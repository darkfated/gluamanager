use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Archive error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Updater error: {0}")]
    Updater(#[from] tauri_plugin_updater::Error),
    #[error("{0}")]
    InvalidGithubUrl(String),
    #[error("{0}")]
    Unexpected(String),
}
