use crate::backend::error::AppError;
use crate::backend::repositories::Repository;
use mysql::prelude::*;
use mysql::{OptsBuilder, Params, Pool, PooledConn, Value};

#[derive(Clone)]
pub struct MySqlRepository {
    pool: Pool,
}

impl MySqlRepository {
    pub fn new_from_env() -> Result<Self, AppError> {
        let db_user = std::env::var("DB_USER")?;
        let db_password = std::env::var("DB_PASSWORD")?;
        let db_host = std::env::var("DB_HOST")?;
        let db_port: u16 = std::env::var("DB_PORT")
            .unwrap_or_else(|_| "3306".to_string())
            .parse()?;
        let db_name = std::env::var("DB_NAME")?;

        // Connect without selecting DB, create it if needed.
        let base_opts = OptsBuilder::new()
            .user(Some(&db_user))
            .pass(Some(&db_password))
            .ip_or_hostname(Some(&db_host))
            .tcp_port(db_port);

        let base_pool = Pool::new(base_opts)?;
        let mut base_conn = base_pool.get_conn()?;
        base_conn.query_drop(format!("CREATE DATABASE IF NOT EXISTS `{}`", db_name))?;

        // Connect with DB selected.
        let full_opts = OptsBuilder::new()
            .user(Some(db_user))
            .pass(Some(db_password))
            .ip_or_hostname(Some(db_host))
            .tcp_port(db_port)
            .db_name(Some(db_name));

        let pool = Pool::new(full_opts)?;
        let repo = Self { pool };
        repo.ensure_schema()?;
        Ok(repo)
    }

    fn with_conn<T>(
        &self,
        f: impl FnOnce(&mut PooledConn) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let mut conn = self.pool.get_conn()?;
        f(&mut conn)
    }

    fn ensure_schema(&self) -> Result<(), AppError> {
        self.with_conn(|conn| {
            conn.query_drop(
                r"CREATE TABLE IF NOT EXISTS fingerprints (
                    hash_key BIGINT UNSIGNED NOT NULL,
                    song_id BIGINT UNSIGNED NOT NULL,
                    anchor_time FLOAT
                )",
            )?;

            conn.query_drop(
                r"CREATE TABLE IF NOT EXISTS songs (
                    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
                    name VARCHAR(255) NOT NULL,
                    artist VARCHAR(255) NOT NULL,
                    UNIQUE KEY unique_song_artist (name, artist)
                )",
            )?;

            Ok(())
        })
    }
}

impl Repository for MySqlRepository {
    fn songs_count(&self) -> Result<u64, AppError> {
        self.with_conn(|conn| {
            let count: Option<u64> = conn.query_first("SELECT COUNT(*) FROM songs")?;
            Ok(count.unwrap_or(0))
        })
    }

    fn insert_song(&self, song_name: &str, artist_name: &str) -> Result<u64, AppError> {
        self.with_conn(|conn| {
            let query = r"INSERT IGNORE INTO songs (name, artist) VALUES (?, ?)";
            conn.exec_drop(query, (song_name, artist_name))?;
            Ok(conn.last_insert_id())
        })
    }

    fn insert_fingerprints(
        &self,
        song_id: u64,
        keys: &[u64],
        anchor_times: &[f32],
    ) -> Result<usize, AppError> {
        if keys.len() != anchor_times.len() {
            return Err(AppError::BadRequest(
                "keys and anchor_times must have the same length".to_string(),
            ));
        }

        self.with_conn(|conn| {
            let mut params_vec = Vec::with_capacity(keys.len());
            for i in 0..keys.len() {
                params_vec.push((keys[i], song_id, anchor_times[i]));
            }

            conn.exec_batch(
                r"INSERT INTO fingerprints (hash_key, song_id, anchor_time)
                  VALUES (?, ?, ?)",
                params_vec,
            )?;
            Ok(keys.len())
        })
    }

    fn get_fingerprints_by_keys(&self, keys: &[u64]) -> Result<Vec<(u64, u64, f32)>, AppError> {
        self.with_conn(|conn| {
            if keys.is_empty() {
                return Ok(Vec::new());
            }

            let placeholders = keys.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
            let query = format!(
                "SELECT * FROM fingerprints WHERE hash_key IN ({}) ORDER BY song_id ASC, anchor_time ASC",
                placeholders
            );

            let values: Vec<Value> = keys.iter().copied().map(Value::from).collect();
            let params = Params::Positional(values);
            let result: Vec<(u64, u64, f32)> = conn.exec(query, params)?;
            Ok(result)
        })
    }

    fn get_song_info(&self, song_id: u64) -> Result<(u64, String, String), AppError> {
        self.with_conn(|conn| {
            let result: Option<(u64, String, String)> =
                conn.exec_first(r"SELECT * FROM songs WHERE id = ?", (song_id,))?;
            match result {
                Some(row) => Ok(row),
                None => Err(AppError::NotFound("song not found".to_string())),
            }
        })
    }
}
