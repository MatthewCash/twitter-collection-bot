use rusqlite::{Connection, Result};
use std::error::Error;
use time::{error::ComponentRange, macros::time, OffsetDateTime};
use tokio::time::Duration;

pub struct CollectionTweet {
    pub id: u64,
    pub file_names: Vec<String>,
    pub text: String,
    pub translated_text: String,
    date: OffsetDateTime,
}

pub fn load_collection() -> Result<Connection, rusqlite::Error> {
    let db_path = std::env::var("COLLECTION_PATH").unwrap_or_else(|_| "./collection.db".into());
    Connection::open(db_path)
}

pub fn get_tweet_from_index(
    conn: &Connection,
    index: usize,
) -> Result<CollectionTweet, Box<dyn Error>> {
    let mut stmt = conn.prepare("SELECT * FROM tweets WHERE rowid = (?) LIMIT 1")?;
    let items = stmt.query_row([index], |row| {
        Ok((
            row.get::<usize, u64>(0)?,
            row.get::<usize, String>(1)?,
            row.get::<usize, String>(2)?,
            row.get::<usize, String>(3)?,
            row.get::<usize, i64>(4)?,
        ))
    })?;

    let file_names = items
        .1
        .split(',')
        .map(|x| x.to_string())
        .collect::<Vec<_>>();

    let date = OffsetDateTime::from_unix_timestamp(items.4)?;

    Ok(CollectionTweet {
        id: items.0,
        file_names,
        text: items.2,
        translated_text: items.3,
        date,
    })
}

pub fn get_tweet_count(conn: &Connection) -> Result<usize, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM tweets")?;
    stmt.query_row([], |row| row.get(0))
}

pub fn get_new_date_for_tweet(tweet: &CollectionTweet) -> Result<OffsetDateTime, ComponentRange> {
    let new_year = tweet.date.year() + 3;

    match tweet.date.replace_year(new_year) {
        // If original day is 2/29 (leap year) set to end of 2/28
        Err(why) if why.name() == "day" && tweet.date.day() == 29 => tweet
            .date
            .replace_time(time!(23:59:59))
            .replace_day(28)?
            .replace_year(new_year),
        date => date,
    }
}

pub fn get_duration_until_tweet(date: OffsetDateTime) -> Duration {
    match date - OffsetDateTime::now_utc() {
        duration if duration.is_positive() => duration.unsigned_abs(),
        _ => Duration::ZERO,
    }
}
