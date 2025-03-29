use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::Json as JsonResponse,
};
use serde_json;
use std::process::Command;
use std::sync::Arc;
use tokio::time::Duration;
use tracing::{error, info};

use crate::types::*;
use crate::AppState;

// Define the handler for input control
pub async fn input_control_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<InputControlRequest>,
) -> Result<JsonResponse<InputControlWithElementsResponse>, (StatusCode, JsonResponse<serde_json::Value>)> {
    info!("input control handler {:?}", payload);
    
    // Execute appropriate input action
    match payload.action {
        InputAction::KeyPress(key) => {
            // Implement key press using appropriate library or command
            let script = format!("tell application \"System Events\" to key code {}", key);
            if let Err(e) = Command::new("osascript").arg("-e").arg(script).output() {
                error!("failed to press key: {}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    JsonResponse(serde_json::json!({"error": format!("failed to press key: {}", e)})),
                ));
            }
        }
        InputAction::MouseMove { x, y } => {
            // Implement mouse move
            let script = format!("tell application \"System Events\" to set mouse position to {{{}, {}}}", x, y);
            if let Err(e) = Command::new("osascript").arg("-e").arg(script).output() {
                error!("failed to move mouse: {}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    JsonResponse(serde_json::json!({"error": format!("failed to move mouse: {}", e)})),
                ));
            }
        }
        InputAction::MouseClick(button) => {
            // Implement mouse click
            let button_num = match button.as_str() {
                "left" => 1,
                "right" => 2,
                _ => {
                    error!("unsupported mouse button: {}", button);
                    return Err((
                        StatusCode::BAD_REQUEST,
                        JsonResponse(serde_json::json!({"error": format!("unsupported mouse button: {}", button)})),
                    ));
                }
            };
            
            let script = format!("tell application \"System Events\" to click button {}", button_num);
            if let Err(e) = Command::new("osascript").arg("-e").arg(script).output() {
                error!("failed to click mouse: {}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    JsonResponse(serde_json::json!({"error": format!("failed to click mouse: {}", e)})),
                ));
            }
        }
        InputAction::WriteText(text) => {
            // Implement text writing
            let script = format!("tell application \"System Events\" to keystroke \"{}\"", text);
            if let Err(e) = Command::new("osascript").arg("-e").arg(script).output() {
                error!("failed to write text: {}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    JsonResponse(serde_json::json!({"error": format!("failed to write text: {}", e)})),
                ));
            }
        }
    }

    // Get elements from cache to find the active application
    let elements_response = {
        let cache = state.element_cache.lock().await;
        match &*cache {
            Some((_, _, cached_app_name)) => {
                // We have a cached app name, so let's refresh elements
                info!("refreshing elements for app: {}", cached_app_name);
                refresh_elements_after_action(state.clone(), cached_app_name.clone(), 500).await
            }
            None => {
                // No cache available, don't try to refresh elements
                info!("no element cache found, skipping element refresh");
                None
            }
        }
    };
    
    // Return combined response
    Ok(JsonResponse(InputControlWithElementsResponse {
        input: InputControlResponse { success: true },
        elements: elements_response,
    }))
}

// Helper function to refresh elements after an action
async fn refresh_elements_after_action(
    state: Arc<AppState>, 
    app_name: String,
    delay_ms: u64
) -> Option<ListInteractableElementsResponse> {
    // Small delay to allow UI to update after action
    info!("waiting for ui to update after action before listing elements");
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    
    // Create request to refresh the elements list
    let elements_request = ListInteractableElementsRequest {
        app_name,
        max_elements: None,
        use_background_apps: Some(false),
        activate_app: Some(true),
    };
    
    // Call the list elements handler
    match crate::handlers::list_elements::list_interactable_elements_handler(State(state), Json(elements_request)).await {
        Ok(response) => Some(response.0),
        Err(e) => {
            // Log the error but don't fail the whole request
            error!("failed to list elements after action: {:?}", e);
            None
        }
    }
}
