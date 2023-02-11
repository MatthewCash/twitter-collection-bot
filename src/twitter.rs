use std::error::Error;

use egg_mode::tweet::DraftTweet;
use egg_mode::KeyPair;
use futures::future::try_join_all;

use crate::collection::CollectionTweet;
use crate::twitter_api;

pub enum TwitterAuth {
    Dev(egg_mode::Token),
    User(twitter_api::Token),
}

pub fn create_dev_token() -> Result<egg_mode::Token, std::env::VarError> {
    Ok(egg_mode::Token::Access {
        consumer: KeyPair::new(
            std::env::var("TWITTER_CONSUMER_KEY")?,
            std::env::var("TWITTER_CONSUMER_SECRET")?,
        ),
        access: KeyPair::new(
            std::env::var("TWITTER_ACCESS_KEY")?,
            std::env::var("TWITTER_ACCESS_SECRET")?,
        ),
    })
}

pub fn create_user_token() -> Result<twitter_api::Token, std::env::VarError> {
    Ok(twitter_api::Token {
        token: std::env::var("TWITTER_USER_TOKEN")?,
        auth_token: std::env::var("TWITTER_USER_AUTH_TOKEN")?,
    })
}

pub async fn upload_media(
    medias: &[(&mime::Mime, &[u8])],
    auth: &TwitterAuth,
) -> Result<Vec<u64>, Box<dyn Error>> {
    try_join_all(medias.iter().map(|(mime, data)| async {
        match auth {
            TwitterAuth::Dev(token) => {
                let id_str = egg_mode::media::upload_media(data, mime, token).await?.id.0;
                Ok(id_str.parse()?)
            }
            TwitterAuth::User(token) => twitter_api::upload_image(token, data, mime).await,
        }
    }))
    .await
}

pub async fn publish_tweet(
    collection_tweet: CollectionTweet,
    media_ids: &[u64],
    auth: &TwitterAuth,
) -> Result<(), Box<dyn Error>> {
    let should_split_tweet =
        collection_tweet.text.len() + collection_tweet.translated_text.len() + 2 > 280;
    let content = if should_split_tweet {
        collection_tweet.text
    } else {
        format!(
            "{}\n\n{}",
            &collection_tweet.text, &collection_tweet.translated_text
        )
    };

    let tweet_id = match auth {
        TwitterAuth::Dev(token) => {
            let mut tweet_draft = DraftTweet::new(content);
            media_ids
                .iter()
                .for_each(|id| tweet_draft.add_media(id.to_string().into()));
            tweet_draft.send(token).await?.id
        }
        TwitterAuth::User(token) => {
            twitter_api::send_tweet(token, content, Some(media_ids), None).await?
        }
    };

    // Tweet content is too long, send translation in reply
    if should_split_tweet {
        match auth {
            TwitterAuth::Dev(token) => {
                DraftTweet::new(collection_tweet.translated_text)
                    .in_reply_to(tweet_id)
                    .send(token)
                    .await?;
            }
            TwitterAuth::User(token) => {
                twitter_api::send_tweet(
                    token,
                    collection_tweet.translated_text,
                    None,
                    Some(tweet_id),
                )
                .await?;
            }
        };
    }

    Ok(())
}
