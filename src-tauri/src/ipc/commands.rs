use crate::hardware::{
    capability::{compute_capability, CapabilityScore},
    profiler::{detect_hardware_default, HardwareProfile},
};

#[tauri::command]
pub fn detect_hardware() -> HardwareProfile {
    detect_hardware_default()
}

#[tauri::command]
pub fn get_capability_score() -> CapabilityScore {
    let profile = detect_hardware_default();
    compute_capability(&profile)
}

#[tauri::command]
pub fn list_models(app: tauri::AppHandle) -> Vec<crate::inference::ModelInfo> {
    use tauri::Manager;
    let models_dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("models");
    let manager = crate::inference::ModelManager::new(models_dir);
    manager.list_models().unwrap_or_default()
}

#[tauri::command]
pub fn health_check() -> String {
    "ok".to_string()
}
