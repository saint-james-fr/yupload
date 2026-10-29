use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use lofty::prelude::*;
use lofty::read_from_path;
use std::path::Path;

/// Audio metadata extracted from ID3 tags or other metadata formats
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioMetadata {
    pub artist: Option<String>,
    pub title: Option<String>,
    pub album: Option<String>,
    /// Base64-encoded album art image data (as data URL)
    pub image_data_url: Option<String>,
}

impl AudioMetadata {
    /// Extract metadata from an audio file
    pub fn from_file(audio_path: &str) -> Result<Self> {
        let path = Path::new(audio_path);

        // Try to read metadata using lofty
        let tagged_file = read_from_path(path)
            .with_context(|| format!("Failed to read audio file: {}", audio_path))?;

        // Get the primary tag (first tag, usually the most relevant)
        let tag = tagged_file.primary_tag();

        let artist = tag.as_ref().and_then(|t| t.artist()).map(|s| s.to_string());

        let title = tag.as_ref().and_then(|t| t.title()).map(|s| s.to_string());

        let album = tag.as_ref().and_then(|t| t.album()).map(|s| s.to_string());

        // Extract album art (first picture) as base64 data URL
        // Only include album art if we have proper metadata (artist or title)
        // This prevents using placeholder/random images when metadata is missing
        let has_proper_metadata = artist.is_some() || title.is_some();
        let image_data_url = if has_proper_metadata {
            tag.as_ref().and_then(|t| {
                let pictures = t.pictures();
                pictures.first().map(|pic| {
                    // Determine mime type - convert MimeType to string
                    let mime_type_str = pic
                        .mime_type()
                        .map(|mt| mt.as_str())
                        .unwrap_or("image/jpeg");
                    // Encode image data as base64
                    let base64_data = STANDARD.encode(pic.data());
                    // Create data URL
                    format!("data:{};base64,{}", mime_type_str, base64_data)
                })
            })
        } else {
            None
        };

        Ok(AudioMetadata {
            artist,
            title,
            album,
            image_data_url,
        })
    }

    /// Generate a display name from metadata: "artist - title" or fallback to title or filename
    pub fn display_name(&self, fallback: &str) -> String {
        match (&self.artist, &self.title) {
            (Some(artist), Some(title)) if !artist.is_empty() && !title.is_empty() => {
                format!("{} - {}", artist.trim(), title.trim())
            }
            (_, Some(title)) if !title.is_empty() => title.trim().to_string(),
            (Some(artist), _) if !artist.is_empty() => artist.trim().to_string(),
            _ => fallback.to_string(),
        }
    }

    /// Check if metadata is available (at least artist or title)
    pub fn has_metadata(&self) -> bool {
        self.artist.is_some() || self.title.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_name() {
        let metadata = AudioMetadata {
            artist: Some("Artist Name".to_string()),
            title: Some("Song Title".to_string()),
            album: None,
            image_data_url: None,
        };
        assert_eq!(
            metadata.display_name("fallback"),
            "Artist Name - Song Title"
        );

        let metadata = AudioMetadata {
            artist: None,
            title: Some("Song Title".to_string()),
            album: None,
            image_data_url: None,
        };
        assert_eq!(metadata.display_name("fallback"), "Song Title");

        let metadata = AudioMetadata {
            artist: Some("Artist Name".to_string()),
            title: None,
            album: None,
            image_data_url: None,
        };
        assert_eq!(metadata.display_name("fallback"), "Artist Name");

        let metadata = AudioMetadata {
            artist: None,
            title: None,
            album: None,
            image_data_url: None,
        };
        assert_eq!(metadata.display_name("fallback"), "fallback");
    }
}
