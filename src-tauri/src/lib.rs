pub mod hardware;
pub mod inference;
pub mod ipc;
pub mod litellm;

use tracing_subscriber::EnvFilter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            ipc::commands::detect_hardware,
            ipc::commands::get_capability_score,
            ipc::commands::list_models,
            ipc::commands::health_check,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
