// src/schema.rs - Enhanced with full rendering logic
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FieldVariant {
    pub base: String,
    #[serde(rename = "override")]
    pub override_class: Option<String>,
    pub extend: Option<String>,
    pub attrs: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Context {
    pub inherits: Option<String>,
    #[serde(flatten)]
    pub fields: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MockRecord {
    #[serde(flatten)]
    pub fields: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TableSchema {
    pub variants: HashMap<String, HashMap<String, FieldVariant>>,
    pub defaults: Option<HashMap<String, String>>,
    pub contexts: HashMap<String, Context>,
    pub mock_data: Option<Vec<MockRecord>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Theme {
    #[serde(flatten)]
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ThemeConfig {
    #[serde(flatten)]
    pub themes: HashMap<String, Theme>,
}

#[derive(Debug, Clone)]
pub struct SchemaRegistry {
    themes: ThemeConfig,
    tables: HashMap<String, TableSchema>,
    current_theme: String,
}

impl SchemaRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_all() -> Self {
        let mut registry = Self::new();

        let themes_content = include_str!("../themes.toml");
        if let Ok(themes) = toml::from_str::<ThemeConfig>(themes_content) {
            registry.themes = themes;
        }

        let table_schemas = [("users", include_str!("../schemas/users/users.toml"))];

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

    pub fn get_table(&self, table: &str) -> Option<&TableSchema> {
        self.tables.get(table)
    }

    pub fn list_tables(&self) -> Vec<&String> {
        self.tables.keys().collect()
    }

    pub fn get_mock_data(&self, table: &str) -> Vec<HashMap<String, String>> {
        self.get_table(table)
            .and_then(|schema| schema.mock_data.as_ref())
            .map(|mock_data| {
                mock_data
                    .iter()
                    .map(|record| record.fields.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_mock_record(&self, table: &str, id: &str) -> Option<HashMap<String, String>> {
        self.get_mock_data(table)
            .into_iter()
            .find(|record| record.get("id") == Some(&id.to_string()))
    }

    pub fn get_mock_records(
        &self,
        table: &str,
        limit: Option<usize>,
    ) -> Vec<HashMap<String, String>> {
        match limit {
            Some(n) => self.get_mock_data(table).into_iter().take(n).collect(),
            None => self.get_mock_data(table),
        }
    }

    pub fn set_theme(&mut self, theme_name: &str) {
        if self.themes.themes.contains_key(theme_name) {
            self.current_theme = theme_name.to_string();
        }
    }

    pub fn get_current_theme(&self) -> &str {
        &self.current_theme
    }

    // 🎯 MAIN RENDERING METHOD - This is where the magic happens
    pub fn render_field(
        &self,
        table: &str,
        field: &str,
        context: &str,
        value: &str,
    ) -> Option<String> {
        let schema = self.get_table(table)?;
        let variant_name = Self::resolve_variant_for_field(schema, field, context)?;
        let field_variants = schema.variants.get(field)?;
        let variant = field_variants.get(&variant_name)?;

        let base_css = self.get_theme_css(&variant.base);
        let css_classes = self.build_css_classes(&base_css, variant);
        let attrs = Self::build_attributes(variant, value, field);

        Some(Self::generate_html(
            &variant.base,
            &css_classes,
            &attrs,
            value,
        ))
    }
    fn resolve_variant_for_field(
        schema: &TableSchema,
        field: &str,
        context: &str,
    ) -> Option<String> {
        // Check if context exists and has this field
        if let Some(ctx) = schema.contexts.get(context) {
            if let Some(variant) = ctx.fields.get(field) {
                return Some(variant.clone());
            }

            // Check inheritance chain recursively
            if let Some(parent_context) = &ctx.inherits {
                return Self::resolve_variant_for_field(schema, field, parent_context);
            }
        }

        // Fall back to defaults
        schema
            .defaults
            .as_ref()
            .and_then(|defaults| defaults.get(field).cloned())
            .or_else(|| {
                // Last resort: use first available variant for this field
                schema
                    .variants
                    .get(field)
                    .and_then(|field_variants| field_variants.keys().next().cloned())
            })
    }

    // Get CSS classes from current theme
    fn get_theme_css(&self, tag: &str) -> String {
        self.themes
            .themes
            .get(&self.current_theme)
            .and_then(|theme| theme.tags.get(tag))
            .cloned()
            .unwrap_or_default()
    }

    // Build final CSS classes (theme + override + extend)
    fn build_css_classes(&self, theme_css: &str, variant: &FieldVariant) -> String {
        match (&variant.override_class, &variant.extend) {
            (Some(override_css), None) => override_css.clone(),
            (None, Some(extend_css)) if theme_css.is_empty() => extend_css.clone(),
            (None, Some(extend_css)) => format!("{} {}", theme_css, extend_css),
            (Some(override_css), Some(extend_css)) => format!("{} {}", override_css, extend_css),
            (None, None) => theme_css.to_string(),
        }
    }

    // Build HTML attributes with value substitution
    fn build_attributes(
        variant: &FieldVariant,
        value: &str,
        field: &str,
    ) -> HashMap<String, String> {
        variant
            .attrs
            .as_ref()
            .map(|attrs| {
                attrs
                    .iter()
                    .map(|(key, attr_value)| {
                        let resolved_value = attr_value
                            .replace("{value}", value)
                            .replace("{field}", field);
                        (key.clone(), resolved_value)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    // Generate final HTML element
    fn generate_html(
        tag: &str,
        css_classes: &str,
        attrs: &HashMap<String, String>,
        value: &str,
    ) -> String {
        let mut html = format!("<{}", tag);

        // Add CSS classes
        if !css_classes.is_empty() {
            html.push_str(&format!(" class=\"{}\"", css_classes));
        }

        // Add other attributes
        for (key, attr_value) in attrs {
            if key != "class" {
                // Don't duplicate class
                html.push_str(&format!(" {}=\"{}\"", key, attr_value));
            }
        }

        // Handle self-closing tags vs content tags
        match tag {
            "img" | "input" | "br" | "hr" => {
                html.push_str(" />");
            }
            _ => {
                html.push('>');
                html.push_str(value);
                html.push_str(&format!("</{}>", tag));
            }
        }

        html
    }

    // end of impl SchemaRegistry
}

impl Default for SchemaRegistry {
    fn default() -> Self {
        Self {
            themes: ThemeConfig {
                themes: HashMap::new(),
            },
            tables: HashMap::new(),
            current_theme: "light".to_string(),
        }
    }
}

use std::sync::OnceLock;
static REGISTRY: OnceLock<SchemaRegistry> = OnceLock::new();

pub fn registry() -> &'static SchemaRegistry {
    REGISTRY.get_or_init(SchemaRegistry::load_all)
}

// Helper function to get a mutable registry for theme switching
pub fn with_registry_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut SchemaRegistry) -> R,
{
    // Note: This is a simplified approach. In production, you'd want
    // proper thread-safe mutable access or per-request theme handling
    let mut registry = SchemaRegistry::load_all();
    f(&mut registry)
}
