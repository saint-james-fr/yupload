use crate::domain::traits::FileHandler;
use crate::domain::FolderContents;
use crate::infrastructure::FileHandlerImpl;

#[tauri::command]
pub async fn select_image() -> Result<Option<String>, String> {
    // File dialogs are handled in the frontend using @tauri-apps/plugin-dialog
    Ok(None)
}

#[tauri::command]
pub async fn select_audio_files() -> Result<Vec<String>, String> {
    // File dialogs are handled in the frontend using @tauri-apps/plugin-dialog
    Ok(Vec::new())
}

#[tauri::command]
pub async fn select_folder() -> Result<Option<String>, String> {
    // File dialogs are handled in the frontend using @tauri-apps/plugin-dialog
    Ok(None)
}

#[tauri::command]
pub async fn select_output_folder() -> Result<Option<String>, String> {
    // File dialogs are handled in the frontend using @tauri-apps/plugin-dialog
    Ok(None)
}

#[tauri::command]
pub async fn list_folder_contents(folder_path: String) -> Result<FolderContents, String> {
    // Validate path to prevent path traversal attacks (defense in depth)
    crate::utils::validate_user_path_str(&folder_path)?;

    let handler = FileHandlerImpl;
    match handler.list_folder_contents(&folder_path) {
        Ok(contents) => {
            tracing::debug!("Listed folder contents: {} images, {} audios, {} videos", 
                contents.images.len(), contents.audios.len(), contents.videos.len());
            Ok(contents)
        }
        Err(e) => {
            tracing::error!("Failed to list folder contents for '{}': {}", 
                crate::infrastructure::logging::redact_sensitive(&folder_path), e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn fetch_image_as_data_url(url: String) -> Result<String, String> {
    let url = url.clone();
    tokio::task::spawn_blocking(move || crate::utils::fetch_image_as_data_url(&url))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_temp_dir() -> Result<String, String> {
    let temp_dir = std::env::temp_dir();
    let youtube_temp_dir = temp_dir.join("yupload_uploads");
    
    // Create directory if it doesn't exist
    if !youtube_temp_dir.exists() {
        std::fs::create_dir_all(&youtube_temp_dir)
            .map_err(|e| format!("Failed to create temp directory: {}", e))?;
    }
    
    Ok(youtube_temp_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn cleanup_temp_videos(paths: Vec<String>) -> Result<(), String> {
    tracing::info!("Starting temp video cleanup for {} files", paths.len());
    
    let mut cleaned = 0;
    let mut skipped = 0;
    let mut errors = Vec::new();
    
    for path in paths {
        // Validate that path is in temp directory to prevent accidental deletion
        let temp_dir = std::env::temp_dir();
        let youtube_temp_dir = temp_dir.join("yupload_uploads");
        let path_buf = std::path::PathBuf::from(&path);
        
        // Only delete if path is within our temp directory
        if path_buf.starts_with(&youtube_temp_dir) || path_buf.starts_with(&temp_dir) {
            match std::fs::remove_file(&path) {
                Ok(_) => {
                    cleaned += 1;
                    tracing::debug!("Deleted temp video file: {}", path);
                }
                Err(e) => {
                    let error_msg = format!("Failed to delete {}: {}", path, e);
                    errors.push(error_msg.clone());
                    tracing::error!("{}", error_msg);
                }
            }
        } else {
            skipped += 1;
            let error_msg = format!("Path {} is not in temp directory, skipping", path);
            errors.push(error_msg.clone());
            tracing::warn!("{}", error_msg);
        }
    }
    
    if !errors.is_empty() {
        tracing::error!("Temp video cleanup completed with errors: {} cleaned, {} skipped, {} errors", 
                       cleaned, skipped, errors.len());
        Err(format!("Cleaned {} files, but encountered {} errors: {}", cleaned, errors.len(), errors.join("; ")))
    } else {
        tracing::info!("Temp video cleanup completed successfully: {} files deleted", cleaned);
        Ok(())
    }
}
