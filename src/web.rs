// src/web.rs - Web API endpoints for component system
use axum::{
    Router,
    extract::{Path, Query},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
};
use serde::Deserialize;

use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

use crate::component_registry::{ComponentError, RenderParams, component_registry};

#[derive(Debug, Deserialize)]
pub struct ComponentParams {
    // Required
    pub id: String,

    // Optional with defaults
    pub context: Option<String>,  // default: "card"
    pub platform: Option<String>, // default: "web"
    pub format: Option<String>,   // default: "html"
    pub theme: Option<String>,    // default: "light"
    pub lang: Option<String>,     // default: "en"
}

// 🚀 Main API endpoint: GET /api/:component
pub async fn render_component_api(
    Path(component_name): Path<String>,
    Query(params): Query<ComponentParams>,
) -> impl IntoResponse {
    let registry = component_registry();

    match registry
        .render_component(
            &component_name,
            &params.id,
            RenderParams {
                context: params.context.as_deref(),
                platform: params.platform.as_deref(),
                theme: params.theme.as_deref(),
                lang: params.lang.as_deref(),
                format: params.format.as_deref(),
            },
        )
        .await
    {
        Ok(html) => {
            // Future: handle different formats here
            match params.format.as_deref().unwrap_or("html") {
                "html" => Html(html).into_response(),
                "text" => html.into_response(), // Plain text
                "json" => {
                    let json_response = serde_json::json!({
                        "component": component_name,
                        "id": params.id,
                        "html": html,
                        "context": params.context.unwrap_or_else(|| "card".to_string()),
                        "theme": params.theme.unwrap_or_else(|| "light".to_string())
                    });
                    axum::Json(json_response).into_response()
                }
                _ => (StatusCode::BAD_REQUEST, "Unsupported format").into_response(),
            }
        }
        Err(ComponentError::ComponentNotFound(name)) => (
            StatusCode::NOT_FOUND,
            format!("Component '{}' not found", name),
        )
            .into_response(),
        Err(ComponentError::RecordNotFound(id)) => (
            StatusCode::NOT_FOUND,
            format!("Record with id '{}' not found", id),
        )
            .into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

// 📋 List all available components
pub async fn list_components_api() -> impl IntoResponse {
    let registry = component_registry();
    let components: Vec<_> = registry.list_components().into_iter().cloned().collect();

    axum::Json(serde_json::json!({
        "components": components,
        "count": components.len(),
        "endpoints": components.iter().map(|name| format!("/api/{}", name)).collect::<Vec<_>>()
    }))
}

// 🔍 Get component info/schema
pub async fn component_info_api(Path(component_name): Path<String>) -> impl IntoResponse {
    let registry = component_registry();

    match registry.get_component(&component_name) {
        Some(component) => axum::Json(serde_json::json!({
            "name": component.name,
            "table": component.table,
            "required_fields": component.required_fields,
            "template_preview": component.template,
            "example_url": format!("/api/{}?id=1&context=card&theme=light", component.name)
        }))
        .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            format!("Component '{}' not found", component_name),
        )
            .into_response(),
    }
}

// 🏠 Root API info
pub async fn api_root() -> impl IntoResponse {
    axum::Json(serde_json::json!({
        "name": "Schema UI Component API",
        "version": "0.1.0",
        "endpoints": {
            "components": "/api/components",
            "render": "/api/:component?id={id}&context={context}&theme={theme}",
            "info": "/api/:component/info"
        },
        "examples": [
            "/api/user_card?id=1",
            "/api/user_card?id=1&context=list&theme=dark",
            "/api/user_card?id=1&format=json"
        ]
    }))
}

// 🌐 Create the web router
pub fn create_router() -> Router {
    Router::new()
        // API routes
        .route("/api", get(api_root))
        .route("/api/components", get(list_components_api))
        .route("/api/:component", get(render_component_api))
        .route("/api/:component/info", get(component_info_api))
        // Add middleware
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive()) // For development
                .into_inner(),
        )
}

// 🚀 Start the web server
pub async fn start_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_router();

    println!(
        "🚀 Schema UI Component API starting on http://localhost:{}",
        port
    );
    println!("📋 Available endpoints:");
    println!("   GET /api/components - List all components");
    println!("   GET /api/user_card?id=1 - Render user card component");
    println!("   GET /api/user_card/info - Get component schema");

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test::TestServer;

    #[tokio::test]
    async fn test_component_api() {
        let app = create_router();
        let server = TestServer::new(app.into_make_service()).unwrap();
        // Test component list
        let response = server.get("/api/components").await;
        assert_eq!(response.status_code(), StatusCode::OK);

        // Test component rendering
        let response = server
            .get("/api/user_card")
            .add_query_param("id", "1")
            .await;
        assert_eq!(response.status_code(), StatusCode::OK);

        // Test component info
        let response = server.get("/api/user_card/info").await;
        assert_eq!(response.status_code(), StatusCode::OK);
    }
}
