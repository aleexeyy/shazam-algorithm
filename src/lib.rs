use crate::backend::repositories::PostgresRepository;

pub mod fingerprinting;
mod paths;

pub mod backend;

pub async fn recognize_with_repo(
    repo: &PostgresRepository,
    recognize_audio_file: &str,
) -> Result<(String, String), backend::error::AppError> {
    let audio = fingerprinting::audio_processing::process_audio(recognize_audio_file)
        .map_err(|e| backend::error::AppError::internal(e.to_string()))?;

    let peaks = fingerprinting::process_spectr::find_spectral_peaks(&audio)
        .map_err(|e| backend::error::AppError::internal(e.to_string()))?;

    let matched_song_id = fingerprinting::create_hash::create_pairs(repo, &peaks, 0, true).await?;
    let (_, matched_name, matched_artist) = repo.get_song_info(matched_song_id).await?;
    Ok((matched_name, matched_artist))
}

pub async fn ingest_with_repo(
    repo: &PostgresRepository,
    song_name: &str,
    artist_name: &str,
    process_audio_file: &str,
) -> Result<(String, String), backend::error::AppError> {
    let song_id = repo.insert_song(song_name, artist_name).await?;

    let audio = fingerprinting::audio_processing::process_audio(process_audio_file)
        .map_err(|e| backend::error::AppError::internal(e.to_string()))?;

    let peaks = fingerprinting::process_spectr::find_spectral_peaks(&audio)
        .map_err(|e| backend::error::AppError::internal(e.to_string()))?;

    let _ = fingerprinting::create_hash::create_pairs(repo, &peaks, song_id, false).await?;
    Ok((song_name.to_string(), artist_name.to_string()))
}
