use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Instant};

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::Json as JsonResponse,
    routing::post,
    Router,
};
use computer_use_ai_sdk::{Desktop, UIElement};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{debug, error, info, level_filters::LevelFilter};
use uuid::Uuid;
use serde_json::{json, Value};

// ================ Types ================

#[derive(Debug, Deserialize, Serialize)]
pub struct ElementSelector {
    app_name: String,
    window_name: Option<String>,
    locator: String,
    index: Option<usize>,
    text: Option<String>,
    label: Option<String>,
    description: Option<String>,
    element_id: Option<String>,
    use_background_apps: Option<bool>,
    activate_app: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FindElementsRequest {
    selector: ElementSelector,
    max_results: Option<usize>,
    max_depth: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClickElementRequest {
    selector: ElementSelector,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TypeTextRequest {
    selector: ElementSelector,
    text: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PressKeyRequest {
    selector: ElementSelector,
    key_combo: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetTextRequest {
    app_name: String,
    window_name: Option<String>,
    max_depth: Option<usize>,
    use_background_apps: Option<bool>,
    activate_app: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ElementPosition {
    x: i32,
    y: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ElementSize {
    width: i32,
    height: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ElementInfo {
    id: Option<String>,
    role: String,
    label: Option<String>,
    description: Option<String>,
    text: Option<String>,
    position: Option<ElementPosition>,
    size: Option<ElementSize>,
    properties: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct FindElementsResponse {
    data: Vec<ElementInfo>,
}

#[derive(Debug, Serialize)]
pub struct ActionResponse {
    success: bool,
    message: String,
}

#[derive(Debug, Serialize)]
pub struct GetTextResponse {
    success: bool,
    text: String,
}

// App state
pub struct AppState {
    element_cache: Arc<Mutex<Option<(Vec<UIElement>, Instant, String)>>>,
}

// Add MCP-specific types
#[derive(Debug, Deserialize, Serialize)]
pub struct MCPRequest {
    jsonrpc: String,
    id: Value,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MCPResponse {
    jsonrpc: String,
    id: Value,
    result: Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MCPErrorResponse {
    jsonrpc: String,
    id: Value,
    error: MCPError,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MCPError {
    code: i32,
    message: String,
    data: Option<Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InitializeParams {
    capabilities: ClientCapabilities,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientCapabilities {
    // MCP client capabilities
    tools: Option<ToolClientCapabilities>,
    resources: Option<ResourceClientCapabilities>,
    // Add other capabilities as needed
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolClientCapabilities {
    execution: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResourceClientCapabilities {
    // Resource capabilities
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerCapabilities {
    tools: Option<ToolServerCapabilities>,
    resources: Option<ResourceServerCapabilities>,
    // Add other capabilities as needed
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolServerCapabilities {
    functions: Vec<ToolFunctionDefinition>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResourceServerCapabilities {
    // Resource capabilities
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolFunctionDefinition {
    name: String,
    description: String,
    parameters: serde_json::Value, // JSON Schema
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExecuteToolFunctionParams {
    function: String,
    arguments: Value,
}

// Add these new structs for scrolling
#[derive(Debug, Deserialize, Serialize)]
pub struct ScrollElementRequest {
    selector: Option<ElementSelector>,
    coordinates: Option<ElementPosition>,
    direction: String,
    amount: f64,
}

// Add these new structs for opening applications
#[derive(Deserialize, Serialize)]
pub struct OpenApplicationRequest {
    app_name: String,
}

#[derive(Serialize)]
pub struct OpenApplicationResponse {
    success: bool,
    message: String,
}

// Add these new structs for opening URLs
#[derive(Deserialize, Serialize)]
pub struct OpenUrlRequest {
    url: String,
    browser: Option<String>,
}

#[derive(Serialize)]
pub struct OpenUrlResponse {
    success: bool,
    message: String,
}

// Add these structs for interactable elements
#[derive(Debug, Deserialize, Serialize)]
pub struct ListInteractableElementsRequest {
    app_name: String,
    window_name: Option<String>,
    with_text_only: Option<bool>,
    interactable_only: Option<bool>,
    include_sometimes_interactable: Option<bool>,
    max_elements: Option<usize>,
    use_background_apps: Option<bool>,
    activate_app: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct InteractableElement {
    index: usize,
    role: String,
    interactability: String, // "definite", "sometimes", "none"
    text: String,
    position: Option<ElementPosition>,
    size: Option<ElementSize>,
    element_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ElementStats {
    total: usize,
    definitely_interactable: usize,
    sometimes_interactable: usize,
    non_interactable: usize,
    by_role: HashMap<String, usize>,
}

#[derive(Debug, Serialize)]
pub struct ElementCacheInfo {
    cache_id: String,
    timestamp: String,
    expires_at: String,
    element_count: usize,
    ttl_seconds: u64,
}

#[derive(Debug, Serialize)]
pub struct ListInteractableElementsResponse {
    elements: Vec<InteractableElement>,
    stats: ElementStats,
    cache_info: ElementCacheInfo,
}

// Add these for index-based operations
#[derive(Debug, Deserialize, Serialize)]
pub struct ClickByIndexRequest {
    element_index: usize,
}

#[derive(Debug, Serialize)]
pub struct ClickByIndexResponse {
    success: bool,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TypeByIndexRequest {
    element_index: usize,
    text: String,
}

#[derive(Debug, Serialize)]
pub struct TypeByIndexResponse {
    success: bool,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PressKeyByIndexRequest {
    element_index: usize,
    key_combo: String,
}

#[derive(Debug, Serialize)]
pub struct PressKeyByIndexResponse {
    success: bool,
    message: String,
}

// Add these for input control
#[derive(Debug, Deserialize)]
struct InputControlRequest {
    action: InputAction,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "data")]
enum InputAction {
    KeyPress(String),
    MouseMove { x: i32, y: i32 },
    MouseClick(String),
    WriteText(String),
}

#[derive(Serialize)]
struct InputControlResponse {
    success: bool,
}

// ================ Handlers ================

async fn find_elements_handler(
    State(_): State<Arc<AppState>>,
    Json(request): Json<FindElementsRequest>,
) -> Result<JsonResponse<FindElementsResponse>, (StatusCode, JsonResponse<serde_json::Value>)> {
    let desktop = match Desktop::new(
        request.selector.use_background_apps.unwrap_or(false),
        request.selector.activate_app.unwrap_or(false),
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

    let app = match desktop.application(&request.selector.app_name) {
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

    debug!("app: {:?}", app.text(1).unwrap_or_default());

    let elements = match app.locator(&*request.selector.locator) {
        Ok(locator) => {
            if request.max_results.unwrap_or(1) > 1 {
                match locator.all() {
                    Ok(elements) => elements,
                    Err(e) => {
                        error!("no matching elements found: {}", e);
                        return Err((
                            StatusCode::NOT_FOUND,
                            JsonResponse(serde_json::json!({ 
                                "error": "no matching elements found" 
                            })),
                        ));
                    }
                }
            } else {
                match locator.first() {
                    Ok(element) => {
                        if let Some(el) = element {
                            vec![el]
                        } else {
                            vec![]
                        }
                    }
                    Err(e) => {
                        error!("no matching element found: {}", e);
                        return Err((
                            StatusCode::NOT_FOUND,
                            JsonResponse(serde_json::json!({ 
                                "error": "no matching element found" 
                            })),
                        ));
                    }
                }
            }
        }
        Err(e) => {
            error!("failed to create locator: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                JsonResponse(serde_json::json!({ 
                    "error": format!("failed to create locator: {}", e) 
                })),
            ));
        }
    };

    if elements.is_empty() {
        error!("no matching elements found");
        return Err((
            StatusCode::NOT_FOUND,
            JsonResponse(serde_json::json!({ "error": "no matching elements found" })),
        ));
    }

    let elements_info: Vec<ElementInfo> = elements
        .into_iter()
        .map(|element| {
            debug!("element: {:?}", element);
            ElementInfo {
                id: element.id(),
                role: element.role(),
                label: element.attributes().label,
                description: element.attributes().description,
                text: element.text(request.max_depth.unwrap_or(10)).ok(),
                position: element.bounds().ok().map(|(x, y, _, _)| ElementPosition {
                    x: x as i32,
                    y: y as i32,
                }),
                size: element.bounds().ok().map(|(_, _, w, h)| ElementSize {
                    width: w as i32,
                    height: h as i32,
                }),
                properties: serde_json::json!(element.attributes().properties),
            }
        })
        .collect();

    Ok(JsonResponse(FindElementsResponse {
        data: elements_info,
    }))
}

async fn click_element_handler(
    State(_): State<Arc<AppState>>,
    Json(request): Json<ClickElementRequest>,
) -> Result<JsonResponse<ActionResponse>, (StatusCode, JsonResponse<serde_json::Value>)> {
    let desktop = match Desktop::new(
        request.selector.use_background_apps.unwrap_or(false),
        request.selector.activate_app.unwrap_or(true), // default to true for click
    ) {
        Ok(d) => d,
        Err(e) => {
            error!("failed to initialize desktop automation: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                JsonResponse(serde_json::json!({
                    "error": format!("failed to initialize desktop automation: {}", e)
                })),
            ));
        }
    };

    let app = match desktop.application(&request.selector.app_name) {
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

    debug!("app: {:?}", app.text(1).unwrap_or_default());

    let element = match app.locator(&*request.selector.locator) {
        Ok(locator) => match locator.first() {
            Ok(element) => element,
            Err(e) => {
                error!("failed to find elements: {}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    JsonResponse(serde_json::json!({
                        "error": format!("failed to find elements: {}", e)
                    })),
                ));
            }
        },
        Err(e) => {
            error!("failed to create locator: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                JsonResponse(serde_json::json!({
                    "error": format!("failed to create locator: {}", e)
                })),
            ));
        }
    };

    match element {
        Some(element) => match element.click() {
            Ok(_) => Ok(JsonResponse(ActionResponse {
                success: true,
                message: format!("clicked element with role: {}", element.role()),
            })),
            Err(e) => {
                error!("failed to click element: {}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    JsonResponse(serde_json::json!({
                        "error": format!("failed to click element: {}", e)
                    })),
                ))
            }
        },
        None => Err((
            StatusCode::NOT_FOUND,
            JsonResponse(serde_json::json!({
                "error": "no matching element found"
            })),
        )),
    }
}

async fn type_text_handler(
    State(_): State<Arc<AppState>>,
    Json(request): Json<TypeTextRequest>,
) -> Result<JsonResponse<ActionResponse>, (StatusCode, JsonResponse<serde_json::Value>)> {
    let desktop = match Desktop::new(
        request.selector.use_background_apps.unwrap_or(false),
        request.selector.activate_app.unwrap_or(true), // default to true for typing
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

    let app = match desktop.application(&request.selector.app_name) {
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

    let element = match app.locator(&*request.selector.locator) {
        Ok(locator) => match locator.first() {
            Ok(element) => element,
            Err(e) => {
                error!("failed to find elements: {}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    JsonResponse(serde_json::json!({
                        "error": format!("failed to find elements: {}", e)
                    })),
                ));
            }
        },
        Err(e) => {
            error!("failed to create locator: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                JsonResponse(serde_json::json!({
                    "error": format!("failed to create locator: {}", e)
                })),
            ));
        }
    };

    match element {
        Some(element) => match element.type_text(&request.text) {
            Ok(_) => Ok(JsonResponse(ActionResponse {
                success: true,
                message: format!("typed text into element with role: {}", element.role()),
            })),
            Err(e) => {
                error!("failed to type text: {}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    JsonResponse(serde_json::json!({
                        "error": format!("failed to type text: {}", e)
                    })),
                ))
            }
        },
        None => Err((
            StatusCode::NOT_FOUND,
            JsonResponse(serde_json::json!({
                "error": "no matching element found"
            })),
        )),
    }
}

async fn get_text_handler(
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

// MCP handlers
async fn mcp_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<MCPRequest>,
) -> JsonResponse<Value> {
    println!("received mcp request: {:?}", request);
    
    // Handle different MCP methods
    match request.method.as_str() {
        "initialize" => handle_initialize(request.id),
        "executeToolFunction" => {
            if let Some(params) = request.params {
                handle_execute_tool_function(state, request.id, params).await
            } else {
                mcp_error_response(request.id, -32602, "Invalid params".to_string(), None)
            }
        }
        _ => mcp_error_response(request.id, -32601, "Method not found".to_string(), None),
    }
}

fn handle_initialize(id: Value) -> JsonResponse<Value> {
    // Define available tool functions
    // let find_elements_schema = json!({
    //     "type": "object",
    //     "properties": {
    //         "selector": {
    //             "type": "object",
    //             "properties": {
    //                 "app_name": {"type": "string"},
    //                 "window_name": {"type": "string"},
    //                 "locator": {"type": "string"},
    //                 // ... other properties ...
    //             },
    //             "required": ["app_name", "locator"]
    //         },
    //         "max_results": {"type": "integer"},
    //         "max_depth": {"type": "integer"}
    //     },
    //     "required": ["selector"]
    // });
    // 
    // let click_element_schema = json!({
    //     "type": "object",
    //     "properties": {
    //         "selector": {
    //             "type": "object",
    //             "properties": {
    //                 "app_name": {"type": "string"},
    //                 "locator": {"type": "string"},
    //                 // ... other properties ...
    //             },
    //             "required": ["app_name", "locator"]
    //         }
    //     },
    //     "required": ["selector"]
    // });
    
    let get_text_schema = json!({
        "type": "object",
        "properties": {
            "app_name": {"type": "string"},
            "window_name": {"type": "string"},
            "max_depth": {"type": "integer"},
            "use_background_apps": {"type": "boolean"},
            "activate_app": {"type": "boolean"}
        },
        "required": ["app_name"]
    });
    
    // let type_text_schema = json!({
    //     "type": "object",
    //     "properties": {
    //         "selector": {
    //             "type": "object",
    //             "properties": {
    //                 "app_name": {"type": "string"},
    //                 "locator": {"type": "string"},
    //                 // ... other properties ...
    //             },
    //             "required": ["app_name", "locator"]
    //         },
    //         "text": {"type": "string"}
    //     },
    //     "required": ["selector", "text"]
    // });
    
    // // Add new schemas for all endpoints
    // let press_key_schema = json!({
    //     "type": "object",
    //     "properties": {
    //         "selector": {
    //             "type": "object",
    //             "properties": {
    //                 "app_name": {"type": "string"},
    //                 "locator": {"type": "string"},
    //                 // ... other properties ...
    //             },
    //             "required": ["app_name", "locator"]
    //         },
    //         "key_combo": {"type": "string"}
    //     },
    //     "required": ["selector", "key_combo"]
    // });
    
    // let scroll_element_schema = json!({
    //     "type": "object",
    //     "properties": {
    //         "selector": {
    //             "type": "object",
    //             "properties": {
    //                 "app_name": {"type": "string"},
    //                 "locator": {"type": "string"},
    //                 // ... other properties ...
    //             },
    //             "required": ["app_name", "locator"]
    //         },
    //         "direction": {"type": "string"},
    //         "amount": {"type": "number"}
    //     },
    //     "required": ["selector", "direction", "amount"]
    // });
    
    let list_interactable_elements_schema = json!({
        "type": "object",
        "properties": {
            "app_name": {"type": "string"},
            "window_name": {"type": "string"},
            "with_text_only": {"type": "boolean"},
            "interactable_only": {"type": "boolean"},
            "include_sometimes_interactable": {"type": "boolean"},
            "max_elements": {"type": "integer"},
            "use_background_apps": {"type": "boolean"},
            "activate_app": {"type": "boolean"}
        },
        "required": ["app_name"]
    });
    
    let click_by_index_schema = json!({
        "type": "object",
        "properties": {
            "element_index": {"type": "integer"}
        },
        "required": ["element_index"]
    });
    
    let type_by_index_schema = json!({
        "type": "object",
        "properties": {
            "element_index": {"type": "integer"},
            "text": {"type": "string"}
        },
        "required": ["element_index", "text"]
    });
    
    let press_key_by_index_schema = json!({
        "type": "object",
        "properties": {
            "element_index": {"type": "integer"},
            "key_combo": {"type": "string"}
        },
        "required": ["element_index", "key_combo"]
    });
    
    let open_application_schema = json!({
        "type": "object",
        "properties": {
            "app_name": {"type": "string"}
        },
        "required": ["app_name"]
    });
    
    let open_url_schema = json!({
        "type": "object",
        "properties": {
            "url": {"type": "string"},
            "browser": {"type": "string"}
        },
        "required": ["url"]
    });
    
    let input_control_schema = json!({
        "type": "object",
        "properties": {
            "action": {
                "type": "object",
                "properties": {
                    "type": {"type": "string", "enum": ["KeyPress", "MouseMove", "MouseClick", "WriteText"]},
                    "data": {"type": ["string", "object"]}
                },
                "required": ["type"]
            }
        },
        "required": ["action"]
    });
    
    // Define tool functions
    let tool_functions = vec![
        // Comment out the functions you don't want to expose
        /*
        ToolFunctionDefinition {
            name: "findElements".to_string(),
            description: "find ui elements in an application window".to_string(),
            parameters: find_elements_schema,
        },
        ToolFunctionDefinition {
            name: "clickElement".to_string(),
            description: "click on a ui element".to_string(),
            parameters: click_element_schema,
        },
        ToolFunctionDefinition {
            name: "typeText".to_string(),
            description: "type text into a ui element".to_string(),
            parameters: type_text_schema,
        },
        */
        // Keep functions you want to expose
        ToolFunctionDefinition {
            name: "getText".to_string(),
            description: "extract text from an application or browser window".to_string(),
            parameters: get_text_schema,
        },
        // Also comment out press_key if needed
        /*
        ToolFunctionDefinition {
            name: "pressKey".to_string(),
            description: "press key combination on a ui element".to_string(),
            parameters: press_key_schema,
        },
        */
        // Keep other functions...
        /*
        ToolFunctionDefinition {
            name: "scroll".to_string(),
            description: "scroll at a location: either by targeting a UI element, at specific screen coordinates, or at the current mouse position".to_string(),
            parameters: scroll_element_schema,
        },
        */
        ToolFunctionDefinition {
            name: "listInteractableElementsByIndex".to_string(),
            description: "list all interactable elements in an application and cache them for subsequent by-index operations. MUST BE CALLED FIRST before using any clickByIndex, typeByIndex, or pressKeyByIndex functions".to_string(),
            parameters: list_interactable_elements_schema,
        },
        ToolFunctionDefinition {
            name: "clickByIndex".to_string(),
            description: "click on a ui element by its index. REQUIRES listInteractableElementsByIndex to be called first to obtain valid indices".to_string(),
            parameters: click_by_index_schema,
        },
        ToolFunctionDefinition {
            name: "typeByIndex".to_string(),
            description: "type text into a ui element by its index. REQUIRES listInteractableElementsByIndex to be called first to obtain valid indices".to_string(),
            parameters: type_by_index_schema,
        },
        ToolFunctionDefinition {
            name: "pressKeyByIndex".to_string(),
            description: "press key combination on a ui element by its index. REQUIRES listInteractableElementsByIndex to be called first to obtain valid indices".to_string(),
            parameters: press_key_by_index_schema,
        },
        ToolFunctionDefinition {
            name: "openApplication".to_string(),
            description: "open an application".to_string(),
            parameters: open_application_schema,
        },
        ToolFunctionDefinition {
            name: "openUrl".to_string(),
            description: "open a url in a browser".to_string(),
            parameters: open_url_schema,
        },
        ToolFunctionDefinition {
            name: "inputControl".to_string(),
            description: "perform direct input control actions".to_string(),
            parameters: input_control_schema,
        },
    ];
    
    let capabilities = ServerCapabilities {
        tools: Some(ToolServerCapabilities {
            functions: tool_functions,
        }),
        resources: None, // Implement if needed
    };
    
    JsonResponse(json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "capabilities": capabilities
        }
    }))
}

async fn handle_execute_tool_function(
    state: Arc<AppState>,
    id: Value,
    params: Value,
) -> JsonResponse<Value> {
    // Parse the params
    let execute_params: ExecuteToolFunctionParams = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => {
            return mcp_error_response(
                id, 
                -32602, 
                format!("invalid params: {}", e), 
                None
            );
        }
    };
    
    info!("executing tool function: {} with args: {}", 
        execute_params.function, execute_params.arguments);
    
    // Execute the appropriate function
    match execute_params.function.as_str() {
        "findElements" => {
            let request: FindElementsRequest = match serde_json::from_value(execute_params.arguments) {
                Ok(r) => r,
                Err(e) => {
                    return mcp_error_response(
                        id, 
                        -32602, 
                        format!("invalid arguments: {}", e), 
                        None
                    );
                }
            };
            
            // Call existing handler and convert response
            match find_elements_handler(State(state), Json(request)).await {
                Ok(response) => {
                    JsonResponse(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "data": response.0.data
                        }
                    }))
                },
                Err((status, error_json)) => {
                    mcp_error_response(
                        id, 
                        status.as_u16() as i32, 
                        error_json.0["error"].as_str().unwrap_or("unknown error").to_string(),
                        None
                    )
                }
            }
        },
        "clickElement" => {
            let request: ClickElementRequest = match serde_json::from_value(execute_params.arguments) {
                Ok(r) => r,
                Err(e) => {
                    return mcp_error_response(
                        id, 
                        -32602, 
                        format!("invalid arguments: {}", e), 
                        None
                    );
                }
            };
            
            match click_element_handler(State(state), Json(request)).await {
                Ok(response) => {
                    JsonResponse(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "success": response.0.success,
                            "message": response.0.message
                        }
                    }))
                },
                Err((status, error_json)) => {
                    mcp_error_response(
                        id, 
                        status.as_u16() as i32, 
                        error_json.0["error"].as_str().unwrap_or("unknown error").to_string(),
                        None
                    )
                }
            }
        },
        "getText" => {
            let request: GetTextRequest = match serde_json::from_value(execute_params.arguments) {
                Ok(r) => r,
                Err(e) => {
                    return mcp_error_response(
                        id, 
                        -32602, 
                        format!("invalid arguments: {}", e), 
                        None
                    );
                }
            };
            
            match get_text_handler(State(state), Json(request)).await {
                Ok(response) => {
                    JsonResponse(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "success": response.0.success,
                            "text": response.0.text
                        }
                    }))
                },
                Err((status, error_json)) => {
                    mcp_error_response(
                        id, 
                        status.as_u16() as i32, 
                        error_json.0["error"].as_str().unwrap_or("unknown error").to_string(),
                        None
                    )
                }
            }
        },
        "typeText" => {
            let request: TypeTextRequest = match serde_json::from_value(execute_params.arguments) {
                Ok(r) => r,
                Err(e) => {
                    return mcp_error_response(
                        id, 
                        -32602, 
                        format!("invalid arguments: {}", e), 
                        None
                    );
                }
            };
            
            match type_text_handler(State(state), Json(request)).await {
                Ok(response) => {
                    JsonResponse(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "success": response.0.success,
                            "message": response.0.message
                        }
                    }))
                },
                Err((status, error_json)) => {
                    mcp_error_response(
                        id, 
                        status.as_u16() as i32, 
                        error_json.0["error"].as_str().unwrap_or("unknown error").to_string(),
                        None
                    )
                }
            }
        },
        // Add new function handlers
        "pressKey" => {
            let request: PressKeyRequest = match serde_json::from_value(execute_params.arguments) {
                Ok(r) => r,
                Err(e) => {
                    return mcp_error_response(
                        id, 
                        -32602, 
                        format!("invalid arguments: {}", e), 
                        None
                    );
                }
            };
            
            match press_key_handler(State(state.clone()), Json(request)).await {
                Ok(response) => {
                    JsonResponse(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "success": response.0.success,
                            "message": response.0.message
                        }
                    }))
                },
                Err((status, error_json)) => {
                    mcp_error_response(
                        id, 
                        status.as_u16() as i32, 
                        error_json.0["error"].as_str().unwrap_or("unknown error").to_string(),
                        None
                    )
                }
            }
        },
        // "scroll" => {
        //     let request: ScrollElementRequest = match serde_json::from_value(execute_params.arguments) {
        //         Ok(r) => r,
        //         Err(e) => {
        //             return mcp_error_response(
        //                 id, 
        //                 -32602, 
        //                 format!("invalid arguments: {}", e), 
        //                 None
        //             );
        //         }
        //     };
            
        //     match scroll_element_handler(State(state.clone()), Json(request)).await {
        //         Ok(response) => {
        //             JsonResponse(json!({
        //                 "jsonrpc": "2.0",
        //                 "id": id,
        //                 "result": {
        //                     "success": response.0.success,
        //                     "message": response.0.message
        //                 }
        //             }))
        //         },
        //         Err((status, error_json)) => {
        //             mcp_error_response(
        //                 id, 
        //                 status.as_u16() as i32, 
        //                 error_json.0["error"].as_str().unwrap_or("unknown error").to_string(),
        //                 None
        //             )
        //         }
        //     }
        // },
        "listInteractableElementsByIndex" => {
            let request: ListInteractableElementsRequest = match serde_json::from_value(execute_params.arguments) {
                Ok(r) => r,
                Err(e) => {
                    return mcp_error_response(
                        id, 
                        -32602, 
                        format!("invalid arguments: {}", e), 
                        None
                    );
                }
            };
            
            match list_interactable_elements_handler(State(state.clone()), Json(request)).await {
                Ok(response) => {
                    JsonResponse(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "elements": response.0.elements,
                            "stats": response.0.stats,
                            "cache_info": response.0.cache_info
                        }
                    }))
                },
                Err((status, error_json)) => {
                    mcp_error_response(
                        id, 
                        status.as_u16() as i32, 
                        error_json.0["error"].as_str().unwrap_or("unknown error").to_string(),
                        None
                    )
                }
            }
        },
        "clickByIndex" => {
            let request: ClickByIndexRequest = match serde_json::from_value(execute_params.arguments) {
                Ok(r) => r,
                Err(e) => {
                    return mcp_error_response(
                        id, 
                        -32602, 
                        format!("invalid arguments: {}", e), 
                        None
                    );
                }
            };
            
            match click_by_index_handler(State(state.clone()), Json(request)).await {
                Ok(response) => {
                    JsonResponse(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "success": response.0.success,
                            "message": response.0.message
                        }
                    }))
                },
                Err((status, error_json)) => {
                    mcp_error_response(
                        id, 
                        status.as_u16() as i32, 
                        error_json.0["error"].as_str().unwrap_or("unknown error").to_string(),
                        None
                    )
                }
            }
        },
        "typeByIndex" => {
            let request: TypeByIndexRequest = match serde_json::from_value(execute_params.arguments) {
                Ok(r) => r,
                Err(e) => {
                    return mcp_error_response(
                        id, 
                        -32602, 
                        format!("invalid arguments: {}", e), 
                        None
                    );
                }
            };
            
            match type_by_index_handler(State(state.clone()), Json(request)).await {
                Ok(response) => {
                    JsonResponse(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "success": response.0.success,
                            "message": response.0.message
                        }
                    }))
                },
                Err((status, error_json)) => {
                    mcp_error_response(
                        id, 
                        status.as_u16() as i32, 
                        error_json.0["error"].as_str().unwrap_or("unknown error").to_string(),
                        None
                    )
                }
            }
        },
        "pressKeyByIndex" => {
            let request: PressKeyByIndexRequest = match serde_json::from_value(execute_params.arguments) {
                Ok(r) => r,
                Err(e) => {
                    return mcp_error_response(
                        id, 
                        -32602, 
                        format!("invalid arguments: {}", e), 
                        None
                    );
                }
            };
            
            match press_key_by_index_handler(State(state.clone()), Json(request)).await {
                Ok(response) => {
                    JsonResponse(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "success": response.0.success,
                            "message": response.0.message
                        }
                    }))
                },
                Err((status, error_json)) => {
                    mcp_error_response(
                        id, 
                        status.as_u16() as i32, 
                        error_json.0["error"].as_str().unwrap_or("unknown error").to_string(),
                        None
                    )
                }
            }
        },
        "openApplication" => {
            let request: OpenApplicationRequest = match serde_json::from_value(execute_params.arguments) {
                Ok(r) => r,
                Err(e) => {
                    return mcp_error_response(
                        id, 
                        -32602, 
                        format!("invalid arguments: {}", e), 
                        None
                    );
                }
            };
            
            match open_application_handler(State(state.clone()), Json(request)).await {
                Ok(response) => {
                    JsonResponse(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "success": response.0.success,
                            "message": response.0.message
                        }
                    }))
                },
                Err((status, error_json)) => {
                    mcp_error_response(
                        id, 
                        status.as_u16() as i32, 
                        error_json.0["error"].as_str().unwrap_or("unknown error").to_string(),
                        None
                    )
                }
            }
        },
        "openUrl" => {
            let request: OpenUrlRequest = match serde_json::from_value(execute_params.arguments) {
                Ok(r) => r,
                Err(e) => {
                    return mcp_error_response(
                        id, 
                        -32602, 
                        format!("invalid arguments: {}", e), 
                        None
                    );
                }
            };
            
            match open_url_handler(State(state.clone()), Json(request)).await {
                Ok(response) => {
                    JsonResponse(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "success": response.0.success,
                            "message": response.0.message
                        }
                    }))
                },
                Err((status, error_json)) => {
                    mcp_error_response(
                        id, 
                        status.as_u16() as i32, 
                        error_json.0["error"].as_str().unwrap_or("unknown error").to_string(),
                        None
                    )
                }
            }
        },
        "inputControl" => {
            let request: InputControlRequest = match serde_json::from_value(execute_params.arguments) {
                Ok(r) => r,
                Err(e) => {
                    return mcp_error_response(
                        id, 
                        -32602, 
                        format!("invalid arguments: {}", e), 
                        None
                    );
                }
            };
            
            match input_control_handler(State(state.clone()), Json(request)).await {
                Ok(response) => {
                    JsonResponse(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "success": response.0.success
                        }
                    }))
                },
                Err((status, error_json)) => {
                    mcp_error_response(
                        id, 
                        status.as_u16() as i32, 
                        error_json.0["error"].as_str().unwrap_or("unknown error").to_string(),
                        None
                    )
                }
            }
        },
        _ => mcp_error_response(
            id, 
            -32601, 
            format!("function not found: {}", execute_params.function), 
            None
        ),
    }
}

fn mcp_error_response(id: Value, code: i32, message: String, data: Option<Value>) -> JsonResponse<Value> {
    JsonResponse(json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message,
            "data": data
        }
    }))
}

// Add the new handler functions

async fn press_key_handler(
    State(_): State<Arc<AppState>>,
    Json(request): Json<PressKeyRequest>,
) -> Result<JsonResponse<ActionResponse>, (StatusCode, JsonResponse<serde_json::Value>)> {
    debug!("pressing key combination: {}", request.key_combo);

    let desktop = match Desktop::new(
        request.selector.use_background_apps.unwrap_or(false),
        request.selector.activate_app.unwrap_or(true), // default to true for key press
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

    let app = match desktop.application(&request.selector.app_name) {
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

    let element = match app.locator(&*request.selector.locator) {
        Ok(locator) => match locator.first() {
            Ok(element) => element,
            Err(e) => {
                error!("failed to find elements: {}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    JsonResponse(json!({
                        "error": format!("failed to find elements: {}", e)
                    })),
                ));
            }
        },
        Err(e) => {
            error!("failed to create locator: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                JsonResponse(json!({
                    "error": format!("failed to create locator: {}", e)
                })),
            ));
        }
    };

    match element {
        Some(element) => match element.press_key(&request.key_combo) {
            Ok(_) => Ok(JsonResponse(ActionResponse {
                success: true,
                message: format!(
                    "successfully pressed key combination '{}' on element with role: {}",
                    request.key_combo,
                    element.role()
                ),
            })),
            Err(e) => {
                error!("failed to press key: {}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    JsonResponse(json!({
                        "error": format!("failed to press key: {}", e)
                    })),
                ))
            }
        },
        None => Err((
            StatusCode::NOT_FOUND,
            JsonResponse(json!({
                "error": "no matching element found"
            })),
        )),
    }
}

// async fn scroll_element_handler(
//     State(_): State<Arc<AppState>>,
//     Json(request): Json<ScrollElementRequest>,
// ) -> Result<JsonResponse<ActionResponse>, (StatusCode, JsonResponse<serde_json::Value>)> {
//     let desktop = match Desktop::new(
//         request.selector.as_ref().and_then(|s| s.use_background_apps).unwrap_or(false),
//         request.selector.as_ref().and_then(|s| s.activate_app).unwrap_or(false),
//     ) {
//         Ok(d) => d,
//         Err(e) => {
//             error!("failed to initialize desktop automation: {}", e);
//             return Err((
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 JsonResponse(json!({
//                     "error": format!("failed to initialize desktop automation: {}", e)
//                 })),
//             ));
//         }
//     };

//     // Determine where to scroll
//     match (&request.selector, &request.coordinates) {
//         // Case 1: Element-based scrolling (current behavior)
//         (Some(selector), _) => {
//             let app = match desktop.application(&selector.app_name) {
//                 Ok(app) => app,
//                 Err(e) => {
//                     error!("application not found: {}", e);
//                     return Err((
//                         StatusCode::NOT_FOUND,
//                         JsonResponse(json!({"error": format!("failed to find application: {}", e)})),
//                     ));
//                 }
//             };

//             let element = match app.locator(&*selector.locator) {
//                 Ok(locator) => locator.first(),
//                 Err(e) => {
//                     error!("failed to find element: {}", e);
//                     return Err((
//                         StatusCode::NOT_FOUND,
//                         JsonResponse(json!({"error": format!("failed to find element: {}", e)})),
//                     ));
//                 }
//             }
//             .map_err(|e| {
//                 error!("failed to find element: {}", e);
//                 (
//                     StatusCode::NOT_FOUND,
//                     JsonResponse(json!({"error": format!("failed to find element: {}", e)})),
//                 )
//             })?;

//             match element {
//                 Some(element) => {
//                     match element.scroll(&request.direction, request.amount) {
//                         Ok(_) => Ok(JsonResponse(ActionResponse {
//                             success: true,
//                             message: format!(
//                                 "successfully scrolled {} by {}",
//                                 request.direction, request.amount
//                             ),
//                         })),
//                         Err(e) => Err((
//                             StatusCode::INTERNAL_SERVER_ERROR,
//                             JsonResponse(json!({
//                                 "error": format!("failed to scroll element: {}", e)
//                             })),
//                         )),
//                     }
//                 }
//                 None => Err((
//                     StatusCode::NOT_FOUND,
//                     JsonResponse(json!({"error": "no element found"})),
//                 )),
//             }
//         },
        
//         // Case 2: Coordinate-based scrolling (new functionality)
//         (None, Some(coords)) => {
//             match desktop.scroll_at_position(coords.x as f64, coords.y as f64, 
//                                             &request.direction, request.amount) {
//                 Ok(_) => Ok(JsonResponse(ActionResponse {
//                     success: true,
//                     message: format!("successfully scrolled {} by {} at position ({}, {})", 
//                         request.direction, request.amount, coords.x, coords.y),
//                 })),
//                 Err(e) => {
//                     debug!("failed to scroll at position: {:?}", e);
//                     Err((
//                         StatusCode::INTERNAL_SERVER_ERROR,
//                         JsonResponse(json!({ "error": format!("failed to scroll at position: {:?}", e) }))
//                     ))
//                 }
//             }
//         },
        
//         // Case 3: Current mouse position scrolling (new functionality)
//         (None, None) => {
//             match desktop.scroll_at_current_position(&request.direction, request.amount) {
//                 Ok(_) => Ok(JsonResponse(ActionResponse {
//                     success: true,
//                     message: format!("successfully scrolled {} by {} at current position", 
//                         request.direction, request.amount),
//                 })),
//                 Err(e) => {
//                     debug!("failed to scroll at current position: {:?}", e);
//                     Err((
//                         StatusCode::INTERNAL_SERVER_ERROR,
//                         JsonResponse(json!({ "error": format!("failed to scroll at current position: {:?}", e) }))
//                     ))
//                 }
//             }
//         }
//     }
// }

async fn list_interactable_elements_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ListInteractableElementsRequest>,
) -> Result<JsonResponse<ListInteractableElementsResponse>, (StatusCode, JsonResponse<serde_json::Value>)> {
    // Set up the definitely and sometimes interactable role sets
    let definitely_interactable: std::collections::HashSet<&str> = [
        "AXButton",
        "AXMenuItem",
        "AXMenuBarItem",
        "AXCheckBox",
        "AXPopUpButton",
        "AXTextField",
        "AXTextArea",
        "AXComboBox",
        "AXLink",
        "AXScrollBar",
    ]
    .iter()
    .cloned()
    .collect();

    let sometimes_interactable: std::collections::HashSet<&str> = [
        "AXImage",
        "AXCell",
        "AXSplitter",
        "AXRow",
        "AXStatusItem",
    ]
    .iter()
    .cloned()
    .collect();

    // Create desktop automation engine
    let desktop = match Desktop::new(
        request.use_background_apps.unwrap_or(false),
        request.activate_app.unwrap_or(false),
    ) {
        Ok(d) => d,
        Err(e) => {
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
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                JsonResponse(json!({
                    "error": format!("failed to get elements: {}", e)
                })),
            ));
        }
    };

    info!("found {} elements in {}", elements.len(), request.app_name);

    // Filter and convert elements
    let mut result_elements = Vec::new();
    let mut stats = ElementStats {
        total: elements.len(),
        definitely_interactable: 0,
        sometimes_interactable: 0,
        non_interactable: 0,
        by_role: HashMap::new(),
    };

    for (i, element) in elements.iter().enumerate() {
        let role = element.role();

        // Count by role
        *stats.by_role.entry(role.clone()).or_insert(0) += 1;

        // Determine interactability
        let interactability = if definitely_interactable.contains(role.as_str()) {
            stats.definitely_interactable += 1;
            "definite"
        } else if sometimes_interactable.contains(role.as_str()) {
            stats.sometimes_interactable += 1;
            "sometimes"
        } else {
            stats.non_interactable += 1;
            "none"
        };

        // Extract text from element
        let text = element.text(10).unwrap_or_default();

        // Apply filters
        let with_text_condition = !request.with_text_only.unwrap_or(false) || !text.is_empty();
        let interactable_condition = !request.interactable_only.unwrap_or(false)
            || (interactability == "definite"
                || (request.include_sometimes_interactable.unwrap_or(false)
                    && interactability == "sometimes"));

        if with_text_condition && interactable_condition {
            let (x, y, width, height) = element.bounds().ok().unwrap_or((0.0, 0.0, 0.0, 0.0));

            result_elements.push(InteractableElement {
                index: i,
                role: role.clone(),
                interactability: interactability.to_string(),
                text,
                position: Some(ElementPosition {
                    x: x as i32,
                    y: y as i32,
                }),
                size: Some(ElementSize {
                    width: width as i32,
                    height: height as i32,
                }),
                element_id: element.id(),
            });
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

async fn click_by_index_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ClickByIndexRequest>,
) -> Result<JsonResponse<ClickByIndexResponse>, (StatusCode, JsonResponse<serde_json::Value>)> {
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
        Some((elements, timestamp, _app_name)) if timestamp.elapsed() < std::time::Duration::from_secs(30) => {
            // Use element_index directly
            if request.element_index < elements.len() {
                let element = &elements[request.element_index];

                match element.click() {
                    Ok(_) => Ok(JsonResponse(ClickByIndexResponse {
                        success: true,
                        message: format!(
                            "successfully clicked element with role: {}",
                            element.role()
                        ),
                    })),
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

async fn type_by_index_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<TypeByIndexRequest>,
) -> Result<JsonResponse<TypeByIndexResponse>, (StatusCode, JsonResponse<serde_json::Value>)> {
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
                                    
                                    Ok(JsonResponse(TypeByIndexResponse {
                                        success: true,
                                        message: format!(
                                            "successfully typed text into element with role: {} (using AppleScript fallback)",
                                            element.role()
                                        ),
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
                            Ok(JsonResponse(TypeByIndexResponse {
                                success: true,
                                message: format!(
                                    "successfully typed text into element with role: {}",
                                    element.role()
                                ),
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

async fn press_key_by_index_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<PressKeyByIndexRequest>,
) -> Result<JsonResponse<PressKeyByIndexResponse>, (StatusCode, JsonResponse<serde_json::Value>)> {
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
                    Ok(_) => Ok(JsonResponse(PressKeyByIndexResponse {
                        success: true,
                        message: format!(
                            "successfully pressed key combination '{}' on element with role: {}",
                            request.key_combo,
                            element.role()
                        ),
                    })),
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

async fn open_application_handler(
    State(_): State<Arc<AppState>>,
    Json(request): Json<OpenApplicationRequest>,
) -> Result<JsonResponse<OpenApplicationResponse>, (StatusCode, JsonResponse<serde_json::Value>)> {
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
        Ok(_) => Ok(JsonResponse(OpenApplicationResponse {
            success: true,
            message: format!("successfully opened application: {}", request.app_name),
        })),
        Err(err) => Err((
            StatusCode::BAD_REQUEST,
            JsonResponse(json!({"error": format!("failed to open application: {}", err)})),
        )),
    }
}

async fn open_url_handler(
    State(_): State<Arc<AppState>>,
    Json(request): Json<OpenUrlRequest>,
) -> Result<JsonResponse<OpenUrlResponse>, (StatusCode, JsonResponse<serde_json::Value>)> {
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

    // Open the URL
    let browser_ref = request.browser.as_deref();
    match desktop.open_url(&request.url, browser_ref) {
        Ok(_) => Ok(JsonResponse(OpenUrlResponse {
            success: true,
            message: format!("successfully opened URL: {}", request.url),
        })),
        Err(err) => Err((
            StatusCode::BAD_REQUEST,
            JsonResponse(json!({"error": format!("failed to open URL: {}", err)})),
        )),
    }
}

// Define the handler for input control
async fn input_control_handler(
    State(_): State<Arc<AppState>>,
    Json(payload): Json<InputControlRequest>,
) -> Result<JsonResponse<InputControlResponse>, (StatusCode, JsonResponse<serde_json::Value>)> {
    use std::process::Command;

    info!("input control handler {:?}", payload);
    
    // Execute appropriate input action
    match payload.action {
        InputAction::KeyPress(key) => {
            // Implement key press using appropriate library or command
            // This is a simplified example
            let script = format!("tell application \"System Events\" to key code {}", key);
            if let Err(e) = Command::new("osascript").arg("-e").arg(script).output() {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    JsonResponse(json!({"error": format!("failed to press key: {}", e)})),
                ));
            }
        }
        InputAction::MouseMove { x, y } => {
            // Implement mouse move
            let script = format!("tell application \"System Events\" to set mouse position to {{{}, {}}}", x, y);
            if let Err(e) = Command::new("osascript").arg("-e").arg(script).output() {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    JsonResponse(json!({"error": format!("failed to move mouse: {}", e)})),
                ));
            }
        }
        InputAction::MouseClick(button) => {
            // Implement mouse click
            let button_num = match button.as_str() {
                "left" => 1,
                "right" => 2,
                _ => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        JsonResponse(json!({"error": format!("unsupported mouse button: {}", button)})),
                    ));
                }
            };
            
            let script = format!("tell application \"System Events\" to click button {}", button_num);
            if let Err(e) = Command::new("osascript").arg("-e").arg(script).output() {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    JsonResponse(json!({"error": format!("failed to click mouse: {}", e)})),
                ));
            }
        }
        InputAction::WriteText(text) => {
            // Implement text writing
            let script = format!("tell application \"System Events\" to keystroke \"{}\"", text);
            if let Err(e) = Command::new("osascript").arg("-e").arg(script).output() {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    JsonResponse(json!({"error": format!("failed to write text: {}", e)})),
                ));
            }
        }
    }

    Ok(JsonResponse(InputControlResponse { success: true }))
}

// ================ Main ================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();
    
    info!("starting ui automation server");
    
    // Check permissions early - add this line
    check_os_permissions();
    
    // Create app state
    let app_state = Arc::new(AppState {
        element_cache: Arc::new(Mutex::new(None)),
    });

    // Create CORS layer
    let cors = CorsLayer::very_permissive();
    
    // Create router with both existing and MCP endpoints plus new endpoints
    let app = Router::new()
        // Existing routes
        // .route("/api/find-elements", post(find_elements_handler))
        // .route("/api/click", post(click_element_handler))
        // .route("/api/type", post(type_text_handler))
        .route("/api/get-text", post(get_text_handler))
        // Add MCP endpoint
        .route("/mcp", post(mcp_handler))
        // New routes matching screenPipe
        // .route("/api/press-key", post(press_key_handler))
        // .route("/api/scroll", post(scroll_element_handler))
        .route("/api/list-interactable-elements", post(list_interactable_elements_handler))
        .route("/api/click-by-index", post(click_by_index_handler))
        .route("/api/type-by-index", post(type_by_index_handler))
        .route("/api/press-key-by-index", post(press_key_by_index_handler))
        .route("/api/open-application", post(open_application_handler))
        .route("/api/open-url", post(open_url_handler))
        .route("/api/input-control", post(input_control_handler))
        .with_state(app_state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());
    
    // Get the address to bind to
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("listening on {}", addr);
    
    // Start the server
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}

// Add this function right after main imports but before the types
fn check_os_permissions() {
    // Only check on macOS
    #[cfg(target_os = "macos")]
    {
        use crate::platforms::macos::check_accessibility_permissions;
        
        match check_accessibility_permissions(true) {
            Ok(granted) => {
                if !granted {
                    info!("accessibility permissions: prompt shown to user");
                    // Sleep to give user time to respond to the prompt
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    
                    // Check again without prompt
                    match check_accessibility_permissions(false) {
                        Ok(_) => info!("accessibility permissions now granted"),
                        Err(e) => {
                            error!("accessibility permissions check failed: {}", e);
                            info!("**************************************************************");
                            info!("* ACCESSIBILITY PERMISSIONS REQUIRED                          *");
                            info!("* Go to System Preferences > Security & Privacy > Privacy >   *");
                            info!("* Accessibility and add this application.                     *");
                            info!("* Without this permission, UI automation will not function.   *");
                            info!("**************************************************************");
                        }
                    }
                } else {
                    info!("accessibility permissions already granted");
                }
            },
            Err(e) => {
                error!("accessibility permissions check failed: {}", e);
                info!("**************************************************************");
                info!("* ACCESSIBILITY PERMISSIONS REQUIRED                          *");
                info!("* Go to System Preferences > Security & Privacy > Privacy >   *");
                info!("* Accessibility and add this application.                     *");
                info!("* Without this permission, UI automation will not function.   *");
                info!("**************************************************************");
            }
        }
    }
}
