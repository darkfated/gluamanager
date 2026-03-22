use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};

const SETTINGS_FILE_NAME: &str = "settings.json";
const DEFAULT_SOURCE_URL: &str =
    "https://raw.githubusercontent.com/darkfated/gluasource/refs/heads/master/source.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default)]
    pub root_path: String,
    #[serde(default = "default_sources")]
    pub sources: Vec<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            root_path: String::new(),
            sources: default_sources(),
        }
    }
}

pub fn load(app: &AppHandle) -> AppResult<AppSettings> {
    let path = settings_path(app)?;
    if !path.exists() {
        return Ok(AppSettings::default());
    }

    let content = fs::read_to_string(path)?;
    let settings: AppSettings = serde_json::from_str(&content)?;
    Ok(normalize_settings(settings))
}

pub fn save(app: &AppHandle, settings: &AppSettings) -> AppResult<()> {
    let path = settings_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let normalized = normalize_settings(settings.clone());
    let content = serde_json::to_vec_pretty(&normalized)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn save_root_path(app: &AppHandle, root_path: &str) -> AppResult<AppSettings> {
    let mut settings = load(app)?;
    settings.root_path = root_path.trim().to_string();
    save(app, &settings)?;
    Ok(settings)
}

pub fn save_sources(app: &AppHandle, sources: &[String]) -> AppResult<AppSettings> {
    let mut settings = load(app)?;
    settings.sources = normalize_sources(sources.to_vec());
    save(app, &settings)?;
    Ok(settings)
}

fn settings_path(app: &AppHandle) -> AppResult<PathBuf> {
    let dir = app.path().app_config_dir().map_err(|error| {
        AppError::Unexpected(format!(
            "Не удалось определить директорию настроек: {error}"
        ))
    })?;
    Ok(dir.join(SETTINGS_FILE_NAME))
}

fn default_sources() -> Vec<String> {
    vec![DEFAULT_SOURCE_URL.to_string()]
}

fn normalize_settings(settings: AppSettings) -> AppSettings {
    AppSettings {
        root_path: settings.root_path.trim().to_string(),
        sources: normalize_sources(settings.sources),
    }
}

fn normalize_sources(items: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut normalized = Vec::new();

    for item in items {
        let value = item.trim();
        if value.is_empty() {
            continue;
        }

        if seen.insert(value.to_string()) {
            normalized.push(value.to_string());
        }
    }

    normalized
}
