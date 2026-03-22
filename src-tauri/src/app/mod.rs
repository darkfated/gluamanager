use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;
use url::Url;

use crate::error::AppResult;

const UPDATE_ENDPOINT: &str =
    "https://github.com/darkfated/gluamanager/releases/latest/download/latest.json";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdateView {
    pub version: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfoView {
    pub version: String,
    pub updater_enabled: bool,
    pub update: Option<AppUpdateView>,
}

pub async fn load_info(app: &AppHandle) -> AppResult<AppInfoView> {
    let version = app.package_info().version.to_string();
    let pubkey = updater_pubkey();
    if pubkey.is_empty() {
        return Ok(AppInfoView {
            version,
            updater_enabled: false,
            update: None,
        });
    }

    let updater = app
        .updater_builder()
        .pubkey(pubkey)
        .endpoints(vec![
            Url::parse(UPDATE_ENDPOINT).expect("valid updater endpoint")
        ])?
        .build()?;
    let update = updater.check().await?;

    Ok(AppInfoView {
        version,
        updater_enabled: true,
        update: update.map(|item| AppUpdateView {
            version: item.version.to_string(),
            notes: item.body.clone(),
        }),
    })
}

pub async fn install_update(app: &AppHandle) -> AppResult<bool> {
    let pubkey = updater_pubkey();
    if pubkey.is_empty() {
        return Ok(false);
    }

    let updater = app
        .updater_builder()
        .pubkey(pubkey)
        .endpoints(vec![
            Url::parse(UPDATE_ENDPOINT).expect("valid updater endpoint")
        ])?
        .build()?;

    if let Some(update) = updater.check().await? {
        update.download_and_install(|_, _| {}, || {}).await?;
        #[cfg(target_os = "windows")]
        {
            return Ok(true);
        }

        #[cfg(not(target_os = "windows"))]
        {
            app.restart();
        }
    }

    Ok(false)
}

fn updater_pubkey() -> String {
    option_env!("GLUAMANAGER_UPDATER_PUBKEY")
        .unwrap_or("")
        .trim()
        .to_string()
}
