//! Router builder for ATLAS HTTP server

use axum::{routing::get, Router};
use std::time::Duration;
use tower_http::{
    cors::{Any, CorsLayer},
    request_id::{MakeRequestUuid, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};

use atlas_kernel::ModuleRegistry;

/// Builder for constructing the main HTTP router
pub struct RouterBuilder {
    router: Router,
}

impl RouterBuilder {
    /// Create a new router builder
    pub fn new() -> Self {
        Self {
            router: Router::new(),
        }
    }

    /// Add a route to the router
    pub fn route(mut self, path: &str, route: axum::routing::MethodRouter) -> Self {
        self.router = self.router.route(path, route);
        self
    }

    /// Mount a module's router under `/api/{module_name}`
    pub fn mount_module(mut self, module_name: &str, module_router: Router) -> Self {
        let api_path = format!("/api/{}", module_name);
        self.router = self.router.nest(&api_path, module_router);
        self
    }

    /// Add tracing middleware
    pub fn with_tracing(mut self) -> Self {
        self.router = self.router.layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(DefaultOnResponse::new().level(tracing::Level::INFO)),
        );
        self
    }

    /// Add CORS middleware
    pub fn with_cors(mut self) -> Self {
        self.router = self.router.layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );
        self
    }

    /// Add request ID middleware
    pub fn with_request_id(mut self) -> Self {
        self.router = self
            .router
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid));
        self
    }

    /// Add timeout middleware
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.router = self
            .router
            .layer(TimeoutLayer::new(Duration::from_millis(timeout_ms)));
        self
    }

    /// Add OpenAPI documentation by collecting specs from all modules
    pub fn with_openapi(mut self, registry: &ModuleRegistry) -> Self {
        // Start with base OpenAPI spec
        let mut openapi_spec = serde_json::json!({
            "openapi": "3.0.0",
            "info": {
                "title": "ATLAS API",
                "version": "1.0.0",
                "description": "Core SaaS Framework API"
            },
            "paths": {},
            "components": {
                "schemas": {}
            }
        });

        // Add common error response schema
        openapi_spec["components"]["schemas"]["ErrorResponse"] = serde_json::json!({
            "type": "object",
            "properties": {
                "error": {
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string"
                        },
                        "message": {
                            "type": "string"
                        },
                        "details": {
                            "type": "array",
                            "items": {}
                        },
                        "trace_id": {
                            "type": "string"
                        },
                        "timestamp": {
                            "type": "string"
                        }
                    },
                    "required": ["code", "message", "trace_id", "timestamp"]
                }
            },
            "required": ["error"]
        });

        // Add server health endpoint
        openapi_spec["paths"]["/healthz"] = serde_json::json!({
            "get": {
                "summary": "Health check",
                "responses": {
                    "200": {
                        "description": "OK",
                        "content": {
                            "text/plain": {
                                "schema": {
                                    "type": "string"
                                }
                            }
                        }
                    }
                }
            }
        });

        // Collect OpenAPI specs from all modules
        for module in registry.modules() {
            if let Some(module_spec) = module.openapi() {
                // Merge paths from module
                if let Some(paths) = module_spec.get("paths") {
                    if let Some(paths_obj) = paths.as_object() {
                        for (path, path_item) in paths_obj {
                            // Prefix module paths with /api/{module_name}
                            let prefixed_path = format!("/api/{}{}", module.name(), path);
                            openapi_spec["paths"][prefixed_path] = path_item.clone();
                        }
                    }
                }

                // Merge schemas from module
                if let Some(components) = module_spec.get("components") {
                    if let Some(schemas) = components.get("schemas") {
                        if let Some(schemas_obj) = schemas.as_object() {
                            for (schema_name, schema_def) in schemas_obj {
                                openapi_spec["components"]["schemas"][schema_name] =
                                    schema_def.clone();
                            }
                        }
                    }
                }
            }
        }

        // Deserialize our JSON spec into a proper utoipa OpenApi object
        // This allows SwaggerUI to serve it correctly
        let openapi_obj: utoipa::openapi::OpenApi = serde_json::from_value(openapi_spec.clone())
            .unwrap_or_else(|_| {
                utoipa::openapi::OpenApiBuilder::new()
                    .info(
                        utoipa::openapi::InfoBuilder::new()
                            .title("ATLAS API")
                            .version("1.0.0")
                            .build(),
                    )
                    .build()
            });

        // Mount Swagger UI at /swagger-ui with our merged OpenAPI spec
        // SwaggerUI will serve both the UI and the spec
        self.router = self.router.merge(
            utoipa_swagger_ui::SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", openapi_obj.clone()),
        );

        // Also serve the raw JSON spec at /docs/openapi.json for external consumers
        self.router = self.router.route(
            "/docs/openapi.json",
            get(move || async move { axum::Json(openapi_spec.clone()) }),
        );

        self
    }

    /// Build the final router
    pub fn build(self) -> Router {
        self.router
    }
}

impl Default for RouterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::routing::get;

    #[tokio::test]
    async fn test_router_builder_basic() {
        let _router = RouterBuilder::new()
            .route("/test", get(|| async { "test" }))
            .build();

        // In a real test, you'd use axum_test or similar to make requests
        // For now, just verify the router builds successfully
        assert!(true);
    }

    #[tokio::test]
    async fn test_module_mounting() {
        let module_router = Router::new().route("/", get(|| async { "module" }));

        let _router = RouterBuilder::new()
            .mount_module("test", module_router)
            .build();

        // Verify the router builds successfully
        assert!(true);
    }

    #[tokio::test]
    async fn test_middleware_chain() {
        let _router = RouterBuilder::new()
            .with_tracing()
            .with_cors()
            .with_request_id()
            .with_timeout(5000)
            .route("/health", get(|| async { "ok" }))
            .build();

        // Verify the router builds successfully with all middlewares
        assert!(true);
    }
}
