// Renderer module - handles HTML generation without database dependency
use crate::schema::{SchemaRegistry, registry};
use std::collections::HashMap;

// Renderer provides high-level rendering utilities
pub struct Renderer {
    registry: &'static SchemaRegistry,
}

impl Renderer {
    // Create new renderer instance
    pub fn new() -> Self {
        Self {
            registry: registry(),
        }
    }

    // Render a single field value
    pub fn render_field(
        &self,
        table: &str,
        field: &str,
        context: &str,
        value: &str,
    ) -> Option<String> {
        self.registry.render_field(table, field, context, value)
    }

    // Render multiple fields for a record (e.g., entire user object)
    pub fn render_record(
        &self,
        table: &str,
        context: &str,
        data: &HashMap<String, String>,
    ) -> HashMap<String, String> {
        let mut rendered = HashMap::new();

        for (field, value) in data {
            if let Some(html) = self.render_field(table, field, context, value) {
                rendered.insert(field.clone(), html);
            }
        }

        rendered
    }

    // Render component template with field substitution
    pub fn render_component(
        &self,
        template: &str,
        table: &str,
        context: &str,
        data: &HashMap<String, String>,
    ) -> String {
        let mut result = template.to_string();

        // Replace {field_name} placeholders with rendered HTML
        for (field, value) in data {
            let placeholder = format!("{{{}}}", field);
            if let Some(rendered_field) = self.render_field(table, field, context, value) {
                result = result.replace(&placeholder, &rendered_field);
            }
        }

        result
    }

    // List available contexts for a table
    pub fn list_contexts(&self, table: &str) -> Vec<String> {
        if let Some(schema) = self.registry.get_table(table) {
            schema.contexts.keys().cloned().collect()
        } else {
            vec![]
        }
    }

    // List available variants for a field
    pub fn list_field_variants(&self, table: &str, field: &str) -> Vec<String> {
        self.registry
            .get_table(table)
            .and_then(|schema| schema.variants.get(field))
            .map(|variants| variants.keys().cloned().collect())
            .unwrap_or_default()
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_creation() {
        let renderer = Renderer::new();
        // Test that renderer can access registry
        let tables = renderer.registry.list_tables();
        println!("Available tables: {:?}", tables);
    }

    #[test]
    fn test_render_simple() {
        let renderer = Renderer::new();

        // Test basic rendering (currently returns simple span)
        if let Some(html) = renderer.render_field("users", "name", "card", "Test User") {
            assert!(html.contains("Test User"));
        }
    }
}
