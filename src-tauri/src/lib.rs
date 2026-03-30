mod addon;
mod api;
mod app;
pub mod cli;
mod error;
mod github;
mod settings;
mod update;

pub fn run_gui() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(api::register())
        .run(tauri::generate_context!())
        .expect("failed to run tauri application");
}
