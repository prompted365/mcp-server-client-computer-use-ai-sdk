use std::sync::Arc;
use axum::extract::{Json, State};
use tokio::time::Duration;
use tracing::{error, info};

use crate::types::*;
use crate::AppState;

use super::list_elements::list_interactable_elements_handler;

pub async fn refresh_elements_after_action(
    state: Arc<AppState>, 
    app_name: String,
    delay_ms: u64
) -> Option<ListInteractableElementsResponse> {
    // Small delay to allow UI to update after action
    info!("waiting for UI to update after action before listing elements");
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    
    // Create request to refresh the elements list
    let elements_request = ListInteractableElementsRequest {
        app_name,
        max_elements: None,
        use_background_apps: Some(false),
        activate_app: Some(true),
    };
    
    // Call the list elements handler
    match list_interactable_elements_handler(State(state), Json(elements_request)).await {
        Ok(response) => Some(response.0),
        Err(e) => {
            // Log the error but don't fail the whole request
            error!("failed to list elements after action: {:?}", e);
            None
        }
    }
}
