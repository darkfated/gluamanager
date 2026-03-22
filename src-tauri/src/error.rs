use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Не удалось прочитать файл: {0}")]
    Io(#[from] std::io::Error),
    #[error("Некорректный JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Ошибка HTTP: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Ошибка архива: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Ошибка автообновления: {0}")]
    Updater(#[from] tauri_plugin_updater::Error),
    #[error("{0}")]
    InvalidGithubUrl(String),
    #[error("{0}")]
    Unexpected(String),
}
