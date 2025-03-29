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

use crate::types::{AppState, TypeByIndexRequest, TypeByIndexResponse, ListInteractableElementsResponse};
use crate::refresh_elements_after_action;

// Response type that combines both results
#[derive(Serialize)]
pub struct TypeByIndexWithElementsResponse {
    pub type_action: TypeByIndexResponse,
    pub elements: Option<ListInteractableElementsResponse>,
}

pub async fn type_by_index_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<TypeByIndexRequest>,
) -> Result<JsonResponse<TypeByIndexWithElementsResponse>, (StatusCode, JsonResponse<serde_json::Value>)> {
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
                
                // Step 1: Try the standard way (setting AXValue)
                match element.type_text(&request.text) {
                    Ok(_) => {
                        debug!("attempted to type text '{}' into element with role: {}", 
                              request.text, element.role());
                        
                        // Step 2: Verify text was actually set by reading it back
                        let verification = match element.text(1) {
                            Ok(actual_text) => {
                                let contains_text = actual_text.contains(&request.text);
                                if contains_text {
                                    debug!("verified text was set correctly: '{}'", actual_text);
                                    true
                                } else {
                                    debug!("verification failed: expected '{}' but got '{}'", 
                                          request.text, actual_text);
                                    false
                                }
                            },
                            Err(e) => {
                                debug!("failed to verify text: {}", e);
                                false
                            }
                        };
                        
                        // Step 3: If verification failed, activate app and try inputControl fallback
                        if !verification {
                            debug!("falling back to inputControl for typing");
                            
                            // Activate the app first, just like we do in press_key_by_index_handler
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
                            
                            // Click the element first to ensure it has focus
                            if let Err(e) = element.click() {
                                debug!("failed to click element before typing: {}", e);
                                // Continue anyway
                            }
                            
                            // Small delay to ensure element is focused
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            
                            // Use inputControl for text input using System Events
                            use std::process::Command;
                            
                            // Escape any quotes in the text to avoid breaking the AppleScript
                            let escaped_text = request.text.replace("\"", "\\\"");
                            let script = format!("tell application \"System Events\" to keystroke \"{}\"", escaped_text);
                            
                            match Command::new("osascript").arg("-e").arg(script).output() {
                                Ok(_) => {
                                    debug!("successfully typed text '{}' using inputControl", request.text);
                                    
                                    let type_response = TypeByIndexResponse {
                                        success: true,
                                        message: format!(
                                            "successfully typed text into element with role: {} (using AppleScript fallback)",
                                            element.role()
                                        ),
                                    };
                                    
                                    // Get refreshed elements using the helper function
                                    let elements_response = refresh_elements_after_action(state, app_name.clone(), 500).await;
                                    
                                    // Return combined response
                                    Ok(JsonResponse(TypeByIndexWithElementsResponse {
                                        type_action: type_response,
                                        elements: elements_response,
                                    }))
                                },
                                Err(e) => {
                                    error!("failed to type text using inputControl: {}", e);
                                    Err((
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                        JsonResponse(json!({
                                            "error": format!("failed to type text using AppleScript: {}", e)
                                        })),
                                    ))
                                }
                            }
                        } else {
                            // Standard approach worked
                            let type_response = TypeByIndexResponse {
                                success: true,
                                message: format!(
                                    "successfully typed text into element with role: {}",
                                    element.role()
                                ),
                            };
                            
                            // Get refreshed elements using the helper function
                            let elements_response = refresh_elements_after_action(state, app_name.clone(), 500).await;
                            
                            // Return combined response
                            Ok(JsonResponse(TypeByIndexWithElementsResponse {
                                type_action: type_response,
                                elements: elements_response,
                            }))
                        }
                    },
                    Err(e) => {
                        error!("failed to type text into element: {}", e);
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            JsonResponse(json!({
                                "error": format!("failed to type text into element: {}", e)
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
