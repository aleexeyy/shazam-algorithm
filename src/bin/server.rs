use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware, web};
use shazam::backend::handlers::{self, AppState};
use shazam::backend::repositories::{MySqlRepository, Repository};
use shazam::backend::services::{AudioTools, FingerprintService, SpotifyService};
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let port: u16 = std::env::var("SERVER_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8000);

    let repo: Arc<dyn Repository> = Arc::new(
        MySqlRepository::new_from_env()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?,
    );
    let spotify = SpotifyService::new_from_env()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    let audio = AudioTools::default();
    audio
        .ensure_dirs()
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    let fingerprint = FingerprintService::new(Arc::clone(&repo));

    let state = AppState {
        repo,
        spotify,
        audio,
        fingerprint,
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
