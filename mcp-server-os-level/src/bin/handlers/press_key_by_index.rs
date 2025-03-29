use std::sync::Arc;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::Json as JsonResponse,
};
use serde::Serialize;
use serde_json::json;
use tracing::{debug, error};
use computer_use_ai_sdk::Desktop;

use crate::types::{AppState, PressKeyByIndexRequest, PressKeyByIndexResponse, ListInteractableElementsResponse};
use crate::refresh_elements_after_action;

// Response type that combines both results
#[derive(Debug, Serialize)]
pub struct PressKeyByIndexWithElementsResponse {
    pub press_key: PressKeyByIndexResponse,
    pub elements: Option<ListInteractableElementsResponse>,
}

pub async fn press_key_by_index_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<PressKeyByIndexRequest>,
) -> Result<JsonResponse<PressKeyByIndexWithElementsResponse>, (StatusCode, JsonResponse<serde_json::Value>)> {
    debug!("pressing key combination by index: element_index={}, key_combo={}", 
        request.element_index, request.key_combo);

    // Get elements from cache
    let elements_opt = {
        let cache = state.element_cache.lock().await;
        cache.clone()
    };

    // Check if cache exists
    if elements_opt.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            JsonResponse(json!({
                "error": "no element cache found - you must call listInteractableElementsByIndex first to index the elements before using by-index operations"
            })),
        ));
    }

    match elements_opt {
        Some((elements, timestamp, app_name)) if timestamp.elapsed() < std::time::Duration::from_secs(30) => {
            // Activate the app first
            debug!("activating app: {}", app_name);
            let desktop = match Desktop::new(false, true) {
                Ok(d) => d,
                Err(e) => {
                    error!("failed to initialize desktop automation: {}", e);
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        JsonResponse(json!({
                            "error": format!("failed to initialize desktop automation: {}", e)
                        })),
                    ));
                }
            };

            // Get and activate the application
            let _ = match desktop.application(&app_name) {
                Ok(app) => app,
                Err(e) => {
                    error!("application not found: {}", e);
                    return Err((
                        StatusCode::NOT_FOUND,
                        JsonResponse(json!({
                            "error": format!("application not found: {}", e)
                        })),
                    ));
                }
            };

            // Use element_index directly
            if request.element_index < elements.len() {
                let element = &elements[request.element_index];

                match element.press_key(&request.key_combo) {
                    Ok(_) => {
                        let press_key_response = PressKeyByIndexResponse {
                            success: true,
                            message: format!(
                                "successfully pressed key combination '{}' on element with role: {}",
                                request.key_combo,
                                element.role()
                            ),
                        };
                        
                        // Get refreshed elements using the helper function
                        let elements_response = refresh_elements_after_action(state, app_name.clone(), 500).await;
                        
                        // Return combined response
                        Ok(JsonResponse(PressKeyByIndexWithElementsResponse {
                            press_key: press_key_response,
                            elements: elements_response,
                        }))
                    },
                    Err(e) => {
                        error!("failed to press key on element: {}", e);
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            JsonResponse(json!({
                                "error": format!("failed to press key on element: {}", e)
                            })),
                        ))
                    }
                }
            } else {
                error!(
                    "element index out of bounds: {} (max: {})",
                    request.element_index,
                    elements.len() - 1
                );
                Err((
                    StatusCode::BAD_REQUEST,
                    JsonResponse(json!({
                        "error": format!("element index out of bounds: {} (max: {})",
                                       request.element_index, elements.len() - 1)
                    })),
                ))
            }
        }
        Some(_) => {
            // Cache entry expired
            Err((
                StatusCode::BAD_REQUEST,
                JsonResponse(json!({
                    "error": "cache entry expired, please list elements again"
                })),
            ))
        }
        None => {
            // Cache miss
            Err((
                StatusCode::NOT_FOUND,
                JsonResponse(json!({
                    "error": "no cache entry found, please list elements again"
                })),
            ))
        }
    }
}
