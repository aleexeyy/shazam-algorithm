use crate::backend::error::AppError;
use crate::backend::repositories::Repository;
use postgres::types::ToSql;
use postgres::{Client, NoTls};
use r2d2::{Pool, PooledConnection};
use r2d2_postgres::PostgresConnectionManager;
use std::io::Write;

#[derive(Clone)]
pub struct PostgresRepository {
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

impl PostgresRepository {
    pub fn new_from_env() -> Result<Self, AppError> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| AppError::Config("DATABASE_URL is not set".to_string()))?;

        let manager = PostgresConnectionManager::new(database_url.parse()?, NoTls);
        let pool = Pool::builder().max_size(8).build(manager)?;
        let repo = Self { pool };
        repo.ensure_schema()?;
        Ok(repo)
    }

    fn conn(&self) -> Result<PooledConnection<PostgresConnectionManager<NoTls>>, AppError> {
        Ok(self.pool.get()?)
    }

    fn ensure_schema(&self) -> Result<(), AppError> {
        let mut conn = self.conn()?;

        conn.batch_execute(
            r#"
            CREATE TABLE IF NOT EXISTS songs (
                id BIGSERIAL PRIMARY KEY,
                name TEXT NOT NULL,
                artist TEXT NOT NULL,
                UNIQUE (name, artist)
            );

            CREATE TABLE IF NOT EXISTS fingerprints (
                hash_key BIGINT NOT NULL,
                song_id BIGINT NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
                anchor_time REAL
            );

            CREATE INDEX IF NOT EXISTS fingerprints_hash_key_idx ON fingerprints(hash_key);
            CREATE INDEX IF NOT EXISTS fingerprints_song_time_idx ON fingerprints(song_id, anchor_time);
            "#,
        )?;

        Ok(())
    }

    fn insert_song_inner(
        conn: &mut Client,
        song_name: &str,
        artist_name: &str,
    ) -> Result<u64, AppError> {
        // Insert if new, otherwise fetch the existing id.
        let row = conn.query_opt(
            r#"
            INSERT INTO songs (name, artist)
            VALUES ($1, $2)
            ON CONFLICT (name, artist) DO NOTHING
            RETURNING id
            "#,
            &[&song_name, &artist_name],
        )?;

        if let Some(row) = row {
            let id: i64 = row.get(0);
            return Ok(id as u64);
        }

        let row = conn.query_one(
            r#"SELECT id FROM songs WHERE name = $1 AND artist = $2"#,
            &[&song_name, &artist_name],
        )?;
        let id: i64 = row.get(0);
        Ok(id as u64)
    }
}

impl Repository for PostgresRepository {
    fn songs_count(&self) -> Result<u64, AppError> {
        let mut conn = self.conn()?;
        let row = conn.query_one("SELECT COUNT(*) FROM songs", &[])?;
        let count: i64 = row.get(0);
        Ok(count as u64)
    }

    fn insert_song(&self, song_name: &str, artist_name: &str) -> Result<u64, AppError> {
        let mut conn = self.conn()?;
        Self::insert_song_inner(&mut conn, song_name, artist_name)
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

        if keys.is_empty() {
            return Ok(0);
        }

        let mut conn = self.conn()?;
        let mut tx = conn.transaction()?;

        // Use COPY for reasonably efficient ingestion without per-row roundtrips.
        let mut writer = tx.copy_in(
            "COPY fingerprints (hash_key, song_id, anchor_time) FROM STDIN WITH (FORMAT csv)",
        )?;
        for i in 0..keys.len() {
            writeln!(writer, "{},{},{}", keys[i], song_id, anchor_times[i])?;
        }
        writer.finish()?;
        tx.commit()?;

        Ok(keys.len())
    }

    fn get_fingerprints_by_keys(&self, keys: &[u64]) -> Result<Vec<(u64, u64, f32)>, AppError> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        let keys_i64: Vec<i64> = keys.iter().map(|&k| k as i64).collect();
        let mut conn = self.conn()?;

        let rows = conn.query(
            r#"
            SELECT hash_key, song_id, anchor_time
            FROM fingerprints
            WHERE hash_key = ANY($1)
            ORDER BY song_id ASC, anchor_time ASC
            "#,
            &[&keys_i64 as &(dyn ToSql + Sync)],
        )?;

        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let hash_key: i64 = row.get(0);
            let song_id: i64 = row.get(1);
            let anchor_time: f32 = row.get(2);
            out.push((hash_key as u64, song_id as u64, anchor_time));
        }
        Ok(out)
    }

    fn get_song_info(&self, song_id: u64) -> Result<(u64, String, String), AppError> {
        let mut conn = self.conn()?;
        let row = conn.query_opt(
            r#"SELECT id, name, artist FROM songs WHERE id = $1"#,
            &[&(song_id as i64)],
        )?;

        match row {
            Some(row) => {
                let id: i64 = row.get(0);
                let name: String = row.get(1);
                let artist: String = row.get(2);
                Ok((id as u64, name, artist))
            }
            None => Err(AppError::NotFound("song not found".to_string())),
        }
    }
}
