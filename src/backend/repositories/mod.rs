use std::time::Duration;

use crate::backend::error::AppError;
use bytes::Bytes;
use deadpool_postgres::{Config as PgConfig, ManagerConfig, Pool, RecyclingMethod};
use futures_util::SinkExt;
use tokio_postgres::types::ToSql;
use tokio_postgres::{Client, CopyInSink, NoTls};

const MAX_DB_CONNECTIONS: usize = 8;

#[derive(Clone)]
pub struct PostgresRepository {
    pool: Pool,
}

impl PostgresRepository {
    pub async fn new_from_env() -> Result<Self, AppError> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| AppError::Config("DATABASE_URL is not set".to_string()))?;

        let mut cfg = PgConfig::new();

        cfg.url = Some(database_url);

        cfg.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });

        cfg.pool = Some(deadpool_postgres::PoolConfig {
            max_size: MAX_DB_CONNECTIONS,
            ..Default::default()
        });

        cfg.connect_timeout = Some(Duration::from_secs(3));

        let pool = cfg.create_pool(None, NoTls)?;
        let repo = Self { pool };

        let _client = repo.pool.get().await?;

        repo.ensure_schema().await?;
        Ok(repo)
    }

    pub async fn conn(&self) -> Result<deadpool_postgres::Client, AppError> {
        Ok(self.pool.get().await?)
    }

    pub async fn ensure_schema(&self) -> Result<(), AppError> {
        let client = self.conn().await?;

        client.batch_execute(
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
        )
        .await?;

        Ok(())
    }

    pub async fn insert_song_inner(
        client: &Client,
        song_name: &str,
        artist_name: &str,
    ) -> Result<u64, AppError> {
        let row = client
            .query_opt(
                r#"
            INSERT INTO songs (name, artist)
            VALUES ($1, $2)
            ON CONFLICT (name, artist) DO NOTHING
            RETURNING id
            "#,
                &[&song_name, &artist_name],
            )
            .await?;

        if let Some(row) = row {
            let id: i64 = row.get(0);
            return Ok(id as u64);
        }

        let row = client
            .query_one(
                r#"SELECT id FROM songs WHERE name = $1 AND artist = $2"#,
                &[&song_name, &artist_name],
            )
            .await?;
        let id: i64 = row.get(0);
        Ok(id as u64)
    }
}

impl PostgresRepository {
    pub async fn songs_count(&self) -> Result<u64, AppError> {
        let client = self.conn().await?;
        let row = client.query_one("SELECT COUNT(*) FROM songs", &[]).await?;
        let count: i64 = row.get(0);
        Ok(count as u64)
    }

    pub async fn insert_song(&self, song_name: &str, artist_name: &str) -> Result<u64, AppError> {
        let client = self.conn().await?;
        Self::insert_song_inner(&client, song_name, artist_name).await
    }

    pub async fn insert_fingerprints(
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

        let mut client = self.conn().await?;
        let tx = client.transaction().await?;

        let writer: CopyInSink<Bytes> = tx
            .copy_in(
                "COPY fingerprints (hash_key, song_id, anchor_time) FROM STDIN WITH (FORMAT csv)",
            )
            .await?;

        tokio::pin!(writer);

        for i in 0..keys.len() {
            let line = format!("{},{},{}\n", keys[i], song_id, anchor_times[i]);
            writer.send(Bytes::from(line)).await?;
        }

        writer.finish().await?;
        tx.commit().await?;

        Ok(keys.len())
    }

    pub async fn get_fingerprints_by_keys(
        &self,
        keys: &[u64],
    ) -> Result<Vec<(u64, u64, f32)>, AppError> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        let keys_i64: Vec<i64> = keys.iter().map(|&k| k as i64).collect();
        let client = self.conn().await?;

        let rows = client
            .query(
                r#"
            SELECT hash_key, song_id, anchor_time
            FROM fingerprints
            WHERE hash_key = ANY($1)
            ORDER BY song_id ASC, anchor_time ASC
            "#,
                &[&keys_i64 as &(dyn ToSql + Sync)],
            )
            .await?;

        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let hash_key: i64 = row.get(0);
            let song_id: i64 = row.get(1);
            let anchor_time: f32 = row.get(2);
            out.push((hash_key as u64, song_id as u64, anchor_time));
        }
        Ok(out)
    }

    pub async fn get_song_info(&self, song_id: u64) -> Result<(u64, String, String), AppError> {
        let client = self.conn().await?;
        let row = client
            .query_opt(
                r#"SELECT id, name, artist FROM songs WHERE id = $1"#,
                &[&(song_id as i64)],
            )
            .await?;

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
