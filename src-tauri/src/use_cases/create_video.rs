use crate::commands::cancel::is_cancelled;
use crate::domain::{
    BulkVideoRequest, BulkVideoResult, ImageInput, ImageProcessor, ImageStyle, ProcessOptions,
    VideoCreator, VideoRequest,
};
use crate::infrastructure::audio_metadata::AudioMetadata;
use crate::infrastructure::audio_normalizer;
use crate::infrastructure::{ImageProcessorImpl, VideoCreatorImpl};
use crate::utils::predefined_backgrounds;
use crate::utils::{calculate_optimal_concurrency, progress, timeout, TempFile};
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

pub struct CreateVideoUseCase {
    #[allow(dead_code)]
    image_processor: Box<dyn ImageProcessor>,
    #[allow(dead_code)]
    video_creator: Box<dyn VideoCreator>,
}

impl CreateVideoUseCase {
    pub fn new(
        image_processor: Box<dyn ImageProcessor>,
        video_creator: Box<dyn VideoCreator>,
    ) -> Self {
        Self {
            image_processor,
            video_creator,
        }
    }

    pub async fn execute(&self, mut request: VideoRequest) -> Result<String> {
        let temp_file = TempFile::new("processed_image", "jpg")?;
        let processed_image_path_str = temp_file.path().to_string_lossy().to_string();
        let config = request.config.clone();
        let style = request.image_style.unwrap_or(ImageStyle::Automatic);
        
        tracing::info!("Starting video creation: title='{}', style={:?}", 
            crate::infrastructure::logging::redact_sensitive(&request.title), style);
        let options = ProcessOptions {
            fit_mode: request.fit_mode,
            background_path: request.background_path.clone(),
            background_url: request.background_url.clone(),
        };
        let main_path = request.image_path.clone();
        let main_url = request.image_url.clone();
        let processed_path_clone = processed_image_path_str.clone();

        if main_path.is_none() && main_url.is_none() {
            anyhow::bail!("No image provided");
        }

        let process_task = tokio::task::spawn_blocking(move || {
            let processor = ImageProcessorImpl;
            processor.process_for_video(
                main_path.as_deref(),
                main_url.as_deref(),
                style,
                &options,
                &processed_path_clone,
                config.width,
                config.height,
            )
        });

        tokio::time::timeout(timeout::IMAGE_PROCESSING, process_task)
            .await
            .map_err(|_| {
                anyhow::anyhow!(
                    "Image processing timed out after {} seconds",
                    timeout::IMAGE_PROCESSING.as_secs()
                )
            })?
            .map_err(|e| anyhow::anyhow!("Task error: {}", e))??;

        request.image_path = Some(processed_image_path_str);

        // Create video - wrap blocking FFmpeg call in spawn_blocking with timeout
        // Note: execute() doesn't have AppHandle, so we pass None
        // Single video: use all CPUs (pass None for unlimited threads)
        let request_clone = request.clone();
        let video_task = tokio::task::spawn_blocking(move || {
            let creator = VideoCreatorImpl;
            creator.create_video(&request_clone, None, None)
        });

        let result = tokio::time::timeout(timeout::VIDEO_CREATION, video_task)
            .await
            .map_err(|_| {
                anyhow::anyhow!(
                    "Video creation timed out after {} minutes",
                    timeout::VIDEO_CREATION.as_secs() / 60
                )
            })?
            .map_err(|e| anyhow::anyhow!("Task error: {}", e))??;

        // temp_file is automatically cleaned up when it goes out of scope
        Ok(result)
    }

    pub async fn execute_with_progress(
        &self,
        mut request: VideoRequest,
        app_handle: AppHandle,
        _cancel_token: Option<CancellationToken>,
    ) -> Result<String> {
        tracing::debug!(
            "Starting video creation: {}x{}, {} audio file(s)",
            request.config.width,
            request.config.height,
            request.audio_paths.len()
        );
        let _ = app_handle.emit(
            "video-progress",
            serde_json::json!({
                "progress": progress::INITIAL,
                "message": "Processing image..."
            }),
        );

        // Create temp file with auto-cleanup (JPEG for faster encoding)
        let temp_file = TempFile::new("processed_image", "jpg")?;
        let processed_image_path_str = temp_file.path().to_string_lossy().to_string();
        let config = request.config.clone();

        let style = request.image_style.unwrap_or(ImageStyle::Automatic);
        let options = ProcessOptions {
            fit_mode: request.fit_mode,
            background_path: request.background_path.clone(),
            background_url: request.background_url.clone(),
        };
        let main_path = request.image_path.clone();
        let main_url = request.image_url.clone();
        let processed_path_clone = processed_image_path_str.clone();

        if main_path.is_none() && main_url.is_none() {
            tracing::error!("Video creation failed: no image provided");
            anyhow::bail!("No image provided");
        }

        if let Some(ref p) = main_path {
            tracing::debug!("Processing image from path: {}", p);
        }
        let process_task = tokio::task::spawn_blocking(move || {
            let processor = ImageProcessorImpl;
            processor.process_for_video(
                main_path.as_deref(),
                main_url.as_deref(),
                style,
                &options,
                &processed_path_clone,
                config.width,
                config.height,
            )
        });

        tokio::time::timeout(timeout::IMAGE_PROCESSING, process_task)
            .await
            .map_err(|_| {
                anyhow::anyhow!(
                    "Image processing timed out after {} seconds",
                    timeout::IMAGE_PROCESSING.as_secs()
                )
            })?
            .map_err(|e| anyhow::anyhow!("Task error: {}", e))??;
        tracing::debug!("Image processed");

        let _ = app_handle.emit(
            "video-progress",
            serde_json::json!({
                "progress": progress::IMAGE_PROCESSED,
                "message": "Image processed. Creating video..."
            }),
        );

        // Update request with processed image path
        request.image_path = Some(processed_image_path_str);

        let _ = app_handle.emit(
            "video-progress",
            serde_json::json!({
                "progress": progress::VIDEO_ENCODING,
                "message": "Encoding video with FFmpeg..."
            }),
        );

        tracing::debug!("Starting FFmpeg video encoding");
        // Create video - wrap blocking FFmpeg call in spawn_blocking with timeout
        // Pass AppHandle to use Tauri's PathResolver API
        // Single video: use all CPUs (pass None for unlimited threads)
        let request_clone = request.clone();
        let app_handle_clone = app_handle.clone();
        let video_task = tokio::task::spawn_blocking(move || {
            let creator = VideoCreatorImpl;
            creator.create_video(&request_clone, Some(&app_handle_clone), None)
        });

        let result = tokio::time::timeout(timeout::VIDEO_CREATION, video_task)
            .await
            .map_err(|_| {
                anyhow::anyhow!(
                    "Video creation timed out after {} minutes",
                    timeout::VIDEO_CREATION.as_secs() / 60
                )
            })?
            .map_err(|e| anyhow::anyhow!("Task error: {}", e))??;

        tracing::info!("Video creation completed: {}", result);
        let _ = app_handle.emit(
            "video-progress",
            serde_json::json!({
                "progress": progress::COMPLETE,
                "message": "Video creation complete!"
            }),
        );

        // temp_file is automatically cleaned up when it goes out of scope
        Ok(result)
    }

    pub async fn execute_bulk_with_progress(
        &self,
        request: BulkVideoRequest,
        app_handle: AppHandle,
        cancel_token: Option<CancellationToken>,
        item_cancel_tokens: Option<Vec<CancellationToken>>,
        video_completion_callback: Option<Box<dyn Fn(usize, String) + Send + Sync>>,
    ) -> Result<BulkVideoResult> {
        tracing::info!("Starting bulk video creation: {} videos, normalize_audio={:?}", 
            request.audio_paths.len(), request.normalize_audio);
        let mut effective_inputs: Vec<ImageInput> = if let Some(ref inputs) = request.image_inputs {
            if inputs.is_empty() {
                request
                    .image_paths
                    .as_ref()
                    .map(|paths| {
                        paths
                            .iter()
                            .map(|p| ImageInput {
                                image_path: Some(p.clone()),
                                ..Default::default()
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            } else {
                inputs.clone()
            }
        } else {
            request
                .image_paths
                .as_ref()
                .map(|paths| {
                    paths
                        .iter()
                        .map(|p| ImageInput {
                            image_path: Some(p.clone()),
                            ..Default::default()
                        })
                        .collect()
                })
                .unwrap_or_default()
        };
        for inp in effective_inputs.iter_mut() {
            if let Some(ref id) = inp.background_predefined_id {
                if let Some(path) =
                    predefined_backgrounds::resolve_predefined_background(&app_handle, id)
                {
                    inp.background_path = Some(path);
                    inp.background_predefined_id = None;
                }
            }
        }
        let mut audio_paths = request.audio_paths;
        // Store original audio paths before normalization for naming purposes
        let original_audio_paths = audio_paths.clone();
        let output_path = request.output_path;
        let config = request.config;
        let normalize_audio = request.normalize_audio.unwrap_or(false);
        let high_pass_filter = request.high_pass_filter.unwrap_or(false);
        let export_normalized_audio = request.export_normalized_audio.unwrap_or(false);
        let audio_export_path = request.audio_export_path.clone();

        // Normalize audio files if requested
        let mut normalized_audio_map: HashMap<String, String> = HashMap::new();
        let mut temp_normalized_files: Vec<String> = Vec::new();
        let mut exported_audio_files: Vec<String> = Vec::new();

        if normalize_audio {
            // Check cancellation before starting normalization
            if let Some(ref token) = cancel_token {
                if is_cancelled(token) {
                    anyhow::bail!("Operation cancelled");
                }
            }

            let _ = app_handle.emit(
                "audio-normalization-start",
                serde_json::json!({
                    "total": audio_paths.len(),
                    "message": format!("Normalizing {} audio file(s)...", audio_paths.len())
                }),
            );

            // Use optimal concurrency calculation for audio normalization
            let (max_jobs, _) = calculate_optimal_concurrency(audio_paths.len(), Some(4));
            
            // Determine export path if audio export is enabled
            let export_path = if export_normalized_audio {
                audio_export_path
            } else {
                None
            };

            let (normalized_results, exported_files) = audio_normalizer::normalize_audio_files_with_export(
                audio_paths.clone(),
                max_jobs,
                high_pass_filter,
                export_path,
                Some(app_handle.clone()),
                cancel_token.clone(),
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to normalize audio files: {}", e))?;

            // Check cancellation after normalization
            if let Some(ref token) = cancel_token {
                if is_cancelled(token) {
                    // Cleanup temp files on cancellation
                    for temp_file in &temp_normalized_files {
                        let _ = std::fs::remove_file(temp_file);
                    }
                    anyhow::bail!("Operation cancelled");
                }
            }

            // Build map and track temp files for cleanup
            for (original, normalized) in &normalized_results {
                normalized_audio_map.insert(original.clone(), normalized.clone());
                temp_normalized_files.push(normalized.clone());
            }

            // Track exported files for reporting
            exported_audio_files = exported_files;

            // Replace audio paths with normalized paths
            audio_paths = audio_paths
                .iter()
                .map(|path| {
                    normalized_audio_map
                        .get(path)
                        .cloned()
                        .unwrap_or_else(|| path.clone())
                })
                .collect();

            let mut completion_message = format!("Successfully normalized {} audio file(s)", normalized_results.len());
            if !exported_audio_files.is_empty() {
                completion_message.push_str(&format!(" and exported {} file(s)", exported_audio_files.len()));
            }

            let _ = app_handle.emit(
                "audio-normalization-complete",
                serde_json::json!({
                    "total": normalized_results.len(),
                    "exported": exported_audio_files.len(),
                    "message": completion_message
                }),
            );
        }

        // Determine number of videos to create (1:1 pairing)
        let video_count = effective_inputs.len().min(audio_paths.len());

        if video_count == 0 {
            // Cleanup temp files on error
            for temp_file in temp_normalized_files {
                let _ = std::fs::remove_file(&temp_file);
            }
            anyhow::bail!("No images or audio files provided");
        }

        // original_audio_paths was already stored before normalization

        let _ = app_handle.emit(
            "bulk-video-progress",
            serde_json::json!({
                "total": video_count,
                "completed": 0,
                "current": 0,
                "message": format!("Starting bulk video creation: {} videos", video_count)
            }),
        );

        // Calculate optimal concurrency and thread allocation
        // Dynamic allocation: ensure each video gets fair CPU share without oversubscription
        let (max_concurrent, threads_per_video) =
            calculate_optimal_concurrency(video_count, Some(4));

        tracing::debug!(
            "Bulk video processing: {} videos, max_concurrent: {}, threads_per_video: {}",
            video_count,
            max_concurrent,
            threads_per_video
        );

        // Limit concurrent video creation to avoid overwhelming the system
        let semaphore = Arc::new(Semaphore::new(max_concurrent));
        let mut handles = Vec::new();

        for i in 0..video_count {
            // Check cancellation before spawning each video task
            if let Some(ref token) = cancel_token {
                if is_cancelled(token) {
                    tracing::info!("Cancellation detected, stopping video creation at video {}/{}", i, video_count);
                    // Cancel all pending tasks by dropping handles
                    handles.clear();
                    // Cleanup temp files on cancellation
                    for temp_file in temp_normalized_files {
                        let _ = std::fs::remove_file(&temp_file);
                    }
                    anyhow::bail!("Operation cancelled");
                }
            }
            let image_input = effective_inputs[i].clone();
            let audio_path = audio_paths[i].clone();
            let original_audio_path = original_audio_paths[i].clone();
            let output_dir = output_path.clone();
            let video_config = config.clone();

            // Extract audio name from original path (not normalized temp file)
            let audio_name = std::path::Path::new(&original_audio_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            let semaphore_clone = semaphore.clone();
            let app_handle_clone = app_handle.clone();
            let video_index = i;
            let video_count_clone = video_count;
            let threads_per_video_clone = threads_per_video;
            // Get per-item cancellation token if available
            let item_cancel_token = item_cancel_tokens.as_ref().and_then(|tokens| tokens.get(i).cloned());
            // Clone cancel token for this task
            let cancel_token_for_task = cancel_token.clone();

            let handle = tokio::spawn(async move {
                // Check per-item cancellation before starting
                if let Some(ref token) = item_cancel_token {
                    if token.is_cancelled() {
                        let _ = app_handle_clone.emit(
                            "video-progress-complete",
                            serde_json::json!({
                                "index": video_index,
                                "success": false,
                                "error": "Cancelled"
                            }),
                        );
                        return Err(anyhow::anyhow!("Item cancelled"));
                    }
                }

                // Acquire semaphore permit
                let _permit = semaphore_clone.acquire().await.unwrap();

                // Check cancellation again after acquiring permit
                if let Some(ref token) = item_cancel_token {
                    if token.is_cancelled() {
                        let _ = app_handle_clone.emit(
                            "video-progress-complete",
                            serde_json::json!({
                                "index": video_index,
                                "success": false,
                                "error": "Cancelled"
                            }),
                        );
                        return Err(anyhow::anyhow!("Item cancelled"));
                    }
                }

                let current_index = video_index + 1;
                let _ = app_handle_clone.emit(
                    "video-progress-start",
                    serde_json::json!({
                        "index": video_index,
                        "audio_path": audio_path.clone(),
                        "audio_name": audio_name.clone()
                    }),
                );

                let _ = app_handle_clone.emit("bulk-video-progress", serde_json::json!({
                    "total": video_count_clone,
                    "completed": current_index - 1,
                    "current": current_index,
                    "message": format!("Processing video {}/{}: {}", current_index, video_count_clone, audio_name)
                }));

                // Check cancellation before image processing
                if let Some(ref token) = item_cancel_token {
                    if token.is_cancelled() {
                        let _ = app_handle_clone.emit(
                            "video-progress-complete",
                            serde_json::json!({
                                "index": video_index,
                                "success": false,
                                "error": "Cancelled"
                            }),
                        );
                        return Err(anyhow::anyhow!("Item cancelled"));
                    }
                }
                if let Some(ref token) = cancel_token_for_task {
                    if is_cancelled(token) {
                        let _ = app_handle_clone.emit(
                            "video-progress-complete",
                            serde_json::json!({
                                "index": video_index,
                                "success": false,
                                "error": "Operation cancelled"
                            }),
                        );
                        return Err(anyhow::anyhow!("Operation cancelled"));
                    }
                }

                // Process image
                let _ = app_handle_clone.emit(
                    "video-progress-update",
                    serde_json::json!({
                        "index": video_index,
                        "progress": progress::IMAGE_PROCESSING,
                        "message": "Processing image..."
                    }),
                );

                // Create temp file with auto-cleanup using unique name
                let temp_file =
                    match TempFile::new(&format!("processed_image_{}", video_index), "jpg") {
                        Ok(f) => f,
                        Err(e) => {
                            let _ = app_handle_clone.emit(
                                "video-progress-complete",
                                serde_json::json!({
                                    "index": video_index,
                                    "success": false,
                                    "error": format!("Failed to create temp file: {}", e)
                                }),
                            );
                            return Err(anyhow::anyhow!("Failed to create temp file: {}", e));
                        }
                    };
                let processed_image_path_str = temp_file.path().to_string_lossy().to_string();

                // Wrap blocking image processing in spawn_blocking with timeout
                let image_input_clone = image_input.clone();
                let processed_path_for_closure = processed_image_path_str.clone();
                let video_config_clone = video_config.clone();

                let process_task = tokio::task::spawn_blocking(move || {
                    let style = image_input_clone
                        .image_style
                        .unwrap_or(ImageStyle::Automatic);
                    let options = ProcessOptions {
                        fit_mode: image_input_clone.fit_mode,
                        background_path: image_input_clone.background_path,
                        background_url: image_input_clone.background_url,
                    };
                    let processor = ImageProcessorImpl;
                    processor.process_for_video(
                        image_input_clone.image_path.as_deref(),
                        image_input_clone.image_url.as_deref(),
                        style,
                        &options,
                        &processed_path_for_closure,
                        video_config_clone.width,
                        video_config_clone.height,
                    )
                });

                let process_result = tokio::time::timeout(timeout::IMAGE_PROCESSING, process_task)
                    .await
                    .map_err(|_| {
                        format!(
                            "Image processing timed out after {} seconds",
                            timeout::IMAGE_PROCESSING.as_secs()
                        )
                    })
                    .and_then(|r| r.map_err(|e| format!("Task error: {}", e)))
                    .and_then(|r| r.map_err(|e| format!("Failed to process image: {}", e)));

                if let Err(e) = process_result {
                    let _ = app_handle_clone.emit(
                        "video-progress-complete",
                        serde_json::json!({
                            "index": video_index,
                            "success": false,
                            "error": format!("Failed to process image: {}", e)
                        }),
                    );
                    return Err(anyhow::anyhow!("Failed to process image for {}: {}", audio_path, e));
                }

                // Check cancellation before video encoding
                if let Some(ref token) = item_cancel_token {
                    if token.is_cancelled() {
                        let _ = app_handle_clone.emit(
                            "video-progress-complete",
                            serde_json::json!({
                                "index": video_index,
                                "success": false,
                                "error": "Cancelled"
                            }),
                        );
                        return Err(anyhow::anyhow!("Item cancelled"));
                    }
                }
                if let Some(ref token) = cancel_token_for_task {
                    if is_cancelled(token) {
                        let _ = app_handle_clone.emit(
                            "video-progress-complete",
                            serde_json::json!({
                                "index": video_index,
                                "success": false,
                                "error": "Operation cancelled"
                            }),
                        );
                        return Err(anyhow::anyhow!("Operation cancelled"));
                    }
                }

                let _ = app_handle_clone.emit(
                    "video-progress-update",
                    serde_json::json!({
                        "index": video_index,
                        "progress": progress::VIDEO_ENCODING,
                        "message": "Encoding video..."
                    }),
                );

                // Create video request with title from metadata or original audio filename (not normalized temp file)
                let title = match AudioMetadata::from_file(&original_audio_path) {
                    Ok(metadata) => {
                        let fallback = Path::new(&original_audio_path)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Untitled");
                        metadata.display_name(fallback)
                    }
                    Err(_) => {
                        Path::new(&original_audio_path)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Untitled")
                            .to_string()
                    }
                };
                
                let video_request = VideoRequest {
                    image_path: Some(processed_image_path_str),
                    image_url: None,
                    image_style: None,
                    fit_mode: None,
                    background_path: None,
                    background_url: None,
                    background_predefined_id: None,
                    audio_paths: vec![audio_path.clone()],
                    title,
                    output_path: output_dir.clone(),
                    config: video_config,
                };

                // Create video - wrap blocking FFmpeg call in spawn_blocking with timeout
                // Pass AppHandle to use Tauri's PathResolver API
                // Limit threads per video to avoid CPU oversubscription in bulk processing
                let video_request_clone = video_request.clone();
                let app_handle_for_video = app_handle_clone.clone();
                let threads_for_this_video = threads_per_video_clone;
                let video_task = tokio::task::spawn_blocking(move || {
                    let creator = VideoCreatorImpl;
                    creator.create_video(
                        &video_request_clone,
                        Some(&app_handle_for_video),
                        Some(threads_for_this_video),
                    )
                });

                let result = match tokio::time::timeout(timeout::VIDEO_CREATION, video_task).await {
                    Ok(Ok(Ok(output_file))) => {
                        let _ = app_handle_clone.emit(
                            "video-progress-complete",
                            serde_json::json!({
                                "index": video_index,
                                "success": true,
                                "output_file": output_file.clone()
                            }),
                        );
                        Ok(output_file)
                    }
                    Ok(Ok(Err(e))) => {
                        let _ = app_handle_clone.emit(
                            "video-progress-complete",
                            serde_json::json!({
                                "index": video_index,
                                "success": false,
                                "error": format!("{}", e)
                            }),
                        );
                        Err(anyhow::anyhow!("Failed to create video for {}: {}", audio_path, e))
                    }
                    Ok(Err(e)) => {
                        let _ = app_handle_clone.emit(
                            "video-progress-complete",
                            serde_json::json!({
                                "index": video_index,
                                "success": false,
                                "error": format!("Task error: {}", e)
                            }),
                        );
                        Err(anyhow::anyhow!(
                            "Failed to create video for {}: Task error: {}",
                            audio_path, e
                        ))
                    }
                    Err(_) => {
                        let _ = app_handle_clone.emit("video-progress-complete", serde_json::json!({
                            "index": video_index,
                            "success": false,
                            "error": format!("Video creation timed out after {} minutes", timeout::VIDEO_CREATION.as_secs() / 60)
                        }));
                        Err(anyhow::anyhow!("Video creation timed out for {}", audio_path))
                    }
                };

                // temp_file is automatically cleaned up when it goes out of scope
                result
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        let mut successful = Vec::new();
        let mut failed = Vec::new();
        let audio_paths_clone = audio_paths.clone();

        for (i, handle) in handles.into_iter().enumerate() {
            // Check cancellation before waiting for each task result
            if let Some(ref token) = cancel_token {
                if is_cancelled(token) {
                    tracing::info!("Cancellation detected while collecting results, stopping at video {}/{}", i, video_count);
                    // Cleanup temp files on cancellation
                    for temp_file in temp_normalized_files {
                        let _ = std::fs::remove_file(&temp_file);
                    }
                    anyhow::bail!("Operation cancelled");
                }
            }

            match handle.await {
                Ok(Ok(output_file)) => {
                    successful.push(output_file.clone());
                    let _ = app_handle.emit(
                        "bulk-video-progress",
                        serde_json::json!({
                            "total": video_count,
                            "completed": i + 1,
                            "current": i + 1,
                            "message": format!("Completed video {}/{}", i + 1, video_count)
                        }),
                    );
                    
                    // Call completion callback if provided
                    if let Some(ref callback) = video_completion_callback {
                        callback(i, output_file);
                    }
                }
                Ok(Err(err)) => {
                    let audio_path = audio_paths_clone
                        .get(i)
                        .cloned()
                        .unwrap_or_else(|| "unknown".to_string());
                    failed.push((audio_path, format!("{}", err)));
                    let _ = app_handle.emit(
                        "bulk-video-progress",
                        serde_json::json!({
                            "total": video_count,
                            "completed": i + 1,
                            "current": i + 1,
                            "message": format!("Failed video {}/{}", i + 1, video_count)
                        }),
                    );
                }
                Err(e) => {
                    let audio_path = audio_paths_clone
                        .get(i)
                        .cloned()
                        .unwrap_or_else(|| "unknown".to_string());
                    failed.push((audio_path, format!("Task error: {}", e)));
                    let _ = app_handle.emit(
                        "bulk-video-progress",
                        serde_json::json!({
                            "total": video_count,
                            "completed": i + 1,
                            "current": i + 1,
                            "message": format!("Failed video {}/{}", i + 1, video_count)
                        }),
                    );
                }
            }
        }

        let _ = app_handle.emit("bulk-video-progress", serde_json::json!({
            "total": video_count,
            "completed": video_count,
            "current": video_count,
            "message": format!("Bulk creation complete: {} successful, {} failed", successful.len(), failed.len())
        }));

        // Cleanup temporary normalized audio files
        for temp_file in temp_normalized_files {
            let _ = std::fs::remove_file(&temp_file);
        }

        Ok(BulkVideoResult { 
            successful, 
            failed,
            exported_audio_files,
        })
    }
}
