use redis::{Commands, Connection, RedisError};

static REDIS_KEY: &str = "zerotwo-collection-bot:last_tweet_index";

pub fn connect() -> Result<Connection, RedisError> {
    let client = redis::Client::open(
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".into()),
    )?;
    client.get_connection()
}

pub fn get_next_index(conn: &mut Connection, tweet_count: usize) -> Result<usize, RedisError> {
    let saved_index = conn
        .get::<&str, Option<usize>>(REDIS_KEY)?
        .expect("No tweet index value exists yet!");

    if saved_index == tweet_count - 1 {
        // Repeat at end
        Ok(0)
    } else {
        Ok(saved_index + 1)
    }
}

pub fn save_tweet_index(conn: &mut Connection, i: usize) -> Result<(), RedisError> {
    conn.set(REDIS_KEY, i)
}
