use crate::commands::cancel::{get_or_create_cancellation_token, get_or_create_item_cancellation_token, OP_TYPE_BULK_VIDEO_CREATION};
use crate::domain::{
    BulkVideoRequest, BulkVideoResult, ImageProcessor, VideoCreator, VideoRequest,
};
use crate::infrastructure::{ImageProcessorImpl, VideoCreatorImpl};
use crate::state::CancellationState;
use crate::use_cases::CreateVideoUseCase;
use crate::utils::{file, validate_user_path_str};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratePreviewArgs {
    image_path: String,
    width: u32,
    height: u32,
    #[serde(default)]
    image_style: Option<String>,
    #[serde(default)]
    fit_mode: Option<String>,
    #[serde(default)]
    background_path: Option<String>,
    #[serde(default)]
    background_url: Option<String>,
    #[serde(default)]
    background_predefined_id: Option<String>,
}

#[tauri::command]
pub async fn create_video(
    request: VideoRequest,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    tracing::info!(
        "Video creation command received: {} audio file(s), output: {}",
        request.audio_paths.len(),
        crate::infrastructure::logging::redact_sensitive(&request.output_path)
    );
    // Validate paths to prevent path traversal attacks (defense in depth)
    if let Some(ref image_path) = request.image_path {
        validate_user_path_str(image_path)?;
    }
    for audio_path in &request.audio_paths {
        validate_user_path_str(audio_path)?;
    }
    validate_user_path_str(&request.output_path)?;
    let image_processor: Box<dyn ImageProcessor> = Box::new(ImageProcessorImpl);
    let video_creator: Box<dyn VideoCreator> = Box::new(VideoCreatorImpl);

    let use_case = CreateVideoUseCase::new(image_processor, video_creator);
    // Single video creation doesn't need cancellation token (quick operation)
    use_case
        .execute_with_progress(request, app_handle, None)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_bulk_videos(
    request: BulkVideoRequest,
    app_handle: tauri::AppHandle,
    cancellation_state: tauri::State<'_, CancellationState>,
) -> Result<BulkVideoResult, String> {
    let video_count = if let Some(ref inputs) = request.image_inputs {
        if inputs.is_empty() {
            0
        } else {
            inputs.len().min(request.audio_paths.len())
        }
    } else if let Some(ref paths) = request.image_paths {
        paths.len().min(request.audio_paths.len())
    } else {
        0
    };
    tracing::info!(
        "Bulk video creation command received: {} videos to create",
        video_count
    );
    
    // Validate file count limit
    if request.audio_paths.len() > file::MAX_FILES_PER_OPERATION {
        return Err(format!(
            "Cannot create more than {} videos at once. Please select fewer audio files.",
            file::MAX_FILES_PER_OPERATION
        ));
    }
    
    // Validate paths to prevent path traversal attacks (defense in depth)
    if let Some(ref inputs) = request.image_inputs {
        for inp in inputs {
            if let Some(ref p) = inp.image_path {
                validate_user_path_str(p)?;
            }
            if let Some(ref p) = inp.background_path {
                validate_user_path_str(p)?;
            }
        }
    } else if let Some(ref paths) = request.image_paths {
        for image_path in paths {
            validate_user_path_str(image_path)?;
        }
    }
    for audio_path in &request.audio_paths {
        validate_user_path_str(audio_path)?;
    }
    validate_user_path_str(&request.output_path)?;

    // Get or create cancellation token for this operation
    let cancel_token = get_or_create_cancellation_token(OP_TYPE_BULK_VIDEO_CREATION, &cancellation_state).await;

    // Create per-item cancellation tokens
    let mut item_tokens = Vec::new();
    for i in 0..video_count {
        let item_id = i.to_string();
        let item_token = get_or_create_item_cancellation_token(
            OP_TYPE_BULK_VIDEO_CREATION,
            &item_id,
            &cancellation_state,
        ).await;
        item_tokens.push(item_token);
    }

    let image_processor: Box<dyn ImageProcessor> = Box::new(ImageProcessorImpl);
    let video_creator: Box<dyn VideoCreator> = Box::new(VideoCreatorImpl);

    let use_case = CreateVideoUseCase::new(image_processor, video_creator);
    let result = use_case
        .execute_bulk_with_progress(request, app_handle, Some(cancel_token), Some(item_tokens), None)
        .await
        .map_err(|e| e.to_string());

    // Clean up cancellation tokens when done
    let mut state = cancellation_state.lock().await;
    state.remove(OP_TYPE_BULK_VIDEO_CREATION);
    for i in 0..video_count {
        let item_key = format!("{}:{}", OP_TYPE_BULK_VIDEO_CREATION, i);
        state.remove(&item_key);
    }

    result
}

#[tauri::command]
pub async fn generate_image_preview(
    app_handle: tauri::AppHandle,
    args: GeneratePreviewArgs,
) -> Result<String, String> {
    if !args.image_path.starts_with("http://") && !args.image_path.starts_with("https://") {
        validate_user_path_str(&args.image_path)?;
    }
    if let Some(ref p) = &args.background_path {
        validate_user_path_str(p)?;
    }
    let mut bg_path = args.background_path;
    if let Some(ref id) = args.background_predefined_id {
        if let Some(resolved) =
            crate::utils::predefined_backgrounds::resolve_predefined_background(&app_handle, id)
        {
            bg_path = Some(resolved);
        }
    }
    crate::infrastructure::thumbnail::generate_image_preview(
        args.image_path,
        args.width,
        args.height,
        args.image_style,
        args.fit_mode,
        bg_path,
        args.background_url,
    )
    .await
}

#[tauri::command]
pub async fn list_predefined_backgrounds(
    app_handle: tauri::AppHandle,
) -> Result<Vec<(String, String, String)>, String> {
    let list =
        crate::utils::predefined_backgrounds::list_predefined_backgrounds_with_paths(&app_handle);
    Ok(list
        .into_iter()
        .filter_map(|(id, label, path)| path.map(|p| (id, label, p)))
        .collect())
}

#[tauri::command]
pub async fn list_user_backgrounds(
    app_handle: tauri::AppHandle,
) -> Result<Vec<(String, String, String)>, String> {
    let list =
        crate::utils::predefined_backgrounds::list_user_backgrounds_with_paths(&app_handle);
    Ok(list
        .into_iter()
        .filter_map(|(id, label, path)| path.map(|p| (id, label, p)))
        .collect())
}

#[tauri::command]
pub async fn generate_image_thumbnail(image_path: String, size: u32) -> Result<String, String> {
    // Validate path to prevent path traversal attacks (defense in depth)
    validate_user_path_str(&image_path)?;

    crate::infrastructure::thumbnail::generate_image_thumbnail(image_path, size).await
}

#[tauri::command]
pub async fn generate_video_thumbnail(
    video_path: String,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    // Validate path to prevent path traversal attacks (defense in depth)
    validate_user_path_str(&video_path)?;

    crate::infrastructure::thumbnail::generate_video_thumbnail(video_path, Some(&app_handle)).await
}

#[tauri::command]
pub async fn save_predefined_background(
    app_handle: tauri::AppHandle,
    image_path: String,
) -> Result<String, String> {
    // Validate path to prevent path traversal attacks (defense in depth)
    validate_user_path_str(&image_path)?;

    crate::utils::predefined_backgrounds::save_predefined_background(&app_handle, &image_path)
}
