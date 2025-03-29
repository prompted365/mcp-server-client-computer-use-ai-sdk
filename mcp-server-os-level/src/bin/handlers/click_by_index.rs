use std::sync::Arc;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::Json as JsonResponse,
};
use serde::Serialize;
use serde_json::json;
use tracing::{debug, error, info};

use crate::types::{AppState, ClickByIndexRequest, ClickByIndexResponse, ListInteractableElementsResponse};
use crate::refresh_elements_after_action;

// Response type that combines both click result and elements
#[derive(Serialize)]
pub struct ClickByIndexWithElementsResponse {
    pub click: ClickByIndexResponse,
    pub elements: Option<ListInteractableElementsResponse>,
}

pub async fn click_by_index_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ClickByIndexRequest>,
) -> Result<JsonResponse<ClickByIndexWithElementsResponse>, (StatusCode, JsonResponse<serde_json::Value>)> {
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
            // Use element_index directly
            if request.element_index < elements.len() {
                let element = &elements[request.element_index];

                match element.click() {
                    Ok(_) => {
                        let click_response = ClickByIndexResponse {
                            success: true,
                            message: format!(
                                "successfully clicked element with role: {}",
                                element.role()
                            ),
                            elements: None,  // add the missing field
                        };
                        
                        // Get refreshed elements using the helper function
                        let elements_response = refresh_elements_after_action(state, app_name.clone(), 500).await;
                        
                        // Return combined response
                        Ok(JsonResponse(ClickByIndexWithElementsResponse {
                            click: click_response,
                            elements: elements_response,
                        }))
                    },
                    Err(e) => {
                        error!("failed to click element: {}", e);
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            JsonResponse(json!({
                                "error": format!("failed to click element: {}", e)
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
