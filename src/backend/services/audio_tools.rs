use crate::backend::error::AppError;
use crate::paths;
use std::path::{Path, PathBuf};
use tokio::process::Command;

#[derive(Clone, Default)]
pub struct AudioTools;

impl AudioTools {
    pub async fn ensure_dirs(&self) -> Result<(), AppError> {
        tokio::fs::create_dir_all(paths::audio_dir()).await?;
        tokio::fs::create_dir_all(paths::log_dir()).await?;
        Ok(())
    }

    pub fn recognize_target_path(&self) -> PathBuf {
        paths::audio_dir().join("audio_to_recognize.wav")
    }

    pub fn process_target_path(&self) -> PathBuf {
        paths::audio_dir().join("song_to_process.wav")
    }

    pub async fn download_with_ytdlp(
        &self,
        song_name: &str,
        artist_name: &str,
    ) -> Result<(), AppError> {
        let search_url = format!("ytsearch1:{} {}", artist_name, song_name);
        let output_template = paths::audio_dir().join("song_to_process.%(ext)s");

        let wav_path = self.process_target_path();
        let _ = tokio::fs::remove_file(&wav_path).await;

        let status = Command::new("yt-dlp")
            .arg("-x")
            .arg("--audio-format")
            .arg("wav")
            .arg("-o")
            .arg(output_template)
            .arg(search_url)
            .status()
            .await?;

        if !status.success() {
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

        Ok(())
    }

    pub async fn convert_to_wav(
        &self,
        input_path: &Path,
        output_path: &Path,
    ) -> Result<(), AppError> {
        let status = Command::new("ffmpeg")
            .arg("-y")
            .arg("-i")
            .arg(input_path)
            .arg(output_path)
            .status()
            .await?;

        if !status.success() {
            return Err(AppError::internal(format!(
                "ffmpeg failed with status: {}",
                status
            )));
        }

        Ok(())
    }

    pub async fn move_or_copy_to(&self, from: &Path, to: &Path) -> Result<(), AppError> {
        match tokio::fs::rename(from, to).await {
            Ok(()) => Ok(()),
            Err(rename_err) => match tokio::fs::copy(from, to).await {
                Ok(_) => {
                    tokio::fs::remove_file(from).await?;
                    Ok(())
                }
                Err(_) => Err(rename_err.into()),
            },
        }
    }
}
