use crate::domain::youtube_models::*;
use crate::utils::{retry, timeout, token};
use anyhow::{Context, Result};
use bytes::Bytes;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use reqwest::Client;
use serde_json::json;
use std::sync::Arc;
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_store::StoreExt;
use tokio::sync::oneshot;

const YOUTUBE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const YOUTUBE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const YOUTUBE_UPLOAD_URL: &str = "https://www.googleapis.com/upload/youtube/v3/videos";
const YOUTUBE_DATA_API_BASE: &str = "https://www.googleapis.com/youtube/v3";

/// Scheme used for deep links; must match tauri.conf.json plugins.deep-link.desktop.schemes.
const DEEP_LINK_SCHEME: &str = "yupload";

/// OAuth callback server state
pub struct OAuthCallbackServer {
    pub port: u16,
    pub sender: Arc<tokio::sync::Mutex<Option<oneshot::Sender<String>>>>,
    // Store PKCE verifier secret as string for YouTube OAuth (None for Discogs OAuth 1.0a)
    pub pkce_verifier_secret: Arc<tokio::sync::Mutex<Option<String>>>,
}

impl OAuthCallbackServer {
    /// Start the OAuth callback server (should be called once at app startup)
    pub fn start() -> Result<Self> {
        let sender = Arc::new(tokio::sync::Mutex::new(None));
        let pkce_verifier_secret = Arc::new(tokio::sync::Mutex::new(None));

        // Spawn persistent server task
        let sender_clone = sender.clone();
        let pkce_verifier_secret_clone = pkce_verifier_secret.clone();
        let (port_tx, port_rx) = std::sync::mpsc::channel();

        // Create a runtime handle or use the existing one
        // Since we're in a sync context, we'll spawn on the existing runtime
        // If no runtime exists, we'll create a new one in a separate thread
        std::thread::spawn(move || {
            // Create a new runtime for this thread
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            rt.block_on(async {
                // Use async TcpListener directly
                let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
                    Ok(l) => l,
                    Err(_) => {
                        return;
                    }
                };

                let port = match listener.local_addr() {
                    Ok(addr) => addr.port(),
                    Err(_) => {
                        return;
                    }
                };

                // Send port back to main thread
                let _ = port_tx.send(port);

                // Accept connections continuously
                loop {
                    match listener.accept().await {
                        Ok((stream, _addr)) => {
                            let io = TokioIo::new(stream);
                            let sender = sender_clone.clone();

                            let pkce_verifier_secret = pkce_verifier_secret_clone.clone();
                            let service = service_fn(move |req: Request<hyper::body::Incoming>| {
                                let sender = sender.clone();
                                let pkce_verifier_secret = pkce_verifier_secret.clone();
                                async move {
                                    handle_oauth_callback(req, sender, pkce_verifier_secret).await
                                }
                            });

                            if let Err(err) =
                                http1::Builder::new().serve_connection(io, service).await
                            {
                                tracing::error!("OAuth callback connection error: {}", err);
                            }
                        }
                        Err(_e) => {
                            tokio::time::sleep(retry::YOUTUBE_RETRY_DELAY).await;
                        }
                    }
                }
            });
        });

        // Wait for the server to start and get the port
        let port = port_rx
            .recv()
            .context("Failed to receive port from server task")?;

        Ok(Self {
            port,
            sender,
            pkce_verifier_secret,
        })
    }
}

pub struct YouTubeClient {
    oauth_client: BasicClient,
    http_client: Client,
    client_id: String,
}

impl YouTubeClient {
    /// Create a new YouTube client using PKCE flow (no client_secret required)
    ///
    /// According to OAuth 2.0 best practices and PKCE specification:
    /// - Desktop apps are public clients and should NOT store secrets
    /// - PKCE eliminates the need for client_secret
    ///
    /// IMPORTANT: When creating OAuth client in Google Cloud Console, use "UWP" (Universal Windows Platform)
    /// type instead of "Desktop app" type. The "Desktop app" type incorrectly requires client_secret even
    /// with PKCE (Google's quirk), while UWP type correctly supports PKCE-only flow.
    pub fn new(client_id: String) -> Result<Self> {
        let client_id_oauth = ClientId::new(client_id.clone());
        let auth_url =
            AuthUrl::new(YOUTUBE_AUTH_URL.to_string()).context("Invalid authorization URL")?;
        let token_url =
            TokenUrl::new(YOUTUBE_TOKEN_URL.to_string()).context("Invalid token URL")?;

        // Create OAuth client without client_secret (PKCE-only flow)
        // Redirect URI will be set dynamically during authentication
        let oauth_client = BasicClient::new(
            client_id_oauth,
            None, // No client_secret - PKCE-only flow
            auth_url,
            Some(token_url),
        );

        let http_client = Client::builder()
            .timeout(timeout::YOUTUBE_API)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            oauth_client,
            http_client,
            client_id,
        })
    }

    pub async fn authenticate(
        &self,
        app_handle: tauri::AppHandle,
        callback_server: &OAuthCallbackServer,
    ) -> Result<AuthToken> {
        tracing::info!("Starting YouTube OAuth authentication");
        let redirect_uri = format!("http://127.0.0.1:{}/callback", callback_server.port);
        tracing::debug!("OAuth redirect URI: {}", redirect_uri);

        // Create a new channel for this authentication attempt
        let (tx, rx) = oneshot::channel::<String>();
        {
            let mut sender_guard = callback_server.sender.lock().await;
            *sender_guard = Some(tx);
        }

        // Generate PKCE code challenge and verifier
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Store the verifier secret as string so callback handler can retrieve it
        {
            let mut verifier_guard = callback_server.pkce_verifier_secret.lock().await;
            *verifier_guard = Some(pkce_verifier.secret().to_string());
        }

        // Create redirect URL and update OAuth client
        let redirect_url =
            RedirectUrl::new(redirect_uri.clone()).context("Invalid redirect URL")?;
        let oauth_client = self.oauth_client.clone().set_redirect_uri(redirect_url);

        // Generate authorization URL with PKCE challenge
        let (auth_url, _csrf_token) = oauth_client
            .authorize_url(oauth2::CsrfToken::new_random)
            .set_pkce_challenge(pkce_challenge)
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/youtube.upload".to_string(),
            ))
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/youtube".to_string(),
            ))
            .url();

        // Open browser using Tauri opener plugin
        tracing::debug!("Opening browser for OAuth authorization");
        match app_handle
            .opener()
            .open_url(&auth_url.to_string(), None::<&str>)
        {
            Ok(_) => {}
            Err(_) => {
                // Fallback to system command
                #[cfg(target_os = "macos")]
                {
                    use crate::utils::create_hidden_command;
                    let _ = create_hidden_command("open").arg(&auth_url.to_string()).spawn();
                }
                #[cfg(target_os = "linux")]
                {
                    use crate::utils::create_hidden_command;
                    let _ = create_hidden_command("xdg-open").arg(&auth_url.to_string()).spawn();
                }
                #[cfg(target_os = "windows")]
                {
                    use crate::utils::create_hidden_command;
                    let _ = create_hidden_command("cmd")
                        .args(["/C", "start", &auth_url.to_string()])
                        .spawn();
                }
            }
        }

        // Wait for callback with timeout
        tracing::debug!(
            "Waiting for OAuth callback (timeout: {}s)",
            timeout::OAUTH_VERIFICATION.as_secs()
        );
        let code = tokio::time::timeout(timeout::OAUTH_VERIFICATION, async {
            rx.await
                .map_err(|e| anyhow::anyhow!("Channel error: {}", e))
        })
        .await
        .context("Authentication timeout - please try again")?
        .context("Failed to receive authorization code")?;

        if code.is_empty() {
            tracing::error!("OAuth authentication was cancelled or failed");
            anyhow::bail!("Authentication was cancelled or failed");
        }

        tracing::debug!("Received authorization code (length: {})", code.len());

        // Retrieve the PKCE verifier secret and reconstruct the verifier
        let verifier_secret = {
            let mut verifier_guard = callback_server.pkce_verifier_secret.lock().await;
            verifier_guard.take().context("PKCE verifier not found")?
        };
        let verifier = PkceCodeVerifier::new(verifier_secret);

        // Exchange code for token using PKCE
        tracing::debug!("Exchanging authorization code for access token");
        let token = self
            .exchange_code(code, &redirect_uri, verifier, &app_handle)
            .await?;
        tracing::info!("YouTube OAuth authentication successful");
        Ok(token)
    }

    async fn exchange_code(
        &self,
        code: String,
        redirect_uri: &str,
        pkce_verifier: PkceCodeVerifier,
        app_handle: &tauri::AppHandle,
    ) -> Result<AuthToken> {
        tracing::debug!("Exchanging OAuth code for token (PKCE flow)");
        // Use OAuth2 crate's built-in PKCE token exchange
        // With UWP client type, this works without client_secret (proper PKCE flow)
        let redirect_url =
            RedirectUrl::new(redirect_uri.to_string()).context("Invalid redirect URL")?;
        let oauth_client = self.oauth_client.clone().set_redirect_uri(redirect_url);

        // Exchange authorization code for token using PKCE
        // The oauth2 crate automatically includes client_id from BasicClient
        // With UWP client type, client_secret is not required (PKCE-only flow)
        let token_result = oauth_client
            .exchange_code(oauth2::AuthorizationCode::new(code.clone()))
            .set_pkce_verifier(pkce_verifier)
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .map_err(|e| {
                // Extract detailed error information from oauth2 error
                let error_details = match &e {
                    oauth2::RequestTokenError::ServerResponse(err) => {
                        format!(
                            "Server error: {} - {}",
                            err.error(),
                            err.error_description().map(|d| d.as_str()).unwrap_or("No description")
                        )
                    }
                    oauth2::RequestTokenError::Request(req_err) => {
                        format!("Request error: {}", req_err)
                    }
                    oauth2::RequestTokenError::Parse(parse_err, _) => {
                        format!("Parse error: {}", parse_err)
                    }
                    oauth2::RequestTokenError::Other(msg) => {
                        format!("Other error: {}", msg)
                    }
                };
                
                anyhow::anyhow!(
                    "Failed to exchange authorization code. {}. \
                    Code length: {}, Redirect URI: {}, Client ID: {}. \
                    Please ensure: redirect_uri is registered in Google Cloud Console, \
                    OAuth app is configured as 'UWP' type (NOT 'Desktop app' - that type has a quirk), \
                    and PKCE is enabled.",
                    error_details, code.len(), redirect_uri, 
                    crate::infrastructure::logging::redact_sensitive(&self.client_id)
                )
            })?;

        // Extract token information from StandardTokenResponse
        let access_token = token_result.access_token().secret().to_string();
        let refresh_token = token_result
            .refresh_token()
            .map(|rt| rt.secret().to_string());

        // Calculate expiration time
        let expires_at = token_result
            .expires_in()
            .map(|duration| chrono::Utc::now().timestamp() + duration.as_secs() as i64);

        let auth_token = AuthToken {
            access_token,
            refresh_token,
            expires_at,
            token_type: "Bearer".to_string(),
        };

        tracing::debug!("Token exchange successful, saving token");
        self.save_token(&auth_token, &app_handle).await?;
        Ok(auth_token)
    }

    async fn load_token(&self, app_handle: &tauri::AppHandle) -> Result<Option<AuthToken>> {
        tracing::debug!("Loading YouTube token from store");
        let path = std::path::PathBuf::from("yupload.store");
        let store = app_handle.store(path).context("Failed to get store")?;

        if let Some(value) = store.get("youtube_token") {
            // Try to decrypt if it's an encrypted string, otherwise fall back to plain JSON
            let token = if let Some(encrypted_str) = value.as_str() {
                // Decrypt and parse
                let decrypted_json = crate::utils::decrypt_token(encrypted_str)
                    .with_context(|| {
                        tracing::error!("Failed to decrypt YouTube token");
                        "Failed to decrypt token"
                    })?;
                serde_json::from_str(&decrypted_json)
                    .with_context(|| {
                        tracing::error!("Failed to parse decrypted YouTube token");
                        "Failed to parse decrypted token"
                    })?
            } else {
                // Legacy format: try to parse as JSON value directly
                serde_json::from_value(value.clone())
                    .with_context(|| {
                        tracing::error!("Failed to parse stored YouTube token (legacy format)");
                        "Failed to parse stored token"
                    })?
            };
            tracing::debug!("YouTube token loaded successfully");
            Ok(Some(token))
        } else {
            tracing::debug!("No YouTube token found in store");
            Ok(None)
        }
    }

    async fn save_token(&self, token: &AuthToken, app_handle: &tauri::AppHandle) -> Result<()> {
        tracing::debug!("Saving YouTube token to store");
        let path = std::path::PathBuf::from("yupload.store");
        let store = app_handle.store(path).context("Failed to get store")?;

        // Serialize to JSON string, then encrypt
        let token_json = serde_json::to_string(token)
            .with_context(|| {
                tracing::error!("Failed to serialize YouTube token");
                "Failed to serialize token"
            })?;
        let encrypted = crate::utils::encrypt_token(&token_json)
            .with_context(|| {
                tracing::error!("Failed to encrypt YouTube token");
                "Failed to encrypt token"
            })?;

        // Store encrypted string as JSON string value
        store.set(
            "youtube_token".to_string(),
            serde_json::Value::String(encrypted),
        );
        store.save().context("Failed to save token to store")?;
        tracing::debug!("YouTube token saved successfully");

        Ok(())
    }

    async fn get_valid_token(&self, app_handle: &tauri::AppHandle) -> Result<String> {
        let mut token = self
            .load_token(app_handle)
            .await?
            .context("No token found. Please authenticate first.")?;

        // Check if token is expired and refresh if needed
        if let Some(expires_at) = token.expires_at {
            if chrono::Utc::now().timestamp() >= expires_at - token::EXPIRY_BUFFER_SECS {
                // Token expires in less than 1 minute, refresh it
                tracing::debug!("Access token expired or expiring soon, refreshing");
                if let Some(refresh_token) = &token.refresh_token {
                    match self
                        .refresh_access_token(refresh_token.clone(), app_handle)
                        .await
                    {
                        Ok(new_token) => {
                            token = new_token;
                            tracing::info!("Access token refreshed successfully");
                        }
                        Err(e) => {
                            // Refresh failed - error message already includes context
                            // Token may have been cleared by refresh_access_token if invalid
                            return Err(e);
                        }
                    }
                } else {
                    tracing::error!("Token expired and no refresh token available");
                    // Clear the invalid token since we can't refresh it
                    if let Err(clear_err) = self.disconnect(app_handle).await {
                        tracing::error!("Failed to clear invalid token: {}", clear_err);
                    }
                    anyhow::bail!(
                        "Your YouTube connection has expired. Please reconnect your YouTube account in Settings."
                    );
                }
            } else {
                tracing::debug!("Access token is still valid, no refresh needed");
            }
        } else {
            tracing::debug!("Access token has no expiration time, assuming valid");
        }

        Ok(token.access_token)
    }

    async fn refresh_access_token(
        &self,
        refresh_token: String,
        app_handle: &tauri::AppHandle,
    ) -> Result<AuthToken> {
        tracing::debug!("Refreshing YouTube access token");
        
        // Validate inputs
        if refresh_token.is_empty() {
            tracing::error!("Refresh token is empty");
            anyhow::bail!("Refresh token is empty. Please reconnect your YouTube account.");
        }
        
        if self.client_id.is_empty() {
            tracing::error!("Client ID is empty");
            anyhow::bail!("Client ID is not configured. Please check your configuration.");
        }
        
        tracing::debug!("Refresh token length: {}, Client ID length: {}", refresh_token.len(), self.client_id.len());
        
        // Build refresh token request - no client_secret needed with UWP client type
        let params = [
            ("client_id", self.client_id.as_str()),
            ("refresh_token", &refresh_token),
            ("grant_type", "refresh_token"),
        ];

        tracing::debug!("Sending refresh token request to {}", YOUTUBE_TOKEN_URL);
        let response = self
            .http_client
            .post(YOUTUBE_TOKEN_URL)
            .form(&params)
            .send()
            .await
            .context("Failed to send refresh token request")?;

        let status = response.status();
        let status_code = status.as_u16();
        tracing::debug!("Refresh token response status: {}", status_code);
        
        if !status.is_success() {
            // Try to parse error as JSON first (Google OAuth errors are JSON)
            let error_text = match response.text().await {
                Ok(text) => {
                    tracing::debug!("Refresh token error response body: {}", 
                                   crate::infrastructure::logging::redact_sensitive(&text));
                    text
                }
                Err(e) => {
                    tracing::error!("Failed to read error response body: {}", e);
                    format!("HTTP {} - Could not read error response", status_code)
                }
            };
            
            let error_message = if error_text.is_empty() {
                format!("HTTP {} - Empty response body", status_code)
            } else if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
                // Extract error and error_description from JSON
                let error_code = error_json["error"].as_str().unwrap_or("unknown_error");
                let error_desc = error_json["error_description"]
                    .as_str()
                    .unwrap_or("No description provided");
                tracing::debug!("Parsed error JSON - code: {}, description: {}", error_code, error_desc);
                format!("{}: {}", error_code, error_desc)
            } else {
                // Fallback to raw text if not JSON
                tracing::debug!("Error response is not JSON, using raw text");
                error_text.clone()
            };
            
            tracing::error!("Failed to refresh token: HTTP {} - {}", status_code, error_message);
            
            // Check if refresh token is invalid/revoked (HTTP 400 with "invalid_grant")
            // This means the user needs to re-authenticate
            let is_invalid_grant = status_code == 400 && (
                error_message.to_lowercase().contains("invalid_grant") || 
                error_text.to_lowercase().contains("\"error\":\"invalid_grant\"") ||
                error_text.to_lowercase().contains("invalid_grant")
            );
            
            if is_invalid_grant {
                tracing::info!("Refresh token is invalid or revoked (HTTP 400), clearing stored token");
                // Clear the invalid token
                if let Err(e) = self.disconnect(app_handle).await {
                    tracing::error!("Failed to clear invalid token: {}", e);
                }
                anyhow::bail!(
                    "Your YouTube connection has expired. Please reconnect your YouTube account in Settings. Error: {}",
                    error_message
                );
            }
            
            anyhow::bail!("Failed to refresh token (HTTP {}): {}", status_code, error_message);
        }

        let token_data: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse refresh response JSON")?;

        tracing::debug!("Refresh token response parsed successfully");

        // Validate required fields
        if !token_data["access_token"].is_string() {
            tracing::error!("Refresh response missing access_token field");
            anyhow::bail!("Invalid refresh response: missing access_token");
        }

        // Load existing token to preserve refresh_token if not provided
        let mut existing_token = self
            .load_token(app_handle)
            .await?
            .unwrap_or_else(|| AuthToken {
                access_token: String::new(),
                refresh_token: Some(refresh_token.clone()),
                expires_at: None,
                token_type: "Bearer".to_string(),
            });

        existing_token.access_token = token_data["access_token"]
            .as_str()
            .context("No access_token in refresh response")?
            .to_string();
        
        // Parse expires_in (typically 3600 seconds = 1 hour)
        existing_token.expires_at = token_data["expires_in"]
            .as_u64()
            .map(|d| {
                let expires_at = chrono::Utc::now().timestamp() + d as i64;
                tracing::debug!("New access token expires in {} seconds (at timestamp {})", d, expires_at);
                expires_at
            });
        
        // Keep existing refresh_token if new one not provided
        // Google may or may not return a new refresh_token
        if let Some(new_refresh) = token_data["refresh_token"].as_str() {
            tracing::debug!("New refresh token provided in response");
            existing_token.refresh_token = Some(new_refresh.to_string());
        } else {
            tracing::debug!("No new refresh token in response, keeping existing one");
        }

        self.save_token(&existing_token, app_handle).await?;
        tracing::info!("Token refreshed and saved successfully");
        Ok(existing_token)
    }

    pub async fn upload_video(
        &self,
        request: &VideoUploadRequest,
        app_handle: &tauri::AppHandle,
    ) -> Result<VideoUploadResult> {
        tracing::info!("Starting YouTube video upload: '{}'", request.title);
        tracing::debug!("Video path: {}, Size: checking...", request.video_path);
        let access_token = self.get_valid_token(app_handle).await?;

        // Step 1: Upload video file
        let video_data = tokio::fs::read(&request.video_path)
            .await
            .with_context(|| format!("Failed to read video file: {}", request.video_path))?;

        tracing::debug!("Video file read successfully: {} bytes", video_data.len());

        // Step 2: Create metadata JSON
        let mut status = json!({
            "privacyStatus": request.privacy_status,
            "selfDeclaredMadeForKids": request.self_declared_made_for_kids
        });
        // Scheduling: publishAt only valid when privacy is "private"
        if request.privacy_status == "private" {
            if let Some(ref publish_at) = request.publish_at {
                if !publish_at.is_empty() {
                    status["publishAt"] = json!(publish_at);
                }
            }
        }

        let metadata = json!({
            "snippet": {
                "title": request.title,
                "description": request.description,
                "tags": request.tags,
                "categoryId": request.category_id
            },
            "status": status
        });

        // Step 3: Upload using resumable upload protocol
        // First, initialize the upload
        tracing::debug!("Initializing YouTube resumable upload");
        let init_response = self
            .http_client
            .post(format!(
                "{}?uploadType=resumable&part=snippet,status",
                YOUTUBE_UPLOAD_URL
            ))
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json; charset=UTF-8")
            .header("X-Upload-Content-Type", "video/*")
            .header("X-Upload-Content-Length", video_data.len())
            .json(&metadata)
            .send()
            .await
            .context("Failed to initialize upload")?;

        let init_status = init_response.status();
        if !init_status.is_success() {
            let error_text = init_response.text().await.unwrap_or_default();
            tracing::error!(
                "Failed to initialize YouTube upload: HTTP {} - {}",
                init_status,
                error_text
            );
            anyhow::bail!("Failed to initialize upload: {}", error_text);
        }

        tracing::debug!("Upload initialized successfully, got upload URL");
        let upload_url = init_response
            .headers()
            .get("Location")
            .and_then(|h| h.to_str().ok())
            .context("No upload URL in response")?
            .to_string();

        // Step 4: Upload the video file
        tracing::debug!("Uploading video file ({} bytes)", video_data.len());
        let upload_response = self
            .http_client
            .put(&upload_url)
            .header("Content-Type", "video/*")
            .header("Content-Length", video_data.len())
            .body(video_data)
            .send()
            .await
            .context("Failed to upload video")?;

        let upload_status = upload_response.status();
        if !upload_status.is_success() {
            let error_text = upload_response.text().await.unwrap_or_default();
            tracing::error!(
                "Video upload failed: HTTP {} - {}",
                upload_status,
                error_text
            );
            anyhow::bail!("Failed to upload video: {}", error_text);
        }

        tracing::debug!("Video upload completed successfully");
        let response_json: serde_json::Value = upload_response
            .json()
            .await
            .context("Failed to parse upload response")?;

        let video_id = response_json["id"]
            .as_str()
            .context("No video ID in response")?
            .to_string();

        tracing::info!("YouTube video uploaded successfully: {}", video_id);

        // Optionally add to playlist (playlistItems.insert)
        if let Some(ref playlist_id) = request.playlist_id {
            if !playlist_id.is_empty() {
                if let Err(e) = self
                    .add_video_to_playlist(playlist_id, &video_id, app_handle)
                    .await
                {
                    tracing::error!("Failed to add video to playlist: {}", e);
                    // Don't fail the upload; video is already uploaded
                } else {
                    tracing::info!("Video {} added to playlist {}", video_id, playlist_id);
                }
            }
        }

        Ok(VideoUploadResult {
            video_id: video_id.clone(),
            video_url: format!("https://www.youtube.com/watch?v={}", video_id),
        })
    }

    /// List the authenticated user's playlists (for "add to playlist").
    pub async fn list_playlists(
        &self,
        app_handle: &tauri::AppHandle,
    ) -> Result<Vec<YouTubePlaylist>> {
        let access_token = self.get_valid_token(app_handle).await?;
        let url = format!(
            "{}?part=snippet,id&mine=true&maxResults=50",
            format!("{}/playlists", YOUTUBE_DATA_API_BASE)
        );
        let resp = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .context("Failed to list playlists")?;
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("playlists.list failed: {}", text);
        }
        let data: serde_json::Value = resp
            .json()
            .await
            .context("Failed to parse playlists response")?;
        let items = data["items"]
            .as_array()
            .context("No items in playlists response")?;
        let playlists: Vec<YouTubePlaylist> = items
            .iter()
            .filter_map(|item| {
                let id = item["id"].as_str()?;
                let title = item["snippet"]["title"].as_str().unwrap_or("").to_string();
                Some(YouTubePlaylist {
                    id: id.to_string(),
                    title,
                })
            })
            .collect();
        Ok(playlists)
    }

    /// Add an uploaded video to a playlist (playlistItems.insert).
    pub async fn add_video_to_playlist(
        &self,
        playlist_id: &str,
        video_id: &str,
        app_handle: &tauri::AppHandle,
    ) -> Result<()> {
        let access_token = self.get_valid_token(app_handle).await?;
        let url = format!(
            "{}?part=snippet",
            format!("{}/playlistItems", YOUTUBE_DATA_API_BASE)
        );
        let body = json!({
            "snippet": {
                "playlistId": playlist_id,
                "resourceId": {
                    "kind": "youtube#video",
                    "videoId": video_id
                }
            }
        });
        let resp = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to add video to playlist")?;
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("playlistItems.insert failed: {}", text);
        }
        Ok(())
    }

    /// Check the processing status of an uploaded video
    pub async fn check_video_processing_status(
        &self,
        video_id: &str,
        app_handle: &tauri::AppHandle,
    ) -> Result<VideoProcessingStatusResponse> {
        let access_token = self.get_valid_token(app_handle).await?;
        let url = format!(
            "{}?part=processingDetails&id={}",
            format!("{}/videos", YOUTUBE_DATA_API_BASE),
            video_id
        );

        tracing::debug!("Checking processing status for video: {}", video_id);
        let resp = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .context("Failed to check video processing status")?;

        if !resp.status().is_success() {
            let status_code = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            
            // 404 or 403 might indicate video was deleted or is inaccessible
            if status_code == 404 || status_code == 403 {
                let checked_at = chrono::Utc::now().to_rfc3339();
                return Ok(VideoProcessingStatusResponse {
                    video_id: video_id.to_string(),
                    status: VideoProcessingStatus::Deleted,
                    failure_reason: Some(format!("Video not found or inaccessible (HTTP {})", status_code)),
                    checked_at,
                });
            }
            
            anyhow::bail!("videos.list failed: {}", text);
        }

        let data: serde_json::Value = resp
            .json()
            .await
            .context("Failed to parse video status response")?;

        let items = data["items"]
            .as_array()
            .context("No items in video status response")?;

        if items.is_empty() {
            // Video not found - likely deleted
            let checked_at = chrono::Utc::now().to_rfc3339();
            return Ok(VideoProcessingStatusResponse {
                video_id: video_id.to_string(),
                status: VideoProcessingStatus::Deleted,
                failure_reason: Some("Video not found on YouTube. It may have been deleted.".to_string()),
                checked_at,
            });
        }

        let item = &items[0];
        let processing_details = item.get("processingDetails");

        let status = if let Some(details) = processing_details {
            let status_str = details["processingStatus"]
                .as_str()
                .unwrap_or("unknown")
                .to_lowercase();

            match status_str.as_str() {
                "uploaded" => VideoProcessingStatus::Uploaded,
                "processing" => VideoProcessingStatus::Processing,
                "processed" | "succeeded" => VideoProcessingStatus::Processed,
                "failed" => VideoProcessingStatus::Failed,
                _ => {
                    tracing::warn!("Unknown processing status: {}", status_str);
                    VideoProcessingStatus::Processing // Default to processing for unknown
                }
            }
        } else {
            // If processingDetails is not available, assume processed (older videos)
            VideoProcessingStatus::Processed
        };

        let failure_reason = processing_details
            .and_then(|d| d["processingFailureReason"].as_str())
            .map(|s| s.to_string());

        let checked_at = chrono::Utc::now().to_rfc3339();

        Ok(VideoProcessingStatusResponse {
            video_id: video_id.to_string(),
            status,
            failure_reason,
            checked_at,
        })
    }

    pub async fn check_auth_status(&self, app_handle: &tauri::AppHandle) -> Result<bool> {
        match self.load_token(app_handle).await? {
            Some(token) => {
                // Check if token is still valid
                let is_valid = if let Some(expires_at) = token.expires_at {
                    chrono::Utc::now().timestamp() < expires_at - token::EXPIRY_BUFFER_SECS
                } else {
                    true // No expiration, assume valid
                };
                
                // If token appears expired, try to refresh it to verify refresh token is still valid
                if !is_valid {
                    tracing::debug!("Token appears expired, verifying refresh token validity");
                    if let Some(refresh_token) = &token.refresh_token {
                        // Try to refresh - this will clear token if refresh token is invalid
                        match self
                            .refresh_access_token(refresh_token.clone(), app_handle)
                            .await
                        {
                            Ok(_) => {
                                tracing::debug!("Refresh token is valid, token refreshed");
                                return Ok(true);
                            }
                            Err(e) => {
                                // Refresh failed - token may have been cleared
                                tracing::debug!("Refresh token verification failed: {}", e);
                                return Ok(false);
                            }
                        }
                    } else {
                        tracing::debug!("No refresh token available");
                        return Ok(false);
                    }
                }
                
                tracing::debug!(
                    "YouTube auth status check: {}",
                    if is_valid { "authenticated" } else { "expired" }
                );
                Ok(is_valid)
            }
            None => {
                tracing::debug!("YouTube auth status check: not authenticated");
                Ok(false)
            }
        }
    }

    /// Clear/disconnect tokens
    pub async fn disconnect(&self, app_handle: &tauri::AppHandle) -> Result<()> {
        tracing::debug!("Starting YouTube account disconnect");
        let path = std::path::PathBuf::from("yupload.store");
        let store = app_handle.store(path).context("Failed to get store")?;

        store.delete("youtube_token");
        store.save().context("Failed to delete token from store")?;

        tracing::info!("YouTube account disconnected successfully");
        Ok(())
    }
}

/// Returns the HTML shown after the OAuth redirect. Copy is intentionally cautious:
/// the callback (browser redirect) can succeed while the app-side handoff or token
/// exchange still fails, so we don't claim "Authentication Successful" here.
fn oauth_callback_page_html() -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Callback received</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            font-family: 'Sora', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background-color: hsl(220 15% 96%);
            color: hsl(220 30% 12%);
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            padding: 1rem;
        }}
        @media (prefers-color-scheme: dark) {{
            body {{ background-color: hsl(220 30% 8%); color: hsl(210 20% 95%); }}
        }}
        .container {{
            background-color: hsl(0 0% 100%);
            border: 1px solid hsl(40 25% 85%);
            border-radius: 0.75rem;
            padding: 2rem;
            max-width: 500px;
            width: 100%;
            text-align: center;
            box-shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.1);
        }}
        @media (prefers-color-scheme: dark) {{
            .container {{ background-color: hsl(220 15% 12%); border-color: hsl(200 25% 20%); }}
        }}
        .icon {{
            width: 64px; height: 64px; margin: 0 auto 1.5rem;
            background-color: hsl(40 95% 52%); border-radius: 50%;
            display: flex; align-items: center; justify-content: center;
            color: hsl(220 30% 12%); font-size: 2rem; font-weight: 600;
        }}
        @media (prefers-color-scheme: dark) {{
            .icon {{ background-color: hsl(40 90% 58%); color: hsl(220 30% 8%); }}
        }}
        h1 {{ font-size: 1.5rem; font-weight: 600; margin-bottom: 0.5rem; color: hsl(220 30% 12%); }}
        @media (prefers-color-scheme: dark) {{ h1 {{ color: hsl(210 20% 95%); }} }}
        p {{ color: hsl(215.4 16.3% 46.9%); font-size: 0.875rem; line-height: 1.5; margin-bottom: 0.75rem; }}
        @media (prefers-color-scheme: dark) {{ p {{ color: hsl(215 20.2% 65.1%); }} }}
        .redirecting {{
            color: hsl(220 70% 45%);
            font-weight: 500;
            margin-top: 1rem;
        }}
        @media (prefers-color-scheme: dark) {{
            .redirecting {{ color: hsl(220 70% 65%); }}
        }}
        .link {{ margin-top: 1rem; }}
        .link a {{
            font-size: 0.875rem;
            color: hsl(220 70% 45%);
            text-decoration: underline;
        }}
        @media (prefers-color-scheme: dark) {{
            .link a {{ color: hsl(220 70% 65%); }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="icon">✓</div>
        <h1>Callback received</h1>
        <p>We got the redirect from the provider. The app may still be finishing sign-in — check the app window to see if you're connected.</p>
        <p class="redirecting" id="redirecting">Redirecting to app...</p>
        <p class="link" id="fallback" style="display: none;">If you're not redirected automatically, <a href="{}://">click here to open the app</a>.</p>
    </div>
    <script>
        (function() {{
            const deepLink = '{}://';
            let redirectAttempted = false;
            
            function attemptRedirect() {{
                if (redirectAttempted) return;
                redirectAttempted = true;
                
                // Try to redirect to the deep link
                window.location.href = deepLink;
                
                // Show fallback link after a short delay if redirect didn't work
                setTimeout(function() {{
                    document.getElementById('fallback').style.display = 'block';
                }}, 500);
            }}
            
            // Attempt redirect immediately (browsers may block it, but we try)
            attemptRedirect();
            
            // Also try after a brief delay to ensure page is fully loaded
            setTimeout(attemptRedirect, 100);
            
            // Show fallback after 2 seconds if still on page
            setTimeout(function() {{
                document.getElementById('redirecting').textContent = 'If you were not redirected, click the link below.';
            }}, 2000);
        }})();
    </script>
</body>
</html>"#,
        DEEP_LINK_SCHEME,
        DEEP_LINK_SCHEME
    )
}

async fn handle_oauth_callback(
    req: Request<hyper::body::Incoming>,
    tx: Arc<tokio::sync::Mutex<Option<oneshot::Sender<String>>>>,
    _pkce_verifier_secret: Arc<tokio::sync::Mutex<Option<String>>>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let (parts, _body) = req.into_parts();

    // Only handle GET requests to /callback
    if parts.method != Method::GET {
        return Ok(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(Full::new(Bytes::from("Method not allowed")))
            .unwrap());
    }

    let path = parts.uri.path();
    if !path.starts_with("/callback") {
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Full::new(Bytes::from("Not found")))
            .unwrap());
    }

    // Parse query parameters
    let query = parts.uri.query().unwrap_or("");
    let params: std::collections::HashMap<_, _> = url::form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect();

    if let Some(code) = params.get("code") {
        // OAuth 2.0 callback (YouTube)
        // Send code to main task
        let mut sender_guard = tx.lock().await;
        if let Some(sender) = sender_guard.take() {
            let _ = sender.send(code.clone());
        }

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .body(Full::new(Bytes::from(oauth_callback_page_html())))
            .unwrap())
    } else if let Some(verifier) = params.get("oauth_verifier") {
        // OAuth 1.0a callback (Discogs)
        // Send verifier to main task
        let mut sender_guard = tx.lock().await;
        if let Some(sender) = sender_guard.take() {
            let _ = sender.send(verifier.clone());
        }

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .body(Full::new(Bytes::from(oauth_callback_page_html())))
            .unwrap())
    } else if let Some(error) = params.get("error") {
        // Handle error
        let error_msg = format!("Authentication error: {}", error);
        let mut sender_guard = tx.lock().await;
        if let Some(sender) = sender_guard.take() {
            let _ = sender.send(String::new()); // Send empty to signal error
        }

        let html = format!(
            r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Authentication Failed</title>
                <style>
                    * {{
                        margin: 0;
                        padding: 0;
                        box-sizing: border-box;
                    }}
                    body {{
                        font-family: 'Sora', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                        background-color: hsl(220 15% 96%);
                        color: hsl(220 30% 12%);
                        display: flex;
                        justify-content: center;
                        align-items: center;
                        min-height: 100vh;
                        padding: 1rem;
                        transition: background-color 0.3s ease, color 0.3s ease;
                    }}
                    @media (prefers-color-scheme: dark) {{
                        body {{
                            background-color: hsl(220 30% 8%);
                            color: hsl(210 20% 95%);
                        }}
                    }}
                    .container {{
                        background-color: hsl(0 0% 100%);
                        border: 1px solid hsl(40 25% 85%);
                        border-radius: 0.75rem;
                        padding: 2rem;
                        max-width: 500px;
                        width: 100%;
                        text-align: center;
                        box-shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.1);
                    }}
                    @media (prefers-color-scheme: dark) {{
                        .container {{
                            background-color: hsl(220 15% 12%);
                            border-color: hsl(200 25% 20%);
                        }}
                    }}
                    .icon {{
                        width: 64px;
                        height: 64px;
                        margin: 0 auto 1.5rem;
                        background-color: hsl(5 75% 60%);
                        border-radius: 50%;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        color: hsl(210 40% 98%);
                        font-size: 2rem;
                        font-weight: 600;
                    }}
                    h1 {{
                        font-size: 1.5rem;
                        font-weight: 600;
                        margin-bottom: 0.5rem;
                        color: hsl(220 30% 12%);
                    }}
                    @media (prefers-color-scheme: dark) {{
                        h1 {{
                            color: hsl(210 20% 95%);
                        }}
                    }}
                    p {{
                        color: hsl(215.4 16.3% 46.9%);
                        font-size: 0.875rem;
                        line-height: 1.5;
                        margin-top: 0.5rem;
                    }}
                    @media (prefers-color-scheme: dark) {{
                        p {{
                            color: hsl(215 20.2% 65.1%);
                        }}
                    }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="icon">✗</div>
                    <h1>Authentication Failed</h1>
                    <p>{}</p>
                    <p style="margin-top: 1rem;">Please close this window and try again.</p>
                </div>
            </body>
            </html>
        "#,
            error_msg
        );

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .body(Full::new(Bytes::from(html)))
            .unwrap())
    } else {
        Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Full::new(Bytes::from("Missing code or error parameter")))
            .unwrap())
    }
}
