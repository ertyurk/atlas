//! HTTP server facade for ATLAS with Axum, error handling, and OpenAPI support.

use anyhow::Context;
use axum::{extract::Request, http::HeaderValue, routing::get, Router};
use tower_http::request_id::{MakeRequestId, RequestId};
use uuid::{Timestamp, Uuid};

use atlas_kernel::ModuleRegistry;

pub mod error;
pub mod router;

use router::RouterBuilder;

/// Start the HTTP server with the given module registry
pub async fn start_server(
    registry: &ModuleRegistry,
    settings: &atlas_kernel::settings::Settings,
) -> anyhow::Result<()> {
    tracing::info!(
        "starting HTTP server on {}:{}",
        settings.server.host,
        settings.server.port
    );

    // Build the main router
    let app = build_router(registry, settings)
        .await
        .context("failed to build HTTP router")?;

    // Create the server
    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", settings.server.host, settings.server.port))
            .await
            .context("failed to bind to address")?;

    tracing::info!(
        "HTTP server listening on http://{}:{}",
        settings.server.host,
        settings.server.port
    );

    // Start serving
    axum::serve(listener, app)
        .await
        .context("HTTP server failed")?;

    Ok(())
}

/// Build the main HTTP router with all module routes mounted
async fn build_router(
    registry: &ModuleRegistry,
    settings: &atlas_kernel::settings::Settings,
) -> anyhow::Result<Router> {
    let mut router_builder = RouterBuilder::new();

    // Add global middlewares
    router_builder = router_builder
        .with_tracing()
        .with_cors()
        .with_request_id()
        .with_timeout(settings.server.request_timeout_ms);

    // Add health check route
    router_builder = router_builder.route("/healthz", get(health_check));

    // Mount module routes
    for module in registry.modules() {
        let module_name = module.name();
        let module_router = module.routes();

        // Check if the module router has any routes by trying to get the first route
        // This is a simple check - in practice, we'll mount all module routers
        tracing::info!(
            module = module_name,
            "mounting module routes under /api/{}",
            module_name
        );
        router_builder = router_builder.mount_module(module_name, module_router);
    }

    // Add OpenAPI documentation
    router_builder = router_builder.with_openapi(registry);

    Ok(router_builder.build())
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "ok"
}

/// Request ID generator for tracing
#[derive(Clone)]
struct MakeRequestUuid;

impl MakeRequestId for MakeRequestUuid {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let timestamp = Timestamp::now(uuid::NoContext);
        let request_id = Uuid::new_v7(timestamp)
            .to_string()
            .parse::<HeaderValue>()
            .ok()?;
        Some(RequestId::new(request_id))
    }
}
