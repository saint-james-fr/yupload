use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

/// Get the architecture directory name from Rust's ARCH constant
fn get_arch_dir() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "arm64",
        arch => arch, // Fallback to raw arch name
    }
}

/// Build platform-specific resource path for FFmpeg binary
fn build_resource_path(arch_dir: &str) -> Option<String> {
    match std::env::consts::OS {
        "macos" => Some(format!("binaries/macos/{}/ffmpeg", arch_dir)),
        "linux" => Some(format!("binaries/linux/{}/ffmpeg", arch_dir)),
        "windows" => Some(format!("binaries/windows/{}/ffmpeg.exe", arch_dir)),
        _ => None,
    }
}

/// Build legacy resource path (without architecture subdirectory)
fn build_legacy_resource_path() -> Option<&'static str> {
    match std::env::consts::OS {
        "macos" => Some("binaries/macos/ffmpeg"),
        "linux" => Some("binaries/linux/ffmpeg"),
        "windows" => Some("binaries/windows/ffmpeg.exe"),
        _ => None,
    }
}

/// Try to resolve a resource path using Tauri's PathResolver
fn try_resolve_resource(app_handle: &AppHandle, resource_path: &str) -> Option<String> {
    app_handle
        .path()
        .resolve(resource_path, BaseDirectory::Resource)
        .ok()
        .filter(|path| path.exists())
        .map(|path| path.to_string_lossy().to_string())
}

/// Get FFmpeg executable path using Tauri's PathResolver API (recommended)
/// Tries bundled FFmpeg first (architecture-specific), then falls back to system FFmpeg
pub fn get_ffmpeg_path_from_app(app_handle: &AppHandle) -> String {
    let arch_dir = get_arch_dir();
    tracing::debug!(
        "FFmpeg path resolution started for architecture: {}",
        arch_dir
    );

    // Try architecture-specific path first
    if let Some(resource_path) = build_resource_path(arch_dir) {
        tracing::debug!("Trying architecture-specific path: {}", resource_path);
        if let Some(path) = try_resolve_resource(app_handle, &resource_path) {
            tracing::debug!("FFmpeg found at: {}", path);
            return path;
        }
        tracing::debug!("Architecture-specific path not found");
    }

    // Try legacy path (without architecture subdirectory)
    if let Some(legacy_path) = build_legacy_resource_path() {
        tracing::debug!("Trying legacy path: {}", legacy_path);
        if let Some(path) = try_resolve_resource(app_handle, legacy_path) {
            tracing::debug!("FFmpeg found at legacy path: {}", path);
            return path;
        }
        tracing::debug!("Legacy path not found");
    }

    // Fallback to manual path resolution if PathResolver fails
    tracing::debug!("PathResolver failed, falling back to manual resolution");
    get_ffmpeg_path_fallback()
}

/// Get FFmpeg executable path using manual path resolution (fallback)
/// This is used when AppHandle is not available or PathResolver fails
pub fn get_ffmpeg_path() -> String {
    get_ffmpeg_path_fallback()
}

/// Get FFmpeg path with optional AppHandle support
/// Uses Tauri PathResolver if AppHandle is provided, otherwise falls back to manual resolution
pub fn get_ffmpeg_path_optional(app_handle: Option<&AppHandle>) -> String {
    if let Some(handle) = app_handle {
        get_ffmpeg_path_from_app(handle)
    } else {
        get_ffmpeg_path()
    }
}

/// Internal fallback implementation using manual path construction
/// Matches Tauri's documented resource_dir locations for each platform
fn get_ffmpeg_path_fallback() -> String {
    let arch_dir = get_arch_dir();
    tracing::debug!(
        "FFmpeg fallback resolution started for architecture: {}",
        arch_dir
    );

    // Try to find bundled FFmpeg first
    if let Ok(exe_dir) = std::env::current_exe() {
        tracing::debug!("Current executable: {:?}", exe_dir);
        if let Some(exe_parent) = exe_dir.parent() {
            #[cfg(target_os = "macos")]
            {
                // In macOS app bundles, resources are in Contents/Resources/
                // Executable is in Contents/MacOS/, so we need to go up and into Resources
                let resources_path = exe_parent
                    .parent()
                    .map(|contents| contents.join("Resources"));

                if let Some(resources) = resources_path {
                    // Try architecture-specific path first
                    let bundled = resources.join(format!("binaries/macos/{}/ffmpeg", arch_dir));
                    tracing::debug!("Checking macOS bundled path: {:?}", bundled);
                    if bundled.exists() {
                        let path = bundled.to_string_lossy().to_string();
                        tracing::debug!("FFmpeg found at: {}", path);
                        return path;
                    }
                    // Fallback to legacy path (for backward compatibility)
                    let legacy = resources.join("binaries/macos/ffmpeg");
                    tracing::debug!("Checking macOS legacy path: {:?}", legacy);
                    if legacy.exists() {
                        let path = legacy.to_string_lossy().to_string();
                        tracing::debug!("FFmpeg found at legacy path: {}", path);
                        return path;
                    }
                }

                // Also try relative to executable (for development/non-bundled runs)
                let bundled = exe_parent.join(format!("binaries/macos/{}/ffmpeg", arch_dir));
                tracing::debug!("Checking macOS dev path: {:?}", bundled);
                if bundled.exists() {
                    let path = bundled.to_string_lossy().to_string();
                    tracing::debug!("FFmpeg found at dev path: {}", path);
                    return path;
                }
                let legacy = exe_parent.join("binaries/macos/ffmpeg");
                tracing::debug!("Checking macOS dev legacy path: {:?}", legacy);
                if legacy.exists() {
                    let path = legacy.to_string_lossy().to_string();
                    tracing::debug!("FFmpeg found at dev legacy path: {}", path);
                    return path;
                }
            }
            #[cfg(target_os = "linux")]
            {
                // On Linux, Tauri resource_dir resolves to:
                // - Development: ${exe_dir}/../lib/${exe_name}
                // - AppImage: ${APPDIR}/usr/lib/${exe_name}
                // - Production: /usr/lib/${exe_name}
                // Try multiple locations for maximum compatibility

                // Try development location: ${exe_dir}/../lib/${exe_name}
                if let Some(exe_name) = exe_dir.file_stem() {
                    let lib_dir = exe_parent.join("lib").join(exe_name);
                    let bundled = lib_dir.join(format!("binaries/linux/{}/ffmpeg", arch_dir));
                    tracing::debug!("Checking Linux dev path: {:?}", bundled);
                    if bundled.exists() {
                        let path = bundled.to_string_lossy().to_string();
                        tracing::debug!("FFmpeg found at: {}", path);
                        return path;
                    }
                    let legacy = lib_dir.join("binaries/linux/ffmpeg");
                    tracing::debug!("Checking Linux dev legacy path: {:?}", legacy);
                    if legacy.exists() {
                        let path = legacy.to_string_lossy().to_string();
                        tracing::debug!("FFmpeg found at legacy path: {}", path);
                        return path;
                    }
                }

                // Try AppImage location: ${APPDIR}/usr/lib/${exe_name}
                if let Ok(appdir) = std::env::var("APPDIR") {
                    tracing::debug!("Checking Linux AppImage path (APPDIR={})", appdir);
                    if let Some(exe_name) = exe_dir.file_stem() {
                        let lib_dir = std::path::Path::new(&appdir).join("usr/lib").join(exe_name);
                        let bundled = lib_dir.join(format!("binaries/linux/{}/ffmpeg", arch_dir));
                        tracing::debug!("Checking Linux AppImage bundled path: {:?}", bundled);
                        if bundled.exists() {
                            let path = bundled.to_string_lossy().to_string();
                            tracing::debug!("FFmpeg found at: {}", path);
                            return path;
                        }
                    }
                }

                // Try relative to executable (fallback for custom setups)
                let bundled = exe_parent.join(format!("binaries/linux/{}/ffmpeg", arch_dir));
                tracing::debug!("Checking Linux relative path: {:?}", bundled);
                if bundled.exists() {
                    let path = bundled.to_string_lossy().to_string();
                    tracing::debug!("FFmpeg found at: {}", path);
                    return path;
                }
                let legacy = exe_parent.join("binaries/linux/ffmpeg");
                tracing::debug!("Checking Linux relative legacy path: {:?}", legacy);
                if legacy.exists() {
                    let path = legacy.to_string_lossy().to_string();
                    tracing::debug!("FFmpeg found at legacy path: {}", path);
                    return path;
                }
            }
            #[cfg(target_os = "windows")]
            {
                // On Windows, Tauri resource_dir resolves to the directory containing the executable
                // So resources are at: ${exe_dir}/../binaries/... (exe_parent is the resource dir)

                // Try relative to executable directory (Windows resource_dir = exe_parent)
                let bundled = exe_parent.join(format!("binaries/windows/{}/ffmpeg.exe", arch_dir));
                tracing::debug!("Checking Windows bundled path: {:?}", bundled);
                if bundled.exists() {
                    let path = bundled.to_string_lossy().to_string();
                    tracing::debug!("FFmpeg found at: {}", path);
                    return path;
                }
                let legacy = exe_parent.join("binaries/windows/ffmpeg.exe");
                tracing::debug!("Checking Windows legacy path: {:?}", legacy);
                if legacy.exists() {
                    let path = legacy.to_string_lossy().to_string();
                    tracing::debug!("FFmpeg found at legacy path: {}", path);
                    return path;
                }
            }
        }
    }

    // Fallback to system FFmpeg
    tracing::debug!("No bundled FFmpeg found, falling back to system FFmpeg");
    #[cfg(target_os = "windows")]
    {
        tracing::debug!("Using system FFmpeg: ffmpeg.exe");
        "ffmpeg.exe".to_string()
    }
    #[cfg(not(target_os = "windows"))]
    {
        tracing::debug!("Using system FFmpeg: ffmpeg");
        "ffmpeg".to_string()
    }
}
