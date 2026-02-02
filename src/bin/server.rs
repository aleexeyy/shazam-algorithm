use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware, web};
use shazam::backend::handlers::{self, AppState};
use shazam::backend::repositories::{PostgresRepository, Repository};
use shazam::backend::services::{AudioTools, FingerprintService, SpotifyService};
use shazam::backend::storage::minio::MinioStorage;
use std::sync::Arc;
use std::time::Duration;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let port: u16 = std::env::var("SERVER_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8000);

    let repo = retry("postgres", Duration::from_secs(30), || async {
        tokio::task::spawn_blocking(PostgresRepository::new_from_env)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    })
    .await?;
    let repo: Arc<dyn Repository> = Arc::new(repo);
    let spotify = SpotifyService::new_from_env()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    let audio = AudioTools::default();
    audio
        .ensure_dirs()
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    let fingerprint = FingerprintService::new(Arc::clone(&repo));
    let minio = retry("minio", Duration::from_secs(30), || async {
        let minio = MinioStorage::new_from_env()
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        minio
            .ensure_bucket()
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        Ok(minio)
    })
    .await?;

    let state = AppState {
        repo,
        spotify,
        audio,
        fingerprint,
        minio,
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(middleware::Logger::default())
            .wrap(Cors::permissive())
            .configure(handlers::configure)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

async fn retry<T, Fut>(
    name: &str,
    timeout: Duration,
    mut f: impl FnMut() -> Fut,
) -> std::io::Result<T>
where
    Fut: std::future::Future<Output = std::io::Result<T>>,
{
    let start = std::time::Instant::now();
    let mut delay = Duration::from_millis(200);

    loop {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                if start.elapsed() >= timeout {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        format!("{name} not ready: {e}"),
                    ));
                }
                tokio::time::sleep(delay).await;
                delay = (delay * 2).min(Duration::from_secs(2));
            }
        }
    }
}
