use crate::domain::discogs_models::*;
use crate::utils::{rate_limit, timeout};
use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use rand::RngCore;
use reqwest::Client;
use sha1::Sha1;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_store::StoreExt;
use tokio::sync::{oneshot, Mutex};
use url::form_urlencoded;

type HmacSha1 = Hmac<Sha1>;

const DISCOGS_API_BASE: &str = "https://api.discogs.com";
const DISCOGS_REQUEST_TOKEN_URL: &str = "https://api.discogs.com/oauth/request_token";
const DISCOGS_AUTHORIZE_URL: &str = "https://www.discogs.com/oauth/authorize";
const DISCOGS_ACCESS_TOKEN_URL: &str = "https://api.discogs.com/oauth/access_token";

/// Rate limiter for Discogs API
struct RateLimiter {
    remaining: Arc<Mutex<u32>>,
    last_reset: Arc<Mutex<Option<std::time::Instant>>>,
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            remaining: Arc::new(Mutex::new(rate_limit::DISCOGS_AUTHENTICATED_LIMIT)),
            last_reset: Arc::new(Mutex::new(None)),
        }
    }

    async fn wait_if_needed(&self) {
        // Check if rate limit window has reset
        let mut last_reset_guard = self.last_reset.lock().await;
        if let Some(last_reset) = *last_reset_guard {
            let elapsed = last_reset.elapsed();
            if elapsed.as_secs() >= rate_limit::DISCOGS_WINDOW_SECS {
                // Reset window has passed, reset the counter
                let mut remaining_guard = self.remaining.lock().await;
                *remaining_guard = rate_limit::DISCOGS_AUTHENTICATED_LIMIT;
                *last_reset_guard = Some(std::time::Instant::now());
            }
        } else {
            // First time, initialize reset time
            *last_reset_guard = Some(std::time::Instant::now());
        }
        drop(last_reset_guard);

        let remaining = *self.remaining.lock().await;
        if remaining < rate_limit::DISCOGS_WAIT_THRESHOLD {
            // Wait for rate limit window to reset
            let wait_secs = rate_limit::DISCOGS_WINDOW_SECS - remaining as u64;
            tokio::time::sleep(tokio::time::Duration::from_secs(wait_secs as u64)).await;
        }
    }

    async fn update_from_headers(&self, headers: &reqwest::header::HeaderMap) {
        if let Some(remaining_header) = headers.get("X-Discogs-RateLimit-Remaining") {
            if let Ok(remaining_str) = remaining_header.to_str() {
                if let Ok(remaining) = remaining_str.parse::<u32>() {
                    let mut guard = self.remaining.lock().await;
                    *guard = remaining;
                    // Update reset time when we get new rate limit info
                    let mut reset_guard = self.last_reset.lock().await;
                    *reset_guard = Some(std::time::Instant::now());
                }
            }
        }
    }
}

pub struct DiscogsClient {
    consumer_key: String,
    consumer_secret: String,
    http_client: Client,
    rate_limiter: RateLimiter,
    access_token: Option<String>,
    access_token_secret: Option<String>,
}

impl DiscogsClient {
    pub fn new(consumer_key: String, consumer_secret: String) -> Result<Self> {
        let http_client = Client::builder()
            .timeout(timeout::DISCOGS_API)
            .user_agent("yupload/1.0")
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            consumer_key,
            consumer_secret,
            http_client,
            rate_limiter: RateLimiter::new(),
            access_token: None,
            access_token_secret: None,
        })
    }

    /// Generate OAuth 1.0a signature
    fn generate_signature(
        &self,
        method: &str,
        url: &str,
        params: &[(String, String)],
        token_secret: Option<&str>,
    ) -> String {
        // Sort parameters
        let mut sorted_params = params.to_vec();
        sorted_params.sort_by(|a, b| match a.0.cmp(&b.0) {
            std::cmp::Ordering::Equal => a.1.cmp(&b.1),
            other => other,
        });

        // Create parameter string
        let param_string = sorted_params
            .iter()
            .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        // Normalize URL for signature (remove query params, normalize port, etc.)
        let normalized_url = normalize_url(url);

        // Create signature base string
        let signature_base = format!(
            "{}&{}&{}",
            method.to_uppercase(),
            percent_encode(&normalized_url),
            percent_encode(&param_string)
        );

        // Create signing key
        let signing_key = format!(
            "{}&{}",
            percent_encode(&self.consumer_secret),
            token_secret.map(percent_encode).unwrap_or_else(String::new)
        );

        // Generate HMAC-SHA1
        let mut mac = HmacSha1::new_from_slice(signing_key.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(signature_base.as_bytes());
        let result = mac.finalize();
        let signature = general_purpose::STANDARD.encode(result.into_bytes());

        signature
    }

    /// Generate OAuth parameters (returns owned strings to avoid lifetime issues)
    fn generate_oauth_params(
        &self,
        token: Option<&str>,
        verifier: Option<&str>,
    ) -> Vec<(String, String)> {
        let mut params = vec![
            ("oauth_consumer_key".to_string(), self.consumer_key.clone()),
            ("oauth_nonce".to_string(), generate_nonce()),
            (
                "oauth_signature_method".to_string(),
                "HMAC-SHA1".to_string(),
            ),
            (
                "oauth_timestamp".to_string(),
                current_timestamp().to_string(),
            ),
            ("oauth_version".to_string(), "1.0".to_string()),
        ];

        if let Some(token) = token {
            params.push(("oauth_token".to_string(), token.to_string()));
        }

        if let Some(verifier) = verifier {
            params.push(("oauth_verifier".to_string(), verifier.to_string()));
        }

        params
    }

    /// Make OAuth-signed request
    async fn make_oauth_request(
        &self,
        method: &str,
        url: &str,
        params: Vec<(String, String)>,
        token_secret: Option<&str>,
    ) -> Result<reqwest::Response> {
        self.rate_limiter.wait_if_needed().await;

        // Generate OAuth params - must include oauth_token if we have access_token
        let oauth_token = self.access_token.as_deref();
        if oauth_token.is_none() {
            anyhow::bail!("No access token available. Please authenticate first.");
        }

        let mut oauth_params = self.generate_oauth_params(oauth_token, None);

        // Combine all params for signature
        let mut all_params = oauth_params.clone();
        all_params.extend(params.iter().cloned());

        // For GET requests, query params go in the URL, not the body
        // The signature base string uses the base URL (without query params)
        // but includes query params in the parameter string
        let request_url = if method == "GET" && !params.is_empty() {
            let mut url_with_params = url.to_string();
            url_with_params.push('?');
            let query_string: Vec<String> = params
                .iter()
                .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
                .collect();
            url_with_params.push_str(&query_string.join("&"));
            url_with_params
        } else {
            url.to_string()
        };

        // Generate signature with token_secret
        // Signature uses base URL, but includes all params (OAuth + query) in parameter string
        let signature = self.generate_signature(method, url, &all_params, token_secret);

        // Add signature to OAuth params
        oauth_params.push(("oauth_signature".to_string(), signature));

        // Build Authorization header
        let auth_header = oauth_params
            .iter()
            .map(|(k, v)| format!("{}=\"{}\"", k, percent_encode(v)))
            .collect::<Vec<_>>()
            .join(", ");

        let auth_header = format!("OAuth {}", auth_header);

        // Make request
        let mut request = match method {
            "GET" => self.http_client.get(&request_url),
            "POST" => self.http_client.post(url),
            _ => anyhow::bail!("Unsupported method: {}", method),
        };

        if method == "POST" && !params.is_empty() {
            let form_params: Vec<(&str, &str)> = params
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            request = request.form(&form_params);
        }

        let response = request
            .header("Authorization", auth_header)
            .send()
            .await
            .context("Failed to send OAuth request")?;

        // Update rate limiter
        self.rate_limiter
            .update_from_headers(response.headers())
            .await;

        Ok(response)
    }

    /// Load stored tokens
    async fn load_token(&self, app_handle: &tauri::AppHandle) -> Result<Option<DiscogsAuthToken>> {
        tracing::debug!("Loading Discogs token from store");
        let path = std::path::PathBuf::from("yupload.store");
        let store = app_handle.store(path).context("Failed to get store")?;

        if let Some(value) = store.get("discogs_token") {
            // Try to decrypt if it's an encrypted string, otherwise fall back to plain JSON
            let token = if let Some(encrypted_str) = value.as_str() {
                // Decrypt and parse
                let decrypted_json = crate::utils::decrypt_token(encrypted_str)
                    .with_context(|| {
                        tracing::error!("Failed to decrypt Discogs token");
                        "Failed to decrypt token"
                    })?;
                serde_json::from_str(&decrypted_json)
                    .with_context(|| {
                        tracing::error!("Failed to parse decrypted Discogs token");
                        "Failed to parse decrypted token"
                    })?
            } else {
                // Legacy format: try to parse as JSON value directly
                serde_json::from_value(value.clone())
                    .with_context(|| {
                        tracing::error!("Failed to parse stored Discogs token (legacy format)");
                        "Failed to parse stored token"
                    })?
            };
            tracing::debug!("Discogs token loaded successfully");
            Ok(Some(token))
        } else {
            tracing::debug!("No Discogs token found in store");
            Ok(None)
        }
    }

    /// Save tokens
    async fn save_token(
        &self,
        token: &DiscogsAuthToken,
        app_handle: &tauri::AppHandle,
    ) -> Result<()> {
        tracing::debug!("Saving Discogs token to store");
        let path = std::path::PathBuf::from("yupload.store");
        let store = app_handle.store(path).context("Failed to get store")?;

        // Serialize to JSON string, then encrypt
        let token_json = serde_json::to_string(token)
            .with_context(|| {
                tracing::error!("Failed to serialize Discogs token");
                "Failed to serialize token"
            })?;
        let encrypted = crate::utils::encrypt_token(&token_json)
            .with_context(|| {
                tracing::error!("Failed to encrypt Discogs token");
                "Failed to encrypt token"
            })?;

        // Store encrypted string as JSON string value
        store.set(
            "discogs_token".to_string(),
            serde_json::Value::String(encrypted),
        );
        store.save().context("Failed to save token to store")?;
        tracing::debug!("Discogs token saved successfully");

        Ok(())
    }

    /// Start OAuth 1.0a flow
    pub async fn authenticate(
        &mut self,
        app_handle: tauri::AppHandle,
        callback_server: &crate::infrastructure::youtube_client::OAuthCallbackServer,
    ) -> Result<DiscogsAuthToken> {
        tracing::info!("Starting Discogs OAuth authentication");
        let redirect_uri = format!("http://127.0.0.1:{}/callback", callback_server.port);
        tracing::debug!("Discogs OAuth redirect URI: {}", redirect_uri);

        // Step 1: Request token
        let mut oauth_params = self.generate_oauth_params(None, None);
        oauth_params.push(("oauth_callback".to_string(), redirect_uri.clone()));

        let signature =
            self.generate_signature("POST", DISCOGS_REQUEST_TOKEN_URL, &oauth_params, None);
        oauth_params.push(("oauth_signature".to_string(), signature));

        let auth_header = oauth_params
            .iter()
            .map(|(k, v)| format!("{}=\"{}\"", k, percent_encode(v)))
            .collect::<Vec<_>>()
            .join(", ");

        let response = self
            .http_client
            .post(DISCOGS_REQUEST_TOKEN_URL)
            .header("Authorization", format!("OAuth {}", auth_header))
            .send()
            .await
            .context("Failed to request token")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!(
                "Failed to get Discogs request token: HTTP {} - {}",
                status,
                crate::infrastructure::logging::redact_sensitive(&error_text)
            );
            anyhow::bail!("Failed to get request token: {}", 
                         crate::infrastructure::logging::redact_sensitive(&error_text));
        }

        tracing::debug!("Discogs request token received successfully");
        let body = response.text().await?;
        let token_params: std::collections::HashMap<String, String> =
            form_urlencoded::parse(body.as_bytes())
                .into_owned()
                .collect();

        let request_token = token_params
            .get("oauth_token")
            .context("No oauth_token in response")?;
        let request_token_secret = token_params
            .get("oauth_token_secret")
            .context("No oauth_token_secret in response")?;

        // Step 2: Authorize
        let auth_url = format!("{}?oauth_token={}", DISCOGS_AUTHORIZE_URL, request_token);

        tracing::debug!("Opening browser for Discogs OAuth authorization");
        match app_handle.opener().open_url(&auth_url, None::<&str>) {
            Ok(_) => {}
            Err(_) => {
                // Fallback to system command
                #[cfg(target_os = "macos")]
                {
                    use crate::utils::create_hidden_command;
                    create_hidden_command("open")
                        .arg(&auth_url)
                        .spawn()
                        .context("Failed to open browser")?;
                }
                #[cfg(target_os = "linux")]
                {
                    use crate::utils::create_hidden_command;
                    create_hidden_command("xdg-open")
                        .arg(&auth_url)
                        .spawn()
                        .context("Failed to open browser")?;
                }
                #[cfg(target_os = "windows")]
                {
                    use crate::utils::create_hidden_command;
                    create_hidden_command("cmd")
                        .args(["/C", "start", &auth_url])
                        .spawn()
                        .context("Failed to open browser")?;
                }
            }
        }

        // Step 3: Wait for callback with verifier
        let (tx, rx) = oneshot::channel::<String>();
        {
            let mut sender_guard = callback_server.sender.lock().await;
            *sender_guard = Some(tx);
        }

        tracing::debug!(
            "Waiting for Discogs OAuth callback (timeout: {}s)",
            timeout::OAUTH_VERIFICATION.as_secs()
        );
        let verifier = tokio::time::timeout(timeout::OAUTH_VERIFICATION, async {
            rx.await
                .map_err(|e| anyhow::anyhow!("Channel error: {}", e))
        })
        .await
        .context("Authentication timeout")?
        .context("Failed to receive verifier")?;

        if verifier.is_empty() {
            tracing::error!("Discogs OAuth authentication was cancelled");
            anyhow::bail!("Authentication was cancelled");
        }

        tracing::debug!("Received Discogs OAuth verifier, exchanging for access token");
        // Step 4: Exchange for access token
        let mut oauth_params = self.generate_oauth_params(Some(request_token), Some(&verifier));
        let signature = self.generate_signature(
            "POST",
            DISCOGS_ACCESS_TOKEN_URL,
            &oauth_params,
            Some(request_token_secret),
        );
        oauth_params.push(("oauth_signature".to_string(), signature));

        let auth_header = oauth_params
            .iter()
            .map(|(k, v)| format!("{}=\"{}\"", k, percent_encode(v)))
            .collect::<Vec<_>>()
            .join(", ");

        let response = self
            .http_client
            .post(DISCOGS_ACCESS_TOKEN_URL)
            .header("Authorization", format!("OAuth {}", auth_header))
            .send()
            .await
            .context("Failed to exchange for access token")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to get access token: {}", error_text);
        }

        let body = response.text().await?;
        let token_params: std::collections::HashMap<String, String> =
            form_urlencoded::parse(body.as_bytes())
                .into_owned()
                .collect();

        let access_token = token_params
            .get("oauth_token")
            .context("No oauth_token in response")?
            .clone();
        let access_token_secret = token_params
            .get("oauth_token_secret")
            .context("No oauth_token_secret in response")?
            .clone();

        let auth_token = DiscogsAuthToken {
            oauth_token: access_token.clone(),
            oauth_token_secret: access_token_secret.clone(),
        };

        self.save_token(&auth_token, &app_handle).await?;

        // Update client's token state
        self.access_token = Some(access_token);
        self.access_token_secret = Some(access_token_secret);

        Ok(auth_token)
    }

    /// Check if authenticated
    pub async fn check_auth_status(&mut self, app_handle: &tauri::AppHandle) -> Result<bool> {
        match self.load_token(app_handle).await? {
            Some(token) => {
                // Load tokens into client
                self.access_token = Some(token.oauth_token.clone());
                self.access_token_secret = Some(token.oauth_token_secret.clone());
                Ok(true)
            }
            None => Ok(false),
        }
    }

    /// Search releases
    pub async fn search_releases(
        &mut self,
        query: &str,
        app_handle: &tauri::AppHandle,
    ) -> Result<DiscogsSearchResponse> {
        tracing::debug!("Searching Discogs releases: '{}'", query);
        // Always reload tokens to ensure they're fresh
        if let Some(token) = self.load_token(app_handle).await? {
            self.access_token = Some(token.oauth_token.clone());
            self.access_token_secret = Some(token.oauth_token_secret.clone());
        } else {
            tracing::error!("Discogs search failed: not authenticated");
            anyhow::bail!("Not authenticated. Please connect your Discogs account first.");
        }

        if self.access_token.is_none() || self.access_token_secret.is_none() {
            tracing::error!("Discogs search failed: tokens not available");
            anyhow::bail!("Not authenticated. Please connect your Discogs account first.");
        }

        let url = format!("{}/database/search", DISCOGS_API_BASE);
        let params = vec![
            ("q".to_string(), query.to_string()),
            ("type".to_string(), "release".to_string()),
        ];

        let response = self
            .make_oauth_request("GET", &url, params, self.access_token_secret.as_deref())
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("Discogs search failed: HTTP {} - {}", status, error_text);
            anyhow::bail!("Search failed: {}", error_text);
        }

        // Debug: Get the raw response text to see what we're trying to parse
        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        tracing::debug!(
            "Response body (first 10000 chars): {}",
            &response_text[..std::cmp::min(10000, response_text.len())]
        );

        let search_response: DiscogsSearchResponse = serde_json::from_str(&response_text)
            .map_err(|e| {
                tracing::debug!("Parsing error: {}", e);
                tracing::debug!("Full response: {}", response_text);
                e
            })
            .context("Failed to parse search response")?;

        tracing::info!(
            "Discogs search completed: {} results found",
            search_response.results.len()
        );
        Ok(search_response)
    }

    /// Get release by ID
    pub async fn get_release(
        &mut self,
        release_id: u64,
        app_handle: &tauri::AppHandle,
    ) -> Result<DiscogsRelease> {
        if self.access_token.is_none() {
            if let Some(token) = self.load_token(app_handle).await? {
                self.access_token = Some(token.oauth_token);
                self.access_token_secret = Some(token.oauth_token_secret);
            }
        }

        let url = format!("{}/releases/{}", DISCOGS_API_BASE, release_id);
        let response = self
            .make_oauth_request("GET", &url, vec![], self.access_token_secret.as_deref())
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to get release: {}", error_text);
        }

        // Debug: Get the raw response text to see what we're trying to parse
        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        tracing::debug!(
            "Release response body (first 2000 chars): {}",
            &response_text[..std::cmp::min(2000, response_text.len())]
        );
        if response_text.len() > 2000 {
            tracing::debug!(
                "... (truncated, total length: {} chars)",
                response_text.len()
            );
        }

        let release: DiscogsRelease = serde_json::from_str(&response_text)
            .map_err(|e| {
                tracing::debug!("Parsing error: {}", e);
                tracing::debug!("Full response: {}", response_text);
                e
            })
            .context("Failed to parse release response")?;

        Ok(release)
    }

    /// Get master release by ID
    pub async fn get_master_release(
        &mut self,
        master_id: u64,
        app_handle: &tauri::AppHandle,
    ) -> Result<DiscogsRelease> {
        if self.access_token.is_none() {
            if let Some(token) = self.load_token(app_handle).await? {
                self.access_token = Some(token.oauth_token);
                self.access_token_secret = Some(token.oauth_token_secret);
            }
        }

        let url = format!("{}/masters/{}", DISCOGS_API_BASE, master_id);
        let response = self
            .make_oauth_request("GET", &url, vec![], self.access_token_secret.as_deref())
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to get master release: {}", error_text);
        }

        let master: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse master response")?;

        // Master releases don't have all fields, so we need to get the main release
        if let Some(main_release_id) = master.get("main_release").and_then(|v| v.as_u64()) {
            self.get_release(main_release_id, app_handle).await
        } else {
            // Try to parse as release anyway
            serde_json::from_value(master).context("Failed to parse master as release")
        }
    }

    /// Clear/disconnect tokens
    pub async fn disconnect(&mut self, app_handle: &tauri::AppHandle) -> Result<()> {
        tracing::debug!("Starting Discogs account disconnect");
        
        self.access_token = None;
        self.access_token_secret = None;

        let path = std::path::PathBuf::from("yupload.store");
        let store = app_handle.store(path).context("Failed to get store")?;

        store.delete("discogs_token");
        store.save().context("Failed to delete token from store")?;

        tracing::info!("Discogs account disconnected successfully");
        Ok(())
    }
}

// Helper functions
/// Percent-encode according to RFC 3986 (used by OAuth 1.0a)
/// Unreserved characters (A-Z, a-z, 0-9, -, ., _, ~) are NOT encoded
/// All other characters are encoded as %XX (uppercase hex)
fn percent_encode(s: &str) -> String {
    s.as_bytes()
        .iter()
        .map(|&byte| {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                    // Unreserved character - don't encode
                    char::from(byte).to_string()
                }
                _ => {
                    // Reserved or other character - percent encode
                    format!("%{:02X}", byte)
                }
            }
        })
        .collect()
}

/// Normalize URL for OAuth signature base string
/// Removes query parameters and fragments, ensures proper scheme/host format
/// According to RFC 5849, the URL should be normalized to scheme://authority/path
fn normalize_url(url: &str) -> String {
    // Parse the URL to extract just the base (scheme + host + path)
    if let Ok(mut parsed) = url::Url::parse(url) {
        // Remove query and fragment
        parsed.set_query(None);
        parsed.set_fragment(None);

        // Build normalized URL: scheme://host[:port]/path
        let mut normalized = format!("{}://{}", parsed.scheme(), parsed.host_str().unwrap_or(""));

        // Only include port if it's not the default port for the scheme
        if let Some(port) = parsed.port() {
            let default_port = match parsed.scheme() {
                "https" => 443,
                "http" => 80,
                _ => 0,
            };
            if port != default_port {
                normalized.push_str(&format!(":{}", port));
            }
        }

        // Add path (ensure it starts with /)
        let path = parsed.path();
        if path.is_empty() {
            normalized.push('/');
        } else {
            normalized.push_str(path);
        }

        normalized
    } else {
        // If parsing fails, return as-is (shouldn't happen with valid URLs)
        url.to_string()
    }
}

fn generate_nonce() -> String {
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 16];
    rng.fill_bytes(&mut bytes);
    general_purpose::STANDARD.encode(bytes)
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
