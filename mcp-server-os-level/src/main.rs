use anyhow::Result;
use computer_use_ai_sdk::Desktop;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::fmt;

fn main() -> Result<()> {
    // initialize tracing
    fmt().with_max_level(LevelFilter::DEBUG).init();
    
    info!("initializing desktop automation sdk");
    
    let desktop = Desktop::new(true, false)?;
    info!("desktop accessibility initialized");
    
    // Your demo code here
    
    Ok(())
}
