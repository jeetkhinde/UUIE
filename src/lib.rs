// Main library entry point
pub mod component_registry;
pub mod renderer;
pub mod schema;
pub mod web;

// Re-export main types for easy access
pub use component_registry::{ComponentRegistry, component_registry};
pub use renderer::Renderer;
pub use schema::{SchemaRegistry, registry};
pub use web::{create_router, start_server};

// Convenience macro for rendering fields
#[macro_export]
macro_rules! render {
    ($table:expr, $field:expr, $context:expr, $value:expr) => {
        $crate::schema::registry().render_field($table, $field, $context, $value)
    };
}

// New: Convenience macro for rendering components
#[macro_export]
macro_rules! render_component {
    ($component:expr, $id:expr) => {
        $crate::component_registry::component_registry()
            .render_component($component, $id, None, None, None, None, None)
    };
    ($component:expr, $id:expr, $context:expr) => {
        $crate::component_registry::component_registry().render_component(
            $component,
            $id,
            Some($context),
            None,
            None,
            None,
            None,
        )
    };
}
