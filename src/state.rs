use anyhow::Result;
use rusqlite::Connection;

pub fn create_state_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS state (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            last_tweet_index INTEGER NOT NULL,
            year_index INTEGER NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO state (id, last_tweet_index, year_index)
VALUES (1, 1, 1)", // both indexes start at 1
        [],
    )?;

    Ok(())
}

// Tweet Index

pub fn get_next_index(conn: &Connection, tweet_count: usize) -> Result<usize> {
    let saved_index: usize = conn
        .prepare("SELECT last_tweet_index FROM state WHERE id = 1")?
        .query_row([], |row| row.get(0))?;

    // Repeat at end
    Ok(if saved_index == tweet_count - 1 {
        log::info!("Reached the end of collection, repeating...");

        let new_year = get_year_index(conn)? + 1;
        set_year_index(conn, new_year)?;

        1
    } else {
        saved_index + 1
    })
}

pub fn save_tweet_index(conn: &Connection, i: usize) -> Result<()> {
    conn.execute("UPDATE state SET last_tweet_index = ? WHERE id = 1", [i])?;
    Ok(())
}

// Year Index

pub fn get_year_index(conn: &Connection) -> Result<usize> {
    Ok(conn
        .prepare("SELECT year_index FROM state WHERE id = 1")?
        .query_row([], |row| row.get(0))?)
}

pub fn set_year_index(conn: &Connection, i: usize) -> Result<()> {
    conn.execute("UPDATE state SET year_index = ? WHERE id = 1", [i])?;
    Ok(())
}
