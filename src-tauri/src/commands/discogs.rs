use crate::domain::discogs_models::*;
use crate::state::OAuthServerState;
use crate::state::{ensure_discogs_client, DiscogsClientState};
use crate::utils::{text::clean_name, validate_search_query};

#[tauri::command]
pub async fn authenticate_discogs(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, DiscogsClientState>,
    oauth_server: tauri::State<'_, OAuthServerState>,
) -> Result<(), String> {
    // Ensure client is initialized
    ensure_discogs_client(state.clone()).await?;

    // Get mutable access to client
    let mut client_guard = state.lock().await;
    let client = client_guard.as_mut().expect("Client should be initialized");

    match client.authenticate(app_handle.clone(), oauth_server.inner()).await {
        Ok(_) => {
            tracing::info!("Discogs authentication successful");
        }
        Err(e) => {
            tracing::error!("Discogs authentication failed: {}", e);
            return Err(format!("Failed to authenticate: {}", e));
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn check_discogs_auth(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, DiscogsClientState>,
) -> Result<bool, String> {
    tracing::debug!("Checking Discogs authentication status");
    // Ensure client is initialized
    ensure_discogs_client(state.clone()).await?;

    let mut client = state.lock().await;
    let client = client.as_mut().expect("Client should be initialized");

    match client.check_auth_status(&app_handle).await {
        Ok(is_authenticated) => {
            tracing::debug!("Discogs authentication status: {}", is_authenticated);
            Ok(is_authenticated)
        }
        Err(e) => {
            tracing::error!("Failed to check Discogs auth status: {}", e);
            Err(format!("Failed to check auth status: {}", e))
        }
    }
}

#[tauri::command]
pub async fn search_discogs_releases(
    query: String,
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, DiscogsClientState>,
) -> Result<DiscogsSearchResponse, String> {
    tracing::info!("Discogs search command received: '{}'", query);
    // Validate search query
    validate_search_query(&query)?;

    // Ensure client is initialized
    ensure_discogs_client(state.clone()).await?;

    let mut client = state.lock().await;
    let client = client.as_mut().expect("Client should be initialized");

    client
        .search_releases(&query, &app_handle)
        .await
        .map_err(|e| format!("Failed to search releases: {}", e))
}

#[tauri::command]
pub async fn get_discogs_release(
    release_id: u64,
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, DiscogsClientState>,
) -> Result<DiscogsRelease, String> {
    // Ensure client is initialized
    ensure_discogs_client(state.clone()).await?;

    let mut client = state.lock().await;
    let client = client.as_mut().expect("Client should be initialized");

    client
        .get_release(release_id, &app_handle)
        .await
        .map_err(|e| format!("Failed to get release: {}", e))
}

#[tauri::command]
pub async fn get_discogs_master(
    master_id: u64,
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, DiscogsClientState>,
) -> Result<DiscogsRelease, String> {
    // Ensure client is initialized
    ensure_discogs_client(state.clone()).await?;

    let mut client = state.lock().await;
    let client = client.as_mut().expect("Client should be initialized");

    client
        .get_master_release(master_id, &app_handle)
        .await
        .map_err(|e| format!("Failed to get master release: {}", e))
}

#[tauri::command]
pub async fn generate_description_from_release(release: DiscogsRelease) -> Result<String, String> {
    // Build simple, concise description
    let mut parts = Vec::new();

    // Artists
    if !release.artists.is_empty() {
        let artist_names: Vec<String> = release
            .artists
            .iter()
            .map(|a| {
                let name = a.anv.as_ref().unwrap_or(&a.name);
                clean_name(name)
            })
            .collect();
        parts.push(format!("Artists: {}", artist_names.join(", ")));
    }

    // Title
    parts.push(format!("Release: {}", release.title));

    // Label
    if !release.labels.is_empty() {
        let label_names: Vec<String> = release.labels.iter().map(|l| clean_name(&l.name)).collect();
        parts.push(format!("Label: {}", label_names.join(", ")));
    }

    // Cat number
    if !release.labels.is_empty() {
        let catnos: Vec<String> = release
            .labels
            .iter()
            .filter_map(|l| l.catno.as_ref())
            .map(|c| c.clone())
            .collect();
        if !catnos.is_empty() {
            parts.push(format!("Catalogue Number: {}", catnos.join(", ")));
        }
    }

    // Genres
    if !release.genres.is_empty() {
        parts.push(format!("Genres: {}", release.genres.join(", ")));
    }

    // Styles
    if !release.styles.is_empty() {
        parts.push(format!("Styles: {}", release.styles.join(", ")));
    }

    // Format - check formats array first (from release endpoint), then format array (from search)
    if !release.formats.is_empty() {
        let format_parts: Vec<String> = release
            .formats
            .iter()
            .flat_map(|f| {
                let mut parts = vec![f.name.clone()];
                parts.extend(f.descriptions.iter().cloned());
                parts
            })
            .collect();
        if !format_parts.is_empty() {
            parts.push(format!("Format: {}", format_parts.join(", ")));
        }
    } else if let Some(formats) = &release.format {
        if !formats.is_empty() {
            parts.push(format!("Format: {}", formats.join(", ")));
        }
    }

    // Country
    if let Some(country) = &release.country {
        parts.push(format!("Country: {}", country));
    }

    // Discogs URL
    if let Some(uri) = &release.uri {
        parts.push(format!("Discogs: {}", uri));
    } else {
        // Fallback: construct URL from ID
        parts.push(format!(
            "Discogs: https://www.discogs.com/release/{}",
            release.id
        ));
    }

    Ok(parts.join("\n"))
}

#[tauri::command]
pub async fn disconnect_discogs(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, DiscogsClientState>,
) -> Result<(), String> {
    tracing::info!("Discogs disconnect command received");
    // Ensure client is initialized
    ensure_discogs_client(state.clone()).await?;

    let mut client = state.lock().await;
    let client = client.as_mut().expect("Client should be initialized");

    match client.disconnect(&app_handle).await {
        Ok(_) => {
            tracing::info!("Discogs disconnect command completed successfully");
            Ok(())
        }
        Err(e) => {
            tracing::error!("Discogs disconnect command failed: {}", e);
            Err(format!("Failed to disconnect: {}", e))
        }
    }
}
