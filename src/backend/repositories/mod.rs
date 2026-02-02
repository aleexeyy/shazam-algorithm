mod postgres_repository;

use crate::backend::error::AppError;

pub use postgres_repository::PostgresRepository;

pub trait Repository: Send + Sync + 'static {
    fn songs_count(&self) -> Result<u64, AppError>;
    fn insert_song(&self, song_name: &str, artist_name: &str) -> Result<u64, AppError>;
    fn insert_fingerprints(
        &self,
        song_id: u64,
        keys: &[u64],
        anchor_times: &[f32],
    ) -> Result<usize, AppError>;
    fn get_fingerprints_by_keys(&self, keys: &[u64]) -> Result<Vec<(u64, u64, f32)>, AppError>;
    fn get_song_info(&self, song_id: u64) -> Result<(u64, String, String), AppError>;
}
