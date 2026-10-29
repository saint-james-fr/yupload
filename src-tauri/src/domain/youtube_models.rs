use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoUploadRequest {
    pub video_path: String,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub category_id: String,
    pub privacy_status: String, // "public", "private", "unlisted"
    pub self_declared_made_for_kids: bool,
    /// Schedule publish time (ISO 8601). Only used when privacy_status is "private".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publish_at: Option<String>,
    /// If set, add the uploaded video to this playlist (playlist ID).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub playlist_id: Option<String>,
}

/// Playlist item returned by playlists.list (mine=true).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubePlaylist {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkVideoUploadRequest {
    pub videos: Vec<VideoUploadRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoUploadResult {
    pub video_id: String,
    pub video_url: String,
}

impl VideoUploadResult {
    pub fn new(video_id: String) -> Self {
        Self {
            video_id: video_id.clone(),
            video_url: format!("https://www.youtube.com/watch?v={}", video_id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkVideoUploadResult {
    pub successful: Vec<VideoUploadResult>,
    pub failed: Vec<(String, String)>, // (video_path, error_message)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<i64>, // Unix timestamp
    pub token_type: String,
}

/// Video processing status from YouTube API
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum VideoProcessingStatus {
    Uploaded,
    Processing,
    Processed,
    Failed,
    Deleted,
}

/// Video processing status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoProcessingStatusResponse {
    pub video_id: String,
    pub status: VideoProcessingStatus,
    pub failure_reason: Option<String>,
    pub checked_at: String, // ISO 8601 timestamp
}
