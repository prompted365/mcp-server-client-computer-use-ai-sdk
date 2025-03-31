use std::{net::SocketAddr, sync::Arc};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

use axum::{
    routing::post,
    Router,
};
use tokio::sync::Mutex;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{error, info, level_filters::LevelFilter};
use serde_json::{json, Value};
mod types;
use types::*;
mod handlers;

// Import only the handlers actually used
use handlers::mcp::mcp_handler;
use handlers::click_by_index::click_by_index_handler;
use handlers::type_by_index::type_by_index_handler;
use handlers::press_key_by_index::press_key_by_index_handler;
use handlers::open_application::open_application_handler;
use handlers::open_url::open_url_handler;
use handlers::input_control::input_control_handler;
use handlers::list_elements_and_attributes::list_elements_and_attributes_handler;
use handlers::utils::*;

// Import mcp_handler helpers but we'll call them directly
use handlers::mcp::{handle_initialize, handle_execute_tool_function, mcp_error_response};

// Import additional tokio features
use tokio::time::{Duration, sleep};

// ================ Main ================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Check if we should use STDIO mode
    let use_stdio = std::env::args().any(|arg| arg == "--stdio");
    
    // initialize tracing with different settings based on mode
    if use_stdio {
        // For STDIO mode, disable colors and only log to stderr
        tracing_subscriber::fmt()
            .with_max_level(LevelFilter::INFO)
            .with_ansi(false)  // Disable ANSI color codes
            .with_writer(std::io::stderr)  // Only write logs to stderr
            .init();
    } else {
        // For HTTP mode, use default settings
        tracing_subscriber::fmt()
            .with_max_level(LevelFilter::DEBUG)
            .init();
    }
    
    info!("starting ui automation server");
    
    // Check permissions early
    check_os_permissions();
    
    // Create app state
    let app_state = Arc::new(AppState {
        element_cache: Arc::new(Mutex::new(None)),
    });

    if use_stdio {
        info!("running in STDIO mode for MCP");
        info!("initializing stdio transport for MCP");
        
        eprintln!("starting stdio server, entering message loop...");
        
        loop {
            info!("starting new STDIO server session");
            match run_stdio_session(app_state.clone()).await {
                Ok(_) => info!("stdio session completed successfully"),
                Err(e) => {
                    if e.to_string().contains("EOF") {
                        info!("EOF detected, preparing for next session");
                    } else {
                        error!("stdio session error: {}", e);
                    }
                }
            }
            
            // Avoid busy waiting - sleep briefly
            sleep(Duration::from_millis(10)).await;
        }
        
        info!("mcp server shutting down");
    } else {
        info!("running in HTTP mode on port 8080");
        run_http_server(app_state).await?;
    }
    
    eprintln!("main function complete - this should only print if the stdin loop ends properly");
    
    Ok(())
}

// New function for handling a single STDIO session
async fn run_stdio_session(app_state: Arc<AppState>) -> anyhow::Result<()> {
    info!("initializing stdio session");
    
    let stdin = tokio::io::stdin();
    let mut stdin_reader = BufReader::new(stdin);
    let mut stdout = tokio::io::stdout();
    
    // Process multiple requests in the same session
    loop {
        let mut line = String::new();
        
        // Read message
        info!("waiting for next message...");
        match stdin_reader.read_line(&mut line).await {
            Ok(0) => {
                info!("stdin closed (EOF), exiting session");
                return Err(anyhow::anyhow!("EOF detected"));
            },
            Ok(bytes) => {
                let trimmed_line = line.trim();
                if trimmed_line.is_empty() {
                    info!("empty message received ({} bytes), continuing", bytes);
                    continue;
                }
                
                info!("received message: {} bytes", trimmed_line.len());
                
                // Process the message
                match process_message(trimmed_line, &app_state).await {
                    Ok(response) => {
                        if !response.is_empty() {
                            info!("sending response: {} bytes", response.len());
                            stdout.write_all(response.as_bytes()).await?;
                            stdout.write_all(b"\n").await?;
                            stdout.flush().await?;
                            info!("response sent successfully, ready for next message");
                        } else {
                            info!("no response needed for this message, ready for next message");
                        }
                    },
                    Err(e) => {
                        error!("error processing message: {}", e);
                        // Send error response
                        let error_response = json!({
                            "jsonrpc": "2.0", 
                            "id": null,
                            "error": {
                                "code": -32603,
                                "message": format!("Internal error: {}", e)
                            }
                        }).to_string();
                        
                        stdout.write_all(error_response.as_bytes()).await?;
                        stdout.write_all(b"\n").await?;
                        stdout.flush().await?;
                        info!("error response sent, continuing session");
                    }
                }
            },
            Err(e) => {
                error!("error reading from stdin: {}", e);
                return Err(anyhow::anyhow!("stdin read error: {}", e));
            }
        }
    }
}

async fn process_message(message: &str, app_state: &Arc<AppState>) -> anyhow::Result<String> {
    let request: Value = serde_json::from_str(message)?;
    
    // Special handling for notifications (no id field)
    if request.get("id").is_none() && request.get("method").is_some() {
        let method = request.get("method").unwrap().as_str().unwrap_or("");
        info!("received notification: {}", method);
        
        if method == "initialized" {
            info!("client sent initialized notification, connection complete");
            // No response needed for notifications
            return Ok(String::new());
        }
        
        // Other notification types can be handled here if needed
        return Ok(String::new());
    }
    
    if let Ok(mcp_request) = serde_json::from_value::<MCPRequest>(request.clone()) {
        let method = mcp_request.method.as_str();
        let id = mcp_request.id.to_string();
        info!("processing request - method: {}, id: {}", method, id);
        
        let response = match method {
            "initialize" => {
                info!("handling initialize request (id: {})", id);
                let response = handle_initialize(mcp_request.id);
                let response_str = serde_json::to_string(&response.0)?;
                info!("initialize response prepared, sending...");
                return Ok(response_str);
            }
            "executeToolFunction" => {
                if let Some(params) = mcp_request.params {
                    handle_execute_tool_function(app_state.clone(), mcp_request.id, params).await
                } else {
                    mcp_error_response(mcp_request.id, -32602, "invalid params".to_string(), None)
                }
            }
            _ => {
                info!("unknown method: {}", method);
                mcp_error_response(mcp_request.id, -32601, "method not found".to_string(), None)
            },
        };
        
        Ok(serde_json::to_string(&response.0)?)
    } else {
        let error_response = json!({
            "jsonrpc": "2.0",
            "id": request.get("id").unwrap_or(&Value::Null),
            "error": {
                "code": -32600,
                "message": "Invalid Request"
            }
        });
        
        Ok(error_response.to_string())
    }
}

async fn run_http_server(app_state: Arc<AppState>) -> anyhow::Result<()> {
    // Create CORS layer
    let cors = CorsLayer::very_permissive();
    
    // Create router with both existing and MCP endpoints plus new endpoints
    let app = Router::new()
        .route("/mcp", post(mcp_handler))
        .route("/api/click-by-index", post(click_by_index_handler))
        .route("/api/type-by-index", post(type_by_index_handler))
        .route("/api/press-key-by-index", post(press_key_by_index_handler))
        .route("/api/open-application", post(open_application_handler))
        .route("/api/open-url", post(open_url_handler))
        .route("/api/input-control", post(input_control_handler))
        .route("/api/list-elements-and-attributes", post(list_elements_and_attributes_handler))
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
        use computer_use_ai_sdk::platforms::macos::check_accessibility_permissions;
        
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
