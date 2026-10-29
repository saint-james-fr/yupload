use crate::infrastructure::logging::{get_all_log_files, get_current_log_path};
use anyhow::Result;
use tauri::{AppHandle, Manager};

#[derive(serde::Serialize)]
pub struct LogFileInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub modified: String,
}

/// Get information about available log files
#[tauri::command]
pub async fn get_log_files(app: AppHandle) -> Result<Vec<LogFileInfo>, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let log_files = get_all_log_files(&app_data_dir)
        .map_err(|e| {
            tracing::error!("Failed to get log files: {}", e);
            format!("Failed to get log files: {}", e)
        })?;
    
    tracing::debug!("Found {} log files", log_files.len());

    let mut file_infos = Vec::new();
    for log_path in log_files {
        if let Ok(metadata) = std::fs::metadata(&log_path) {
            let modified = metadata
                .modified()
                .map(|time| {
                    let datetime: chrono::DateTime<chrono::Local> = time.into();
                    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                })
                .unwrap_or_else(|_| "Unknown".to_string());

            file_infos.push(LogFileInfo {
                path: log_path.to_string_lossy().to_string(),
                name: log_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                size: metadata.len(),
                modified,
            });
        }
    }

    Ok(file_infos)
}

/// Export all log files as a single text file
#[tauri::command]
pub async fn export_logs(app: AppHandle) -> Result<String, String> {
    use tauri_plugin_dialog::{DialogExt, FilePath};

    tracing::info!("Starting log export");

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let log_files = get_all_log_files(&app_data_dir)
        .map_err(|e| format!("Failed to get log files: {}", e))?;
    
    tracing::debug!("Found {} log files to export", log_files.len());

    // Generate default filename
    let default_filename = format!(
        "yupload-logs-{}.txt",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    );

    // Show save dialog
    let file_path = app
        .dialog()
        .file()
        .set_title("Save logs as...")
        .set_file_name(&default_filename)
        .add_filter("Text files", &["txt"])
        .add_filter("All files", &["*"])
        .blocking_save_file();

    let export_path = match file_path {
        Some(FilePath::Path(path)) => path,
        Some(FilePath::Url(_)) => {
            tracing::error!("Log export failed: URL paths not supported");
            return Err("URL paths not supported for log export".to_string());
        }
        None => {
            tracing::debug!("Log export cancelled by user");
            return Err("Export cancelled by user".to_string());
        }
    };

    // Generate combined logs content
    let mut combined_logs = String::new();
    combined_logs.push_str(&format!(
        "Yupload Logs Export - Generated: {}\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    ));
    combined_logs.push_str(&format!("Total log files: {}\n", log_files.len()));
    combined_logs.push_str("=".repeat(80).as_str());
    combined_logs.push('\n');

    let mut files_processed = 0;
    let mut files_failed = 0;
    for log_path in log_files {
        let file_name = log_path.file_name().unwrap_or_default().to_string_lossy();
        combined_logs.push_str(&format!("\n--- Log File: {} ---\n", file_name));

        match std::fs::read_to_string(&log_path) {
            Ok(content) => {
                combined_logs.push_str(&content);
                if !content.ends_with('\n') {
                    combined_logs.push('\n');
                }
                files_processed += 1;
            }
            Err(e) => {
                combined_logs.push_str(&format!("Error reading file: {}\n", e));
                files_failed += 1;
                tracing::error!("Failed to read log file '{}': {}", file_name, e);
            }
        }
    }

    // Write to selected file
    match std::fs::write(&export_path, combined_logs) {
        Ok(_) => {
            tracing::info!("Logs exported successfully to: {} ({} files processed, {} failed)", 
                          export_path.display(), files_processed, files_failed);
            Ok(export_path.to_string_lossy().to_string())
        }
        Err(e) => {
            tracing::error!("Failed to write export file to '{}': {}", export_path.display(), e);
            Err(format!("Failed to write export file: {}", e))
        }
    }
}

/// Get the current log file content (for debugging)
#[tauri::command]
pub async fn get_current_log_content(app: AppHandle) -> Result<String, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let current_log_path = get_current_log_path(&app_data_dir);

    tracing::debug!("Reading current log file: {}", current_log_path.display());

    if current_log_path.exists() {
        match std::fs::read_to_string(&current_log_path) {
            Ok(content) => {
                tracing::debug!("Successfully read current log file ({} bytes)", content.len());
                Ok(content)
            }
            Err(e) => {
                tracing::error!("Failed to read current log file '{}': {}", current_log_path.display(), e);
                Err(format!("Failed to read current log: {}", e))
            }
        }
    } else {
        tracing::debug!("Current log file does not exist: {}", current_log_path.display());
        Ok("No current log file found.".to_string())
    }
}

/// Get system information for debugging
#[tauri::command]
pub async fn get_system_info() -> Result<serde_json::Value, String> {
    let mut info = serde_json::Map::new();

    info.insert("os".to_string(), serde_json::Value::String(std::env::consts::OS.to_string()));
    info.insert("arch".to_string(), serde_json::Value::String(std::env::consts::ARCH.to_string()));
    info.insert("family".to_string(), serde_json::Value::String(std::env::consts::FAMILY.to_string()));
    info.insert("cpu_cores".to_string(), serde_json::Value::Number(serde_json::Number::from(num_cpus::get())));

    // Check FFmpeg availability using proper path resolution
    let ffmpeg_path = crate::utils::ffmpeg::get_ffmpeg_path();
    let ffmpeg_available = std::process::Command::new(&ffmpeg_path)
        .arg("-version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);
    info.insert("ffmpeg_available".to_string(), serde_json::Value::Bool(ffmpeg_available));
    info.insert("ffmpeg_path".to_string(), serde_json::Value::String(ffmpeg_path));

    Ok(serde_json::Value::Object(info))
}
