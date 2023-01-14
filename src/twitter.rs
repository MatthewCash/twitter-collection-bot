use crate::collection::CollectionTweet;
use egg_mode::media::upload_media;
use egg_mode::{KeyPair, Token};

use egg_mode::tweet::DraftTweet;

pub fn create_token() -> Result<Token, std::env::VarError> {
    Ok(Token::Access {
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

pub async fn publish_tweet(
    collection_tweet: CollectionTweet,
    files: &[Vec<u8>],
    token: &Token,
) -> Result<(), egg_mode::error::Error> {
    let media_ids = futures::future::try_join_all(
        files
            .iter()
            .map(|file| async { upload_media(file, &mime::IMAGE_JPEG, token).await }),
    )
    .await?
    .iter()
    .map(|handle| handle.id.to_owned())
    .collect::<Vec<_>>();

    let split_tweet =
        collection_tweet.text.len() + collection_tweet.translated_text.len() + 2 > 280;
    let content = if split_tweet {
        collection_tweet.text
    } else {
        format!(
            "{}\n\n{}",
            &collection_tweet.text, &collection_tweet.translated_text
        )
    };

    let mut tweet_draft = DraftTweet::new(content);
    media_ids
        .iter()
        .for_each(|id| tweet_draft.add_media(id.to_owned()));
    let tweet = tweet_draft.send(token).await?;

    if split_tweet {
        DraftTweet::new(collection_tweet.translated_text)
            .in_reply_to(tweet.id)
            .send(token)
            .await?;
    }

    Ok(())
}
