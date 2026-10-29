use crate::domain::{ImageStyle, ProcessOptions, VideoRequest};
use anyhow::Result;

pub trait ImageProcessor: Send + Sync {
    fn process_image_for_youtube(
        &self,
        input_path: &str,
        output_path: &str,
        width: u32,
        height: u32,
    ) -> Result<()>;

    fn download_and_process_image(
        &self,
        url: &str,
        output_path: &str,
        width: u32,
        height: u32,
    ) -> Result<()>;

    /// Process main image for video with style and options (fit mode, custom background).
    /// main_path or main_url must be set; background_path/background_url used for CustomBackground.
    fn process_for_video(
        &self,
        main_path: Option<&str>,
        main_url: Option<&str>,
        style: ImageStyle,
        options: &ProcessOptions,
        output_path: &str,
        width: u32,
        height: u32,
    ) -> Result<()>;
}

pub trait VideoCreator: Send + Sync {
    fn create_video(
        &self,
        request: &VideoRequest,
        app_handle: Option<&tauri::AppHandle>,
        max_threads: Option<usize>,
    ) -> Result<String>;
}

pub trait FileHandler: Send + Sync {
    fn get_audio_title(&self, audio_path: &str) -> Result<String>;
    fn get_audio_metadata(&self, audio_path: &str) -> Result<crate::infrastructure::audio_metadata::AudioMetadata>;
    fn list_folder_contents(&self, folder_path: &str) -> Result<crate::domain::FolderContents>;
}
