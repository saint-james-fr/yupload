use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
}

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            fps: crate::utils::video::DEFAULT_FPS,
        }
    }
}

/// How to produce the 16:9 frame from the main image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ImageStyle {
    /// Blurred/darkened background from same image + centered square crop (current behavior).
    Automatic,
    /// User image scaled/cropped to fill 16:9; fit_mode controls contain vs cover.
    Fit16x9,
    /// Centered main image over a custom background (second image or predefined).
    CustomBackground,
    /// Use only the background (no centered main image).
    BackgroundOnly,
}

impl Default for ImageStyle {
    fn default() -> Self {
        Self::Automatic
    }
}

/// For Fit16x9: how to fit the image into 16:9.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FitMode {
    /// Letterbox/pillarbox so the whole image is visible.
    Contain,
    /// Crop to fill 16:9, centered.
    Cover,
}

impl Default for FitMode {
    fn default() -> Self {
        Self::Contain
    }
}

/// Options for process_for_video: fit mode and background source (path/url; predefined id is resolved by caller).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcessOptions {
    #[serde(default)]
    pub fit_mode: Option<FitMode>,
    #[serde(default)]
    pub background_path: Option<String>,
    #[serde(default)]
    pub background_url: Option<String>,
}

/// Per-video image input: main image + style + optional background.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImageInput {
    /// Main/foreground image path (local file).
    #[serde(default)]
    pub image_path: Option<String>,
    /// Main/foreground image URL (when not using path).
    #[serde(default)]
    pub image_url: Option<String>,
    /// How to produce the 16:9 frame. Defaults to Automatic if absent.
    #[serde(default)]
    pub image_style: Option<ImageStyle>,
    /// For Fit16x9: contain or cover. Defaults to Contain if absent.
    #[serde(default)]
    pub fit_mode: Option<FitMode>,
    /// For CustomBackground: path to background image.
    #[serde(default)]
    pub background_path: Option<String>,
    /// For CustomBackground: URL of background image.
    #[serde(default)]
    pub background_url: Option<String>,
    /// For CustomBackground: id of predefined background.
    #[serde(default)]
    pub background_predefined_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoRequest {
    pub image_path: Option<String>,
    pub image_url: Option<String>,
    /// Image style; defaults to Automatic if absent.
    #[serde(default)]
    pub image_style: Option<ImageStyle>,
    #[serde(default)]
    pub fit_mode: Option<FitMode>,
    #[serde(default)]
    pub background_path: Option<String>,
    #[serde(default)]
    pub background_url: Option<String>,
    #[serde(default)]
    pub background_predefined_id: Option<String>,
    pub audio_paths: Vec<String>,
    pub title: String,
    pub output_path: String,
    pub config: VideoConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub is_image: bool,
    pub is_audio: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderContents {
    pub images: Vec<FileInfo>,
    pub audios: Vec<FileInfo>,
    pub videos: Vec<FileInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkVideoRequest {
    /// Per-video image configs. When present, length must match audio_paths.
    #[serde(default)]
    pub image_inputs: Option<Vec<ImageInput>>,
    /// Legacy: when image_inputs is None, use these paths with Automatic style.
    #[serde(default)]
    pub image_paths: Option<Vec<String>>,
    pub audio_paths: Vec<String>,
    pub output_path: String,
    pub config: VideoConfig,
    #[serde(default)]
    pub normalize_audio: Option<bool>,
    #[serde(default)]
    pub high_pass_filter: Option<bool>,
    /// Whether to export normalized audio files to a permanent location
    #[serde(default)]
    pub export_normalized_audio: Option<bool>,
    /// Directory path where normalized audio files should be exported
    #[serde(default)]
    pub audio_export_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkVideoResult {
    pub successful: Vec<String>,
    pub failed: Vec<(String, String)>,
    /// List of successfully exported normalized audio files
    #[serde(default)]
    pub exported_audio_files: Vec<String>,
}
