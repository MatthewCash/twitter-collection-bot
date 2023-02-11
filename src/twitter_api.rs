use reqwest::{header::HeaderMap, multipart, Client, ClientBuilder};
use serde_json::{json, Value};
use std::error::Error;
use tokio::time::{sleep, Duration};

pub struct Token {
    pub token: String,
    pub auth_token: String,
}

fn get_client(auth: &Token) -> Result<Client, Box<dyn Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "User-Agent",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/111.0"
            .parse()?,
    );
    headers.insert("Accept", "*/*".parse()?);
    headers.insert("Accept-Language", "en-US,en;q=0.5".parse()?);
    headers.insert("Prefer", "safe".parse()?);
    headers.insert("authorization", format!("Bearer {}", auth.token).parse()?);
    headers.insert("Cookie", format!("ct0=df660175801ded4bd218ef4c3d1c57091468a5d101129f4ca1e54cad82a91b61d1774293ed6affa17dc28bd93772743a3039a71a6fcb7e044f294b49b783e452dafbe270b706de05892ac248dff85e9e; auth_token={}; dnt=1; lang=en", auth.auth_token).parse()?);
    headers.insert("Referer", "https://twitter.com/".parse()?);
    headers.insert("x-csrf-token", "df660175801ded4bd218ef4c3d1c57091468a5d101129f4ca1e54cad82a91b61d1774293ed6affa17dc28bd93772743a3039a71a6fcb7e044f294b49b783e452dafbe270b706de05892ac248dff85e9e".parse()?);
    headers.insert("x-twitter-client-language", "en".parse()?);
    headers.insert("x-twitter-active-user", "yes".parse()?);
    headers.insert("x-twitter-auth-type", "OAuth2Session".parse()?);
    headers.insert("Sec-Fetch-Dest", "empty".parse()?);
    headers.insert("Sec-Fetch-Mode", "cors".parse()?);
    headers.insert("Sec-Fetch-Site", "same-site".parse()?);
    headers.insert("Pragma", "no-cache".parse()?);
    headers.insert("Cache-Control", "no-cache".parse()?);

    Ok(ClientBuilder::new().default_headers(headers).build()?)
}

pub async fn send_tweet(
    token: &Token,
    text: impl Into<String>,
    media_ids: Option<&[u64]>,
    reply_to: Option<u64>,
) -> Result<u64, Box<dyn Error>> {
    let media_entities = match media_ids {
        Some(media_ids) => media_ids
            .iter()
            .map(|id| {
                json!({
                    "media_id": *id,
                    "tagged_users": [],
                })
            })
            .collect(),
        None => vec![],
    };

    let mut body = json!({
        "features": {
            "longform_notetweets_consumption_enabled": true,
            "tweetypie_unmention_optimization_enabled": true,
            "vibe_api_enabled": true,
            "responsive_web_edit_tweet_api_enabled": true,
            "graphql_is_translatable_rweb_tweet_is_translatable_enabled": true,
            "view_counts_everywhere_api_enabled": true,
            "interactive_text_enabled": true,
            "responsive_web_text_conversations_enabled": false,
            "responsive_web_twitter_blue_verified_badge_is_enabled": true,
            "verified_phone_label_enabled": false,
            "standardized_nudges_misinfo": true,
            "tweet_with_visibility_results_prefer_gql_limited_actions_policy_enabled": false,
            "responsive_web_graphql_timeline_navigation_enabled": true,
            "responsive_web_enhance_cards_enabled": false
        },
        "variables": {
            "tweet_text": text.into(),
            "dark_request": false,
            "media": {
                "media_entities": media_entities,
                "possibly_sensitive": false
            },
            "withDownvotePerspective": false,
            "withReactionsMetadata": false,
            "withReactionsPerspective": false,
            "withSuperFollowsTweetFields": true,
            "withSuperFollowsUserFields": true,
            "semantic_annotation_ids": []
        },
        "queryId": "MYy_64Dv_JRBlPN5OZjQXw"
    });

    if let Some(reply_to) = reply_to {
        body["variables"]["reply"] = json!({
            "exclude_reply_user_ids": [],
            "in_reply_to_tweet_id": reply_to.to_string()
        });
    }

    let res = get_client(token)?
        .post("https://api.twitter.com/graphql/uDKNAYFNOggQWtel6bilRw/CreateTweet")
        .json(&body)
        .send()
        .await?;

    let res_json = res.json::<Value>().await?;

    match &res_json["data"]["create_tweet"]["tweet_results"]["result"]["rest_id"] {
        Value::String(id) => Ok(id.parse::<u64>()?),
        _ => Err("Could not determine tweet id!".into()),
    }
}

pub async fn upload_image(
    token: &Token,
    data: &[u8],
    mime: &mime::Mime,
) -> Result<u64, Box<dyn Error>> {
    let client = get_client(token)?;
    let media_category = match (mime.type_(), mime.subtype()) {
        (mime::IMAGE, mime::GIF) => "tweet_gif",
        (mime::VIDEO, mime::MP4) => "tweet_video",
        _ => "tweet_image",
    };

    let query = vec![
        ("command", "INIT".to_string()),
        ("total_bytes", data.len().to_string()),
        ("media_type", mime.to_string()),
        ("media_category", media_category.to_string()),
    ];

    let res = client
        .post("https://upload.twitter.com/i/media/upload.json")
        .query(&query)
        .send()
        .await?;

    let res_json = res.json::<Value>().await?;

    let media_id: u64 = res_json["media_id_string"]
        .as_str()
        .and_then(|x| x.parse().ok())
        .ok_or("Could not determine media id!")?;

    for (i, chunk) in data.chunks(1024 * 1024).enumerate() {
        let query = vec![
            ("command", "APPEND".to_string()),
            ("media_id", media_id.to_string()),
            ("segment_index", i.to_string()),
        ];

        let form = multipart::Form::new().part(
            "media",
            multipart::Part::bytes(chunk.to_owned())
                .file_name("blob")
                .mime_str(mime::APPLICATION_OCTET_STREAM.to_string().as_str())?,
        );

        client
            .post("https://upload.twitter.com/i/media/upload.json")
            .query(&query)
            .multipart(form)
            .send()
            .await?;
    }

    let query = vec![
        ("command", "FINALIZE".to_string()),
        ("media_id", media_id.to_string()),
    ];

    let res = client
        .post("https://upload.twitter.com/i/media/upload.json")
        .query(&query)
        .send()
        .await?;

    let res_json = res.json::<Value>().await?;

    let state = &res_json["processing_info"]["state"];

    let mut finished = state.is_null() || state.as_str().map(|x| x == "succeeded").unwrap_or(false);

    let mut check_after_secs = res_json["processing_info"]["check_after_secs"]
        .as_u64()
        .unwrap_or(5);

    while !finished {
        sleep(Duration::from_secs(check_after_secs)).await;

        let query = vec![
            ("command", "STATUS".to_string()),
            ("media_id", media_id.to_string()),
        ];

        let res = client
            .get("https://upload.twitter.com/i/media/upload.json")
            .query(&query)
            .send()
            .await?;

        let res_json = res.json::<Value>().await?;

        finished = matches!(&res_json["processing_info"]["state"], Value::String(state) if state == "succeeded");

        check_after_secs = res_json["processing_info"]["check_after_secs"]
            .as_u64()
            .unwrap_or(6);
    }

    Ok(media_id)
}
