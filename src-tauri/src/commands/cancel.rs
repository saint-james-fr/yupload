use crate::state::CancellationState;
use tokio_util::sync::CancellationToken;

/// Operation types that can be cancelled
pub const OP_TYPE_BULK_VIDEO_CREATION: &str = "bulk_video_creation";
pub const OP_TYPE_AUDIO_NORMALIZATION: &str = "audio_normalization";
pub const OP_TYPE_YOUTUBE_UPLOAD: &str = "youtube_upload";

/// Cancel an entire operation (all items)
#[tauri::command]
pub async fn cancel_operation(
    operation_type: String,
    cancellation_state: tauri::State<'_, CancellationState>,
) -> Result<(), String> {
    let mut state = cancellation_state.lock().await;
    
    if let Some(token) = state.get(&operation_type) {
        token.cancel();
        tracing::info!("Cancellation requested for operation: {}", operation_type);
        // Remove the token after cancelling
        state.remove(&operation_type);
        // Also cancel all per-item tokens for this operation
        let keys_to_remove: Vec<String> = state
            .keys()
            .filter(|k| k.starts_with(&format!("{}:", operation_type)))
            .cloned()
            .collect();
        for key in keys_to_remove {
            if let Some(item_token) = state.get(&key) {
                item_token.cancel();
            }
            state.remove(&key);
        }
        Ok(())
    } else {
        Err(format!("No active operation of type: {}", operation_type))
    }
}

/// Cancel a specific item within an operation
#[tauri::command]
pub async fn cancel_item(
    operation_type: String,
    item_id: String,
    cancellation_state: tauri::State<'_, CancellationState>,
) -> Result<(), String> {
    let mut state = cancellation_state.lock().await;
    let item_key = format!("{}:{}", operation_type, item_id);
    
    if let Some(token) = state.get(&item_key) {
        token.cancel();
        tracing::info!("Cancellation requested for item {} in operation: {}", item_id, operation_type);
        // Remove the token after cancelling
        state.remove(&item_key);
        Ok(())
    } else {
        Err(format!("No active item {} in operation: {}", item_id, operation_type))
    }
}

/// Get or create a cancellation token for an operation
pub async fn get_or_create_cancellation_token(
    operation_type: &str,
    cancellation_state: &tauri::State<'_, CancellationState>,
) -> CancellationToken {
    let mut state = cancellation_state.lock().await;
    
    // Remove any existing token for this operation type
    state.remove(operation_type);
    
    // Create new token
    let token = CancellationToken::new();
    state.insert(operation_type.to_string(), token.clone());
    
    token
}

/// Get or create a cancellation token for a specific item within an operation
pub async fn get_or_create_item_cancellation_token(
    operation_type: &str,
    item_id: &str,
    cancellation_state: &tauri::State<'_, CancellationState>,
) -> CancellationToken {
    let mut state = cancellation_state.lock().await;
    let item_key = format!("{}:{}", operation_type, item_id);
    
    // Remove any existing token for this item
    state.remove(&item_key);
    
    // Create new token
    let token = CancellationToken::new();
    state.insert(item_key, token.clone());
    
    token
}

/// Check if a specific item has been cancelled
pub fn is_item_cancelled(
    operation_type: &str,
    item_id: &str,
    cancellation_state: &CancellationState,
) -> bool {
    let state = cancellation_state.blocking_lock();
    let item_key = format!("{}:{}", operation_type, item_id);
    if let Some(token) = state.get(&item_key) {
        token.is_cancelled()
    } else {
        false
    }
}

/// Check if an operation has been cancelled
pub fn is_cancelled(token: &CancellationToken) -> bool {
    token.is_cancelled()
}
