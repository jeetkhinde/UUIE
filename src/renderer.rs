// Renderer module - handles HTML generation and template processing
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
        if let Some(schema) = self.registry.get_table(table) {
            if let Some(field_variants) = schema.variants.get(field) {
                return field_variants.keys().cloned().collect();
            }
        }
        vec![]
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
    use std::collections::HashMap;

    #[test]
    fn test_render_single_field() {
        let renderer = Renderer::new();

        // Test rendering a user name in card context
        if let Some(html) = renderer.render_field("users", "name", "card", "John Doe") {
            assert!(html.contains("John Doe"));
            assert!(html.contains("<h2"));
        }
    }

    #[test]
    fn test_render_record() {
        let renderer = Renderer::new();

        let mut user_data = HashMap::new();
        user_data.insert("name".to_string(), "Jane Smith".to_string());
        user_data.insert("email".to_string(), "jane@example.com".to_string());

        let rendered = renderer.render_record("users", "card", &user_data);

        assert!(rendered.contains_key("name"));
        assert!(rendered.contains_key("email"));
    }

    #[test]
    fn test_render_component() {
        let renderer = Renderer::new();

        let template = r#"
        <div class="user-card">
            {name}
            {email}
        </div>
        "#;

        let mut user_data = HashMap::new();
        user_data.insert("name".to_string(), "Bob Wilson".to_string());
        user_data.insert("email".to_string(), "bob@example.com".to_string());

        let result = renderer.render_component(template, "users", "card", &user_data);

        assert!(result.contains("Bob Wilson"));
        assert!(result.contains("bob@example.com"));
        assert!(result.contains("<div class=\"user-card\">"));
    }
}
