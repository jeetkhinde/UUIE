// Simple rendering example without database dependency
use schema_ui_system::{Renderer, registry, render};
use std::collections::HashMap;

fn main() {
    println!("=== Schema UI System - Simple Rendering Demo ===\n");

    // Initialize renderer
    let renderer = Renderer::new();
    let schema_registry = registry();

    // Show available tables
    println!("Available tables: {:?}", schema_registry.list_tables());

    // Test basic rendering with macro
    println!("\n--- Basic Rendering ---");
    if let Some(html) = render!("users", "name", "card", "John Doe") {
        println!("Name field: {}", html);
    }

    if let Some(html) = render!("users", "email", "card", "john@example.com") {
        println!("Email field: {}", html);
    }

    // Test renderer methods
    println!("\n--- Renderer Methods ---");

    // Create sample user data
    let mut user_data = HashMap::new();
    user_data.insert("name".to_string(), "Alice Johnson".to_string());
    user_data.insert("email".to_string(), "alice@example.com".to_string());
    user_data.insert(
        "avatar_url".to_string(),
        "https://example.com/alice.jpg".to_string(),
    );
    user_data.insert("created_at".to_string(), "2024-01-15T10:30:00Z".to_string());

    // Render complete record
    let rendered = renderer.render_record("users", "card", &user_data);
    println!("Rendered record:");
    for (field, html) in &rendered {
        println!("  {}: {}", field, html);
    }

    // Test component rendering
    println!("\n--- Component Template ---");
    let template = r#"
<div class="user-card">
    <h3>User Profile</h3>
    <p>Name: {name}</p>
    <p>Email: {email}</p>
    <p>Joined: {created_at}</p>
</div>"#;

    let component_html = renderer.render_component(template, "users", "card", &user_data);
    println!("Component HTML: {}", component_html);

    // List schema information
    println!("\n--- Schema Information ---");
    println!(
        "Contexts for 'users': {:?}",
        renderer.list_contexts("users")
    );
    println!(
        "Variants for 'users.name': {:?}",
        renderer.list_field_variants("users", "name")
    );
    // ADD: Test mock data functionality
    println!("\n--- Mock Data Demo ---");

    // Get all mock records
    let mock_records = schema_registry.get_mock_data("users");
    println!("Mock data count: {}", mock_records.len());

    // Render each mock record
    for (i, record) in mock_records.iter().enumerate() {
        println!("\nMock User {}:", i + 1);
        let rendered = renderer.render_record("users", "card", record);
        for (field, html) in &rendered {
            println!("  {}: {}", field, html);
        }
    }

    // Get specific record by ID
    if let Some(record) = schema_registry.get_mock_record("users", "2") {
        println!("\nSpecific user (ID=2):");
        println!(
            "Name: {}",
            record.get("name").unwrap_or(&"Unknown".to_string())
        );
        println!(
            "Email: {}",
            record.get("email").unwrap_or(&"Unknown".to_string())
        );
    }

    // Get limited records
    let limited = schema_registry.get_mock_records("users", Some(2));
    println!("\nLimited to 2 records: {} found", limited.len());

    println!("\n=== Demo Complete ===");
}
