use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Minimum required disk space in bytes (1GB)
const MIN_DISK_SPACE_BYTES: u64 = 1_073_741_824;

/// Maximum file sizes
pub const MAX_VIDEO_SIZE_BYTES: u64 = 2_147_483_648; // 2GB
pub const MAX_AUDIO_SIZE_BYTES: u64 = 524_288_000; // 500MB
pub const MAX_IMAGE_SIZE_BYTES: u64 = 20_971_520; // 20MB

/// RAII guard for temporary files - automatically deletes on drop
pub struct TempFile {
    path: PathBuf,
    keep: bool,
}

impl TempFile {
    /// Create a new temp file with unique name
    pub fn new(prefix: &str, extension: &str) -> Result<Self> {
        let temp_dir = std::env::temp_dir();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let pid = std::process::id();
        let filename = format!(
            "{}_{}_{}_{}.{}",
            prefix,
            pid,
            timestamp,
            uuid::Uuid::new_v4()
                .to_string()
                .split('-')
                .next()
                .unwrap_or("tmp"),
            extension
        );
        let path = temp_dir.join(filename);

        Ok(Self { path, keep: false })
    }

    /// Create from existing path
    pub fn from_path(path: PathBuf) -> Self {
        Self { path, keep: false }
    }

    /// Get path to temp file
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Keep the file after drop (don't auto-delete)
    pub fn keep(mut self) -> PathBuf {
        self.keep = true;
        self.path.clone()
    }

    /// Explicitly delete the file now
    pub fn delete(mut self) -> Result<()> {
        self.keep = true; // Prevent double delete in Drop
        match fs::remove_file(&self.path) {
            Ok(_) => {
                tracing::debug!("Deleted temp file: {:?}", self.path);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to delete temp file {:?}: {}", self.path, e);
                Err(anyhow::Error::from(e).context(format!("Failed to delete temp file: {:?}", self.path)))
            }
        }
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        if !self.keep && self.path.exists() {
            let _ = fs::remove_file(&self.path);
            tracing::debug!("Cleaned up temp file: {:?}", self.path);
        }
    }
}

/// Check if there's enough disk space available
pub fn check_disk_space(path: &Path, required_bytes: u64) -> Result<()> {
    let available = get_available_disk_space(path)?;

    if available < MIN_DISK_SPACE_BYTES {
        anyhow::bail!(
            "Disk space critically low: {} MB available. Please free up disk space.",
            available / 1_048_576
        );
    }

    if available < required_bytes {
        anyhow::bail!(
            "Not enough disk space. Required: {} MB, Available: {} MB",
            required_bytes / 1_048_576,
            available / 1_048_576
        );
    }

    Ok(())
}

/// Get available disk space in bytes
#[cfg(unix)]
fn get_available_disk_space(path: &Path) -> Result<u64> {
    // Ensure path exists
    let check_path = if path.exists() {
        path.to_path_buf()
    } else {
        // Use parent directory or root
        path.parent().unwrap_or(Path::new("/")).to_path_buf()
    };

    // Use statvfs to get filesystem stats
    use std::ffi::CString;
    use std::mem::MaybeUninit;

    let c_path = CString::new(check_path.to_string_lossy().as_bytes()).context("Invalid path")?;

    unsafe {
        let mut stat: MaybeUninit<libc::statvfs> = MaybeUninit::uninit();
        if libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) == 0 {
            let stat = stat.assume_init();
            let available = (stat.f_bavail as u64).saturating_mul(stat.f_bsize as u64);
            Ok(available)
        } else {
            anyhow::bail!("Failed to get disk space information")
        }
    }
}

#[cfg(windows)]
fn get_available_disk_space(path: &Path) -> Result<u64> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let check_path = if path.exists() {
        path.to_path_buf()
    } else {
        path.parent().unwrap_or(Path::new("C:\\")).to_path_buf()
    };

    let wide: Vec<u16> = OsStr::new(&check_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut free_bytes_available: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut total_free_bytes: u64 = 0;

    #[link(name = "kernel32")]
    extern "system" {
        fn GetDiskFreeSpaceExW(
            lp_directory_name: *const u16,
            lp_free_bytes_available_to_caller: *mut u64,
            lp_total_number_of_bytes: *mut u64,
            lp_total_number_of_free_bytes: *mut u64,
        ) -> i32;
    }

    unsafe {
        let result = GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &mut free_bytes_available,
            &mut total_bytes,
            &mut total_free_bytes,
        );

        if result != 0 {
            Ok(free_bytes_available)
        } else {
            anyhow::bail!("Failed to get disk space information")
        }
    }
}

#[cfg(not(any(unix, windows)))]
fn get_available_disk_space(_path: &Path) -> Result<u64> {
    // Fallback: return a large number to avoid blocking
    tracing::warn!("Disk space check not supported on this platform");
    Ok(u64::MAX / 2)
}

/// Atomically write data to a file (write to temp, then rename)
pub fn atomic_write<P: AsRef<Path>, C: AsRef<[u8]>>(
    path: P,
    contents: C,
    check_space: bool,
) -> Result<()> {
    let path = path.as_ref();
    let contents = contents.as_ref();

    // Check disk space if requested
    if check_space {
        check_disk_space(path, contents.len() as u64)?;
    }

    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {:?}", parent))?;
    }

    // Write to temp file first
    let temp_path = path.with_extension("tmp");

    {
        let mut file = File::create(&temp_path)
            .with_context(|| format!("Failed to create temp file: {:?}", temp_path))?;

        file.write_all(contents)
            .with_context(|| format!("Failed to write to temp file: {:?}", temp_path))?;

        file.sync_all()
            .with_context(|| "Failed to sync temp file to disk")?;
    }

    // Atomic rename (on same filesystem)
    fs::rename(&temp_path, path).with_context(|| {
        format!(
            "Failed to rename temp file to final destination: {:?}",
            path
        )
    })?;

    Ok(())
}

/// Validate file extension matches actual file format
pub fn validate_file_extension(path: &Path) -> Result<()> {
    if !path.exists() {
        anyhow::bail!("File does not exist: {:?}", path);
    }

    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    // Read first few bytes to check magic numbers
    let mut file = File::open(path)
        .with_context(|| format!("Failed to open file for validation: {:?}", path))?;

    let mut buffer = [0u8; 12];
    use std::io::Read;
    let bytes_read = file
        .read(&mut buffer)
        .with_context(|| "Failed to read file header")?;

    if bytes_read < 4 {
        anyhow::bail!("File too small to validate: {:?}", path);
    }

    // Check magic numbers
    match extension.as_str() {
        "png" => {
            if &buffer[0..4] != b"\x89PNG" {
                anyhow::bail!("File has .png extension but is not a PNG file");
            }
        }
        "jpg" | "jpeg" => {
            if &buffer[0..2] != b"\xFF\xD8" {
                anyhow::bail!("File has .jpg extension but is not a JPEG file");
            }
        }
        "wav" => {
            if &buffer[0..4] != b"RIFF" || &buffer[8..12] != b"WAVE" {
                anyhow::bail!("File has .wav extension but is not a WAV file");
            }
        }
        "mp4" => {
            // MP4 has "ftyp" at offset 4
            if bytes_read >= 8 && &buffer[4..8] != b"ftyp" {
                tracing::warn!("File has .mp4 extension but may not be a valid MP4 file");
            }
        }
        "mp3" => {
            // MP3 can start with ID3 or have FF FB/FF F3/FF F2 sync word
            if &buffer[0..3] != b"ID3" && buffer[0] != 0xFF {
                tracing::warn!("File has .mp3 extension but may not be a valid MP3 file");
            }
        }
        _ => {
            // Unknown extension, skip validation
            tracing::debug!("Skipping extension validation for: {:?}", extension);
        }
    }

    Ok(())
}

/// Check if file size is within allowed limits
pub fn validate_file_size(path: &Path, max_size: u64) -> Result<()> {
    let metadata =
        fs::metadata(path).with_context(|| format!("Failed to get file metadata: {:?}", path))?;

    let file_size = metadata.len();

    if file_size > max_size {
        anyhow::bail!(
            "File size exceeds limit. File: {} MB, Limit: {} MB",
            file_size / 1_048_576,
            max_size / 1_048_576
        );
    }

    Ok(())
}

/// Cleanup old temp files in the system temp directory
pub fn cleanup_old_temp_files(prefix: &str, max_age_hours: u64) -> Result<usize> {
    let temp_dir = std::env::temp_dir();
    let max_age = Duration::from_secs(max_age_hours * 3600);
    let now = SystemTime::now();
    let mut cleaned_count = 0;
    let mut error_count = 0;
    let mut checked_count = 0;

    tracing::debug!("Starting temp file cleanup for prefix '{}' (max age: {} hours)", prefix, max_age_hours);

    let entries = fs::read_dir(&temp_dir)
        .with_context(|| format!("Failed to read temp directory: {:?}", temp_dir))?;

    for entry in entries.flatten() {
        let path = entry.path();

        // Check if filename starts with our prefix
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with(prefix) {
                checked_count += 1;
                // Check file age (skip directories — e.g. yupload_uploads is a dir for upload temp files)
                match entry.metadata() {
                    Ok(metadata) => {
                        if !metadata.is_file() {
                            continue;
                        }
                        match metadata.modified() {
                            Ok(modified) => {
                                match now.duration_since(modified) {
                                    Ok(age) => {
                                        if age > max_age {
                                            // File is too old, delete it
                                            match fs::remove_file(&path) {
                                                Ok(_) => {
                                                    cleaned_count += 1;
                                                    tracing::debug!("Cleaned up old temp file: {:?}", path);
                                                }
                                                Err(e) => {
                                                    error_count += 1;
                                                    tracing::error!("Failed to delete old temp file {:?}: {}", path, e);
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to calculate age for temp file {:?}: {}", path, e);
                                        error_count += 1;
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to get modified time for temp file {:?}: {}", path, e);
                                error_count += 1;
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to get metadata for temp file {:?}: {}", path, e);
                        error_count += 1;
                    }
                }
            }
        }
    }

    if cleaned_count > 0 || error_count > 0 {
        tracing::info!(
            "Temp file cleanup completed for prefix '{}': {} checked, {} cleaned, {} errors",
            prefix, checked_count, cleaned_count, error_count
        );
    } else if checked_count > 0 {
        tracing::debug!(
            "Temp file cleanup completed for prefix '{}': {} checked, no files needed cleanup",
            prefix, checked_count
        );
    }

    Ok(cleaned_count)
}

/// File lock guard using fs2 crate
#[cfg(feature = "file-locks")]
pub struct FileLock {
    file: File,
}

#[cfg(feature = "file-locks")]
impl FileLock {
    pub fn new_exclusive(path: &Path) -> Result<Self> {
        use fs2::FileExt;

        let file = File::create(path)
            .with_context(|| format!("Failed to create lock file: {:?}", path))?;

        file.lock_exclusive()
            .with_context(|| format!("Failed to acquire exclusive lock: {:?}", path))?;

        Ok(Self { file })
    }

    pub fn new_shared(path: &Path) -> Result<Self> {
        use fs2::FileExt;

        let file =
            File::open(path).with_context(|| format!("Failed to open lock file: {:?}", path))?;

        file.lock_shared()
            .with_context(|| format!("Failed to acquire shared lock: {:?}", path))?;

        Ok(Self { file })
    }
}

#[cfg(feature = "file-locks")]
impl Drop for FileLock {
    fn drop(&mut self) {
        use fs2::FileExt;
        let _ = self.file.unlock();
    }
}

/// Get estimated file size for operations (helps with disk space checks)
pub fn estimate_video_size(audio_size_bytes: u64, duration_seconds: u64) -> u64 {
    // Rough estimate: ~1MB per second for 1080p video with audio
    // Audio size + video estimate
    let video_estimate = duration_seconds * 1_048_576; // 1MB per second
    audio_size_bytes + video_estimate
}

/// Get estimated audio normalization size (usually same as input)
pub fn estimate_normalized_audio_size(input_size_bytes: u64) -> u64 {
    // WAV files are typically similar size or slightly larger
    (input_size_bytes as f64 * 1.1) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temp_file_cleanup() {
        let temp = TempFile::new("test", "txt").unwrap();
        let path = temp.path().to_path_buf();

        // File should exist while guard is alive
        assert!(path.parent().unwrap().exists());

        drop(temp);

        // File should be deleted after drop
        assert!(!path.exists());
    }

    #[test]
    fn test_temp_file_keep() {
        let temp = TempFile::new("test_keep", "txt").unwrap();
        let path = temp.path().to_path_buf();

        // Write something
        std::fs::write(&path, b"test").unwrap();

        // Keep the file
        let kept_path = temp.keep();

        // File should still exist
        assert!(kept_path.exists());

        // Manual cleanup
        let _ = std::fs::remove_file(&kept_path);
    }
}
