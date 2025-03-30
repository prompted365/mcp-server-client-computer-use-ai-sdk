use axum::{
    extract::{Json, State},
    response::Json as JsonResponse,
};
use serde_json::{self, json, Value};
use std::sync::Arc;
use tracing::{info, error};

use crate::types::{AppState, ExecuteToolFunctionParams, GetTextRequest, 
                   ListInteractableElementsRequest, MCPRequest, ServerCapabilities, 
                   ToolFunctionDefinition, ToolServerCapabilities,
                   ClickByIndexRequest, TypeByIndexRequest, PressKeyByIndexRequest,
                   OpenApplicationRequest, InputControlRequest, OpenUrlRequest};

// Update handler imports
use crate::handlers::get_text::get_text_handler;
use crate::handlers::list_elements_and_attributes::list_elements_and_attributes_handler;
use crate::handlers::click_by_index::click_by_index_handler;
use crate::handlers::type_by_index::type_by_index_handler;
use crate::handlers::press_key_by_index::press_key_by_index_handler;
use crate::handlers::open_application::open_application_handler;
use crate::handlers::open_url::open_url_handler;
use crate::handlers::input_control::input_control_handler;

// MCP handler
pub async fn mcp_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<MCPRequest>,
) -> JsonResponse<Value> {
    info!("received mcp request: {:?}", request);
    
    // Handle different MCP methods
    match request.method.as_str() {
        "initialize" => handle_initialize(request.id),
        "executeToolFunction" => {
            if let Some(params) = request.params {
                handle_execute_tool_function(state, request.id, params).await
            } else {
                mcp_error_response(request.id, -32602, "invalid params".to_string(), None)
            }
        }
        _ => mcp_error_response(request.id, -32601, "method not found".to_string(), None),
    }
}

// Handler for initialize method
pub fn handle_initialize(id: Value) -> JsonResponse<Value> {
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
        ToolFunctionDefinition {
            name: "getText".to_string(),
            description: "extract text from an application or browser window".to_string(),
            parameters: get_text_schema,
        },
        ToolFunctionDefinition {
            name: "clickByIndex".to_string(),
            description: "click on a ui element by its index and returns the updated element list".to_string(),
            parameters: click_by_index_schema,
        },
        ToolFunctionDefinition {
            name: "typeByIndex".to_string(),
            description: "type text into a ui element by its index and returns the updated element list".to_string(),
            parameters: type_by_index_schema,
        },
        ToolFunctionDefinition {
            name: "pressKeyByIndex".to_string(),
            description: "press key combination on a ui element by its index and returns the updated element list".to_string(),
            parameters: press_key_by_index_schema,
        },
        ToolFunctionDefinition {
            name: "openApplication".to_string(),
            description: "open an application and return the list of interactable elements in the app".to_string(),
            parameters: open_application_schema,
        },
        ToolFunctionDefinition {
            name: "openUrl".to_string(),
            description: "open a url in a browser and return the list of interactable elements in the browser".to_string(),
            parameters: open_url_schema,
        },
        ToolFunctionDefinition {
            name: "inputControl".to_string(),
            description: "perform direct input control actions and return the list of interactable elements from the current app".to_string(),
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

// Handler for executeToolFunction method
pub async fn handle_execute_tool_function(
    state: Arc<AppState>,
    id: Value,
    params: Value,
) -> JsonResponse<Value> {
    // Parse the params
    let execute_params: ExecuteToolFunctionParams = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => {
            error!("invalid params: {}", e);
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
        "getText" => {
            let request: GetTextRequest = match serde_json::from_value(execute_params.arguments) {
                Ok(r) => r,
                Err(e) => {
                    error!("invalid arguments: {}", e);
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
        "listInteractableElementsByIndex" => {
            let request: ListInteractableElementsRequest = match serde_json::from_value(execute_params.arguments) {
                Ok(r) => r,
                Err(e) => {
                    error!("invalid arguments: {}", e);
                    return mcp_error_response(
                        id, 
                        -32602, 
                        format!("invalid arguments: {}", e), 
                        None
                    );
                }
            };
            
            match list_elements_and_attributes_handler(State(state.clone()), Json(request)).await {
                Ok(response) => {
                    JsonResponse(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "elements": response.0.elements,
                            "stats": response.0.stats,
                            "cache_info": response.0.cache_info,
                            "processing_time_seconds": response.0.processing_time_seconds
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
                    error!("invalid arguments: {}", e);
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
                            "click": {
                                "success": response.0.click.success,
                                "message": response.0.click.message
                            },
                            "elements": response.0.elements
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
                    error!("invalid arguments: {}", e);
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
                            "type_action": {
                                "success": response.0.type_action.success,
                                "message": response.0.type_action.message
                            },
                            "elements": response.0.elements
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
                    error!("invalid arguments: {}", e);
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
                            "press_key": {
                                "success": response.0.press_key.success,
                                "message": response.0.press_key.message
                            },
                            "elements": response.0.elements
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
                    error!("invalid arguments: {}", e);
                    return mcp_error_response(
                        id, 
                        -32602, 
                        format!("invalid arguments: {}", e), 
                        None
                    );
                }
            };
            
            match open_application_handler(State(state), Json(request)).await {
                Ok(response) => {
                    JsonResponse(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "application": {
                                "success": response.0.application.success,
                                "message": response.0.application.message
                            },
                            "elements": response.0.elements
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
            let types_request: OpenUrlRequest = match serde_json::from_value(execute_params.arguments) {
                Ok(r) => r,
                Err(e) => {
                    error!("invalid arguments: {}", e);
                    return mcp_error_response(
                        id, 
                        -32602, 
                        format!("invalid arguments: {}", e), 
                        None
                    );
                }
            };
            
            // Convert from types::OpenUrlRequest to the handler's OpenUrlRequest
            let request = crate::handlers::open_url::OpenUrlRequest {
                url: types_request.url,
                browser: types_request.browser,
            };
            
            match open_url_handler(State(state.clone()), Json(request)).await {
                Ok(response) => {
                    JsonResponse(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "url": {
                                "success": response.0.url.success,
                                "message": response.0.url.message
                            },
                            "elements": response.0.elements
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
                    error!("invalid arguments: {}", e);
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
                            "input": {
                                "success": response.0.input.success
                            },
                            "elements": response.0.elements
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

// Helper function for MCP error responses
pub fn mcp_error_response(id: Value, code: i32, message: String, data: Option<Value>) -> JsonResponse<Value> {
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
