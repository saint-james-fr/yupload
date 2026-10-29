use std::path::PathBuf;
use std::sync::Arc;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const STORE_PATH: &str = "yupload.store";

/// Get the store instance
pub fn get_store(
    app_handle: &AppHandle,
) -> Result<Arc<tauri_plugin_store::Store<tauri::Wry>>, String> {
    let path = PathBuf::from(STORE_PATH);
    app_handle
        .store(path)
        .map_err(|e| format!("Failed to get store: {}", e))
}

/// Get mutable store instance (returns Arc for consistency, but you can clone it)
pub fn get_store_mut(
    app_handle: &AppHandle,
) -> Result<Arc<tauri_plugin_store::Store<tauri::Wry>>, String> {
    let path = PathBuf::from(STORE_PATH);
    app_handle
        .store(path)
        .map_err(|e| format!("Failed to get store: {}", e))
}

/// Save store changes
pub fn save_store(store: &tauri_plugin_store::Store<tauri::Wry>) -> Result<(), String> {
    match store.save() {
        Ok(_) => {
            tracing::debug!("Store saved successfully");
            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to save store: {}", e);
            Err(format!("Failed to save store: {}", e))
        }
    }
}
