use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("unsupported media type: {0}")]
    UnsupportedMediaType(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("configuration error: {0}")]
    Config(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Db(#[from] mysql::Error),

    #[error(transparent)]
    HttpClient(#[from] reqwest::Error),

    #[error(transparent)]
    Join(#[from] tokio::task::JoinError),

    #[error("internal error: {0}")]
    Internal(String),
}

impl From<std::env::VarError> for AppError {
    fn from(value: std::env::VarError) -> Self {
        Self::Config(value.to_string())
    }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::Config(value.to_string())
    }
}

impl AppError {
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::UnsupportedMediaType(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::HttpClient(_) => StatusCode::BAD_GATEWAY,
            Self::Join(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ErrorBody {
            error: self.to_string(),
        })
    }
}
