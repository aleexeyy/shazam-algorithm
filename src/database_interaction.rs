use mysql::*;
use mysql::prelude::*;

pub fn set_up_connection() -> Result<PooledConn, Box<dyn std::error::Error>> {
    let url = "mysql://root:Aradchenko2021@localhost:3306";
    let pool = Pool::new(url)?;
    let mut conn: PooledConn = pool.get_conn()?;

    conn.query_drop("CREATE DATABASE IF NOT EXISTS  shazam_db")?;
    conn.query_drop("USE shazam_db")?;


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