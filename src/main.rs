// Main application entry point for testing and CLI usage
use dotenv::dotenv;
use schema_ui_system::{registry, render};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    // Initialize the schema registry (loads all themes and table schemas)
    let registry = registry();

    // Test rendering examples
    println!("=== Testing Schema Rendering ===");

    // Test user field rendering in different contexts
    if let Some(html) = render!("users", "name", "card", "John Doe") {
        println!("Name in card: {}", html);
    }

    if let Some(html) = render!("users", "email", "list", "john@example.com") {
        println!("Email in list: {}", html);
    }

    // Show available tables
    println!("\nAvailable tables: {:?}", registry.list_tables());

    Ok(())
}
