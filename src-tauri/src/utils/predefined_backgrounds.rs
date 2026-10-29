//! Predefined background images for custom-background style.
//! Images are auto-discovered from resources/backgrounds/ (jpg, jpeg, png, webp, gif).
//! Optional static PREDEFINED_BACKGROUNDS can override or add entries.
//!
//! In dev, the resource root may not contain bundled files, so we fall back to
//! CARGO_MANIFEST_DIR/resources/backgrounds (i.e. src-tauri/resources/backgrounds).

use std::path::{Path, PathBuf};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};
use std::fs;

/// Allowed image extensions for auto-discovery (lowercase).
const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif"];

/// Relative path to backgrounds dir under Tauri Resource root (matches bundle.resources "resources/**/*").
const RESOURCE_BACKGROUNDS_SUBDIR: &str = "resources/backgrounds";

/// (id, label, filename under resources/backgrounds/)
/// Optional: add entries here to override or supplement auto-discovery.
const PREDEFINED_BACKGROUNDS: &[(&str, &str, &str)] = &[];

/// Returns the path to the predefined backgrounds directory (bundled resources).
/// Tries Tauri Resource first (bundled/production), then CARGO_MANIFEST_DIR/resources/backgrounds (dev).
fn predefined_backgrounds_dir(app_handle: &AppHandle) -> Option<PathBuf> {
    // 1) Bundled: Resource root + "resources/backgrounds" (bundle.resources has "resources/**/*")
    if let Ok(p) = app_handle
        .path()
        .resolve(RESOURCE_BACKGROUNDS_SUBDIR, BaseDirectory::Resource)
    {
        if p.is_dir() {
            return Some(p);
        }
    }
    // 2) Dev fallback: crate root resources/backgrounds (src-tauri/resources/backgrounds)
    let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/backgrounds");
    if dev_path.is_dir() {
        return Some(dev_path);
    }
    None
}

/// Returns the path to the user-imported backgrounds directory (app data).
fn user_backgrounds_dir(app_handle: &AppHandle) -> Option<PathBuf> {
    if let Ok(app_data_dir) = app_handle.path().app_data_dir() {
        let user_bg_dir = app_data_dir.join("backgrounds");
        // Create directory if it doesn't exist
        if let Err(_) = fs::create_dir_all(&user_bg_dir) {
            return None;
        }
        return Some(user_bg_dir);
    }
    None
}


fn is_image_file(name: &str) -> bool {
    Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| IMAGE_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn stem_to_label(stem: &str) -> String {
    stem.replace(['-', '_'], " ")
        .split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().chain(c).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Resolve background id to filesystem path.
/// Tries predefined backgrounds first, then user-imported backgrounds.
pub fn resolve_predefined_background(app_handle: &AppHandle, id: &str) -> Option<String> {
    // Static list: try Resource path then dev fallback dir
    if let Some((_, _, filename)) = PREDEFINED_BACKGROUNDS.iter().find(|(i, _, _)| *i == id) {
        let resource_path = format!("{}/{}", RESOURCE_BACKGROUNDS_SUBDIR, filename);
        if let Ok(p) = app_handle
            .path()
            .resolve(&resource_path, BaseDirectory::Resource)
        {
            if p.exists() {
                return Some(p.to_string_lossy().to_string());
            }
        }
        if let Some(dir) = predefined_backgrounds_dir(app_handle) {
            let p = dir.join(filename);
            if p.is_file() {
                return Some(p.to_string_lossy().to_string());
            }
        }
    }

    // Discovery: find file whose stem == id in predefined backgrounds dir
    if let Some(dir_path) = predefined_backgrounds_dir(app_handle) {
        let entries = std::fs::read_dir(&dir_path).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if stem == id {
                        return Some(path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    // Try user-imported backgrounds
    if let Some(dir_path) = user_backgrounds_dir(app_handle) {
        let entries = std::fs::read_dir(&dir_path).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if stem == id {
                        return Some(path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    None
}

/// List predefined backgrounds for the UI: (id, label, absolute_path).
/// Uses static list if non-empty, otherwise auto-discovers image files in resources/backgrounds/.
pub fn list_predefined_backgrounds_with_paths(
    app_handle: &AppHandle,
) -> Vec<(String, String, Option<String>)> {
    let from_static: Vec<(String, String, Option<String>)> = PREDEFINED_BACKGROUNDS
        .iter()
        .map(|(id, label, _filename)| {
            let path = resolve_predefined_background(app_handle, id);
            (id.to_string(), label.to_string(), path)
        })
        .collect();

    if !from_static.is_empty() {
        return from_static
            .into_iter()
            .filter_map(|(id, label, path)| path.map(|p| (id, label, p)))
            .map(|(id, label, p)| (id, label, Some(p)))
            .collect();
    }

    // Auto-discover from resources/backgrounds/ (Resource or dev fallback)
    let Some(dir_path) = predefined_backgrounds_dir(app_handle) else {
        return vec![];
    };
    let Ok(entries) = std::fs::read_dir(&dir_path) else {
        return vec![];
    };

    let mut out: Vec<(String, String, Option<String>)> = entries
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            if !path.is_file() {
                return None;
            }
            let name = path.file_name().and_then(|n| n.to_str())?;
            if !is_image_file(name) {
                return None;
            }
            let stem = path.file_stem().and_then(|s| s.to_str())?.to_string();
            let label = stem_to_label(&stem);
            let path_str = path.to_string_lossy().to_string();
            Some((stem, label, Some(path_str)))
        })
        .collect();

    out.sort_by(|a, b| a.1.cmp(&b.1));
    out
}

/// List user-imported backgrounds for the UI: (id, label, absolute_path).
/// Auto-discovers image files in app data/backgrounds/.
pub fn list_user_backgrounds_with_paths(
    app_handle: &AppHandle,
) -> Vec<(String, String, Option<String>)> {
    let Some(dir_path) = user_backgrounds_dir(app_handle) else {
        return vec![];
    };
    let Ok(entries) = std::fs::read_dir(&dir_path) else {
        return vec![];
    };

    let mut out: Vec<(String, String, Option<String>)> = entries
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            if !path.is_file() {
                return None;
            }
            let name = path.file_name().and_then(|n| n.to_str())?;
            if !is_image_file(name) {
                return None;
            }
            let stem = path.file_stem().and_then(|s| s.to_str())?.to_string();
            let label = stem_to_label(&stem);
            let path_str = path.to_string_lossy().to_string();
            Some((stem, label, Some(path_str)))
        })
        .collect();

    out.sort_by(|a, b| a.1.cmp(&b.1));
    out
}

/// List predefined backgrounds (id, label only). For compatibility.
pub fn list_predefined_backgrounds() -> Vec<(String, String)> {
    PREDEFINED_BACKGROUNDS
        .iter()
        .map(|(id, label, _)| (id.to_string(), label.to_string()))
        .collect()
}

/// Save an image file to the user-imported backgrounds directory.
/// Returns the id (filename stem) of the saved background.
pub fn save_predefined_background(
    app_handle: &AppHandle,
    source_path: &str,
) -> Result<String, String> {
    // Validate source path
    let source = PathBuf::from(source_path);
    if !source.is_file() {
        return Err(format!("Source file does not exist: {}", source_path));
    }
    
    // Get the filename and extension
    let filename = source
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Invalid filename".to_string())?;
    
    if !is_image_file(filename) {
        return Err(format!("File is not a supported image format: {}", filename));
    }
    
    // Get the user backgrounds directory (create if needed)
    let dir_path = user_backgrounds_dir(app_handle)
        .ok_or_else(|| "Could not access or create user backgrounds directory".to_string())?;
    
    // Generate a unique filename if one already exists
    let mut dest_path = dir_path.join(filename);
    let mut counter = 1;
    while dest_path.exists() {
        let stem = source
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("image");
        let ext = source
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png");
        let new_filename = format!("{}_{}.{}", stem, counter, ext);
        dest_path = dir_path.join(new_filename);
        counter += 1;
    }
    
    // Copy the file
    fs::copy(&source, &dest_path)
        .map_err(|e| format!("Failed to copy file: {}", e))?;
    
    // Return the id (filename stem without extension)
    dest_path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Failed to generate background id".to_string())
}
