use actix_web::Error;
use actix_web::dev::Service;
use actix_web::http::header;
use actix_web::{App, HttpServer, middleware, web};
use shazam::backend::handlers::{self, AppState};
use shazam::backend::repositories::PostgresRepository;
use shazam::backend::services::{AudioTools, FingerprintService, SpotifyService};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

const CONTENT_SECURITY_POLICY_VALUE: &str = "default-src 'self'; base-uri 'self'; object-src 'none'; frame-ancestors 'none'; form-action 'self'; img-src 'self' data:; font-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self'; connect-src 'self'";
const PERMISSIONS_POLICY_VALUE: &str = "camera=(), microphone=(), geolocation=(), payment=()";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let port: u16 = std::env::var("SERVER_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8000);

    let repo = retry("postgres", Duration::from_secs(30), || async {
        PostgresRepository::new_from_env()
            .await
            .map_err(|e| std::io::Error::other(e.to_string()))
    })
    .await?;
    let repo: Arc<PostgresRepository> = Arc::new(repo);
    let spotify =
        SpotifyService::new_from_env().map_err(|e| std::io::Error::other(e.to_string()))?;
    let audio = AudioTools;
    audio
        .ensure_dirs()
        .await
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    let fingerprint = FingerprintService::new(Arc::clone(&repo));

    let state = AppState {
        repo,
        spotify,
        audio,
        fingerprint,
    };

    let frontend_dist = std::env::var("FRONTEND_DIST")
        .ok()
        .map(PathBuf::from)
        .or_else(|| Some(PathBuf::from("front-end/dist")))
        .filter(|dir| dir.join("index.html").is_file());
    if let Some(dir) = &frontend_dist {
        log::info!("Serving frontend from {}", dir.display());
    }

    HttpServer::new(move || {
        let mut app = App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(middleware::Logger::default())
            .wrap_fn({
                move |req, srv| {
                    let path = req.path().to_string();
                    let fut = srv.call(req);

                    async move {
                        let mut res = fut.await?;
                        apply_security_headers(res.headers_mut(), &path);
                        Ok::<_, Error>(res)
                    }
                }
            })
            .configure(handlers::configure);

        if let Some(dir) = frontend_dist.clone() {
            app = app.configure(|cfg| handlers::frontend::configure(cfg, dir.clone()));
        }

        app
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

fn apply_security_headers(headers: &mut header::HeaderMap, path: &str) {
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        header::HeaderValue::from_static(CONTENT_SECURITY_POLICY_VALUE),
    );
    headers.insert(
        header::REFERRER_POLICY,
        header::HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        header::HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::X_FRAME_OPTIONS,
        header::HeaderValue::from_static("DENY"),
    );
    headers.insert(
        header::HeaderName::from_static("permissions-policy"),
        header::HeaderValue::from_static(PERMISSIONS_POLICY_VALUE),
    );
    headers.insert(
        header::HeaderName::from_static("cross-origin-opener-policy"),
        header::HeaderValue::from_static("same-origin"),
    );
    headers.insert(
        header::HeaderName::from_static("cross-origin-resource-policy"),
        header::HeaderValue::from_static("same-origin"),
    );

    if path.starts_with("/assets/") {
        headers.insert(
            header::CACHE_CONTROL,
            header::HeaderValue::from_static("public, max-age=31536000, immutable"),
        );
    } else {
        headers.insert(
            header::CACHE_CONTROL,
            header::HeaderValue::from_static("no-store"),
        );
    }
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
