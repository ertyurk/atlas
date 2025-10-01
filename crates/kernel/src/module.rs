use async_trait::async_trait;
use axum::Router;

/// Context provided to modules during initialization
pub struct InitCtx<'a> {
    pub settings: &'a crate::settings::Settings,
    // TODO: Add db and events when those crates are implemented
    // pub db: &'a surrealdb::Surreal<surrealdb::engine::remote::ws::Client>,
    // pub events: &'a crate::events::EventBus,
}

/// Migration definition for modules
#[derive(Debug, Clone)]
pub struct Migration {
    pub id: &'static str,
    pub up: &'static str,
}

/// Core module trait that all ATLAS modules must implement
#[async_trait]
pub trait Module: Sync + Send {
    /// Unique name for this module
    fn name(&self) -> &'static str;

    /// Initialize the module with the provided context
    /// Called during application startup before migrations
    async fn init(&self, _ctx: &InitCtx<'_>) -> anyhow::Result<()> {
        Ok(())
    }

    /// Return the Axum router for this module's routes
    /// Routes will be mounted under `/api/{module_name}`
    fn routes(&self) -> Router {
        Router::new()
    }

    /// Return OpenAPI specification fragment for this module as JSON
    /// Will be merged with other modules' specs
    fn openapi(&self) -> Option<serde_json::Value> {
        None
    }

    /// Return migrations contributed by this module
    /// Migrations are executed in the order returned
    fn migrations(&self) -> Vec<Migration> {
        vec![]
    }

    /// Start background tasks for this module
    /// Called after migrations are complete
    async fn start(&self, _ctx: &InitCtx<'_>) -> anyhow::Result<()> {
        Ok(())
    }

    /// Stop the module and clean up resources
    /// Called during application shutdown
    async fn stop(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
