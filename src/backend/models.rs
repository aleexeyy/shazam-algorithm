use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct PongResponse {
    pub message: &'static str,
}

#[derive(Debug, Serialize)]
pub struct SongsCountResponse {
    pub count: u64,
}

#[derive(Debug, Deserialize)]
pub struct UploadSongRequest {
    #[serde(rename = "songId")]
    pub song_id: String,
    #[serde(rename = "toRecognize")]
    pub to_recognize: bool,
}

#[derive(Debug, Serialize)]
pub struct UploadSongResponse {
    #[serde(rename = "uploadStatus")]
    pub upload_status: &'static str,
}

#[derive(Debug, Serialize)]
pub struct RecognizeSongResponse {
    pub name: String,
    pub artist: String,
}
