use crate::backend::error::AppError;
use base64::Engine;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::Deserialize;

#[derive(Clone)]
pub struct SpotifyService {
    http: reqwest::Client,
    client_id: String,
    client_secret: String,
}

impl SpotifyService {
    pub fn new_from_env() -> Result<Self, AppError> {
        Ok(Self {
            http: reqwest::Client::new(),
            client_id: std::env::var("CLIENT_ID")?,
            client_secret: std::env::var("CLIENT_SECRET")?,
        })
    }

    async fn token(&self) -> Result<String, AppError> {
        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
        }

        let auth = base64::engine::general_purpose::STANDARD
            .encode(format!("{}:{}", self.client_id, self.client_secret));

        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Basic {}", auth))
                .map_err(|e| AppError::internal(format!("invalid auth header: {e}")))?,
        );

        let resp = self
            .http
            .post("https://accounts.spotify.com/api/token")
            .headers(headers)
            .body("grant_type=client_credentials")
            .send()
            .await?
            .error_for_status()?;

        let data: TokenResponse = resp.json().await?;
        Ok(data.access_token)
    }

    pub async fn track_info(&self, track_id: &str) -> Result<TrackInfo, AppError> {
        #[derive(Deserialize)]
        struct Artist {
            name: String,
        }

        #[derive(Deserialize)]
        struct TrackResponse {
            name: String,
            artists: Vec<Artist>,
        }

        let token = self.token().await?;
        let resp = self
            .http
            .get(format!("https://api.spotify.com/v1/tracks/{}", track_id))
            .bearer_auth(token)
            .send()
            .await?
            .error_for_status()?;

        let track: TrackResponse = resp.json().await?;
        let artist = track
            .artists
            .get(0)
            .ok_or_else(|| AppError::BadRequest("track has no artists".to_string()))?
            .name
            .clone();

        Ok(TrackInfo {
            name: track.name,
            artist,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TrackInfo {
    pub name: String,
    pub artist: String,
}
