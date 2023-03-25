use futures::future::try_join_all;
use mime_sniffer::MimeTypeSniffer;
use std::error::Error;
use tokio::time::{sleep, Duration};
use twitter::{create_dev_token, create_user_token, TwitterAuth};

mod collection;
mod images;
mod redis;
mod twitter;
mod twitter_api;

static MEDIA_UPLOAD_SECS: u64 = 60;

async fn run_tweet_iteration(
    redis_conn: &mut ::redis::Connection,
    db_conn: &rusqlite::Connection,
    tw_auth: &TwitterAuth,
    tweet_index: usize,
) -> Result<(), Box<dyn Error>> {
    let next_tweet = collection::get_tweet_from_index(db_conn, tweet_index)?;

    let files = try_join_all(
        next_tweet
            .file_names
            .iter()
            .map(|name| images::get_image_file(name)),
    )
    .await?;

    let mimes = files
        .iter()
        .map(|data| {
            data.sniff_mime_type()
                .and_then(|x| x.parse().ok())
                .unwrap_or(mime::IMAGE_JPEG)
        })
        .collect::<Vec<_>>();

    let medias = mimes
        .iter()
        .zip(files.iter())
        .map(|(a, b)| (a, b.as_slice()))
        .collect::<Vec<_>>();

    let date = collection::get_new_date_for_tweet(redis_conn, &next_tweet)?;

    match collection::get_duration_until_tweet(date) {
        duration if !duration.is_zero() => {
            println!(
                "Next tweet scheduled for {}, waiting for {:.2}h",
                date,
                duration.as_secs() as f64 / 3600.0
            );

            sleep(duration - Duration::from_secs(MEDIA_UPLOAD_SECS)).await;
        }
        _ => {
            // Immediately publish if duration is zero (it is overdue)
            println!("Publishing missed tweet for {} in 5s...", date);
            sleep(Duration::from_secs(5)).await;
        }
    };

    println!(
        "Uploading {} medias for tweet {}...",
        medias.len(),
        next_tweet.id
    );
    let media_ids = twitter::upload_media(&medias, tw_auth).await?;

    sleep(collection::get_duration_until_tweet(date)).await;

    println!("Publishing tweet {}...", next_tweet.id);
    twitter::publish_tweet(next_tweet, &media_ids, tw_auth).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    let db_conn = collection::load_collection().expect("Failed to load collection!");
    let mut redis_conn = redis::connect().expect("Failed to connect to redis!");
    let tweet_count = collection::get_tweet_count(&db_conn).expect("Failed to get tweet count!");

    let tw_auth = if std::env::var("TWITTER_USE_USER").is_ok() {
        TwitterAuth::User(create_user_token().expect("Failed to create twitter user token!"))
    } else {
        TwitterAuth::Dev(create_dev_token().expect("Failed to create twitter dev token!"))
    };

    loop {
        let i = redis::get_next_index(&mut redis_conn, tweet_count)
            .expect("Failed to get next tweet index!");

        match run_tweet_iteration(&mut redis_conn, &db_conn, &tw_auth, i).await {
            Ok(_) => {
                redis::save_tweet_index(&mut redis_conn, i).expect("Failed to save tweet index!")
            }
            Err(why) => println!("Tweet loop failed for index {}: {}", i, why),
        }
    }
}
