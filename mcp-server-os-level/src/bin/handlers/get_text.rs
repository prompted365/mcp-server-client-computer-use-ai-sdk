use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::Json as JsonResponse,
};
use computer_use_ai_sdk::Desktop;
use tracing::error;
use std::sync::Arc;

use crate::types::*;
use crate::AppState;

pub async fn get_text_handler(
    State(_): State<Arc<AppState>>,
    Json(request): Json<GetTextRequest>,
) -> Result<JsonResponse<GetTextResponse>, (StatusCode, JsonResponse<serde_json::Value>)> {
    let desktop = match Desktop::new(
        request.use_background_apps.unwrap_or(false),
        request.activate_app.unwrap_or(false),
    ) {
        Ok(d) => d,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                JsonResponse(serde_json::json!({
                    "error": format!("failed to initialize desktop automation: {}", e)
                })),
            ));
        }
    };

    let app = match desktop.application(&request.app_name) {
        Ok(app) => app,
        Err(e) => {
            error!("application not found: {}", e);
            return Err((
                StatusCode::NOT_FOUND,
                JsonResponse(serde_json::json!({
                    "error": format!("application not found: {}", e)
                })),
            ));
        }
    };

    let text = match app.text(request.max_depth.unwrap_or(10)) {
        Ok(text) => text,
        Err(e) => {
            error!("failed to extract text: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                JsonResponse(serde_json::json!({
                    "error": format!("failed to extract text: {}", e)
                })),
            ));
        }
    };

    Ok(JsonResponse(GetTextResponse {
        success: true,
        text,
    }))
}
