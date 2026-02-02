use thiserror::Error;

#[derive(Debug, Error)]
pub enum FingerprintError {
    #[error("audio is empty")]
    EmptyAudio,

    #[error("wav read error: {0}")]
    Wav(#[from] hound::Error),

    #[error("resampler construction error: {0}")]
    ResamplerConstruction(#[from] rubato::ResamplerConstructionError),

    #[error("resampling error: {0}")]
    Resample(#[from] rubato::ResampleError),

    #[error("invalid audio: {0}")]
    InvalidAudio(&'static str),
}
