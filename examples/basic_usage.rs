// Basic usage example - demonstrates complete workflow
use dotenv::dotenv;
use schema_ui_system::{Database, Renderer, registry};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv().ok();

    println!("=== Schema UI System Demo ===\n");

    // Step 1: Initialize components
    let renderer = Renderer::new();
    let registry = registry();

    // Step 2: Connect to database (optional)
    match Database::new().await {
        Ok(db) => {
            println!("✓ Connected to Supabase database");

            // Load table schemas into database
            if let Err(e) = db.load_table_schema("users").await {
                println!("⚠ Warning: Could not load users schema: {}", e);
            } else {
                println!("✓ Loaded users table schema");
            }
        }
        Err(e) => {
            println!("⚠ Database connection failed: {}", e);
            println!("  (Continuing with schema rendering demo...)");
        }
    }

    // Step 3: Demo schema rendering
    println!("\n=== Schema Rendering Demo ===");

    // Create sample user data
    let mut user_data = HashMap::new();
    user_data.insert("name".to_string(), "Alice Johnson".to_string());
    user_data.insert("email".to_string(), "alice@example.com".to_string());
    user_data.insert(
        "avatar_url".to_string(),
        "https://example.com/alice.jpg".to_string(),
    );
    user_data.insert("created_at".to_string(), "2024-01-15T10:30:00Z".to_string());

    // Demo 1: Render individual fields in different contexts
    println!("\n--- Individual Field Rendering ---");

    let contexts = ["card", "list"];
    let fields = ["name", "email", "avatar_url", "created_at"];

    for context in &contexts {
        println!("\n{} context:", context.to_uppercase());
        for field in &fields {
            if let Some(value) = user_data.get(*field) {
                if let Some(html) = renderer.render_field("users", field, context, value) {
                    println!("  {}: {}", field, html);
                }
            }
        }
    }

    // Demo 2: Render complete record
    println!("\n--- Complete Record Rendering ---");

    let rendered_card = renderer.render_record("users", "card", &user_data);
    println!("\nCard context - all fields:");
    for (field, html) in &rendered_card {
        println!("  {}: {}", field, html);
    }

    // Demo 3: Render component template
    println!("\n--- Component Template Rendering ---");

    let card_template = r#"
<div class="bg-white rounded-lg shadow-md p-6 max-w-sm">
    <div class="flex items-center space-x-4">
        {avatar_url}
        <div class="flex-1">
            {name}
            {email}
            <div class="mt-2">
                {created_at}
            </div>
        </div>
    </div>
</div>"#;

    let rendered_component = renderer.render_component(card_template, "users", "card", &user_data);
    println!("\nUser Card Component:");
    println!("{}", rendered_component);

    // Demo 4: List available schemas and variants
    println!("\n--- Schema Information ---");

    println!("\nAvailable tables: {:?}", registry.list_tables());

    println!(
        "\nAvailable contexts for 'users': {:?}",
        renderer.list_contexts("users")
    );

    println!(
        "\nAvailable variants for 'users.name': {:?}",
        renderer.list_field_variants("users", "name")
    );

    // Demo 5: Theme switching (if implemented)
    println!("\n--- Theme Demo ---");

    // Note: Theme switching would require mutable registry
    println!("Current theme: light (theme switching demo would go here)");

    // Demo 6: Error handling
    println!("\n--- Error Handling Demo ---");

    // Try to render non-existent field
    match renderer.render_field("users", "nonexistent_field", "card", "test") {
        Some(html) => println!("Unexpected success: {}", html),
        None => println!("✓ Correctly handled non-existent field"),
    }

    // Try to render non-existent context
    match renderer.render_field("users", "name", "nonexistent_context", "test") {
        Some(html) => println!("Unexpected success: {}", html),
        None => println!("✓ Correctly handled non-existent context"),
    }

    println!("\n=== Demo Complete ===");

    Ok(())
}
