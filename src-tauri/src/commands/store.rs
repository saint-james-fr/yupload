use crate::infrastructure::store::{get_store, get_store_mut, save_store};
use serde::{Deserialize, Serialize};

/// Key under which the full preferences blob is stored in the plugin store.
const APP_PREFERENCES_KEY: &str = "app_preferences";

/// Description template block for upload descriptions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescriptionTemplateBlock {
    pub id: String,
    /// "tag" or "text"
    #[serde(rename = "type")]
    pub kind: String,
    pub enabled: bool,
    #[serde(rename = "tagKey", alias = "tag_key")]
    pub tag_key: Option<String>,
    pub content: Option<String>,
}

/// Structured application preferences persisted in the store.
///
/// All fields are optional so we can evolve this over time without breaking
/// existing data; the frontend applies sensible defaults when a field is `None`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppPreferences {
    /// Legacy single output path (maintained for backward compatibility).
    pub preferred_output_path: Option<String>,

    /// Enhanced output paths - separate for audio and video
    pub preferred_audio_output_path: Option<String>,
    pub preferred_video_output_path: Option<String>,

    /// Predefined input paths for file selection
    pub preferred_audio_input_path: Option<String>,
    pub preferred_image_input_path: Option<String>,

    // Creation defaults
    pub default_normalize_for_creation: Option<bool>,
    pub default_high_pass_when_normalizing: Option<bool>,
    pub default_image_style: Option<String>,
    pub default_fit_mode: Option<String>,
    pub default_background_predefined_id: Option<String>,

    // Upload defaults
    pub default_privacy_status: Option<String>,
    pub default_category_id: Option<String>,
    pub default_playlist_id: Option<String>,

    /// Notify via desktop notification when uploads complete.
    pub notify_on_upload_complete: Option<bool>,

    /// Standalone audio normalization output behaviour.
    /// When true or None: add "-normalized" suffix to filenames (recommended).
    /// When false: overwrite originals in the chosen output folder.
    pub normalize_audio_add_suffix: Option<bool>,

    // Last-used paths per flow
    pub last_creation_output: Option<String>,
    pub last_normalize_output: Option<String>,
    pub last_export_output: Option<String>,
    pub last_upload_source: Option<String>,

    // Description template
    pub description_template_blocks: Option<Vec<DescriptionTemplateBlock>>,

    // Unified upload defaults
    pub default_save_videos_locally: Option<bool>,
    pub default_upload_to_youtube: Option<bool>,
}

fn load_app_preferences(app_handle: &tauri::AppHandle) -> Result<AppPreferences, String> {
    let store = get_store(app_handle)?;

    // Try new structured preferences blob first
    let mut prefs = if let Some(value) = store.get(APP_PREFERENCES_KEY) {
        serde_json::from_value::<AppPreferences>(value.clone())
            .map_err(|e| format!("Failed to parse app preferences: {}", e))?
    } else {
        AppPreferences::default()
    };

    // Backwards compatibility: if the old standalone preferred_output_path key
    // exists and the structured field is empty, migrate it into the struct.
    if prefs.preferred_output_path.is_none() {
        if let Some(legacy) = store.get("preferred_output_path") {
            if let Ok(path_str) = serde_json::from_value::<String>(legacy.clone()) {
                prefs.preferred_output_path = Some(path_str);
            }
        }
    }

    // Migration: if new separate output paths are not set but legacy path exists,
    // migrate the legacy path to both audio and video output paths
    if prefs.preferred_audio_output_path.is_none() && prefs.preferred_video_output_path.is_none() {
        if let Some(legacy_path) = &prefs.preferred_output_path {
            prefs.preferred_audio_output_path = Some(legacy_path.clone());
            prefs.preferred_video_output_path = Some(legacy_path.clone());
            tracing::debug!("Migrated legacy preferred_output_path to separate audio/video paths");
        }
    }

    // Backwards compatibility: migrate legacy notify_on_upload_complete key.
    if prefs.notify_on_upload_complete.is_none() {
        if let Some(legacy) = store.get(NOTIFY_ON_UPLOAD_COMPLETE_KEY) {
            if let Ok(enabled) = serde_json::from_value::<bool>(legacy.clone()) {
                prefs.notify_on_upload_complete = Some(enabled);
            }
        }
    }

    Ok(prefs)
}

fn save_app_preferences(app_handle: &tauri::AppHandle, prefs: &AppPreferences) -> Result<(), String> {
    let store = get_store_mut(app_handle)?;
    let value = serde_json::to_value(prefs)
        .map_err(|e| {
            tracing::error!("Failed to serialize app preferences: {}", e);
            format!("Failed to serialize app preferences: {}", e)
        })?;
    store.set(APP_PREFERENCES_KEY.to_string(), value);
    save_store(&store)?;
    tracing::debug!("App preferences saved successfully");
    Ok(())
}

#[tauri::command]
pub async fn get_app_preferences(app_handle: tauri::AppHandle) -> Result<AppPreferences, String> {
    load_app_preferences(&app_handle)
}

#[tauri::command]
pub async fn set_app_preferences(
    prefs: AppPreferences,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    tracing::debug!("Setting app preferences");
    save_app_preferences(&app_handle, &prefs)
}

#[tauri::command]
pub async fn get_preferred_output_path(
    app_handle: tauri::AppHandle,
) -> Result<Option<String>, String> {
    let prefs = load_app_preferences(&app_handle)?;
    Ok(prefs.preferred_output_path)
}

#[tauri::command]
pub async fn set_preferred_output_path(
    output_path: Option<String>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    tracing::debug!("Setting preferred output path: {:?}", output_path);
    let mut prefs = load_app_preferences(&app_handle)?;
    prefs.preferred_output_path = output_path;
    save_app_preferences(&app_handle, &prefs)
}

const NOTIFY_ON_UPLOAD_COMPLETE_KEY: &str = "notify_on_upload_complete";

#[tauri::command]
pub async fn get_notify_on_upload_complete(app_handle: tauri::AppHandle) -> Result<bool, String> {
    let prefs = load_app_preferences(&app_handle)?;
    Ok(prefs.notify_on_upload_complete.unwrap_or(false))
}

#[tauri::command]
pub async fn set_notify_on_upload_complete(
    enabled: bool,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    tracing::debug!("Setting notify on upload complete: {}", enabled);
    let mut prefs = load_app_preferences(&app_handle)?;
    prefs.notify_on_upload_complete = Some(enabled);
    save_app_preferences(&app_handle, &prefs)
}

#[tauri::command]
pub async fn save_upload_history(
    uploads: Vec<serde_json::Value>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    tracing::debug!("Saving upload history with {} entries", uploads.len());
    
    let mut store = get_store_mut(&app_handle)?;

    let uploads_value = serde_json::to_value(&uploads)
        .map_err(|e| {
            tracing::error!("Failed to serialize upload history: {}", e);
            format!("Failed to serialize uploads: {}", e)
        })?;
    store.set("upload_history".to_string(), uploads_value);

    save_store(&mut store)?;

    tracing::info!("Upload history saved successfully with {} entries", uploads.len());
    Ok(())
}

#[tauri::command]
pub async fn get_upload_history(
    app_handle: tauri::AppHandle,
) -> Result<Vec<serde_json::Value>, String> {
    let store = get_store(&app_handle)?;

    if let Some(value) = store.get("upload_history") {
        let uploads: Vec<serde_json::Value> = serde_json::from_value(value.clone())
            .map_err(|e| format!("Failed to parse upload history: {}", e))?;
        Ok(uploads)
    } else {
        Ok(Vec::new())
    }
}

/// Keys in the persistent store that should be cleared when user runs "Clear all data"
const STORE_KEYS_TO_CLEAR: &[&str] = &[
    "preferred_output_path", // legacy key (kept for backward compatibility)
    APP_PREFERENCES_KEY,
    "notify_on_upload_complete",
    "upload_history",
    "youtube_token",
    "discogs_token",
];

/// Temp file prefixes used by the app (must match lib.rs cleanup and infrastructure usage)
const TEMP_PREFIXES: &[&str] = &[
    "yupload_",
    "processed_image",
    "thumb_",
    "video_thumb_",
    "normalized_",
];

#[tauri::command]
pub async fn clear_all_data(app_handle: tauri::AppHandle) -> Result<(), String> {
    use crate::utils::cleanup_old_temp_files;

    tracing::info!("Starting data deletion - clearing {} store keys and {} temp file prefixes", 
                   STORE_KEYS_TO_CLEAR.len(), TEMP_PREFIXES.len());

    let mut store = get_store_mut(&app_handle)?;
    let mut deleted_keys = 0;
    for key in STORE_KEYS_TO_CLEAR {
        if store.has(key.to_string()) {
            store.delete(key.to_string());
            deleted_keys += 1;
            tracing::debug!("Deleted store key: {}", key);
        }
    }
    save_store(&mut store)?;
    tracing::debug!("Store saved after deleting {} keys", deleted_keys);

    let mut cleaned_prefixes = 0;
    for prefix in TEMP_PREFIXES {
        match cleanup_old_temp_files(prefix, 0) {
            Ok(_) => {
                cleaned_prefixes += 1;
                tracing::debug!("Cleaned temp files with prefix: {}", prefix);
            }
            Err(e) => {
                tracing::error!("Failed to clean temp files with prefix '{}': {}", prefix, e);
            }
        }
    }

    tracing::info!("Data deletion completed successfully - deleted {} store keys, cleaned {} temp file prefixes", 
                   deleted_keys, cleaned_prefixes);
    Ok(())
}
