use crate::backend::error::AppError;

#[derive(Clone)]
pub struct MinioStorage {
    endpoint: String,
    bucket: String,
    http: reqwest::Client,
}

impl MinioStorage {
    pub async fn new_from_env() -> Result<Self, AppError> {
        let endpoint = std::env::var("S3_ENDPOINT")
            .map_err(|_| AppError::Config("S3_ENDPOINT is not set".to_string()))?;
        let bucket = std::env::var("S3_BUCKET")
            .map_err(|_| AppError::Config("S3_BUCKET is not set".to_string()))?;

        // Still validate these are present so Docker/Compose configs are complete, even if the
        // current code only uses the MinIO health endpoint.
        let _ = std::env::var("S3_ACCESS_KEY")
            .map_err(|_| AppError::Config("S3_ACCESS_KEY is not set".to_string()))?;
        let _ = std::env::var("S3_SECRET_KEY")
            .map_err(|_| AppError::Config("S3_SECRET_KEY is not set".to_string()))?;

        Ok(Self {
            endpoint: endpoint.trim_end_matches('/').to_string(),
            bucket,
            http: reqwest::Client::new(),
        })
    }

    pub async fn ensure_bucket(&self) -> Result<(), AppError> {
        // For MinIO, the unauthenticated readiness endpoint is the most robust startup check.
        // Bucket creation and authenticated S3 operations can be added once the app stores
        // objects and needs them.
        self.ensure_ready().await
    }

    pub async fn ensure_ready(&self) -> Result<(), AppError> {
        let url = format!("{}/minio/health/ready", self.endpoint);
        self.http.get(url).send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn bucket(&self) -> Result<String, AppError> {
        Ok(self.bucket.clone())
    }
}
