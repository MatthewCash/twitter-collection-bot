use redis::{Commands, Connection, RedisError};

static LAST_TWEET_INDEX_KEY: &str = "zerotwo-collection-bot:last_tweet_index";
static YEAR_INDEX_KEY: &str = "zerotwo-collection-bot:year_index";

pub fn connect() -> Result<Connection, RedisError> {
    let client = redis::Client::open(
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".into()),
    )?;
    client.get_connection()
}

// Tweet Index

pub fn get_next_index(conn: &mut Connection, tweet_count: usize) -> Result<usize, RedisError> {
    let saved_index = conn
        .get::<&str, Option<usize>>(LAST_TWEET_INDEX_KEY)?
        .expect("No tweet index value exists yet!");

    if saved_index == tweet_count - 1 {
        // Repeat at end
        println!("Reached the end of collection, repeating...");

        let new_year = get_year_index(conn)? + 1;
        set_year_index(conn, new_year)?;

        Ok(1)
    } else {
        Ok(saved_index + 1)
    }
}

pub fn save_tweet_index(conn: &mut Connection, i: usize) -> Result<(), RedisError> {
    conn.set(LAST_TWEET_INDEX_KEY, i)
}

// Year Index

pub fn get_year_index(conn: &mut Connection) -> Result<i32, RedisError> {
    conn.get::<&str, Option<i32>>(YEAR_INDEX_KEY)
        .map(|x| x.expect("No year index value exists yet!"))
}

pub fn set_year_index(conn: &mut Connection, i: i32) -> Result<(), RedisError> {
    conn.set(YEAR_INDEX_KEY, i)
}
