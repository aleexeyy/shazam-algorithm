use crate::backend::error::AppError;
use crate::backend::models::{
    PongResponse, RecognizeSongResponse, SongsCountResponse, UploadSongRequest, UploadSongResponse,
};
use crate::backend::repositories::Repository;
use crate::backend::services::{AudioTools, FingerprintService, SpotifyService};
use actix_multipart::Multipart;
use actix_web::{HttpResponse, web};
use futures_util::StreamExt;
use mime::Mime;
use std::sync::Arc;

pub mod frontend;

#[derive(Clone)]
pub struct AppState {
    pub repo: Arc<dyn Repository>,
    pub spotify: SpotifyService,
    pub audio: AudioTools,
    pub fingerprint: FingerprintService,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/ping", web::get().to(ping))
        .route("/healthz", web::get().to(healthz))
        .route("/songs/count", web::get().to(songs_count))
        .route("/upload-song", web::post().to(upload_song))
        .route("/recognize-song", web::post().to(recognize_song));
}

async fn ping() -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(PongResponse { message: "Pong" }))
}

async fn healthz(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let repo = Arc::clone(&state.repo);
    let _ = tokio::task::spawn_blocking(move || repo.songs_count()).await??;
    Ok(HttpResponse::Ok().json(PongResponse { message: "OK" }))
}

async fn songs_count(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let repo = Arc::clone(&state.repo);
    let count = tokio::task::spawn_blocking(move || repo.songs_count()).await??;
    Ok(HttpResponse::Ok().json(SongsCountResponse { count }))
}

async fn upload_song(
    state: web::Data<AppState>,
    body: web::Json<UploadSongRequest>,
) -> Result<HttpResponse, AppError> {
    if body.to_recognize {
        return Err(AppError::BadRequest(
            "upload-song does not support toRecognize=true".to_string(),
        ));
    }

    state.audio.ensure_dirs().await?;
    let track = state.spotify.track_info(&body.song_id).await?;
    let download_id = uuid::Uuid::new_v4();
    let wav_file = state
        .audio
        .download_with_ytdlp(&track.name, &track.artist, download_id)
        .await?;

    let _ = state
        .fingerprint
        .ingest_from_file(track.name, track.artist, wav_file.clone())
        .await?;

    let _ = tokio::fs::remove_file(crate::paths::audio_dir().join(&wav_file)).await;

    Ok(HttpResponse::Ok().json(UploadSongResponse {
        upload_status: "OK",
    }))
}

async fn recognize_song(
    state: web::Data<AppState>,
    mut multipart: Multipart,
) -> Result<HttpResponse, AppError> {
    state.audio.ensure_dirs().await?;

    let mut saved_path = None::<std::path::PathBuf>;
    let mut content_type = None::<Mime>;
    let upload_id = uuid::Uuid::new_v4();

    while let Some(field) = multipart.next().await {
        let mut field = field.map_err(|e| AppError::BadRequest(e.to_string()))?;
        if field.name() != Some("audio") {
            continue;
        }

        content_type = field.content_type().cloned();

        let file_name = format!("upload-{}.bin", upload_id);
        let temp_path = crate::paths::audio_dir().join(file_name);
        let mut file = tokio::fs::File::create(&temp_path).await?;

        while let Some(chunk) = field.next().await {
            let data = chunk.map_err(|e| AppError::BadRequest(e.to_string()))?;
            tokio::io::AsyncWriteExt::write_all(&mut file, &data).await?;
        }

        saved_path = Some(temp_path);
        break;
    }

    let temp_path =
        saved_path.ok_or_else(|| AppError::BadRequest("missing 'audio' file field".to_string()))?;

    let recognize_wav_file = format!("audio_to_recognize-{}.wav", upload_id);
    let target_path = crate::paths::audio_dir().join(&recognize_wav_file);
    let mime_type = content_type.unwrap_or(mime::APPLICATION_OCTET_STREAM);

    match mime_type.essence_str() {
        "audio/wav" | "audio/x-wav" => {
            state
                .audio
                .move_or_copy_to(&temp_path, &target_path)
                .await?;
        }
        "audio/mpeg" | "audio/ogg" => {
            state.audio.convert_to_wav(&temp_path, &target_path).await?;
            let _ = tokio::fs::remove_file(&temp_path).await;
        }
        other => {
            let _ = tokio::fs::remove_file(&temp_path).await;
            return Err(AppError::UnsupportedMediaType(other.to_string()));
        }
    }

    let result = state
        .fingerprint
        .recognize_from_file(recognize_wav_file)
        .await;

    let _ = tokio::fs::remove_file(&target_path).await;
    let (name, artist) = result?;

    Ok(HttpResponse::Ok().json(RecognizeSongResponse { name, artist }))
}
