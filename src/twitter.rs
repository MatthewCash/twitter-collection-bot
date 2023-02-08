use std::error::Error;

use egg_mode::tweet::DraftTweet;
use egg_mode::{KeyPair, Token};
use futures::future::try_join_all;

use crate::collection::CollectionTweet;

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

pub async fn upload_media(
    medias: &[(&mime::Mime, &[u8])],
    token: &Token,
) -> Result<Vec<u64>, Box<dyn Error>> {
    try_join_all(medias.iter().map(|(mime, data)| async {
        let id_str = egg_mode::media::upload_media(data, mime, token).await?.id.0;
        Ok(id_str.parse()?)
    }))
    .await
}

pub async fn publish_tweet(
    collection_tweet: CollectionTweet,
    media_ids: &[u64],
    token: &Token,
) -> Result<(), egg_mode::error::Error> {
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

    let mut tweet_draft = DraftTweet::new(content);
    media_ids
        .iter()
        .for_each(|id| tweet_draft.add_media(id.to_string().into()));
    let tweet = tweet_draft.send(token).await?;

    if should_split_tweet {
        DraftTweet::new(collection_tweet.translated_text)
            .in_reply_to(tweet.id)
            .send(token)
            .await?;
    }

    Ok(())
}
