use std::env;

use anyhow::Result;
use rusqlite::{named_params, Connection, Error::QueryReturnedNoRows, Row};
use time::{macros::time, OffsetDateTime};
use tokio::time::Duration;

use crate::state::get_year_index;

pub struct CollectionTweet {
    pub id: u64,
    pub file_names: Vec<String>,
    pub text: String,
    pub translated_text: String,
    date: OffsetDateTime,
}

pub fn load_collection() -> Result<Connection> {
    let db_path = env::var("DB_PATH").unwrap_or_else(|_| "./db.sqlite".into());
    Ok(rusqlite::Connection::open(db_path)?)
}

fn convert_row_to_tweet(row: &Row) -> Result<CollectionTweet> {
    let file_names = row
        .get::<&str, String>("file_names")?
        .split(',')
        .map(|x| x.into())
        .collect::<Vec<_>>();

    let date = OffsetDateTime::from_unix_timestamp(row.get::<&str, i64>("date")?)?;

    Ok(CollectionTweet {
        id: row.get::<&str, u64>("id")?,
        file_names,
        text: row.get::<&str, String>("text")?,
        translated_text: row.get::<&str, String>("translated_text")?,
        date,
    })
}

pub fn get_tweet_from_index(conn: &Connection, index: usize) -> Result<CollectionTweet> {
    let mut stmt = conn.prepare("SELECT * FROM tweets WHERE rowid = (:index) LIMIT 1")?;

    let mut rows = stmt.query(named_params! { ":index": index, })?;
    let row = rows.next()?.ok_or(QueryReturnedNoRows)?;

    convert_row_to_tweet(row)
}

pub fn get_tweet_count(conn: &Connection) -> Result<usize> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM tweets")?;
    Ok(stmt.query_row([], |row| row.get(0))?)
}

fn get_year_offset_for_tweet(conn: &Connection) -> Result<usize> {
    get_year_index(conn).map(|year_index| {
        year_index
            * env::var("COLLECTION_DURATION_YEARS")
                .ok()
                .and_then(|x| x.parse().ok())
                .unwrap_or(3)
    })
}

pub fn get_new_date_for_tweet(
    conn: &Connection,
    tweet: &CollectionTweet,
) -> Result<OffsetDateTime> {
    let new_year = tweet.date.year() + get_year_offset_for_tweet(conn)? as i32;

    Ok(match tweet.date.replace_year(new_year) {
        // If original day is 2/29 (leap day on non leap year) set to end of 2/28
        Err(why) if why.name() == "day" && tweet.date.day() == 29 => tweet
            .date
            .replace_time(time!(23:59:59))
            .replace_day(28)?
            .replace_year(new_year),
        date => date,
    }?)
}

pub fn get_duration_until_tweet(date: OffsetDateTime) -> Duration {
    match date - OffsetDateTime::now_utc() {
        duration if duration.is_positive() => duration.unsigned_abs(),
        _ => Duration::ZERO,
    }
}
