// Main library entry point
pub mod renderer;
pub mod schema;

// Re-export main types for easy access
pub use renderer::Renderer;
pub use schema::{SchemaRegistry, registry};

// Convenience macro for rendering fields
#[macro_export]
macro_rules! render {
    ($table:expr, $field:expr, $context:expr, $value:expr) => {
        $crate::schema::registry().render_field($table, $field, $context, $value)
    };
}
