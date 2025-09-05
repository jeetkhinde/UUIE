// Main library entry point - exports all public modules
pub mod database;
pub mod renderer;
pub mod schema;

// Re-export main types for easy access
pub use database::Database;
pub use renderer::Renderer;
pub use schema::{SchemaRegistry, registry};

// Convenience macro for rendering fields
#[macro_export]
macro_rules! render {
    ($table:expr, $field:expr, $context:expr, $value:expr) => {
        crate::schema::registry().render_field($table, $field, $context, $value)
    };
}
