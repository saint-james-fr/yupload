use crate::commands::cancel::{get_or_create_cancellation_token, get_or_create_item_cancellation_token, is_cancelled, OP_TYPE_YOUTUBE_UPLOAD};
use crate::domain::youtube_models::*;
use crate::state::OAuthServerState;
use crate::state::{ensure_youtube_client, CancellationState, YouTubeClientState};
use crate::utils::{
    file, validate_user_path_str, validate_youtube_description, validate_youtube_tags,
    validate_youtube_title,
};
use tauri::Emitter;

#[tauri::command]
pub async fn authenticate_youtube(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, YouTubeClientState>,
    oauth_server: tauri::State<'_, OAuthServerState>,
) -> Result<(), String> {
    // Ensure client is initialized
    ensure_youtube_client(state.clone()).await?;

    // Get mutable access to client
    let mut client_guard = state.lock().await;
    let client = client_guard.as_mut().expect("Client should be initialized");

    match client.authenticate(app_handle.clone(), oauth_server.inner()).await {
        Ok(_) => {
            tracing::info!("YouTube authentication successful");
        }
        Err(e) => {
            tracing::error!("YouTube authentication failed: {}", e);
            return Err(format!("Failed to authenticate: {}", e));
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn check_youtube_auth(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, YouTubeClientState>,
) -> Result<bool, String> {
    tracing::debug!("Checking YouTube authentication status");
    // Ensure client is initialized
    ensure_youtube_client(state.clone()).await?;

    let client = state.lock().await;
    let client = client.as_ref().expect("Client should be initialized");

    match client.check_auth_status(&app_handle).await {
        Ok(is_authenticated) => {
            tracing::debug!("YouTube authentication status: {}", is_authenticated);
            Ok(is_authenticated)
        }
        Err(e) => {
            tracing::error!("Failed to check YouTube auth status: {}", e);
            Err(format!("Failed to check auth status: {}", e))
        }
    }
}

#[tauri::command]
pub async fn list_youtube_playlists(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, YouTubeClientState>,
) -> Result<Vec<YouTubePlaylist>, String> {
    ensure_youtube_client(state.clone()).await?;
    let client = state.lock().await;
    let client = client.as_ref().expect("Client should be initialized");
    client
        .list_playlists(&app_handle)
        .await
        .map_err(|e| format!("Failed to list playlists: {}", e))
}

#[tauri::command]
pub async fn upload_video_to_youtube(
    request: VideoUploadRequest,
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, YouTubeClientState>,
) -> Result<VideoUploadResult, String> {
    tracing::info!("YouTube upload command received: '{}'", request.title);
    // Validate inputs
    validate_youtube_title(&request.title)?;
    validate_youtube_description(&request.description)?;
    validate_youtube_tags(&request.tags)?;
    validate_user_path_str(&request.video_path)?;

    // Ensure client is initialized
    ensure_youtube_client(state.clone()).await?;

    let client = state.lock().await;
    let client = client.as_ref().expect("Client should be initialized");

    // Emit progress event
    let _ = app_handle.emit(
        "youtube-upload-progress",
        serde_json::json!({
            "video_path": request.video_path,
            "progress": crate::utils::progress::INITIAL,
            "message": "Starting upload..."
        }),
    );

    let result = client
        .upload_video(&request, &app_handle)
        .await
        .map_err(|e| format!("Failed to upload video: {}", e))?;

    let _ = app_handle.emit(
        "youtube-upload-progress",
        serde_json::json!({
            "video_path": request.video_path,
            "progress": crate::utils::progress::COMPLETE,
            "message": "Upload complete!",
            "video_id": result.video_id
        }),
    );

    Ok(result)
}

#[tauri::command]
pub async fn upload_bulk_videos_to_youtube(
    request: BulkVideoUploadRequest,
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, YouTubeClientState>,
    cancellation_state: tauri::State<'_, CancellationState>,
) -> Result<BulkVideoUploadResult, String> {
    // Validate file count limit
    if request.videos.len() > file::MAX_FILES_PER_OPERATION {
        return Err(format!(
            "Cannot upload more than {} videos at once. Please select fewer videos.",
            file::MAX_FILES_PER_OPERATION
        ));
    }

    // Validate inputs for all videos
    for video_request in &request.videos {
        validate_youtube_title(&video_request.title)?;
        validate_youtube_description(&video_request.description)?;
        validate_youtube_tags(&video_request.tags)?;
        validate_user_path_str(&video_request.video_path)?;
    }

    // Ensure client is initialized
    ensure_youtube_client(state.clone()).await?;

    // Get or create cancellation token for this operation
    let cancel_token = get_or_create_cancellation_token(OP_TYPE_YOUTUBE_UPLOAD, &cancellation_state).await;

    // Create per-item cancellation tokens
    let mut item_tokens = Vec::new();
    for i in 0..request.videos.len() {
        let item_id = i.to_string();
        let item_token = get_or_create_item_cancellation_token(
            OP_TYPE_YOUTUBE_UPLOAD,
            &item_id,
            &cancellation_state,
        ).await;
        item_tokens.push(item_token);
    }

    let client = state.lock().await;
    let client = client.as_ref().expect("Client should be initialized");

    let mut successful = Vec::new();
    let mut failed = Vec::new();

    // Process uploads sequentially for now to avoid lifetime issues
    // The main concurrency benefit comes from starting uploads as videos complete
    for (index, video_request) in request.videos.iter().enumerate() {
        // Check global cancellation before each upload
        if is_cancelled(&cancel_token) {
            tracing::info!("Cancellation detected, stopping YouTube upload at video {}/{}", index, request.videos.len());
            // Clean up cancellation tokens
            let mut state = cancellation_state.lock().await;
            state.remove(OP_TYPE_YOUTUBE_UPLOAD);
            for i in 0..request.videos.len() {
                let item_key = format!("{}:{}", OP_TYPE_YOUTUBE_UPLOAD, i);
                state.remove(&item_key);
            }
            return Err("Operation cancelled".to_string());
        }

        // Check per-item cancellation
        if let Some(item_token) = item_tokens.get(index) {
            if item_token.is_cancelled() {
                tracing::info!("Item {} cancelled, skipping", index);
                let _ = app_handle.emit(
                    "youtube-upload-complete",
                    serde_json::json!({
                        "index": index,
                        "success": false,
                        "error": "Cancelled"
                    }),
                );
                failed.push((video_request.video_path.clone(), "Cancelled".to_string()));
                continue;
            }
        }

        let _ = app_handle.emit(
            "youtube-upload-start",
            serde_json::json!({
                "index": index,
                "video_path": video_request.video_path,
                "title": video_request.title
            }),
        );

        match client.upload_video(video_request, &app_handle).await {
            Ok(result) => {
                let video_id = result.video_id.clone();
                successful.push(result);
                let _ = app_handle.emit(
                    "youtube-upload-complete",
                    serde_json::json!({
                        "index": index,
                        "success": true,
                        "video_id": video_id
                    }),
                );
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                failed.push((video_request.video_path.clone(), error_msg.clone()));
                let _ = app_handle.emit(
                    "youtube-upload-complete",
                    serde_json::json!({
                        "index": index,
                        "success": false,
                        "error": error_msg
                    }),
                );
            }
        }
    }

    // Clean up cancellation tokens when done
    let mut state = cancellation_state.lock().await;
    state.remove(OP_TYPE_YOUTUBE_UPLOAD);
    for i in 0..request.videos.len() {
        let item_key = format!("{}:{}", OP_TYPE_YOUTUBE_UPLOAD, i);
        state.remove(&item_key);
    }

    Ok(BulkVideoUploadResult { successful, failed })
}

#[tauri::command]
pub async fn disconnect_youtube(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, YouTubeClientState>,
) -> Result<(), String> {
    tracing::info!("YouTube disconnect command received");
    // Ensure client is initialized
    ensure_youtube_client(state.clone()).await?;

    let client = state.lock().await;
    let client = client.as_ref().expect("Client should be initialized");

    match client.disconnect(&app_handle).await {
        Ok(_) => {
            tracing::info!("YouTube disconnect command completed successfully");
            Ok(())
        }
        Err(e) => {
            tracing::error!("YouTube disconnect command failed: {}", e);
            Err(format!("Failed to disconnect: {}", e))
        }
    }
}

#[tauri::command]
pub async fn check_video_processing_status(
    video_id: String,
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, YouTubeClientState>,
) -> Result<crate::domain::youtube_models::VideoProcessingStatusResponse, String> {
    ensure_youtube_client(state.clone()).await?;
    let client = state.lock().await;
    let client = client.as_ref().expect("Client should be initialized");
    
    client
        .check_video_processing_status(&video_id, &app_handle)
        .await
        .map_err(|e| format!("Failed to check video processing status: {}", e))
}
