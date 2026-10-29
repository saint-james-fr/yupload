use crate::domain::traits::ImageProcessor;
use crate::domain::{FitMode, ImageStyle, ProcessOptions};
use crate::infrastructure::ImageProcessorImpl;
use crate::utils::ffmpeg::get_ffmpeg_path_optional;
use crate::utils::{
    create_hidden_command, download_image_safely, timeout, validate_file_size, TempFile, MAX_IMAGE_SIZE_BYTES,
};
use base64::{engine::general_purpose, Engine as _};
use image::ImageReader;

/// Generate image preview for YouTube (processed image).
/// When image_style, fit_mode, or background_* are provided, uses process_for_video for WYSIWYG preview.
/// background_predefined_id is resolved to path by the caller before calling this.
pub async fn generate_image_preview(
    image_path: String,
    width: u32,
    height: u32,
    image_style: Option<String>,
    fit_mode: Option<String>,
    background_path: Option<String>,
    background_url: Option<String>,
) -> Result<String, String> {
    let processor = ImageProcessorImpl;

    // Handle data URLs by saving to temp file
    let (main_path, main_url, _data_url_temp_file) = if image_path.starts_with("data:") {
        // Extract base64 data from data URL
        let base64_data = image_path
            .splitn(2, ',')
            .nth(1)
            .ok_or_else(|| "Invalid data URL format".to_string())?;
        
        // Decode base64
        let image_data = general_purpose::STANDARD
            .decode(base64_data)
            .map_err(|e| format!("Failed to decode base64 data URL: {}", e))?;
        
        // Validate size
        if image_data.len() > MAX_IMAGE_SIZE_BYTES as usize {
            return Err(format!("Image data too large: {} bytes", image_data.len()));
        }
        
        // Save to temp file (keep it alive by storing in _data_url_temp_file)
        let temp_file = TempFile::new("data_url_image", "jpg")
            .map_err(|e| format!("Failed to create temp file for data URL: {}", e))?;
        std::fs::write(temp_file.path(), &image_data)
            .map_err(|e| format!("Failed to write data URL to temp file: {}", e))?;
        
        let temp_path = temp_file.path().to_string_lossy().to_string();
        (Some(temp_path), None, Some(temp_file))
    } else {
        // Validate input file size if it's not a URL
        if !image_path.starts_with("http://") && !image_path.starts_with("https://") {
            validate_file_size(std::path::Path::new(&image_path), MAX_IMAGE_SIZE_BYTES)
                .map_err(|e| format!("Input image too large: {}", e))?;
        }
        
        let path = if image_path.starts_with("http://") || image_path.starts_with("https://") {
            None
        } else {
            Some(image_path.clone())
        };
        let url = if image_path.starts_with("http://") || image_path.starts_with("https://") {
            Some(image_path.clone())
        } else {
            None
        };
        (path, url, None)
    };

    let temp_file = TempFile::new("yupload_preview", "jpg")
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    let preview_path_str = temp_file.path().to_string_lossy().to_string();

    let style = parse_image_style(image_style.as_deref());
    let options = ProcessOptions {
        fit_mode: parse_fit_mode(fit_mode.as_deref()),
        background_path,
        background_url,
    };

    // Clone main_path and main_url for the closure (data_url_temp_file stays in scope)
    let main_path_clone = main_path.clone();
    let main_url_clone = main_url.clone();
    let process_task = tokio::task::spawn_blocking(move || {
        processor.process_for_video(
            main_path_clone.as_deref(),
            main_url_clone.as_deref(),
            style,
            &options,
            &preview_path_str,
            width,
            height,
        )
    });

    tokio::time::timeout(timeout::IMAGE_THUMBNAIL, process_task)
        .await
        .map_err(|_| {
            format!(
                "Image processing timed out after {} seconds",
                timeout::IMAGE_THUMBNAIL.as_secs()
            )
        })?
        .map_err(|e| format!("Task error: {}", e))?
        .map_err(|e| e.to_string())?;

    let image_data =
        std::fs::read(temp_file.path()).map_err(|e| format!("Failed to read preview: {}", e))?;
    let base64 = general_purpose::STANDARD.encode(&image_data);
    Ok(format!("data:image/jpeg;base64,{}", base64))
}

fn parse_image_style(s: Option<&str>) -> ImageStyle {
    match s {
        Some("fit16x9") => ImageStyle::Fit16x9,
        Some("custom-background") => ImageStyle::CustomBackground,
        Some("background-only") => ImageStyle::BackgroundOnly,
        _ => ImageStyle::Automatic,
    }
}

fn parse_fit_mode(s: Option<&str>) -> Option<FitMode> {
    match s {
        Some("cover") => Some(FitMode::Cover),
        Some("contain") => Some(FitMode::Contain),
        _ => None,
    }
}

/// Generate image thumbnail
pub async fn generate_image_thumbnail(image_path: String, size: u32) -> Result<String, String> {
    // Create temp file with auto-cleanup using unique name
    let temp_file =
        TempFile::new("thumb", "png").map_err(|e| format!("Failed to create temp file: {}", e))?;

    // Validate input file size if it's not a URL
    if !image_path.starts_with("http://") && !image_path.starts_with("https://") {
        validate_file_size(std::path::Path::new(&image_path), MAX_IMAGE_SIZE_BYTES)
            .map_err(|e| format!("Input image too large: {}", e))?;
    }

    // Use spawn_blocking to run blocking operations in a separate thread
    let image_path_clone = image_path.clone();

    let img_task = tokio::task::spawn_blocking(move || {
        // Load and resize image for thumbnail
        if image_path_clone.starts_with("http://") || image_path_clone.starts_with("https://") {
            // Use safe download function with URL validation, size limits, and Content-Type checks
            let bytes = download_image_safely(&image_path_clone)
                .map_err(|e| format!("Failed to safely download image: {}", e))?;
            ImageReader::new(std::io::Cursor::new(bytes))
                .with_guessed_format()
                .map_err(|e| format!("Failed to guess format: {}", e))?
                .decode()
                .map_err(|e| format!("Failed to decode image: {}", e))
        } else {
            ImageReader::open(&image_path_clone)
                .map_err(|e| format!("Failed to open image: {}", e))?
                .with_guessed_format()
                .map_err(|e| format!("Failed to guess format: {}", e))?
                .decode()
                .map_err(|e| format!("Failed to decode image: {}", e))
        }
    });

    let img = tokio::time::timeout(timeout::IMAGE_THUMBNAIL, img_task)
        .await
        .map_err(|_| {
            format!(
                "Image loading timed out after {} seconds",
                timeout::IMAGE_THUMBNAIL.as_secs()
            )
        })?
        .map_err(|e| format!("Task error: {}", e))??;

    // Resize and save thumbnail
    let thumbnail_path = temp_file.path().to_path_buf();
    let save_task = tokio::task::spawn_blocking(move || {
        let thumbnail = img.thumbnail(size, size);
        thumbnail
            .save(&thumbnail_path)
            .map_err(|e| format!("Failed to save thumbnail: {}", e))
    });

    tokio::time::timeout(timeout::IMAGE_THUMBNAIL_SAVE, save_task)
        .await
        .map_err(|_| {
            format!(
                "Thumbnail save timed out after {} seconds",
                timeout::IMAGE_THUMBNAIL_SAVE.as_secs()
            )
        })?
        .map_err(|e| format!("Task error: {}", e))??;

    // Convert to data URL
    let image_data =
        std::fs::read(temp_file.path()).map_err(|e| format!("Failed to read thumbnail: {}", e))?;
    let base64 = general_purpose::STANDARD.encode(&image_data);

    // temp_file is automatically cleaned up when it goes out of scope
    Ok(format!("data:image/png;base64,{}", base64))
}

/// Generate video thumbnail using FFmpeg
pub async fn generate_video_thumbnail(
    video_path: String,
    app_handle: Option<&tauri::AppHandle>,
) -> Result<String, String> {
    // Create temp file with auto-cleanup using unique name
    let temp_file = TempFile::new("video_thumb", "png")
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    // Get FFmpeg path - use Tauri PathResolver if AppHandle is available
    let ffmpeg_path = get_ffmpeg_path_optional(app_handle);

    // Clone thumbnail_path before moving into closure
    let thumbnail_path = temp_file.path().to_path_buf();

    // Extract frame at 1 second (or first frame if video is shorter)
    let ffmpeg_task = tokio::task::spawn_blocking(move || {
        create_hidden_command(&ffmpeg_path)
            .arg("-i")
            .arg(&video_path)
            .arg("-ss")
            .arg("00:00:01")
            .arg("-vframes")
            .arg("1")
            .arg("-vf")
            .arg("scale=320:-1") // Scale to 320px width, maintain aspect ratio
            .arg(&thumbnail_path)
            .output()
            .map_err(|e| format!("Failed to execute FFmpeg: {}", e))
    });

    let output = tokio::time::timeout(timeout::VIDEO_THUMBNAIL, ffmpeg_task)
        .await
        .map_err(|_| {
            format!(
                "FFmpeg thumbnail extraction timed out after {} seconds",
                timeout::VIDEO_THUMBNAIL.as_secs()
            )
        })?
        .map_err(|e| format!("Task error: {}", e))??;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFmpeg error: {}", stderr));
    }

    // Convert to data URL
    let image_data =
        std::fs::read(temp_file.path()).map_err(|e| format!("Failed to read thumbnail: {}", e))?;
    let base64 = general_purpose::STANDARD.encode(&image_data);

    // temp_file is automatically cleaned up when it goes out of scope
    Ok(format!("data:image/png;base64,{}", base64))
}
