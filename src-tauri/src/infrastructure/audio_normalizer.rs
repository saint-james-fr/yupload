use crate::utils::{
    check_disk_space, create_hidden_command, estimate_normalized_audio_size, find_unique_filename,
    get_ffmpeg_path_optional, timeout, validate_file_size, TempFile, MAX_AUDIO_SIZE_BYTES,
};
use anyhow::{Context, Result};
use std::path::Path;
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::Semaphore;

const TARGET_PEAK: &str = "-0.1";
const HIGH_PASS_FILTER: &str = "highpass=f=30,";
const LOUDNORM: &str = "loudnorm=I=-16:TP=-0.5:LRA=10";
const WAV_ENCODE: &[&str] = &["-ar", "44100", "-ac", "2", "-sample_fmt", "s16"];

/// FFmpeg encode args for step 4 (volume-adjusted output). Output format matches input extension.
fn encode_args_for_extension(ext: &str) -> &'static [&'static str] {
    match ext {
        "mp3" => &[
            "-ar",
            "44100",
            "-ac",
            "2",
            "-c:a",
            "libmp3lame",
            "-b:a",
            "320k",
        ],
        "wav" => WAV_ENCODE,
        "flac" => &["-ar", "44100", "-ac", "2", "-c:a", "flac"],
        "m4a" | "aac" => &["-ar", "44100", "-ac", "2", "-c:a", "aac", "-b:a", "256k"],
        "ogg" => &[
            "-ar",
            "44100",
            "-ac",
            "2",
            "-c:a",
            "libvorbis",
            "-q:a",
            "10",
        ],
        "aif" | "aiff" => &[
            "-ar",
            "44100",
            "-ac",
            "2",
            "-c:a",
            "pcm_s16be",
            "-f",
            "aiff",
        ],
        _ => WAV_ENCODE,
    }
}

/// Normalizes a single audio file using loudnorm + exact peak adjustment
pub fn normalize_audio_file(
    input_path: &str,
    output_path: &str,
    high_pass_filter: bool,
    app_handle: Option<&tauri::AppHandle>,
) -> Result<()> {
    // Validate input file size
    let input_path_buf = Path::new(input_path);
    validate_file_size(input_path_buf, MAX_AUDIO_SIZE_BYTES)
        .with_context(|| format!("Input audio file too large: {}", input_path))?;

    // Get input file size to estimate output size
    let input_size = std::fs::metadata(input_path_buf)
        .with_context(|| "Failed to get input file metadata")?
        .len();

    let estimated_output_size = estimate_normalized_audio_size(input_size);

    // Check disk space for output
    let output_path_buf = Path::new(output_path);
    if let Some(parent) = output_path_buf.parent() {
        check_disk_space(parent, estimated_output_size * 2) // 2x for temp + final file
            .with_context(|| "Not enough disk space for audio normalization")?;
    }

    // Use Tauri PathResolver if AppHandle is available, otherwise fallback
    let ffmpeg_path = get_ffmpeg_path_optional(app_handle);

    // Step 1: Apply loudnorm + optionally highpass filter
    // Use TempFile for automatic cleanup
    let temp_file = TempFile::new("normalized_temp", "wav")
        .with_context(|| "Failed to create temp file for normalization")?;
    let temp_output = temp_file.path().to_string_lossy().to_string();

    let filter_chain = if high_pass_filter {
        format!("{}{}", HIGH_PASS_FILTER, LOUDNORM)
    } else {
        LOUDNORM.to_string()
    };

    let mut cmd = create_hidden_command(&ffmpeg_path);
    cmd.arg("-i").arg(input_path);
    cmd.arg("-af").arg(&filter_chain);
    cmd.args(WAV_ENCODE);
    cmd.arg(&temp_output);
    cmd.arg("-loglevel").arg("error");
    cmd.arg("-stats");

    let output = cmd
        .output()
        .with_context(|| format!("Failed to execute FFmpeg for loudnorm: {}", input_path))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("FFmpeg loudnorm error for {}: {}", input_path, stderr);
    }

    // Step 2: Measure peak of normalized file
    let mut cmd = create_hidden_command(&ffmpeg_path);
    cmd.arg("-i").arg(&temp_output);
    cmd.arg("-af").arg("volumedetect");
    cmd.arg("-f").arg("null");
    cmd.arg("/dev/null");
    // Don't use -loglevel error here, we need to see the volumedetect output
    cmd.arg("-loglevel").arg("info");

    let output = cmd
        .output()
        .with_context(|| format!("Failed to execute FFmpeg for volumedetect: {}", temp_output))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("FFmpeg volumedetect error: {}", stderr);
    }

    // Parse max_volume from both stdout and stderr (FFmpeg outputs volumedetect info to stderr)
    // The bash script uses: grep "max_volume" | awk '{print $5}' | tr -d 'dB'
    // So we need to find the line with "max_volume" and get the 5th whitespace-separated field
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined_output = format!("{}\n{}", stdout, stderr);

    let max_volume = combined_output
        .lines()
        .find(|line| line.contains("max_volume"))
        .and_then(|line| {
            // Split by whitespace and get the 5th field (index 4)
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() >= 5 {
                // The 5th field should be something like "-12.5dB" or "-12.5 dB"
                fields[4]
                    .trim_end_matches("dB")
                    .trim_end_matches(" dB")
                    .parse::<f64>()
                    .ok()
            } else {
                None
            }
        })
        .or_else(|| {
            // Fallback: try to find any number followed by "dB" in the line
            combined_output
                .lines()
                .find(|line| line.contains("max_volume"))
                .and_then(|line| {
                    // Look for pattern like "max_volume: -12.5 dB" or "max_volume=-12.5dB"
                    for word in line.split_whitespace() {
                        if word.ends_with("dB") {
                            if let Ok(val) = word.trim_end_matches("dB").parse::<f64>() {
                                return Some(val);
                            }
                        }
                    }
                    None
                })
        })
        .with_context(|| {
            format!(
                "Could not find max_volume in FFmpeg output. Stderr: {}\nStdout: {}",
                stderr, stdout
            )
        })?;

    // Step 3: Calculate exact gain to reach target peak
    let target_peak: f64 = TARGET_PEAK.parse().unwrap();
    let gain_value = target_peak - max_volume;

    // Step 4: Apply volume adjustment and encode to target format (matches input extension)
    let ext = output_path_buf
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("wav")
        .to_lowercase();
    let encode_args = encode_args_for_extension(&ext);
    let mut cmd = create_hidden_command(&ffmpeg_path);
    cmd.arg("-i").arg(&temp_output);
    cmd.arg("-af").arg(&format!("volume={}dB", gain_value));
    cmd.args(encode_args);
    cmd.arg(output_path);
    cmd.arg("-loglevel").arg("error");
    cmd.arg("-stats");

    let output = cmd.output().with_context(|| {
        format!(
            "Failed to execute FFmpeg for volume adjustment: {}",
            temp_output
        )
    })?;

    // temp_file is automatically cleaned up when it goes out of scope

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("FFmpeg volume adjustment error: {}", stderr);
    }

    // Validate output file
    validate_file_size(output_path_buf, MAX_AUDIO_SIZE_BYTES)
        .with_context(|| "Output audio file exceeds maximum size")?;

    Ok(())
}

/// Normalizes multiple audio files in parallel and saves to output directory
/// Returns successful results, failures are collected separately
pub async fn normalize_audio_files_to_directory(
    audio_paths: Vec<String>,
    output_dir: &str,
    max_jobs: usize,
    high_pass_filter: bool,
    add_suffix: bool,
    app_handle: Option<tauri::AppHandle>,
    cancel_token: Option<tokio_util::sync::CancellationToken>,
) -> Result<(Vec<(String, String)>, Vec<(String, String)>)> {
    use std::path::Path;

    let semaphore = Arc::new(Semaphore::new(max_jobs));
    let mut handles = Vec::new();

    let output_path = Path::new(output_dir);
    // Ensure output directory exists
    std::fs::create_dir_all(output_path)
        .with_context(|| format!("Failed to create output directory: {}", output_dir))?;

    // Check disk space before starting
    let total_input_size: u64 = audio_paths
        .iter()
        .filter_map(|path| std::fs::metadata(path).ok())
        .map(|m| m.len())
        .sum();
    let estimated_total_output =
        estimate_normalized_audio_size(total_input_size) * audio_paths.len() as u64;
    check_disk_space(output_path, estimated_total_output)
        .with_context(|| "Not enough disk space for batch audio normalization")?;

    for audio_path in audio_paths.iter() {
        // Check cancellation before spawning each normalization task
        if let Some(ref token) = cancel_token {
            if token.is_cancelled() {
                tracing::info!("Cancellation detected, stopping audio normalization");
                anyhow::bail!("Operation cancelled");
            }
        }

        let audio_path_clone = audio_path.clone();
        let output_dir_clone = output_dir.to_string();
        let semaphore_clone = semaphore.clone();
        let high_pass_filter_clone = high_pass_filter;
        let add_suffix_clone = add_suffix;
        let app_handle_clone = app_handle.clone();

        let handle = tokio::spawn(async move {
            let _permit = semaphore_clone.acquire().await.unwrap();

            // Create output path preserving original filename and extension (input format respected)
            let input_path = Path::new(&audio_path_clone);
            let file_stem = input_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("audio");
            let ext = input_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("wav")
                .to_lowercase();
            let base_name = if add_suffix_clone {
                format!("{}-normalized", file_stem)
            } else {
                file_stem.to_string()
            };
            let output_dir_path = Path::new(&output_dir_clone);

            // Find unique filename in case of duplicates; output extension matches input
            let output_path = find_unique_filename(output_dir_path, &base_name, &ext);
            let output_path_str = output_path.to_string_lossy().to_string();

            // Normalize the file with timeout - pass AppHandle if available
            let audio_path_for_task = audio_path_clone.clone();
            let output_path_for_task = output_path_str.clone();
            let normalize_task = tokio::task::spawn_blocking(move || {
                normalize_audio_file(
                    &audio_path_for_task,
                    &output_path_for_task,
                    high_pass_filter_clone,
                    app_handle_clone.as_ref(),
                )
            });

            match tokio::time::timeout(timeout::AUDIO_NORMALIZATION, normalize_task).await {
                Ok(Ok(Ok(()))) => Ok((audio_path_clone, output_path_str)),
                Ok(Ok(Err(e))) => Err((audio_path_clone, format!("{}", e))),
                Ok(Err(e)) => Err((audio_path_clone, format!("Task error: {}", e))),
                Err(_) => Err((
                    audio_path_clone,
                    "Audio normalization timed out after 5 minutes".to_string(),
                )),
            }
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete and collect results
    let mut successful = Vec::new();
    let mut failed = Vec::new();

    for handle in handles {
        // Check cancellation before waiting for each task result
        if let Some(ref token) = cancel_token {
            if token.is_cancelled() {
                tracing::info!("Cancellation detected while collecting normalization results");
                anyhow::bail!("Operation cancelled");
            }
        }

        match handle.await {
            Ok(Ok((original, normalized))) => successful.push((original, normalized)),
            Ok(Err((original, error))) => failed.push((original, error)),
            Err(e) => {
                // Task error - we don't have the original path here, so we'll skip it
                // This shouldn't happen in normal operation
                tracing::error!("Task error during normalization: {}", e);
            }
        }
    }

    Ok((successful, failed))
}

/// Normalizes multiple audio files in parallel (creates temp files for video creation)
pub async fn normalize_audio_files_parallel(
    audio_paths: Vec<String>,
    max_jobs: usize,
    high_pass_filter: bool,
    app_handle: Option<tauri::AppHandle>,
    cancel_token: Option<tokio_util::sync::CancellationToken>,
) -> Result<Vec<(String, String)>> {
    let semaphore = Arc::new(Semaphore::new(max_jobs));
    let mut handles = Vec::new();

    let temp_dir = std::env::temp_dir();

    // Check disk space before starting
    let total_input_size: u64 = audio_paths
        .iter()
        .filter_map(|path| std::fs::metadata(path).ok())
        .map(|m| m.len())
        .sum();
    let estimated_total_output =
        estimate_normalized_audio_size(total_input_size) * audio_paths.len() as u64;
    check_disk_space(&temp_dir, estimated_total_output)
        .with_context(|| "Not enough disk space for audio normalization")?;

    for (index, audio_path) in audio_paths.iter().enumerate() {
        // Check cancellation before spawning each normalization task
        if let Some(ref token) = cancel_token {
            if token.is_cancelled() {
                tracing::info!("Cancellation detected, stopping audio normalization");
                anyhow::bail!("Operation cancelled");
            }
        }

        let audio_path_clone = audio_path.clone();
        let semaphore_clone = semaphore.clone();
        let temp_dir_clone = temp_dir.clone();
        let high_pass_filter_clone = high_pass_filter;
        let app_handle_clone = app_handle.clone();

        let handle = tokio::spawn(async move {
            let _permit = semaphore_clone.acquire().await.unwrap();

            // Create temp output path with unique name to avoid conflicts
            let input_path = Path::new(&audio_path_clone);
            let file_stem = input_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("audio");
            let pid = std::process::id();
            let filename = format!("normalized_{}_{}_{}.wav", pid, index, file_stem);
            let output_path = temp_dir_clone.join(filename);
            let output_path_str = output_path.to_string_lossy().to_string();

            // Normalize the file with timeout - pass AppHandle if available
            let audio_path_for_task = audio_path_clone.clone();
            let output_path_for_task = output_path_str.clone();
            let normalize_task = tokio::task::spawn_blocking(move || {
                normalize_audio_file(
                    &audio_path_for_task,
                    &output_path_for_task,
                    high_pass_filter_clone,
                    app_handle_clone.as_ref(),
                )
            });

            match tokio::time::timeout(timeout::AUDIO_NORMALIZATION, normalize_task).await {
                Ok(Ok(Ok(()))) => Ok((audio_path_clone, output_path_str)),
                Ok(Ok(Err(e))) => Err(format!("Failed to normalize {}: {}", audio_path_clone, e)),
                Ok(Err(e)) => Err(format!("Task error: {}", e)),
                Err(_) => Err(format!(
                    "Audio normalization timed out after 5 minutes: {}",
                    audio_path_clone
                )),
            }
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    let mut results = Vec::new();
    for handle in handles {
        // Check cancellation before waiting for each task result
        if let Some(ref token) = cancel_token {
            if token.is_cancelled() {
                tracing::info!("Cancellation detected while collecting normalization results");
                anyhow::bail!("Operation cancelled");
            }
        }

        match handle.await {
            Ok(Ok((original, normalized))) => results.push((original, normalized)),
            Ok(Err(e)) => anyhow::bail!("{}", e),
            Err(e) => anyhow::bail!("Task error: {}", e),
        }
    }

    Ok(results)
}

/// Normalizes audio files with dual output: temp files for video creation + optional permanent export
/// Returns (temp_file_mapping, exported_files)
pub async fn normalize_audio_files_with_export(
    audio_paths: Vec<String>,
    max_jobs: usize,
    high_pass_filter: bool,
    export_path: Option<String>,
    app_handle: Option<tauri::AppHandle>,
    cancel_token: Option<tokio_util::sync::CancellationToken>,
) -> Result<(Vec<(String, String)>, Vec<String>)> {
    use std::path::Path;

    let semaphore = Arc::new(Semaphore::new(max_jobs));
    let mut handles = Vec::new();

    let temp_dir = std::env::temp_dir();

    // If export is requested, ensure export directory exists
    if let Some(ref export_dir) = export_path {
        let export_path_buf = Path::new(export_dir);
        std::fs::create_dir_all(export_path_buf)
            .with_context(|| format!("Failed to create export directory: {}", export_dir))?;
    }

    // Check disk space before starting
    let total_input_size: u64 = audio_paths
        .iter()
        .filter_map(|path| std::fs::metadata(path).ok())
        .map(|m| m.len())
        .sum();
    let estimated_total_output =
        estimate_normalized_audio_size(total_input_size) * audio_paths.len() as u64;
    
    // Check temp directory space
    check_disk_space(&temp_dir, estimated_total_output)
        .with_context(|| "Not enough disk space for temporary audio normalization files")?;
    
    // Check export directory space if needed
    if let Some(ref export_dir) = export_path {
        check_disk_space(Path::new(export_dir), estimated_total_output)
            .with_context(|| "Not enough disk space for audio export")?;
    }

    for (index, audio_path) in audio_paths.iter().enumerate() {
        // Check cancellation before spawning each normalization task
        if let Some(ref token) = cancel_token {
            if token.is_cancelled() {
                tracing::info!("Cancellation detected, stopping audio normalization");
                anyhow::bail!("Operation cancelled");
            }
        }

        let audio_path_clone = audio_path.clone();
        let semaphore_clone = semaphore.clone();
        let temp_dir_clone = temp_dir.clone();
        let export_path_clone = export_path.clone();
        let high_pass_filter_clone = high_pass_filter;
        let app_handle_clone = app_handle.clone();

        let handle = tokio::spawn(async move {
            let _permit = semaphore_clone.acquire().await.unwrap();

            // Create temp output path with unique name to avoid conflicts
            let input_path = Path::new(&audio_path_clone);
            let file_stem = input_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("audio");
            let pid = std::process::id();
            let temp_filename = format!("normalized_{}_{}_{}.wav", pid, index, file_stem);
            let temp_output_path = temp_dir_clone.join(&temp_filename);

            // Normalize to temp file first
            let temp_result = tokio::task::spawn_blocking({
                let audio_path_clone = audio_path_clone.clone();
                let temp_output_path_clone = temp_output_path.clone();
                let app_handle_clone = app_handle_clone.clone();
                move || {
                    normalize_audio_file(
                        &audio_path_clone,
                        temp_output_path_clone.to_str().unwrap(),
                        high_pass_filter_clone,
                        app_handle_clone.as_ref(),
                    )
                }
            })
            .await;

            match temp_result {
                Ok(Ok(())) => {
                    let temp_path_str = temp_output_path.to_string_lossy().to_string();
                    let mut export_path_opt = None;

                    // If export is requested, also create permanent file
                    if let Some(export_dir) = export_path_clone {
                        let input_path = Path::new(&audio_path_clone);
                        let file_stem = input_path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("audio");
                        let extension = input_path
                            .extension()
                            .and_then(|s| s.to_str())
                            .unwrap_or("wav");
                        
                        let base_name = format!("{}-normalized", file_stem);
                        let export_output_path = find_unique_filename(
                            Path::new(&export_dir),
                            &base_name,
                            extension,
                        );

                        // Normalize to export file with format preservation
                        let export_result = tokio::task::spawn_blocking({
                            let audio_path_clone = audio_path_clone.clone();
                            let export_output_path_clone = export_output_path.clone();
                            let app_handle_clone = app_handle_clone.clone();
                            move || {
                                normalize_audio_file(
                                    &audio_path_clone,
                                    export_output_path_clone.to_str().unwrap(),
                                    high_pass_filter_clone,
                                    app_handle_clone.as_ref(),
                                )
                            }
                        })
                        .await;

                        match export_result {
                            Ok(Ok(())) => {
                                export_path_opt = Some(export_output_path.to_string_lossy().to_string());
                                
                                // Emit progress event for export completion
                                if let Some(ref app_handle) = app_handle_clone {
                                    let _ = app_handle.emit(
                                        "audio-export-progress",
                                        serde_json::json!({
                                            "file": audio_path_clone,
                                            "exported_path": export_output_path.to_string_lossy(),
                                            "message": format!("Exported normalized audio: {}", file_stem)
                                        }),
                                    );
                                }
                            }
                            Ok(Err(e)) => {
                                tracing::warn!("Failed to export audio file {}: {}", audio_path_clone, e);
                                // Continue with temp file even if export fails
                            }
                            Err(e) => {
                                tracing::warn!("Export task failed for {}: {}", audio_path_clone, e);
                                // Continue with temp file even if export fails
                            }
                        }
                    }

                    Ok((audio_path_clone, temp_path_str, export_path_opt))
                }
                Ok(Err(e)) => Err(anyhow::anyhow!("Normalization failed for {}: {}", audio_path_clone, e)),
                Err(e) => Err(anyhow::anyhow!("Task failed for {}: {}", audio_path_clone, e)),
            }
        });

        handles.push(handle);
    }

    // Collect results
    let mut temp_mapping = Vec::new();
    let mut exported_files = Vec::new();

    for handle in handles {
        match handle.await {
            Ok(Ok((original_path, temp_path, export_path_opt))) => {
                temp_mapping.push((original_path, temp_path));
                if let Some(export_path) = export_path_opt {
                    exported_files.push(export_path);
                }
            }
            Ok(Err(e)) => {
                tracing::error!("Audio normalization error: {}", e);
                // Continue processing other files
            }
            Err(e) => {
                tracing::error!("Task join error: {}", e);
                // Continue processing other files
            }
        }
    }

    Ok((temp_mapping, exported_files))
}
