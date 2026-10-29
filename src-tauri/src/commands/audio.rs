use crate::commands::cancel::{get_or_create_cancellation_token, OP_TYPE_AUDIO_NORMALIZATION};
use crate::domain::traits::FileHandler;
use crate::infrastructure::audio_normalizer;
use crate::state::CancellationState;
use crate::utils::{calculate_optimal_concurrency, file, validate_user_path_str};
use tauri::Emitter;

#[tauri::command]
pub async fn normalize_audio_files_standalone(
    audio_paths: Vec<String>,
    output_path: String,
    high_pass_filter: Option<bool>,
    add_suffix: Option<bool>,
    app_handle: tauri::AppHandle,
    cancellation_state: tauri::State<'_, CancellationState>,
) -> Result<serde_json::Value, String> {
    // Validate file count limit
    if audio_paths.len() > file::MAX_FILES_PER_OPERATION {
        return Err(format!(
            "Cannot normalize more than {} audio files at once. Please select fewer files.",
            file::MAX_FILES_PER_OPERATION
        ));
    }
    
    // Validate paths to prevent path traversal attacks (defense in depth)
    for path in &audio_paths {
        validate_user_path_str(path)?;
    }
    validate_user_path_str(&output_path)?;

    // Get or create cancellation token for this operation
    let cancel_token = get_or_create_cancellation_token(OP_TYPE_AUDIO_NORMALIZATION, &cancellation_state).await;

    // Calculate optimal concurrency for parallel processing
    let (max_jobs, _) = calculate_optimal_concurrency(audio_paths.len(), Some(4));
    let high_pass_filter = high_pass_filter.unwrap_or(false);

    let _ = app_handle.emit(
        "audio-normalization-start",
        serde_json::json!({
            "total": audio_paths.len(),
            "message": format!("Starting normalization of {} audio file(s)...", audio_paths.len())
        }),
    );

    // Normalize files in parallel to output directory
    let result = audio_normalizer::normalize_audio_files_to_directory(
        audio_paths.clone(),
        &output_path,
        max_jobs,
        high_pass_filter,
        add_suffix.unwrap_or(true),
        Some(app_handle.clone()),
        Some(cancel_token),
    )
    .await
    .map_err(|e| format!("Failed to normalize audio files: {}", e));

    // Clean up cancellation token when done
    let mut state = cancellation_state.lock().await;
    state.remove(OP_TYPE_AUDIO_NORMALIZATION);

    let (successful_results, failed_results) = result?;

    // Build response
    let successful: Vec<String> = successful_results
        .iter()
        .map(|(_, normalized)| normalized.clone())
        .collect();

    let failed = failed_results;

    let _ = app_handle.emit(
        "audio-normalization-complete",
        serde_json::json!({
            "total": audio_paths.len(),
            "message": format!("Successfully normalized {} audio file(s)", successful.len())
        }),
    );

    Ok(serde_json::json!({
        "successful": successful,
        "failed": failed
    }))
}

#[tauri::command]
pub async fn normalize_audio_files(
    audio_paths: Vec<String>,
    high_pass_filter: Option<bool>,
    app_handle: tauri::AppHandle,
) -> Result<std::collections::HashMap<String, String>, String> {
    // Validate file count limit
    if audio_paths.len() > file::MAX_FILES_PER_OPERATION {
        return Err(format!(
            "Cannot normalize more than {} audio files at once. Please select fewer files.",
            file::MAX_FILES_PER_OPERATION
        ));
    }
    
    // Validate paths to prevent path traversal attacks (defense in depth)
    for path in &audio_paths {
        validate_user_path_str(path)?;
    }

    // Calculate optimal concurrency for parallel processing
    let (max_jobs, _) = calculate_optimal_concurrency(audio_paths.len(), Some(4));
    let high_pass_filter = high_pass_filter.unwrap_or(false);

    let _ = app_handle.emit(
        "audio-normalization-start",
        serde_json::json!({
            "total": audio_paths.len(),
            "message": format!("Starting normalization of {} audio file(s)...", audio_paths.len())
        }),
    );

    // Normalize files in parallel (no cancellation token for internal use)
    let results = audio_normalizer::normalize_audio_files_parallel(
        audio_paths.clone(),
        max_jobs,
        high_pass_filter,
        Some(app_handle.clone()),
        None, // No cancellation for internal use
    )
    .await
    .map_err(|e| format!("Failed to normalize audio files: {}", e))?;

    // Convert to HashMap for easier lookup
    let mut normalized_map = std::collections::HashMap::new();
    for (original, normalized) in &results {
        normalized_map.insert(original.clone(), normalized.clone());
    }

    let _ = app_handle.emit(
        "audio-normalization-complete",
        serde_json::json!({
            "total": results.len(),
            "message": format!("Successfully normalized {} audio file(s)", results.len())
        }),
    );

    Ok(normalized_map)
}

#[tauri::command]
pub async fn get_audio_title(audio_path: String) -> Result<String, String> {
    // Validate path to prevent path traversal attacks (defense in depth)
    validate_user_path_str(&audio_path)?;

    use crate::infrastructure::FileHandlerImpl;
    let handler = FileHandlerImpl;
    handler
        .get_audio_title(&audio_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_audio_metadata(audio_path: String) -> Result<crate::infrastructure::audio_metadata::AudioMetadata, String> {
    // Validate path to prevent path traversal attacks (defense in depth)
    validate_user_path_str(&audio_path)?;

    use crate::infrastructure::FileHandlerImpl;
    let handler = FileHandlerImpl;
    handler
        .get_audio_metadata(&audio_path)
        .map_err(|e| e.to_string())
}
