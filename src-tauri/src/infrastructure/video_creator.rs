use crate::domain::VideoCreator;
use crate::domain::VideoRequest;
use crate::utils::{
    check_disk_space, create_hidden_command, estimate_video_size, find_unique_filename, get_ffmpeg_path_optional,
    validate_file_extension, validate_file_size, MAX_AUDIO_SIZE_BYTES,
};
use anyhow::{Context, Result};
use std::path::Path;

pub struct VideoCreatorImpl;

impl VideoCreator for VideoCreatorImpl {
    fn create_video(
        &self,
        request: &VideoRequest,
        app_handle: Option<&tauri::AppHandle>,
        max_threads: Option<usize>,
    ) -> Result<String> {
        let base_filename = sanitize_filename(&request.title);
        let output_dir = Path::new(&request.output_path);

        // Validate input files first
        if let Some(image_path) = &request.image_path {
            let image_path_buf = Path::new(image_path);
            validate_file_extension(image_path_buf)
                .with_context(|| format!("Invalid image file: {}", image_path))?;
        }

        // Validate audio files and calculate total size
        let mut total_audio_size: u64 = 0;
        let mut max_audio_duration: u64 = 0;

        for audio_path in &request.audio_paths {
            let audio_path_buf = Path::new(audio_path);

            // Validate file size
            validate_file_size(audio_path_buf, MAX_AUDIO_SIZE_BYTES)
                .with_context(|| format!("Audio file too large: {}", audio_path))?;

            // Validate extension
            validate_file_extension(audio_path_buf)
                .with_context(|| format!("Invalid audio file: {}", audio_path))?;

            // Get file size for disk space estimation
            if let Ok(metadata) = std::fs::metadata(audio_path_buf) {
                total_audio_size += metadata.len();
                // Rough duration estimate: 44100 Hz * 2 channels * 2 bytes = ~176KB/sec for WAV
                let estimated_duration = metadata.len() / 176_000;
                max_audio_duration = max_audio_duration.max(estimated_duration);
            }
        }

        // Estimate output video size and check disk space
        let estimated_output_size = estimate_video_size(total_audio_size, max_audio_duration);
        check_disk_space(output_dir, estimated_output_size)
            .with_context(|| "Not enough disk space for video creation")?;

        // Ensure output directory exists
        std::fs::create_dir_all(output_dir)
            .with_context(|| format!("Failed to create output directory: {:?}", output_dir))?;

        // Find a unique filename by checking for duplicates
        let output_file = find_unique_filename(output_dir, &base_filename, "mp4");
        let output_path = output_file.to_string_lossy().to_string();

        // Build ffmpeg command - use Tauri PathResolver if AppHandle is available, otherwise fallback
        let ffmpeg_path = get_ffmpeg_path_optional(app_handle);
        let mut cmd = create_hidden_command(&ffmpeg_path);

        // Input image (loop for duration)
        if let Some(image_path) = &request.image_path {
            cmd.arg("-loop").arg("1");
            cmd.arg("-i").arg(image_path);
        } else {
            anyhow::bail!("No image path provided");
        }

        // Input audio files
        if request.audio_paths.is_empty() {
            anyhow::bail!("No audio files provided");
        }

        // Add all audio inputs
        for audio_path in &request.audio_paths {
            cmd.arg("-i").arg(audio_path);
        }

        // Build filter_complex for audio concatenation if multiple files
        if request.audio_paths.len() > 1 {
            // Create concat filter for audio
            let audio_inputs: Vec<String> = (1..=request.audio_paths.len())
                .map(|i| format!("[{}:a]", i))
                .collect();
            let filter_complex = format!(
                "{}concat=n={}:v=0:a=1[outa]",
                audio_inputs.join(""),
                request.audio_paths.len()
            );
            cmd.arg("-filter_complex").arg(&filter_complex);
            cmd.arg("-map").arg("0:v");
            cmd.arg("-map").arg("[outa]");
        } else {
            cmd.arg("-map").arg("0:v");
            cmd.arg("-map").arg("1:a");
        }

        // Video codec and settings
        cmd.arg("-c:v").arg("libx264");

        // Limit threads if specified (for bulk processing to avoid CPU oversubscription)
        if let Some(threads) = max_threads {
            cmd.arg("-threads").arg(threads.to_string());
        }

        // Use 'fast' preset for better encoding speed with minimal quality loss
        // This is ~2x faster than 'medium' with only ~10% larger file size
        cmd.arg("-preset").arg("fast");
        cmd.arg("-crf").arg("23");
        cmd.arg("-pix_fmt").arg("yuv420p");

        // Audio codec and settings
        cmd.arg("-c:a").arg("aac");
        cmd.arg("-b:a").arg("192k");
        cmd.arg("-shortest");

        // Video resolution
        cmd.arg("-s").arg(format!(
            "{}x{}",
            request.config.width, request.config.height
        ));
        cmd.arg("-r").arg(request.config.fps.to_string());

        // Output file (no -y flag needed since we've already ensured unique filename)
        cmd.arg(&output_path);

        // Execute ffmpeg
        let output = cmd.output().with_context(|| {
            "Failed to execute ffmpeg command. Make sure ffmpeg is installed and in PATH."
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("FFmpeg error: {}", stderr);
        }

        // Validate output file was created and has content
        let output_path_buf = Path::new(&output_path);
        if !output_path_buf.exists() {
            anyhow::bail!(
                "FFmpeg completed but output file was not created: {}",
                output_path
            );
        }

        let output_size = std::fs::metadata(output_path_buf)
            .with_context(|| "Failed to get output file metadata")?
            .len();

        if output_size == 0 {
            anyhow::bail!("Output video file is empty: {}", output_path);
        }

        tracing::info!(
            "Video created successfully: {} ({} MB)",
            output_path,
            output_size / 1_048_576
        );

        Ok(output_path)
    }
}

fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}
