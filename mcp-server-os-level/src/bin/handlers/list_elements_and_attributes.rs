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
use serde_json::{self, json, Value};
use tracing::{error, info, debug};
use uuid::Uuid;
use core_foundation::string::CFString;

use crate::types::*;
use crate::AppState;
use accessibility::AXUIElement;
use computer_use_ai_sdk::platforms::tree_search::{TreeVisitor, TreeWalkerWithWindows, TreeWalkerFlow};

// Define the request structure
#[derive(serde::Deserialize)]
pub struct ListElementAttributesRequest {
    pub app_name: String,
    pub max_elements: Option<usize>,
    pub max_depth: Option<usize>,
    pub use_background_apps: Option<bool>,
    pub activate_app: Option<bool>,
    pub full_tree: Option<bool>,
}

// Define the response structure
#[derive(serde::Serialize)]
pub struct ListElementAttributesResponse {
    pub elements: Vec<ElementAttributesInfo>,
    pub stats: ElementStatsExtended,
    pub cache_info: ElementCacheInfo,
}

// Extended element info with all attributes
#[derive(serde::Serialize)]
pub struct ElementAttributesInfo {
    pub index: usize,
    pub role: String,
    pub attributes: HashMap<String, Value>,
    pub bounds: Option<(f64, f64, f64, f64)>,
    pub depth: usize,
    pub parent_index: Option<usize>,
    pub children_indices: Vec<usize>,
}

// Extended stats
#[derive(serde::Serialize)]
pub struct ElementStatsExtended {
    pub total: usize,
    pub by_role: HashMap<String, usize>,
    pub by_attribute: HashMap<String, usize>,
    pub max_depth: usize,
}

// Element collector that builds the full tree structure
struct ElementTreeCollector {
    elements: Vec<(AXUIElement, usize)>, // Element and depth
    current_depth: usize,
    max_depth: Option<usize>,
    max_elements: Option<usize>,
}

impl ElementTreeCollector {
    fn new(max_depth: Option<usize>, max_elements: Option<usize>) -> Self {
        Self {
            elements: Vec::new(),
            current_depth: 0,
            max_depth,
            max_elements,
        }
    }
    
    fn get_elements(&self) -> &Vec<(AXUIElement, usize)> {
        &self.elements
    }
}

// Modified implementation with interior mutability
impl TreeVisitor for ElementTreeCollector {
    fn enter_element(&self, element: &AXUIElement) -> TreeWalkerFlow {
        // Add the element to our collection with its depth
        if let Some(max_elements) = self.max_elements {
            if self.elements.len() >= max_elements {
                debug!("reached max elements limit of {}", max_elements);
                return TreeWalkerFlow::Exit;
            }
        }
        
        // Cannot modify self directly, so clone the element and temporarily create a mutable binding
        let mut elements = std::cell::RefCell::borrow_mut(&std::cell::RefCell::new(&self.elements));
        elements.push((element.clone(), self.current_depth));
        drop(elements);
        
        // Check if we've hit the max depth
        if let Some(max_depth) = self.max_depth {
            if self.current_depth >= max_depth {
                return TreeWalkerFlow::SkipSubtree;
            }
        }
        
        // Cannot modify self.current_depth directly
        // Just return based on the current depth without modifying it
        TreeWalkerFlow::Continue
    }
    
    fn exit_element(&self, _element: &AXUIElement) {
        // Cannot modify self.current_depth, so no-op
    }
}

// Wrapper to handle incremental depth tracking externally
struct DepthTrackingTreeWalker {
    collector: ElementTreeCollector,
    current_depth: usize,
}

impl DepthTrackingTreeWalker {
    fn new(max_depth: Option<usize>, max_elements: Option<usize>) -> Self {
        Self {
            collector: ElementTreeCollector::new(max_depth, max_elements),
            current_depth: 0,
        }
    }
    
    fn walk(&mut self, element: &AXUIElement) {
        self.walk_element(element, 0);
    }
    
    fn walk_element(&mut self, element: &AXUIElement, depth: usize) {
        // Check max elements limit
        if let Some(max) = self.collector.max_elements {
            if self.collector.elements.len() >= max {
                return;
            }
        }
        
        // Add current element with its depth
        self.collector.elements.push((element.clone(), depth));
        
        // Check depth limit
        if let Some(max_depth) = self.collector.max_depth {
            if depth >= max_depth {
                return;
            }
        }
        
        // Get children and traverse them
        if let Ok(children) = element.children() {
            for child in children {
                self.walk_element(&child, depth + 1);
            }
        }
    }
    
    fn get_collector(&self) -> &ElementTreeCollector {
        &self.collector
    }
}

pub async fn list_element_attributes_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ListElementAttributesRequest>,
) -> Result<JsonResponse<ListElementAttributesResponse>, (StatusCode, JsonResponse<serde_json::Value>)> {
    info!("listing all element attributes for app: {}", request.app_name);
    
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
    
    // Get the raw AXUIElement using our fixed method
    let app_ax = match desktop.get_application_ax(&request.app_name) {
        Ok(ax) => ax,
        Err(e) => {
            error!("failed to get AXUIElement from application: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                JsonResponse(json!({
                    "error": format!("failed to get AXUIElement from application: {}", e)
                })),
            ));
        }
    };
    
    // Use our custom tree walker
    let start_time = Instant::now();
    
    let mut element_infos = Vec::new();
    let mut stats = ElementStatsExtended {
        total: 0,
        by_role: HashMap::new(),
        by_attribute: HashMap::new(),
        max_depth: 0,
    };
    
    // Collect all elements using our custom traversal approach
    let mut tree_walker = DepthTrackingTreeWalker::new(
        request.max_depth,
        request.max_elements
    );
    
    tree_walker.walk(&app_ax);
    
    let elements_with_depth = &tree_walker.get_collector().get_elements();
    
    // Process collected elements
    // Instead of using HashMap with AXUIElement as key, use a Vec and index-based lookup
    let mut element_indices: HashMap<usize, usize> = HashMap::new(); // Map from element ptr address to index
    
    for (i, (element, depth)) in elements_with_depth.iter().enumerate() {
        // Track maximum depth
        if *depth > stats.max_depth {
            stats.max_depth = *depth;
        }
        
        // Get all attributes
        let mut all_attributes = HashMap::new();
        
        // Get element bounds (if available)
        let bounds = if let Ok(b) = element.bounds() {
            Some(b)
        } else {
            None
        };
        
        // Get role and convert to String
        let role = match element.role() {
            Ok(cf_role) => cf_role.to_string(),
            Err(_) => "unknown".to_string()
        };
        
        // Count by role for stats
        *stats.by_role.entry(role.clone()).or_insert(0) += 1;
        
        // Store element ptr for lookup
        let element_ptr = element.as_impl_ref() as *const _ as usize;
        element_indices.insert(element_ptr, i);
        
        // Get available attributes
        // Use accessibility API's available attribute names
        if let Ok(attr_names) = element.attributes() {
            for attr_name in attr_names {
                let name = attr_name.to_string();
                
                // Try to get the attribute value
                let value_json = match element.attribute_value(&attr_name) {
                    Ok(Some(val)) => {
                        // Convert to appropriate JSON type
                        if let Ok(s) = val.as_string() {
                            Value::String(s)
                        } else if let Ok(n) = val.as_number() {
                            match serde_json::Number::from_f64(n) {
                                Some(num) => Value::Number(num),
                                None => Value::Null
                            }
                        } else if let Ok(b) = val.as_bool() {
                            Value::Bool(b)
                        } else {
                            Value::String(format!("{:?}", val))
                        }
                    },
                    _ => Value::Null
                };
                
                all_attributes.insert(name.clone(), value_json);
                
                // Count attributes for stats
                *stats.by_attribute.entry(name).or_insert(0) += 1;
            }
        }
        
        // Add to results with empty children list (will be filled later)
        element_infos.push(ElementAttributesInfo {
            index: i,
            role: role.clone(),
            attributes: all_attributes,
            bounds,
            depth: *depth,
            parent_index: None, // Will fill this later
            children_indices: vec![], // Will fill this later
        });
    }
    
    // Now that we have all elements, build parent-child relationships
    for (i, (element, _)) in elements_with_depth.iter().enumerate() {
        // Get element's parent
        if let Ok(Some(parent)) = element.parent() {
            let parent_ptr = parent.as_impl_ref() as *const _ as usize;
            if let Some(&parent_idx) = element_indices.get(&parent_ptr) {
                // Set parent index in element info
                element_infos[i].parent_index = Some(parent_idx);
                
                // Add this element as child of parent
                if parent_idx < element_infos.len() {
                    element_infos[parent_idx].children_indices.push(i);
                }
            }
        }
    }
    
    // Update stats
    stats.total = element_infos.len();
    
    // Apply max_elements limit if specified (and not already applied during collection)
    if let Some(max) = request.max_elements {
        if element_infos.len() > max {
            element_infos.truncate(max);
        }
    }
    
    // Generate a cache ID and store elements in cache (we don't actually cache the raw elements)
    let cache_id = Uuid::new_v4().to_string();
    let cache_timestamp = Instant::now();
    let ttl_seconds: u64 = 30;
    
    info!("found {} elements with attributes in {} in {}ms", 
          element_infos.len(), 
          request.app_name,
          cache_timestamp.duration_since(start_time).as_millis());
    
    // Create cache info for response
    let now = chrono::Utc::now();
    let expires_at = now + chrono::Duration::seconds(ttl_seconds as i64);
    
    let cache_info = ElementCacheInfo {
        cache_id: cache_id.clone(),
        timestamp: now.to_rfc3339(),
        expires_at: expires_at.to_rfc3339(),
        element_count: element_infos.len(),
        ttl_seconds,
    };
    
    Ok(JsonResponse(ListElementAttributesResponse {
        elements: element_infos,
        stats,
        cache_info,
    }))
}
