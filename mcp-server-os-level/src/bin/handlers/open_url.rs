use std::sync::Arc;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::Json as JsonResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, error, info};
use computer_use_ai_sdk::Desktop;

use crate::types::AppState;
use crate::refresh_elements_and_attributes_after_action;
use crate::types::ListElementsAndAttributesResponse;

#[derive(Deserialize)]
pub struct OpenUrlRequest {
    pub url: String,
    pub browser: Option<String>,
}

#[derive(Serialize)]
pub struct OpenUrlResponse {
    pub success: bool,
    pub message: String,
}

// First, create a new response type that combines both results
#[derive(Serialize)]
pub struct OpenUrlWithElementsResponse {
    pub url: OpenUrlResponse,
    pub elements: Option<ListElementsAndAttributesResponse>,
}

pub async fn open_url_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<OpenUrlRequest>,
) -> Result<JsonResponse<OpenUrlWithElementsResponse>, (StatusCode, JsonResponse<serde_json::Value>)> {
    info!("handling request to open url: {}", request.url);
    
    // Create Desktop automation instance
    let desktop = match Desktop::new(false, true) {
        Ok(desktop) => desktop,
        Err(err) => {
            error!("failed to initialize automation: {}", err);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                JsonResponse(json!({"error": format!("failed to initialize automation: {}", err)})),
            ));
        }
    };

    // Open the URL
    let browser_name = request.browser.clone().unwrap_or_else(|| "Safari".to_string());
    let browser_ref = request.browser.as_deref();
    
    debug!("opening url {} in browser {}", request.url, browser_name);
    
    match desktop.open_url(&request.url, browser_ref) {
        Ok(_) => {
            // URL opened successfully
            info!("successfully opened url: {}", request.url);
            let url_response = OpenUrlResponse {
                success: true,
                message: format!("successfully opened URL: {}", request.url),
            };
            
            // Get refreshed elements using the helper function - use a longer delay for page loading
            let elements_response = refresh_elements_and_attributes_after_action(state, browser_name, 2000).await;
            
            // Return combined response
            Ok(JsonResponse(OpenUrlWithElementsResponse {
                url: url_response,
                elements: elements_response,
            }))
        },
        Err(err) => {
            error!("failed to open url {}: {}", request.url, err);
            Err((
                StatusCode::BAD_REQUEST,
                JsonResponse(json!({"error": format!("failed to open URL: {}", err)})),
            ))
        },
    }
}