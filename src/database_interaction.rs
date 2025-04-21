use mysql::*;
use mysql::prelude::*;
use std::env;
pub fn set_up_connection() -> Result<PooledConn, Box<dyn std::error::Error>> {
    let db_user = env::var("DB_USER")?;
    let db_password = env::var("DB_PASSWORD")?;
    let db_host = env::var("DB_HOST")?;
    let db_port = env::var("DB_PORT")?.parse::<u16>()?;
    let db_name = env::var("DB_NAME")?;

    // First, connect WITHOUT db_name to create the DB if needed
    let base_opts = OptsBuilder::new()
        .user(Some(&db_user))
        .pass(Some(&db_password))
        .ip_or_hostname(Some(&db_host))
        .tcp_port(db_port);

    let base_pool = Pool::new(base_opts)?;
    let mut base_conn = base_pool.get_conn()?;

    base_conn.query_drop(format!("CREATE DATABASE IF NOT EXISTS `{}`", db_name))?;

    // Now connect WITH db_name
    let full_opts = OptsBuilder::new()
        .user(Some(db_user))
        .pass(Some(db_password))
        .ip_or_hostname(Some(db_host))
        .tcp_port(db_port)
        .db_name(Some(db_name.clone()));

    let pool = Pool::new(full_opts)?;
    let mut conn: PooledConn = pool.get_conn()?;

    conn.query_drop(
        r"CREATE TABLE IF NOT EXISTS fingerprints (
            hash_key BIGINT UNSIGNED NOT NULL,
            song_id BIGINT UNSIGNED NOT NULL,
            anchor_time FLOAT
        )"
    )?;

    conn.query_drop(
        r"CREATE TABLE IF NOT EXISTS songs (
            id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            artist VARCHAR(255) NOT NULL,
            UNIQUE KEY unique_song_artist (name, artist)
        )"
    )?;

    Ok(conn)
}

pub fn insert_fingerprint(conn : &mut PooledConn, keys : Vec<u64>, values : Vec<f64>, song_id : u64) -> Result< usize, Box<dyn std::error::Error>> {
    if keys.len() != values.len() {
        return Err("Keys and values vectors must have the same length".into());
    }
    let mut params_vec = Vec::with_capacity(keys.len());

    for i in 0..keys.len() {
        params_vec.push((
            keys[i],
            song_id, // song_id
            values[i], // anchor_time
        ));
    }

    // Batch insert query
    conn.exec_batch(
        r"INSERT INTO fingerprints (hash_key, song_id, anchor_time)
          VALUES (?, ?, ?)",
        params_vec,
    )?;


    Ok(keys.len())
}

pub fn get_song(conn: &mut PooledConn, keys: &Vec<u64>) -> Result<Vec<(u64, u64, f64)>, Box<dyn std::error::Error>> {
    
    let placeholders = keys.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let query = format!("SELECT * FROM fingerprints WHERE hash_key IN ({}) ORDER BY song_id ASC, anchor_time ASC", placeholders);

    let result: Vec<(u64, u64, f64)> = conn.exec(query, keys)?;

    Ok(result)
}

pub fn insert_song(conn: &mut PooledConn, song_name: &str, artist_name: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let query = r"INSERT IGNORE INTO songs (name, artist) VALUES (?, ?)";
    conn.exec_drop(query, (song_name, artist_name))?;

    let song_id = conn.last_insert_id();
    Ok(song_id)
}

// pub fn clear_database(conn: &mut PooledConn) -> Result<(), Box<dyn std::error::Error>> {
//     conn.query_drop(r"TRUNCATE TABLE fingerprints;")?;
//     conn.query_drop(r"TRUNCATE TABLE songs;")?;
//     Ok(())
// }

pub fn get_song_info(conn: &mut PooledConn, song_id: u64) -> Result<(u64, String, String), Box<dyn std::error::Error>> {
    let result: Vec<(u64, String, String)> = conn.exec(r"SELECT * FROM songs WHERE id = ?", (song_id,))?;
    println!("Got the result based on songs id: {:?}", result);

    if result.is_empty() {
        return Err("Song not found".into());
    }
    Ok(result[0].clone())
}