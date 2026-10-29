#[macro_use]
pub mod logging;

pub mod concurrency;
pub mod constants;
pub mod encryption;
pub mod ffmpeg;
pub mod file_ops;
pub mod path;
pub mod predefined_backgrounds;
pub mod process;
pub mod text;
pub mod validation;

pub use concurrency::calculate_optimal_concurrency;
pub use constants::{cleanup, file, progress, rate_limit, retry, timeout, token, video, window};
pub use encryption::{decrypt_token, encrypt_token};
pub use ffmpeg::{get_ffmpeg_path, get_ffmpeg_path_from_app, get_ffmpeg_path_optional};
pub use file_ops::{
    atomic_write, check_disk_space, cleanup_old_temp_files, estimate_normalized_audio_size,
    estimate_video_size, validate_file_extension, validate_file_size, TempFile,
    MAX_AUDIO_SIZE_BYTES, MAX_IMAGE_SIZE_BYTES, MAX_VIDEO_SIZE_BYTES,
};
pub use path::{
    download_image_safely, fetch_image_as_data_url, find_unique_filename, validate_image_url,
    validate_user_path, validate_user_path_str,
};
pub use process::create_hidden_command;
pub use text::clean_name;
pub use validation::{
    validate_search_query, validate_youtube_description, validate_youtube_tags,
    validate_youtube_title,
};
