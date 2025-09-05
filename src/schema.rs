// src/schema.rs
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

// ADD: Mock data record structure
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
    // ADD: Mock data for testing
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
    #[allow(dead_code)]
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

    // Get schema for a specific table
    pub fn get_table(&self, table: &str) -> Option<&TableSchema> {
        self.tables.get(table)
    }

    pub fn list_tables(&self) -> Vec<&String> {
        self.tables.keys().collect()
    }

    // functional programming style methods
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

    // ADD: Get single mock record by ID
    pub fn get_mock_record(&self, table: &str, id: &str) -> Option<HashMap<String, String>> {
        self.get_mock_data(table)
            .into_iter()
            .find(|record| record.get("id") == Some(&id.to_string()))
    }

    // ADD: Get first N mock records
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
    #[allow(unused_variables)]
    pub fn render_field(
        &self,
        table: &str,
        field: &str,
        context: &str,
        value: &str,
    ) -> Option<String> {
        // TODO: Implement proper rendering
        Some(format!("<span>{}</span>", value))
    }
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
