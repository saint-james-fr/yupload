use crate::config;
use crate::infrastructure::{DiscogsClient, OAuthCallbackServer, YouTubeClient};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

// Global client state types
pub type YouTubeClientState = Arc<Mutex<Option<YouTubeClient>>>;
pub type DiscogsClientState = Arc<Mutex<Option<DiscogsClient>>>;
pub type OAuthServerState = Arc<OAuthCallbackServer>;

// Cancellation state for long-running operations
// Key format: "operation_type" for global, "operation_type:item_id" for per-item
pub type CancellationState = Arc<Mutex<HashMap<String, CancellationToken>>>;

/// Ensure YouTube client is initialized
pub async fn ensure_youtube_client(
    state: tauri::State<'_, YouTubeClientState>,
) -> Result<(), String> {
    let mut client_guard = state.lock().await;
    if client_guard.is_none() {
        // Get client ID from environment variables
        // No client_secret needed - PKCE-only flow with UWP client type
        // IMPORTANT: Use "UWP" client type in Google Cloud Console, NOT "Desktop app"
        // The "Desktop app" type incorrectly requires client_secret even with PKCE
        let client_id = config::get_youtube_client_id();

        // PKCE-only flow (no client_secret) - works correctly with UWP client type
        let client = YouTubeClient::new(client_id)
            .map_err(|e| format!("Failed to initialize YouTube client: {}", e))?;
        *client_guard = Some(client);
    }
    Ok(())
}

/// Ensure Discogs client is initialized
pub async fn ensure_discogs_client(
    state: tauri::State<'_, DiscogsClientState>,
) -> Result<(), String> {
    let mut client_guard = state.lock().await;
    if client_guard.is_none() {
        // Get credentials from environment variables (will panic if not found)
        let consumer_key = config::get_discogs_consumer_key();
        let consumer_secret = config::get_discogs_consumer_secret();

        let client = DiscogsClient::new(consumer_key, consumer_secret)
            .map_err(|e| format!("Failed to initialize Discogs client: {}", e))?;
        *client_guard = Some(client);
    }
    Ok(())
}
