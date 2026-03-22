use std::path::PathBuf;

use tauri::generate_handler;
use tauri::AppHandle;

use crate::addon::{AddonView, AvailableAddonView, InstallPlanView};
use crate::app::AppInfoView;
use crate::error::AppResult;
use crate::settings::AppSettings;
use crate::update;

#[tauri::command]
async fn load_app_info(app: AppHandle) -> Result<AppInfoView, String> {
    handle(crate::app::load_info(&app).await)
}

#[tauri::command]
async fn install_app_update(app: AppHandle) -> Result<bool, String> {
    handle(crate::app::install_update(&app).await)
}

#[tauri::command]
async fn load_settings(app: AppHandle) -> Result<AppSettings, String> {
    handle(crate::settings::load(&app))
}

#[tauri::command]
async fn save_root_path(app: AppHandle, root_path: String) -> Result<AppSettings, String> {
    handle(crate::settings::save_root_path(&app, &root_path))
}

#[tauri::command]
async fn save_sources(app: AppHandle, sources: Vec<String>) -> Result<AppSettings, String> {
    handle(crate::settings::save_sources(&app, &sources))
}

#[tauri::command]
async fn scan_root(root_path: String) -> Result<Vec<AddonView>, String> {
    handle(update::scan_root(&PathBuf::from(root_path)).await)
}

#[tauri::command]
async fn check_updates(root_path: String) -> Result<Vec<AddonView>, String> {
    handle(update::check_updates(&PathBuf::from(root_path)).await)
}

#[tauri::command]
async fn check_addon(addon_path: String) -> Result<AddonView, String> {
    handle(update::check_addon(&PathBuf::from(addon_path)).await)
}

#[tauri::command]
async fn load_addon_readme(addon_path: String) -> Result<Option<String>, String> {
    handle(update::load_addon_readme(&PathBuf::from(addon_path)).await)
}

#[tauri::command]
async fn load_available_addon(
    root_path: String,
    repository_url: String,
    branch: String,
) -> Result<AvailableAddonView, String> {
    handle(update::load_available_addon(&PathBuf::from(root_path), &repository_url, &branch).await)
}

#[tauri::command]
async fn load_available_addon_readme(
    repository_url: String,
    branch: String,
) -> Result<Option<String>, String> {
    handle(update::load_available_addon_readme(&repository_url, &branch).await)
}

#[tauri::command]
async fn update_addon(addon_path: String) -> Result<AddonView, String> {
    handle(update::update_addon(&PathBuf::from(addon_path)).await)
}

#[tauri::command]
async fn list_available_addons(
    root_path: String,
    source_urls: Vec<String>,
) -> Result<Vec<AvailableAddonView>, String> {
    handle(update::list_available_addons(&PathBuf::from(root_path), &source_urls).await)
}

#[tauri::command]
async fn install_addon(
    root_path: String,
    repository_url: String,
    branch: String,
) -> Result<AddonView, String> {
    handle(update::install_addon(&PathBuf::from(root_path), &repository_url, &branch).await)
}

#[tauri::command]
async fn preview_install(
    root_path: String,
    repository_url: String,
    branch: String,
) -> Result<InstallPlanView, String> {
    handle(update::preview_install(&PathBuf::from(root_path), &repository_url, &branch).await)
}

fn handle<T>(result: AppResult<T>) -> Result<T, String> {
    result.map_err(|error| error.to_string())
}

pub fn register() -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool {
    generate_handler![
        load_settings,
        load_app_info,
        install_app_update,
        save_root_path,
        save_sources,
        scan_root,
        check_updates,
        check_addon,
        load_addon_readme,
        load_available_addon,
        load_available_addon_readme,
        update_addon,
        list_available_addons,
        install_addon,
        preview_install
    ]
}
