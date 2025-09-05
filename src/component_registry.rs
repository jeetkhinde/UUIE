// src/component_registry.rs - New file for component discovery
use crate::schema::{SchemaRegistry, registry};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ComponentTemplate {
    pub name: String,
    pub table: String,                // which table this component belongs to
    pub template: String,             // HTML template with {field} placeholders
    pub required_fields: Vec<String>, // fields needed for this component
}
// Add this struct before ComponentRegistry:
#[derive(Debug, Default)]
pub struct RenderParams<'a> {
    pub context: Option<&'a str>,
    pub theme: Option<&'a str>,
    pub platform: Option<&'a str>,
    pub format: Option<&'a str>,
    pub lang: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct ComponentRegistry {
    components: HashMap<String, ComponentTemplate>,
    schema_registry: &'static SchemaRegistry,
}
impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
impl ComponentRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            components: HashMap::new(),
            schema_registry: registry(),
        };

        // Auto-discover all components from schema files
        registry.discover_components();
        registry
    }

    // 🔍 Auto-discover components from SQL files
    fn discover_components(&mut self) {
        // For now, hardcoded discovery - later we'll scan directories
        let component_definitions = [
            (
                "user_card",
                "users",
                r#"<div class="bg-white rounded-lg shadow-md p-6">
                    <div class="flex items-center space-x-4">
                        {avatar_url}
                        <div>
                            {name}
                            {email}
                            {created_at}
                        </div>
                    </div>
                </div>"#,
            ),
            // Future components auto-discovered here:
            // ("user_list_item", "users", template),
            // ("product_card", "products", template),
        ];

        for (name, table, template) in component_definitions {
            let required_fields = self.extract_field_placeholders(template);

            self.components.insert(
                name.to_string(),
                ComponentTemplate {
                    name: name.to_string(),
                    table: table.to_string(),
                    template: template.to_string(),
                    required_fields,
                },
            );
        }
    }

    // Extract {field} placeholders from template
    fn extract_field_placeholders(&self, template: &str) -> Vec<String> {
        let mut fields = Vec::new();
        let mut chars = template.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                let mut field = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '}' {
                        chars.next(); // consume '}'
                        break;
                    }
                    field.push(chars.next().unwrap());
                }
                if !field.is_empty() {
                    fields.push(field);
                }
            }
        }

        fields.sort();
        fields.dedup();
        fields
    }

    // 🎯 Main API: Render component with parameters
    pub async fn render_component(
        &self,
        component_name: &str,
        record_id: &str,
        params: RenderParams<'_>,
    ) -> Result<String, ComponentError> {
        // 1. Find component template
        let component =
            self.components
                .get(component_name)
                .ok_or(ComponentError::ComponentNotFound(
                    component_name.to_string(),
                ))?;

        // 2. Get data for this record (mock data for now)
        let record_data = self
            .schema_registry
            .get_mock_record(&component.table, record_id)
            .ok_or(ComponentError::RecordNotFound(record_id.to_string()))?;

        // 3. Apply theme (future: per-request theme switching)
        let context = params.context.unwrap_or("card");

        // 4. Render each field with schema styling
        let rendered_fields: HashMap<_, _> = component
            .required_fields
            .iter()
            .filter_map(|field| {
                record_data
                    .get(field)
                    .and_then(|field_value| {
                        self.schema_registry.render_field(
                            &component.table,
                            field,
                            context,
                            field_value,
                        )
                    })
                    .map(|rendered_html| (field.clone(), rendered_html))
            })
            .collect();

        // 5. Substitute fields in template
        let final_html = self.substitute_template(&component.template, &rendered_fields)?;

        Ok(final_html)
    }

    // Replace {field} placeholders with rendered HTML
    fn substitute_template(
        &self,
        template: &str,
        rendered_fields: &HashMap<String, String>,
    ) -> Result<String, ComponentError> {
        let mut result = template.to_string();

        for (field, rendered_html) in rendered_fields {
            let placeholder = format!("{{{}}}", field);
            result = result.replace(&placeholder, rendered_html);
        }

        // Check for unresolved placeholders
        if result.contains('{') && result.contains('}') {
            return Err(ComponentError::UnresolvedPlaceholders);
        }

        Ok(result)
    }

    // List all available components
    pub fn list_components(&self) -> Vec<&String> {
        self.components.keys().collect()
    }

    // Get component info
    pub fn get_component(&self, name: &str) -> Option<&ComponentTemplate> {
        self.components.get(name)
    }
}

#[derive(Debug, Clone)]
pub enum ComponentError {
    ComponentNotFound(String),
    RecordNotFound(String),
    UnresolvedPlaceholders,
    DatabaseError(String),
}

impl std::fmt::Display for ComponentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentError::ComponentNotFound(name) => write!(f, "Component '{}' not found", name),
            ComponentError::RecordNotFound(id) => write!(f, "Record with id '{}' not found", id),
            ComponentError::UnresolvedPlaceholders => {
                write!(f, "Template has unresolved placeholders")
            }
            ComponentError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for ComponentError {}

// Global component registry
use std::sync::OnceLock;
static COMPONENT_REGISTRY: OnceLock<ComponentRegistry> = OnceLock::new();

pub fn component_registry() -> &'static ComponentRegistry {
    COMPONENT_REGISTRY.get_or_init(ComponentRegistry::new)
}
