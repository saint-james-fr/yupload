use std::time::Duration;

/// Timeout constants for various operations
pub mod timeout {
    use super::Duration;

    /// Image processing timeout (1 minute)
    pub const IMAGE_PROCESSING: Duration = Duration::from_secs(60);

    /// Image thumbnail generation timeout (30 seconds)
    pub const IMAGE_THUMBNAIL: Duration = Duration::from_secs(30);

    /// Image thumbnail save timeout (10 seconds)
    pub const IMAGE_THUMBNAIL_SAVE: Duration = Duration::from_secs(10);

    /// Video thumbnail extraction timeout (30 seconds)
    pub const VIDEO_THUMBNAIL: Duration = Duration::from_secs(30);

    /// Audio normalization timeout per file (5 minutes)
    pub const AUDIO_NORMALIZATION: Duration = Duration::from_secs(300);

    /// Video creation timeout per video (10 minutes)
    pub const VIDEO_CREATION: Duration = Duration::from_secs(600);

    /// HTTP request timeout (30 seconds)
    pub const HTTP_REQUEST: Duration = Duration::from_secs(30);

    /// YouTube API request timeout (5 minutes for large uploads)
    pub const YOUTUBE_API: Duration = Duration::from_secs(300);

    /// OAuth token verification timeout (5 minutes)
    pub const OAUTH_VERIFICATION: Duration = Duration::from_secs(300);

    /// Discogs API request timeout (30 seconds)
    pub const DISCOGS_API: Duration = Duration::from_secs(30);
}

/// Progress percentage constants for UI updates
pub mod progress {
    /// Initial progress (10%)
    pub const INITIAL: u8 = 10;

    /// Image processing complete (30%)
    pub const IMAGE_PROCESSED: u8 = 30;

    /// Image processing in progress (20%)
    pub const IMAGE_PROCESSING: u8 = 20;

    /// Video encoding started (50%)
    pub const VIDEO_ENCODING: u8 = 50;

    /// Operation complete (100%)
    pub const COMPLETE: u8 = 100;
}

/// Rate limiting constants
pub mod rate_limit {
    /// Discogs API rate limit window in seconds (60 seconds)
    pub const DISCOGS_WINDOW_SECS: u64 = 60;

    /// Discogs API authenticated requests per window
    pub const DISCOGS_AUTHENTICATED_LIMIT: u32 = 60;

    /// Discogs API unauthenticated requests per window
    pub const DISCOGS_UNAUTHENTICATED_LIMIT: u32 = 25;

    /// Minimum remaining requests before waiting for reset
    pub const DISCOGS_WAIT_THRESHOLD: u32 = 5;
}

/// Token expiry constants
pub mod token {
    /// Buffer time before token expiry to consider it expired (60 seconds)
    pub const EXPIRY_BUFFER_SECS: i64 = 60;
}

/// File processing constants
pub mod file {
    /// Maximum directory traversal depth
    pub const MAX_DEPTH: u32 = 10;

    /// Maximum number of files that can be processed in a single operation (videos or audio)
    /// This limit helps prevent overwhelming the system and hitting YouTube API quotas
    pub const MAX_FILES_PER_OPERATION: usize = 100;
}

/// Video configuration defaults
pub mod video {
    /// Default frames per second
    pub const DEFAULT_FPS: u32 = 30;
}

/// Retry and backoff constants
pub mod retry {
    use super::Duration;

    /// Retry delay for YouTube API (1 second)
    pub const YOUTUBE_RETRY_DELAY: Duration = Duration::from_secs(1);
}

/// Main window dimensions (used when creating the app window)
pub mod window {
    /// Main window default width
    pub const MAIN_WIDTH: f64 = 1070.0;

    /// Main window default height
    pub const MAIN_HEIGHT: f64 = 670.0;
}

/// Cleanup and maintenance constants
pub mod cleanup {
    use super::Duration;

    /// Periodic temp file cleanup interval (1 hour)
    pub const TEMP_FILE_CLEANUP_INTERVAL: Duration = Duration::from_secs(3600);

    /// Maximum age for temp files before cleanup (24 hours)
    pub const TEMP_FILE_MAX_AGE_HOURS: u64 = 24;
}
