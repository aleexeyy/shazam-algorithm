use crate::backend::error::AppError;
use crate::backend::repositories::Repository;
use std::sync::Arc;

#[derive(Clone)]
pub struct FingerprintService {
    repo: Arc<dyn Repository>,
}

impl FingerprintService {
    pub fn new(repo: Arc<dyn Repository>) -> Self {
        Self { repo }
    }

    pub async fn run(
        &self,
        song_name: String,
        artist_name: String,
        to_recognize: bool,
    ) -> Result<(String, String), AppError> {
        let repo = Arc::clone(&self.repo);
        tokio::task::spawn_blocking(move || {
            crate::run_shazam_with_repo(repo.as_ref(), &song_name, &artist_name, to_recognize)
        })
        .await?
    }
}
