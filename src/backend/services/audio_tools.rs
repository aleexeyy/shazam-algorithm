use crate::backend::error::AppError;
use crate::paths;
use log::{debug, info, warn};
use std::path::Path;
use tokio::process::Command;
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct AudioTools;

impl AudioTools {
    pub async fn ensure_dirs(&self) -> Result<(), AppError> {
        debug!(
            "Ensuring audio dirs exist: audio_dir={}, log_dir={}",
            paths::audio_dir().display(),
            paths::log_dir().display()
        );
        tokio::fs::create_dir_all(paths::audio_dir()).await?;
        tokio::fs::create_dir_all(paths::log_dir()).await?;
        Ok(())
    }

    pub async fn download_with_ytdlp(
        &self,
        song_name: &str,
        artist_name: &str,
        download_id: Uuid,
    ) -> Result<String, AppError> {
        let search_url = format!("ytsearch1:{} {}", artist_name, song_name);
        let output_template =
            paths::audio_dir().join(format!("song_to_process-{}.%(ext)s", download_id));

        let wav_file = format!("song_to_process-{}.wav", download_id);
        let wav_path = paths::audio_dir().join(&wav_file);
        let _ = tokio::fs::remove_file(&wav_path).await;

        info!(
            "Downloading audio via yt-dlp (id={}): \"{}\" - \"{}\"",
            download_id, artist_name, song_name
        );
        let status = Command::new("yt-dlp")
            .arg("--no-playlist")
            .arg("--default-search")
            .arg("ytsearch1")
            .arg("-f")
            .arg("bestaudio/best")
            .arg("-x")
            .arg("--audio-format")
            .arg("wav")
            .arg("-o")
            .arg(output_template)
            .arg(search_url)
            .status()
            .await?;

        if !status.success() {
            warn!("yt-dlp failed (id={}): status={}", download_id, status);
            return Err(AppError::internal(format!(
                "yt-dlp failed with status: {}",
                status
            )));
        }

        if tokio::fs::metadata(&wav_path).await.is_err() {
            return Err(AppError::internal(
                "yt-dlp completed but wav not found".to_string(),
            ));
        }

        debug!(
            "yt-dlp produced wav (id={}): {}",
            download_id,
            wav_path.display()
        );
        Ok(wav_file)
    }

    pub async fn convert_to_wav(
        &self,
        input_path: &Path,
        output_path: &Path,
    ) -> Result<(), AppError> {
        info!(
            "Converting to wav via ffmpeg: {} -> {}",
            input_path.display(),
            output_path.display()
        );
        let status = Command::new("ffmpeg")
            .arg("-y")
            .arg("-i")
            .arg(input_path)
            .arg(output_path)
            .status()
            .await?;

        if !status.success() {
            warn!(
                "ffmpeg conversion failed: {} -> {} (status={})",
                input_path.display(),
                output_path.display(),
                status
            );
            return Err(AppError::internal(format!(
                "ffmpeg failed with status: {}",
                status
            )));
        }

        debug!("ffmpeg conversion complete: {}", output_path.display());
        Ok(())
    }

    pub async fn move_or_copy_to(&self, from: &Path, to: &Path) -> Result<(), AppError> {
        debug!(
            "Moving uploaded file: {} -> {}",
            from.display(),
            to.display()
        );
        match tokio::fs::rename(from, to).await {
            Ok(()) => Ok(()),
            Err(rename_err) => match tokio::fs::copy(from, to).await {
                Ok(_) => {
                    debug!(
                        "Rename failed; copied instead: {} -> {} (removing source)",
                        from.display(),
                        to.display()
                    );
                    tokio::fs::remove_file(from).await?;
                    Ok(())
                }
                Err(_) => Err(rename_err.into()),
            },
        }
    }
}
