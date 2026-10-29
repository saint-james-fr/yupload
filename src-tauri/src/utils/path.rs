use anyhow::{Context, Result};
use reqwest::blocking::Client;
use std::path::Path;
use url::Url;

/// Validate user-provided path to prevent path traversal attacks
///
/// This is a basic defense-in-depth measure. Since this is a desktop app where
/// users select files through OS dialogs, the risk is low, but this prevents
/// accidental issues with malicious file names or path manipulation bugs.
///
/// Returns true if the path appears safe, false if it contains suspicious patterns.
pub fn validate_user_path(path: &Path) -> bool {
    // Check for obvious path traversal sequences (defense in depth)
    let path_str = path.to_string_lossy();

    // Reject paths containing ".." (parent directory traversal)
    if path_str.contains("..") {
        return false;
    }

    // Reject paths with null bytes (potential for path manipulation)
    if path_str.contains('\0') {
        return false;
    }

    true
}

/// Validate a path string and return an error if invalid
pub fn validate_user_path_str(path: &str) -> Result<(), String> {
    let path = Path::new(path);
    if !validate_user_path(path) {
        return Err("Invalid path: contains path traversal sequences or null bytes".to_string());
    }
    Ok(())
}

/// Validate an image URL to prevent security issues
///
/// According to security best practices:
/// - Only HTTPS URLs are allowed (not HTTP)
/// - Block file:// and other non-HTTP schemes
/// - Prevent SSRF attacks by validating the scheme
///
/// Returns Ok(()) if valid, Err with message if invalid
pub fn validate_image_url(url: &str) -> Result<(), String> {
    let parsed = Url::parse(url).map_err(|e| format!("Invalid URL format: {}", e))?;

    // Only HTTPS (not HTTP) - more secure
    if parsed.scheme() != "https" {
        return Err("Only HTTPS URLs are allowed for security".to_string());
    }

    Ok(())
}

/// Maximum size for downloaded images (20MB)
const MAX_IMAGE_SIZE: u64 = 20 * 1024 * 1024;

/// Download an image from a URL with security limits.
/// Returns (bytes, content_type) e.g. ("image/jpeg").
fn download_image_safely_with_content_type(url: &str) -> Result<(Vec<u8>, String)> {
    validate_image_url(url).map_err(|e| anyhow::anyhow!("URL validation failed: {}", e))?;

    let client = Client::builder()
        .timeout(crate::utils::timeout::HTTP_REQUEST)
        .build()
        .context("Failed to create HTTP client")?;

    let response = client
        .get(url)
        .header("Range", format!("bytes=0-{}", MAX_IMAGE_SIZE))
        .send()
        .context("Failed to download image")?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "HTTP error: {} - {}",
            response.status(),
            response.status().canonical_reason().unwrap_or("Unknown")
        ));
    }

    let content_type: String = response
        .headers()
        .get("content-type")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("image/jpeg")
        .to_string();

    if !content_type.starts_with("image/") {
        return Err(anyhow::anyhow!(
            "URL does not point to an image. Content-Type: {:?}",
            content_type
        ));
    }

    let bytes = response.bytes().context("Failed to read image bytes")?;
    if bytes.len() as u64 > MAX_IMAGE_SIZE {
        return Err(anyhow::anyhow!(
            "Image too large: {} bytes (max: {} bytes)",
            bytes.len(),
            MAX_IMAGE_SIZE
        ));
    }

    let mime = content_type.split(';').next().unwrap_or("image/jpeg").trim();
    Ok((bytes.to_vec(), mime.to_string()))
}

/// Download an image from a URL with security limits (HTTPS, timeout, size limit, Content-Type check).
/// Returns the image bytes if successful.
pub fn download_image_safely(url: &str) -> Result<Vec<u8>> {
    download_image_safely_with_content_type(url).map(|(bytes, _)| bytes)
}

/// Download an image from a URL and return it as a data URL for use in the frontend (avoids CORS).
pub fn fetch_image_as_data_url(url: &str) -> Result<String> {
    use base64::{engine::general_purpose, Engine as _};
    let (bytes, mime) = download_image_safely_with_content_type(url)?;
    let b64 = general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:{};base64,{}", mime, b64))
}

/// Finds a unique filename by appending a suffix (-1, -2, etc.) if the file already exists
///
/// This is useful for preventing file overwrites when saving files with user-provided names.
/// The function will try suffixes from 1 to 1000, and if all are taken, it falls back to
/// using a timestamp suffix.
///
/// # Arguments
///
/// * `output_dir` - The directory where the file will be saved
/// * `base_name` - The base filename without extension
/// * `extension` - The file extension (without the dot)
///
/// # Returns
///
/// A `PathBuf` pointing to a unique filename that doesn't exist yet
pub fn find_unique_filename(
    output_dir: &Path,
    base_name: &str,
    extension: &str,
) -> std::path::PathBuf {
    let base_path = output_dir.join(format!("{}.{}", base_name, extension));

    // If the file doesn't exist, return it as-is
    if !base_path.exists() {
        return base_path;
    }

    // Try with suffix -1, -2, etc.
    for i in 1..=1000 {
        let candidate = output_dir.join(format!("{}-{}.{}", base_name, i, extension));
        if !candidate.exists() {
            return candidate;
        }
    }

    // If we've exhausted all options (unlikely), return the original with a timestamp
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    output_dir.join(format!("{}-{}.{}", base_name, timestamp, extension))
}
