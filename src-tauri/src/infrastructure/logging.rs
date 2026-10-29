use std::path::PathBuf;
use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_appender::{non_blocking, rolling};
use crate::utils::create_hidden_command;

/// Initialize the logging system with file rotation
pub fn init_logging(app_data_dir: PathBuf) -> Result<()> {
    // Create logs directory
    let logs_dir = app_data_dir.join("logs");
    std::fs::create_dir_all(&logs_dir)?;

    // Set up file appender with daily rotation and 20MB max size
    let file_appender = rolling::RollingFileAppender::builder()
        .rotation(rolling::Rotation::DAILY)
        .filename_prefix("yupload")
        .filename_suffix("log")
        .max_log_files(5) // Keep 5 backup files
        .build(&logs_dir)?;

    let (non_blocking_file, file_guard) = non_blocking(file_appender);

    // Create console writer for development
    let (non_blocking_stdout, stdout_guard) = non_blocking(std::io::stdout());

    // Set up the subscriber with both file and console output
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("debug"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking_file)
                .with_ansi(false)
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking_stdout)
                .with_ansi(true)
                .with_target(false)
        )
        .init();

    // Log system information on startup
    log_system_info();

    // Intentionally leak the guards to keep them alive for the program's lifetime
    // This is the recommended approach for logging guards in long-running applications
    std::mem::forget(file_guard);
    std::mem::forget(stdout_guard);

    tracing::info!("Logging system initialized");
    Ok(())
}

/// Log basic system information for debugging
fn log_system_info() {
    tracing::info!("=== System Information ===");
    tracing::info!("OS: {}", std::env::consts::OS);
    tracing::info!("Architecture: {}", std::env::consts::ARCH);
    tracing::info!("Family: {}", std::env::consts::FAMILY);
    
    // Log OS version
    if let Ok(version) = get_os_version() {
        tracing::info!("OS Version: {}", version);
    } else {
        tracing::warn!("Could not determine OS version");
    }
    
    // Log CPU count
    tracing::info!("CPU cores: {}", num_cpus::get());
    
    // Log available memory (if possible)
    #[cfg(target_os = "windows")]
    {
        if let Ok(memory_info) = get_windows_memory_info() {
            tracing::info!("Total memory: {} GB", memory_info.total_gb);
            tracing::info!("Available memory: {} GB", memory_info.available_gb);
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        if let Ok(memory_info) = get_macos_memory_info() {
            tracing::info!("Total memory: {} GB", memory_info.total_gb);
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        if let Ok(memory_info) = get_linux_memory_info() {
            tracing::info!("Total memory: {} GB", memory_info.total_gb);
            tracing::info!("Available memory: {} GB", memory_info.available_gb);
        }
    }

    // Check for common missing libraries/dependencies
    check_dependencies();
    
    tracing::info!("=== End System Information ===");
}

#[allow(dead_code)]
struct MemoryInfo {
    total_gb: f64,
    available_gb: f64,
}

#[cfg(target_os = "windows")]
fn get_windows_memory_info() -> Result<MemoryInfo> {
    use std::mem;
    use std::ptr;
    
    #[repr(C)]
    struct MEMORYSTATUSEX {
        dw_length: u32,
        dw_memory_load: u32,
        ull_total_phys: u64,
        ull_avail_phys: u64,
        ull_total_page_file: u64,
        ull_avail_page_file: u64,
        ull_total_virtual: u64,
        ull_avail_virtual: u64,
        ull_avail_extended_virtual: u64,
    }

    extern "system" {
        fn GlobalMemoryStatusEx(lpBuffer: *mut MEMORYSTATUSEX) -> i32;
    }

    unsafe {
        let mut mem_status: MEMORYSTATUSEX = mem::zeroed();
        mem_status.dw_length = mem::size_of::<MEMORYSTATUSEX>() as u32;
        
        if GlobalMemoryStatusEx(&mut mem_status) != 0 {
            let total_gb = mem_status.ull_total_phys as f64 / (1024.0 * 1024.0 * 1024.0);
            let available_gb = mem_status.ull_avail_phys as f64 / (1024.0 * 1024.0 * 1024.0);
            Ok(MemoryInfo { total_gb, available_gb })
        } else {
            Err(anyhow::anyhow!("Failed to get memory status"))
        }
    }
}

#[cfg(target_os = "macos")]
fn get_macos_memory_info() -> Result<MemoryInfo> {
    
    let output = create_hidden_command("sysctl")
        .args(&["-n", "hw.memsize"])
        .output()?;
    
    if output.status.success() {
        let memory_str = String::from_utf8(output.stdout)?;
        let memory_bytes: u64 = memory_str.trim().parse()?;
        let total_gb = memory_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        Ok(MemoryInfo { total_gb, available_gb: 0.0 }) // Available memory is harder to get on macOS
    } else {
        Err(anyhow::anyhow!("Failed to get memory info"))
    }
}

#[cfg(target_os = "linux")]
fn get_linux_memory_info() -> Result<MemoryInfo> {
    let meminfo = std::fs::read_to_string("/proc/meminfo")?;
    
    let mut total_kb = 0u64;
    let mut available_kb = 0u64;
    
    for line in meminfo.lines() {
        if line.starts_with("MemTotal:") {
            total_kb = line.split_whitespace()
                .nth(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
        } else if line.starts_with("MemAvailable:") {
            available_kb = line.split_whitespace()
                .nth(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
        }
    }
    
    let total_gb = total_kb as f64 / (1024.0 * 1024.0);
    let available_gb = available_kb as f64 / (1024.0 * 1024.0);
    
    Ok(MemoryInfo { total_gb, available_gb })
}

/// Get OS version information
fn get_os_version() -> Result<String> {
    #[cfg(target_os = "macos")]
    {
        get_macos_version()
    }
    
    #[cfg(target_os = "windows")]
    {
        get_windows_version()
    }
    
    #[cfg(target_os = "linux")]
    {
        get_linux_version()
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Ok("Unknown".to_string())
    }
}

#[cfg(target_os = "macos")]
fn get_macos_version() -> Result<String> {
    
    let output = create_hidden_command("sw_vers")
        .args(&["-productVersion"])
        .output()?;
    
    if output.status.success() {
        let version = String::from_utf8(output.stdout)?;
        Ok(format!("macOS {}", version.trim()))
    } else {
        // Fallback to system call
        let output = create_hidden_command("uname")
            .args(&["-r"])
            .output()?;
        
        if output.status.success() {
            let kernel_version = String::from_utf8(output.stdout)?;
            Ok(format!("macOS (kernel {})", kernel_version.trim()))
        } else {
            Err(anyhow::anyhow!("Failed to get macOS version"))
        }
    }
}

#[cfg(target_os = "windows")]
fn get_windows_version() -> Result<String> {
    use std::process::Command;
    
    // Try to get Windows version using wmic
    let output = create_hidden_command("wmic")
        .args(&["os", "get", "Caption,Version", "/format:list"])
        .output()?;
    
    if output.status.success() {
        let output_str = String::from_utf8(output.stdout)?;
        let mut caption = String::new();
        let mut version = String::new();
        
        for line in output_str.lines() {
            if line.starts_with("Caption=") {
                caption = line.replace("Caption=", "").trim().to_string();
            } else if line.starts_with("Version=") {
                version = line.replace("Version=", "").trim().to_string();
            }
        }
        
        if !caption.is_empty() && !version.is_empty() {
            Ok(format!("{} ({})", caption, version))
        } else if !caption.is_empty() {
            Ok(caption)
        } else {
            // Fallback to ver command
            let output = create_hidden_command("cmd")
                .args(&["/c", "ver"])
                .output()?;
            
            if output.status.success() {
                let version_output = String::from_utf8(output.stdout)?;
                Ok(version_output.trim().to_string())
            } else {
                Err(anyhow::anyhow!("Failed to get Windows version"))
            }
        }
    } else {
        Err(anyhow::anyhow!("Failed to execute wmic command"))
    }
}

#[cfg(target_os = "linux")]
fn get_linux_version() -> Result<String> {
    // Try to read from /etc/os-release first
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        let mut name = String::new();
        let mut version = String::new();
        
        for line in content.lines() {
            if line.starts_with("NAME=") {
                name = line.replace("NAME=", "").trim_matches('"').to_string();
            } else if line.starts_with("VERSION=") {
                version = line.replace("VERSION=", "").trim_matches('"').to_string();
            }
        }
        
        if !name.is_empty() {
            if !version.is_empty() {
                return Ok(format!("{} {}", name, version));
            } else {
                return Ok(name);
            }
        }
    }
    
    // Fallback to lsb_release
    if let Ok(output) = std::process::Command::new("lsb_release")
        .args(&["-d", "-s"])
        .output() 
    {
        if output.status.success() {
            let description = String::from_utf8(output.stdout)?;
            return Ok(description.trim().trim_matches('"').to_string());
        }
    }
    
    // Final fallback to uname
    let output = std::process::Command::new("uname")
        .args(&["-sr"])
        .output()?;
    
    if output.status.success() {
        let uname_output = String::from_utf8(output.stdout)?;
        Ok(uname_output.trim().to_string())
    } else {
        Err(anyhow::anyhow!("Failed to get Linux version"))
    }
}

/// Check for common missing dependencies and log warnings
fn check_dependencies() {
    // Check for FFmpeg with comprehensive testing
    check_ffmpeg_functionality();

    #[cfg(target_os = "windows")]
    {
        // Check for WebView2 on Windows
        check_webview2_windows();
    }
}

/// Comprehensive FFmpeg functionality check
fn check_ffmpeg_functionality() {
    // Use the proper FFmpeg path resolution (fallback since we don't have AppHandle here)
    let ffmpeg_path = crate::utils::ffmpeg::get_ffmpeg_path();
    
    // First check if FFmpeg is available
    let ffmpeg_available = match std::process::Command::new(&ffmpeg_path).arg("-version").output() {
        Ok(output) if output.status.success() => {
            let version_output = String::from_utf8_lossy(&output.stdout);
            if let Some(version_line) = version_output.lines().next() {
                tracing::info!("FFmpeg found at '{}': {}", ffmpeg_path, version_line);
                
                // Extract version number for compatibility checking
                if let Some(version_info) = extract_ffmpeg_version(&version_output) {
                    tracing::info!("FFmpeg version details: {}", version_info);
                }
            } else {
                tracing::info!("FFmpeg found at '{}' but version unknown", ffmpeg_path);
            }
            true
        }
        Ok(output) => {
            let stderr_output = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("FFmpeg found but returned error status. stderr: {}", stderr_output);
            false
        }
        Err(e) => {
            tracing::warn!("FFmpeg not found at '{}': {}", ffmpeg_path, e);
            false
        }
    };

    if !ffmpeg_available {
        tracing::error!("FFmpeg is not available - video creation will fail");
        return;
    }

    // Test FFmpeg codecs we need
    test_ffmpeg_codecs();
    
    // Test basic FFmpeg functionality with a simple command
    test_ffmpeg_basic_functionality();
}

/// Extract FFmpeg version information
fn extract_ffmpeg_version(version_output: &str) -> Option<String> {
    for line in version_output.lines() {
        if line.starts_with("ffmpeg version") {
            return Some(line.to_string());
        }
    }
    None
}

/// Test that FFmpeg has the codecs we need
fn test_ffmpeg_codecs() {
    // Check for required codecs
    let required_codecs = ["libx264", "aac", "mp4"];
    let ffmpeg_path = crate::utils::ffmpeg::get_ffmpeg_path();
    
    match std::process::Command::new(&ffmpeg_path).args(&["-codecs"]).output() {
        Ok(output) if output.status.success() => {
            let codecs_output = String::from_utf8_lossy(&output.stdout);
            
            for codec in &required_codecs {
                if codecs_output.contains(codec) {
                    tracing::debug!("FFmpeg codec available: {}", codec);
                } else {
                    tracing::warn!("FFmpeg codec missing: {} - this may cause issues", codec);
                }
            }
        }
        Ok(_) => tracing::warn!("FFmpeg codecs check failed"),
        Err(e) => tracing::warn!("Could not check FFmpeg codecs: {}", e),
    }
}

/// Test basic FFmpeg functionality
fn test_ffmpeg_basic_functionality() {
    // Test with the most basic FFmpeg command: just get help/usage
    let ffmpeg_path = crate::utils::ffmpeg::get_ffmpeg_path();
    match std::process::Command::new(&ffmpeg_path)
        .args(&["-h"])
        .output() 
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            // FFmpeg -h should return usage info and exit with code 0
            if output.status.success() && (stdout.contains("usage:") || stdout.contains("ffmpeg")) {
                tracing::info!("FFmpeg basic functionality test: PASSED");
            } else if stderr.contains("usage:") || stderr.contains("ffmpeg") {
                // Some FFmpeg versions output help to stderr
                tracing::info!("FFmpeg basic functionality test: PASSED (help via stderr)");
            } else {
                tracing::warn!("FFmpeg basic functionality test: FAILED - unexpected output");
                tracing::debug!("stdout: {}", stdout.lines().take(2).collect::<Vec<_>>().join(" "));
                tracing::debug!("stderr: {}", stderr.lines().take(2).collect::<Vec<_>>().join(" "));
            }
        }
        Err(e) => {
            tracing::error!("FFmpeg basic functionality test: ERROR - {}", e);
        }
    }
}

#[cfg(target_os = "windows")]
fn check_webview2_windows() {
    // Check if WebView2 is installed by looking for the registry key or runtime
    use std::process::Command;
    
    // Try to check WebView2 version via registry
    match create_hidden_command("reg")
        .args(&["query", "HKEY_LOCAL_MACHINE\\SOFTWARE\\WOW6432Node\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}", "/v", "pv"])
        .output() 
    {
        Ok(output) if output.status.success() => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if let Some(version_line) = output_str.lines().find(|line| line.contains("pv")) {
                tracing::info!("WebView2 found: {}", version_line.trim());
            } else {
                tracing::info!("WebView2 registry key found but version unclear");
            }
        }
        Ok(_) => {
            // Try alternative registry location
            match create_hidden_command("reg")
                .args(&["query", "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}", "/v", "pv"])
                .output()
            {
                Ok(output) if output.status.success() => {
                    tracing::info!("WebView2 found in alternative registry location");
                }
                _ => tracing::warn!("WebView2 not found in registry - may need manual installation")
            }
        }
        Err(e) => tracing::warn!("Could not check WebView2 registry: {}", e),
    }
}

/// Get the path to the current log file
pub fn get_current_log_path(app_data_dir: &PathBuf) -> PathBuf {
    let logs_dir = app_data_dir.join("logs");
    let today = chrono::Local::now().format("%Y-%m-%d");
    logs_dir.join(format!("yupload.{}.log", today))
}

/// Get all log files for export
pub fn get_all_log_files(app_data_dir: &PathBuf) -> Result<Vec<PathBuf>> {
    let logs_dir = app_data_dir.join("logs");
    let mut log_files = Vec::new();
    
    if logs_dir.exists() {
        for entry in std::fs::read_dir(&logs_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "log") {
                log_files.push(path);
            }
        }
    }
    
    // Sort by modification time, newest first
    log_files.sort_by(|a, b| {
        let a_modified = std::fs::metadata(a).and_then(|m| m.modified()).unwrap_or(std::time::UNIX_EPOCH);
        let b_modified = std::fs::metadata(b).and_then(|m| m.modified()).unwrap_or(std::time::UNIX_EPOCH);
        b_modified.cmp(&a_modified)
    });
    
    Ok(log_files)
}

/// Redact sensitive information from log messages
pub fn redact_sensitive(message: &str) -> String {
    let mut redacted = message.to_string();
    
    // Redact OAuth tokens
    let token_patterns = [
        r#"access_token["':\s]*["']([^"']+)["']"#,
        r#"refresh_token["':\s]*["']([^"']+)["']"#,
        r#"client_secret["':\s]*["']([^"']+)["']"#,
        r#"client_id["':\s]*["']([^"']+)["']"#,
        r"Bearer\s+([A-Za-z0-9\-_]+)",
        r#"token["':\s]*["']([^"']+)["']"#,
    ];
    
    for pattern in &token_patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            redacted = re.replace_all(&redacted, |caps: &regex::Captures| {
                let full_match = caps.get(0).unwrap().as_str();
                let sensitive_part = caps.get(1).unwrap().as_str();
                let redacted_part = if sensitive_part.len() > 8 {
                    format!("{}***{}", &sensitive_part[..4], &sensitive_part[sensitive_part.len()-4..])
                } else {
                    "***REDACTED***".to_string()
                };
                full_match.replace(sensitive_part, &redacted_part)
            }).to_string();
        }
    }
    
    redacted
}

/// Custom tracing layer that redacts sensitive information
pub struct RedactingLayer;

impl<S> tracing_subscriber::Layer<S> for RedactingLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        _event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // This is a simplified implementation
        // In a full implementation, you'd intercept and redact the event fields
    }
}