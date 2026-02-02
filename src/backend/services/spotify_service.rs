use crate::backend::error::AppError;
use log::{debug, info};
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::Deserialize;

#[derive(Clone)]
pub struct SpotifyService {
    http: reqwest::Client,
    client_id: String,
    client_secret: String,
}

impl SpotifyService {
    pub fn new_from_env() -> Result<Self, AppError> {
        debug!("Initializing SpotifyService from env");
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

        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );

        debug!("Requesting Spotify access token");
        let resp = self
            .http
            .post("https://accounts.spotify.com/api/token")
            .headers(headers)
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .body("grant_type=client_credentials")
            .send()
            .await?
            .error_for_status()?;

        let data: TokenResponse = resp.json().await?;
        debug!("Spotify access token acquired");
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
        info!("Fetching Spotify track info: {}", track_id);
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
            .first()
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
