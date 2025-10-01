mod modules;
mod utils;

use anyhow::Context;
use atlas_kernel::{settings::Settings, InitCtx, ModuleRegistry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::try_init().ok();

    let settings = Settings::load().with_context(|| "failed to load ATLAS settings")?;

    tracing::info!(
        env = ?settings.environment,
        db = %settings.database.endpoint,
        "atlas-app bootstrap starting"
    );

    // Create module registry and register modules
    let mut registry = ModuleRegistry::new();

    // Register custom modules (core modules will be registered by their respective crates)
    modules::register_all(&mut registry);

    tracing::info!(
        core_modules = registry.core_module_count(),
        custom_modules = registry.custom_module_count(),
        "found {} core modules and {} custom modules",
        registry.core_module_count(),
        registry.custom_module_count()
    );

    // List all registered modules
    for module in registry.modules() {
        tracing::info!(module = module.name(), "registered module");
    }

    // Create initialization context
    let ctx = InitCtx {
        settings: &settings,
    };

    // Phase 1: Initialize core modules in order
    registry.init_core_modules(&ctx).await?;

    // Phase 2: Initialize custom modules
    registry.init_custom_modules(&ctx).await?;

    // Collect and display migrations
    let migrations = registry.collect_migrations();
    tracing::info!(
        migration_count = migrations.len(),
        "collected {} migrations",
        migrations.len()
    );

    for (module_name, migration) in &migrations {
        tracing::info!(
            module = module_name,
            migration_id = migration.id,
            "migration: {}:{}",
            module_name,
            migration.id
        );
    }

    // Phase 3: Start core modules in order
    registry.start_core_modules(&ctx).await?;

    // Phase 4: Start custom modules
    registry.start_custom_modules(&ctx).await?;

    tracing::info!("atlas-app bootstrap complete");

    // Simulate some runtime
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Shutdown Phase 1: Stop custom modules first
    registry.stop_custom_modules().await?;

    // Shutdown Phase 2: Stop core modules in reverse order
    registry.stop_core_modules().await?;

    tracing::info!("atlas-app shutdown complete");
    Ok(())
}
