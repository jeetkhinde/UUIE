// Schema module - handles loading and managing TOML schemas
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// FieldVariant represents a single way to render a field (e.g., name.h1, email.link)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FieldVariant {
    pub base: String, // HTML tag (h1, span, input, etc.)
    #[serde(rename = "override")]
    pub override_class: Option<String>, // Replace theme CSS completely
    pub extend: Option<String>, // Add CSS to theme defaults
    pub attrs: Option<HashMap<String, String>>, // HTML attributes
}

// Context defines which variant to use for each field in a UI context
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Context {
    pub inherits: Option<String>, // Inherit from another context
    #[serde(flatten)]
    pub fields: HashMap<String, String>, // field_name -> variant_name mapping
}

// TableSchema represents the complete schema for one table
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TableSchema {
    pub variants: HashMap<String, HashMap<String, FieldVariant>>, // field -> variants
    pub defaults: Option<HashMap<String, String>>,                // field -> default_variant
    pub contexts: HashMap<String, Context>,                       // context_name -> context
}

// Theme holds global CSS defaults for HTML tags
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Theme {
    #[serde(flatten)]
    pub tags: HashMap<String, String>, // tag_name -> css_classes
}

// ThemeConfig holds all available themes (light, dark, etc.)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ThemeConfig {
    #[serde(flatten)]
    pub themes: HashMap<String, Theme>, // theme_name -> theme
}

// SchemaRegistry manages all schemas and provides rendering functionality
#[derive(Debug, Clone)]
pub struct SchemaRegistry {
    themes: ThemeConfig,                  // Global themes
    tables: HashMap<String, TableSchema>, // table_name -> schema
    current_theme: String,                // Active theme name
}

impl SchemaRegistry {
    // Create new empty registry
    pub fn new() -> Self {
        Self {
            themes: ThemeConfig {
                themes: HashMap::new(),
            },
            tables: HashMap::new(),
            current_theme: "light".to_string(),
        }
    }

    // Load all schemas from embedded files
    pub fn load_all() -> Self {
        let mut registry = Self::new();

        // Load global themes
        let themes_content = include_str!("../../themes.toml");
        if let Ok(themes) = toml::from_str::<ThemeConfig>(themes_content) {
            registry.themes = themes;
        } else {
            eprintln!("Failed to load themes.toml");
        }

        // Load table schemas (add more tables here as needed)
        let table_schemas = [
            ("users", include_str!("../../schemas/users/users.toml")),
            // Add more tables: ("products", include_str!("../../schemas/products/products.toml")),
        ];

        for (table_name, content) in table_schemas {
            match toml::from_str::<TableSchema>(content) {
                Ok(schema) => {
                    registry.tables.insert(table_name.to_string(), schema);
                }
                Err(e) => {
                    eprintln!("Failed to load schema for {}: {}", table_name, e);
                }
            }
        }

        registry
    }

    // Get schema for a specific table
    pub fn get_table(&self, table: &str) -> Option<&TableSchema> {
        self.tables.get(table)
    }

    // Switch active theme (light/dark)
    pub fn set_theme(&mut self, theme_name: &str) {
        if self.themes.themes.contains_key(theme_name) {
            self.current_theme = theme_name.to_string();
        }
    }

    // Get CSS classes for a tag from current theme
    fn get_theme_css(&self, tag: &str) -> Option<&str> {
        self.themes
            .themes
            .get(&self.current_theme)?
            .tags
            .get(tag)
            .map(|s| s.as_str())
    }

    // List all available tables
    pub fn list_tables(&self) -> Vec<&String> {
        self.tables.keys().collect()
    }

    // Main rendering function - converts field to HTML
    pub fn render_field(
        &self,
        table: &str,
        field: &str,
        context: &str,
        value: &str,
    ) -> Option<String> {
        let table_schema = self.get_table(table)?;

        // Step 1: Resolve context with inheritance
        let resolved_context = self.resolve_context(table_schema, context)?;

        // Step 2: Get variant name for this field in this context
        let variant_name = resolved_context.fields.get(field).or_else(|| {
            // Fallback to model defaults
            table_schema.defaults.as_ref()?.get(field)
        })?;

        // Step 3: Get field variant definition
        let variant = table_schema.variants.get(field)?.get(variant_name)?;

        // Step 4: Build HTML with CSS classes
        self.build_html(variant, value)
    }

    // Resolve context inheritance (e.g., list inherits from card)
    fn resolve_context(&self, schema: &TableSchema, context_name: &str) -> Option<Context> {
        let context = schema.contexts.get(context_name)?;

        if let Some(ref inherits_from) = context.inherits {
            // Merge inherited context with current context
            let mut base_context = self.resolve_context(schema, inherits_from)?;

            // Override with current context fields
            for (field, variant) in &context.fields {
                base_context.fields.insert(field.clone(), variant.clone());
            }

            Some(base_context)
        } else {
            Some(context.clone())
        }
    }

    // Build final HTML string with CSS classes and attributes
    fn build_html(&self, variant: &FieldVariant, value: &str) -> Option<String> {
        let mut html = format!("<{}", variant.base);

        // Determine final CSS classes
        let css_classes = if let Some(ref override_css) = variant.override_class {
            // Use override CSS, ignore theme
            override_css.clone()
        } else if let Some(ref extend_css) = variant.extend {
            // Combine theme CSS + extend CSS
            let theme_css = self.get_theme_css(&variant.base).unwrap_or("");
            format!("{} {}", theme_css, extend_css)
        } else {
            // Use theme CSS only
            self.get_theme_css(&variant.base).unwrap_or("").to_string()
        };

        // Add CSS class attribute if we have classes
        if !css_classes.is_empty() {
            html.push_str(&format!(" class=\"{}\"", css_classes.trim()));
        }

        // Add custom HTML attributes
        if let Some(ref attrs) = variant.attrs {
            for (key, val) in attrs {
                let attr_value = val.replace("{value}", value);
                html.push_str(&format!(" {}=\"{}\"", key, attr_value));
            }
        }

        // Close opening tag and add content
        html.push_str(&format!(">{}}</{}>", value, variant.base));
        Some(html)
    }
}

// Global singleton registry instance
use std::sync::OnceLock;
static REGISTRY: OnceLock<SchemaRegistry> = OnceLock::new();

// Get global schema registry instance
pub fn registry() -> &'static SchemaRegistry {
    REGISTRY.get_or_init(|| SchemaRegistry::load_all())
}
