use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::Json as JsonResponse,
};
use computer_use_ai_sdk::Desktop;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use chrono;
use serde_json::{self, json};
use tracing::{error, info};
use uuid::Uuid;

use crate::types::*;
use crate::AppState;

pub async fn list_interactable_elements_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ListInteractableElementsRequest>,
) -> Result<JsonResponse<ListInteractableElementsResponse>, (StatusCode, JsonResponse<serde_json::Value>)> {
    // Create desktop automation engine
    let desktop = match Desktop::new(
        request.use_background_apps.unwrap_or(false),
        request.activate_app.unwrap_or(false),
    ) {
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

    // Get application
    let app = match desktop.application(&request.app_name) {
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

    // Get elements from the application
    let locator = match app.locator("") {
        Ok(locator) => locator,
        Err(e) => {
            error!("failed to get elements: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                JsonResponse(json!({
                    "error": format!("failed to get elements: {}", e)
                })),
            ));
        }
    };

    let elements = match locator.all() {
        Ok(elements) => elements,
        Err(e) => {
            error!("failed to get elements: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                JsonResponse(json!({
                    "error": format!("failed to get elements: {}", e)
                })),
            ));
        }
    };

    info!("found {} elements in {}", elements.len(), request.app_name);

    // Create simple stats
    let mut stats = ElementStats {
        total: elements.len(),
        definitely_interactable: 0,
        sometimes_interactable: 0,
        non_interactable: 0,
        by_role: HashMap::new(),
    };

    // Collect elements with text
    let mut result_elements = Vec::new();
    for (i, element) in elements.iter().enumerate() {
        let role = element.role();

        // Count by role for stats
        *stats.by_role.entry(role.clone()).or_insert(0) += 1;
        
        // Extract text from element's attributes
        let attrs = element.attributes();
        let mut text_parts = Vec::new();

        // Collect text from direct attributes
        if let Some(value) = &attrs.value { 
            if !value.is_empty() { text_parts.push(value.clone()); }
        }
        if let Some(label) = &attrs.label { 
            if !label.is_empty() { text_parts.push(label.clone()); }
        }
        if let Some(desc) = &attrs.description { 
            if !desc.is_empty() { text_parts.push(desc.clone()); }
        }

        // Join non-empty text parts with spaces
        let text = text_parts.join(" ").trim().to_string();
        
        // Roles that are almost never interactable
        let non_interactable_roles = [
            "AXGroup", "AXStaticText", "AXUnknown", "AXSeparator", 
            "AXHeading", "AXLayoutArea", "AXHelpTag", "AXGrowArea"
        ];

        // Include if:
        // 1. The role is likely interactable (not in our non-interactable list)
        // OR
        // 2. The element has any text content
        if !non_interactable_roles.contains(&role.as_str()) || !text.is_empty() {
            // Create an object instead of an array
            result_elements.push(json!({
                "index": i,
                "role": role.clone(),
                "text": text
            }));
        }
    }

    // Apply max_elements limit if specified
    if let Some(max) = request.max_elements {
        if result_elements.len() > max {
            result_elements.truncate(max);
        }
    }

    // Generate a cache ID and store elements in cache
    let cache_id = Uuid::new_v4().to_string();
    let cache_timestamp = Instant::now();
    let ttl_seconds: u64 = 30;

    {
        let mut cache = state.element_cache.lock().await;
        *cache = Some((elements.clone(), cache_timestamp, request.app_name.clone()));
    }

    // Create cache info for response
    let now = chrono::Utc::now();
    let expires_at = now + chrono::Duration::seconds(ttl_seconds as i64);

    let cache_info = ElementCacheInfo {
        cache_id: cache_id.clone(),
        timestamp: now.to_rfc3339(),
        expires_at: expires_at.to_rfc3339(),
        element_count: elements.len(),
        ttl_seconds,
    };

    Ok(JsonResponse(ListInteractableElementsResponse {
        elements: result_elements,
        stats,
        cache_info,
    }))
}

#[allow(dead_code)]
pub async fn refresh_elements_after_action(
    state: Arc<AppState>,
    app_name: String,
    delay_ms: u64,
) -> Option<ListInteractableElementsResponse> {
    // Add a small delay to allow UI to update
    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
    
    info!("refreshing elements for app: {} after action", app_name);
    
    // Create request for list elements
    let list_request = ListInteractableElementsRequest {
        app_name,
        max_elements: None,
        use_background_apps: Some(false),
        activate_app: Some(true),
    };
    
    // Call the handler to get fresh elements
    match list_interactable_elements_handler(State(state), Json(list_request)).await {
        Ok(response) => Some(response.0),
        Err(e) => {
            error!("failed to refresh elements after action: {:?}", e);
            None
        }
    }
}
