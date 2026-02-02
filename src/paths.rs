use std::path::PathBuf;

pub fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn audio_dir() -> PathBuf {
    project_root().join("audio")
}

pub fn log_dir() -> PathBuf {
    project_root().join("log")
}
