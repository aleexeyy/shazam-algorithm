use crate::backend::error::AppError;
use crate::backend::repositories::Repository;
use log::{debug, info};
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone)]
pub struct FingerprintService {
    repo: Arc<dyn Repository>,
}

impl FingerprintService {
    pub fn new(repo: Arc<dyn Repository>) -> Self {
        Self { repo }
    }

    pub async fn recognize_from_file(
        &self,
        recognize_audio_file: String,
    ) -> Result<(String, String), AppError> {
        let start = Instant::now();
        info!("Fingerprint recognize start: {}", recognize_audio_file);
        let repo = Arc::clone(&self.repo);
        let result = tokio::task::spawn_blocking(move || {
            crate::recognize_with_repo(repo.as_ref(), &recognize_audio_file)
        })
        .await?;
        debug!("Fingerprint recognize done in {:?}", start.elapsed());
        result
    }

    pub async fn ingest_from_file(
        &self,
        song_name: String,
        artist_name: String,
        process_audio_file: String,
    ) -> Result<(String, String), AppError> {
        let start = Instant::now();
        info!(
            "Fingerprint ingest start: \"{}\" - \"{}\" (file={})",
            artist_name, song_name, process_audio_file
        );
        let repo = Arc::clone(&self.repo);
        let result = tokio::task::spawn_blocking(move || {
            crate::ingest_with_repo(repo.as_ref(), &song_name, &artist_name, &process_audio_file)
        })
        .await?;
        debug!("Fingerprint ingest done in {:?}", start.elapsed());
        result
    }
}
