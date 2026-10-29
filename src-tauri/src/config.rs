// OAuth credentials configuration
// Credentials MUST be provided via environment variables or .env file
// No hardcoded fallbacks - application will fail if credentials are missing

/// Get YouTube OAuth client ID
///
/// According to Google's official documentation:
/// https://developers.google.com/identity/protocols/oauth2/native-app
///
/// Desktop apps "cannot keep secrets" and use PKCE-only flow (no client_secret needed).
///
/// IMPORTANT: When creating OAuth client in Google Cloud Console, use "UWP" (Universal Windows Platform)
/// type instead of "Desktop app" type. The "Desktop app" type incorrectly requires client_secret even
/// with PKCE (Google's quirk), while UWP type correctly supports PKCE-only flow without client_secret.
///
/// # Returns
///
/// Returns the client ID as a String, or panics if not found.
///
/// # Panics
///
/// Panics if neither `YOUTUBE_CLIENT_ID` environment variable nor `YOUTUBE_CLIENT_ID_BUILD`
/// (build-time injected) is available.
pub fn get_youtube_client_id() -> String {
    // Priority order:
    // 1. Runtime environment variable (for development/testing)
    // 2. Build-time environment variable (from .env file baked into binary)
    std::env::var("YOUTUBE_CLIENT_ID")
        .or_else(|_| {
            // Try build-time injected value (from .env loaded during cargo build)
            option_env!("YOUTUBE_CLIENT_ID_BUILD")
                .map(|s| s.to_string())
                .ok_or(())
        })
        .expect("YOUTUBE_CLIENT_ID not found. Please set YOUTUBE_CLIENT_ID environment variable or provide it in .env file during build.")
}

/// Get Discogs OAuth consumer key from environment
///
/// # Returns
///
/// Returns the consumer key as a String, or panics if not found.
///
/// # Panics
///
/// Panics if neither `DISCOGS_CONSUMER_KEY` environment variable nor `DISCOGS_CONSUMER_KEY_BUILD`
/// (build-time injected) is available.
pub fn get_discogs_consumer_key() -> String {
    // Priority: runtime env -> build-time env
    std::env::var("DISCOGS_CONSUMER_KEY")
        .or_else(|_| {
            option_env!("DISCOGS_CONSUMER_KEY_BUILD")
                .map(|s| s.to_string())
                .ok_or(())
        })
        .expect("DISCOGS_CONSUMER_KEY not found. Please set DISCOGS_CONSUMER_KEY environment variable or provide it in .env file during build.")
}

/// Get Discogs OAuth consumer secret from environment
///
/// # Returns
///
/// Returns the consumer secret as a String, or panics if not found.
///
/// # Panics
///
/// Panics if neither `DISCOGS_CONSUMER_SECRET` environment variable nor `DISCOGS_CONSUMER_SECRET_BUILD`
/// (build-time injected) is available.
pub fn get_discogs_consumer_secret() -> String {
    // Priority: runtime env -> build-time env
    std::env::var("DISCOGS_CONSUMER_SECRET")
        .or_else(|_| {
            option_env!("DISCOGS_CONSUMER_SECRET_BUILD")
                .map(|s| s.to_string())
                .ok_or(())
        })
        .expect("DISCOGS_CONSUMER_SECRET not found. Please set DISCOGS_CONSUMER_SECRET environment variable or provide it in .env file during build.")
}
