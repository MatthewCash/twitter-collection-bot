use futures::future::try_join_all;
use std::error::Error;
use tokio::time::{sleep, Duration};

mod collection;
mod images;
mod redis;
mod twitter;

async fn run_tweet_iteration(
    db_conn: &rusqlite::Connection,
    tw_token: &egg_mode::Token,
    tweet_index: usize,
) -> Result<(), Box<dyn Error>> {
    let next_tweet = collection::get_tweet_from_index(db_conn, tweet_index)?;

    let files = try_join_all(
        next_tweet
            .file_names
            .iter()
            .map(|name| async { images::get_image_file(name).await }),
    )
    .await?;

    let date = collection::get_new_date_for_tweet(&next_tweet)?;

    match collection::get_duration_until_tweet(date)? {
        Some(duration) => {
            println!(
                "Next tweet scheduled for {}, waiting for {}...",
                date, duration
            );
            sleep(duration.unsigned_abs()).await;
        }
        None => {
            // Immediately publish if no date provided (it is overdue)
            println!("Publishing missed tweet for {} in 5s...", date);
            sleep(Duration::from_secs(5)).await;
        }
    };

    println!("Publishing tweet {}...", next_tweet.id);
    twitter::publish_tweet(next_tweet, &files, tw_token).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    let db_conn = collection::load_collection().expect("Failed to load collection!");
    let mut redis_conn = redis::connect().expect("Failed to connect to redis!");
    let tw_token = twitter::create_token().expect("Failed to create twitter token!");
    let tweet_count = collection::get_tweet_count(&db_conn).expect("Failed to get tweet count!");

    loop {
        let i = redis::get_next_index(&mut redis_conn, tweet_count)
            .expect("Failed to get next tweet index!");

        match run_tweet_iteration(&db_conn, &tw_token, i).await {
            Ok(_) => {
                redis::save_tweet_index(&mut redis_conn, i).expect("Failed to save tweet index!")
            }
            Err(why) => println!("Tweet loop failed for index {}: {}", i, &why),
        }
    }
}
