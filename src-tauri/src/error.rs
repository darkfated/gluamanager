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
    Unexpected(String),
}

impl AppError {
    pub fn user_message(&self) -> String {
        match self {
            Self::Io(error) => match error.kind() {
                std::io::ErrorKind::NotFound => "Not found.".into(),
                std::io::ErrorKind::PermissionDenied => "No access to the file or folder.".into(),
                std::io::ErrorKind::AlreadyExists => "That file or folder already exists.".into(),
                std::io::ErrorKind::InvalidData => {
                    "The file is damaged or has an invalid format.".into()
                }
                _ => format!("File error: {error}"),
            },
            Self::Json(_) => "The .addon file or settings are damaged.".into(),
            Self::Http(_) => "Could not fetch data from the link.".into(),
            Self::Zip(_) => "Could not unpack the archive.".into(),
            Self::Updater(_) => "Could not check for updates.".into(),
            Self::Unexpected(message) => message.clone(),
        }
    }
}
