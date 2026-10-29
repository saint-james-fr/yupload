use crate::domain::FileHandler;
use crate::domain::{FileInfo, FolderContents};
use crate::infrastructure::audio_metadata::AudioMetadata;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub struct FileHandlerImpl;

impl FileHandlerImpl {
    fn traverse_directory_recursive(
        &self,
        folder_path: &Path,
        current_depth: u32,
        max_depth: u32,
        images: &mut Vec<FileInfo>,
        audios: &mut Vec<FileInfo>,
        videos: &mut Vec<FileInfo>,
    ) -> Result<()> {
        // Stop if we've reached max depth
        if current_depth > max_depth {
            return Ok(());
        }

        let entries = fs::read_dir(folder_path)
            .with_context(|| format!("Failed to read directory: {}", folder_path.display()))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // Skip if path is a symlink that points to a directory (avoid infinite loops)
            if path.is_symlink() {
                continue;
            }

            if path.is_dir() {
                // Recursively traverse subdirectories
                self.traverse_directory_recursive(
                    &path,
                    current_depth + 1,
                    max_depth,
                    images,
                    audios,
                    videos,
                )?;
            } else if path.is_file() {
                // Process file
                let path_str = path.to_string_lossy().to_string();
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    let is_image = matches!(
                        ext_lower.as_str(),
                        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp"
                    );
                    let is_audio = matches!(
                        ext_lower.as_str(),
                        "mp3" | "wav" | "flac" | "aac" | "m4a" | "ogg"
                    );
                    let is_video = matches!(
                        ext_lower.as_str(),
                        "mp4" | "mov" | "avi" | "mkv" | "webm" | "m4v" | "flv" | "wmv"
                    );

                    let file_info = FileInfo {
                        path: path_str.clone(),
                        name: name.clone(),
                        is_image,
                        is_audio,
                    };

                    if is_image {
                        images.push(file_info);
                    } else if is_audio {
                        audios.push(file_info);
                    } else if is_video {
                        videos.push(file_info);
                    }
                }
            }
        }

        Ok(())
    }
}

impl FileHandler for FileHandlerImpl {
    fn get_audio_title(&self, audio_path: &str) -> Result<String> {
        // Try to extract metadata first
        match AudioMetadata::from_file(audio_path) {
            Ok(metadata) => {
                // Use metadata if available, otherwise fallback to filename
                let fallback = Path::new(audio_path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Untitled");
                Ok(metadata.display_name(fallback))
            }
            Err(_) => {
                // If metadata extraction fails, fallback to filename
                let path = Path::new(audio_path);
                let file_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Untitled");
                Ok(file_name.to_string())
            }
        }
    }

    fn get_audio_metadata(&self, audio_path: &str) -> Result<AudioMetadata> {
        AudioMetadata::from_file(audio_path)
    }

    fn list_folder_contents(&self, folder_path: &str) -> Result<FolderContents> {
        let mut images = Vec::new();
        let mut audios = Vec::new();
        let mut videos = Vec::new();

        let path = Path::new(folder_path);
        const MAX_DEPTH: u32 = crate::utils::file::MAX_DEPTH;

        self.traverse_directory_recursive(
            path,
            0,
            MAX_DEPTH,
            &mut images,
            &mut audios,
            &mut videos,
        )?;

        Ok(FolderContents {
            images,
            audios,
            videos,
        })
    }
}
