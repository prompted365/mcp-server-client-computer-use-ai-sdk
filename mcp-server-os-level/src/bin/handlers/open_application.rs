use std::sync::Arc;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::Json as JsonResponse,
};
use serde::Serialize;
use serde_json::json;
use tracing::{debug, error, info};
use computer_use_ai_sdk::Desktop;

use crate::types::{AppState, OpenApplicationRequest, OpenApplicationResponse, ListInteractableElementsResponse};
use crate::refresh_elements_after_action;

// Response type that combines both results
#[derive(Serialize)]
pub struct OpenApplicationWithElementsResponse {
    pub application: OpenApplicationResponse,
    pub elements: Option<ListInteractableElementsResponse>,
}

pub async fn open_application_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<OpenApplicationRequest>,
) -> Result<JsonResponse<OpenApplicationWithElementsResponse>, (StatusCode, JsonResponse<serde_json::Value>)> {
    // Create Desktop automation instance
    let desktop = match Desktop::new(false, true) {
        Ok(desktop) => desktop,
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                JsonResponse(json!({"error": format!("failed to initialize automation: {}", err)})),
            ));
        }
    };

    // Open the application
    match desktop.open_application(&request.app_name) {
        Ok(_) => {
            // Application opened successfully
            let app_response = OpenApplicationResponse {
                success: true,
                message: format!("successfully opened application: {}", request.app_name),
            };
            
            // Get refreshed elements using the helper function - use a longer delay for app startup
            let elements_response = refresh_elements_after_action(state, request.app_name.clone(), 1000).await;
            
            // Return combined response
            Ok(JsonResponse(OpenApplicationWithElementsResponse {
                application: app_response,
                elements: elements_response,
            }))
        },
        Err(err) => Err((
            StatusCode::BAD_REQUEST,
            JsonResponse(json!({"error": format!("failed to open application: {}", err)})),
        )),
    }
}
