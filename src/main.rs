// Main application entry point for testing and CLI usage
// src/main.rs
use dotenv::dotenv;
use schema_ui_system::{component_registry, start_server};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv().ok();

    // Initialize registries (this loads all schemas and components)
    let _component_registry = component_registry();

    println!("=== Schema UI Component System ===");
    println!("🔧 Initialized schema registry");
    println!(
        "🧩 Discovered components: {:?}",
        _component_registry.list_components()
    );

    // Start web server
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    start_server(port).await?;

    Ok(())
}
