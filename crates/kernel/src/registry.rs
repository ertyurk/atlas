use anyhow::Context;
use std::sync::Arc;

use crate::module::{InitCtx, Module};

/// Core module initialization order (excluding HTTP server)
const CORE_MODULE_ORDER: &[&str] = &[
    "kernel",    // Kernel must be first
    "telemetry", // Telemetry for logging
    "db",        // Database connection
    "authz",     // Authorization
    "events",    // Event bus
                 // Note: HTTP server is started separately after all modules are initialized
];

/// Module registry for managing module lifecycle with core/custom separation
pub struct ModuleRegistry {
    core_modules: Vec<Arc<dyn Module>>,
    custom_modules: Vec<Arc<dyn Module>>,
}

impl ModuleRegistry {
    /// Create a new module registry
    pub fn new() -> Self {
        Self {
            core_modules: Vec::new(),
            custom_modules: Vec::new(),
        }
    }

    /// Register a core module with the registry
    pub fn register_core(&mut self, module: Arc<dyn Module>) {
        self.core_modules.push(module);
    }

    /// Register a custom module with the registry
    pub fn register_custom(&mut self, module: Arc<dyn Module>) {
        self.custom_modules.push(module);
    }

    /// Get all registered modules (core + custom)
    pub fn modules(&self) -> Vec<&Arc<dyn Module>> {
        let mut all_modules = Vec::new();
        all_modules.extend(self.core_modules.iter());
        all_modules.extend(self.custom_modules.iter());
        all_modules
    }

    /// Get a module by name (searches both core and custom modules)
    pub fn get_module(&self, name: &str) -> Option<&Arc<dyn Module>> {
        self.core_modules
            .iter()
            .find(|module| module.name() == name)
            .or_else(|| {
                self.custom_modules
                    .iter()
                    .find(|module| module.name() == name)
            })
    }

    /// Get the number of core modules
    pub fn core_module_count(&self) -> usize {
        self.core_modules.len()
    }

    /// Get the number of custom modules
    pub fn custom_module_count(&self) -> usize {
        self.custom_modules.len()
    }

    /// Initialize core modules in the correct order
    pub async fn init_core_modules(&self, ctx: &InitCtx<'_>) -> anyhow::Result<()> {
        tracing::info!(
            "initializing core modules in order: {:?}",
            CORE_MODULE_ORDER
        );

        for &module_name in CORE_MODULE_ORDER {
            if let Some(module) = self.core_modules.iter().find(|m| m.name() == module_name) {
                tracing::info!(module = module.name(), "initializing core module");

                module.init(ctx).await.with_context(|| {
                    format!("failed to initialize core module '{}'", module.name())
                })?;
            }
        }

        Ok(())
    }

    /// Initialize custom modules
    pub async fn init_custom_modules(&self, ctx: &InitCtx<'_>) -> anyhow::Result<()> {
        tracing::info!("initializing {} custom modules", self.custom_modules.len());

        for module in &self.custom_modules {
            tracing::info!(module = module.name(), "initializing custom module");

            module.init(ctx).await.with_context(|| {
                format!("failed to initialize custom module '{}'", module.name())
            })?;
        }

        Ok(())
    }

    /// Start core modules in the correct order
    pub async fn start_core_modules(&self, ctx: &InitCtx<'_>) -> anyhow::Result<()> {
        tracing::info!("starting core modules in order: {:?}", CORE_MODULE_ORDER);

        for &module_name in CORE_MODULE_ORDER {
            if let Some(module) = self.core_modules.iter().find(|m| m.name() == module_name) {
                tracing::info!(module = module.name(), "starting core module");

                module
                    .start(ctx)
                    .await
                    .with_context(|| format!("failed to start core module '{}'", module.name()))?;
            }
        }

        Ok(())
    }

    /// Start custom modules
    pub async fn start_custom_modules(&self, ctx: &InitCtx<'_>) -> anyhow::Result<()> {
        tracing::info!("starting {} custom modules", self.custom_modules.len());

        for module in &self.custom_modules {
            tracing::info!(module = module.name(), "starting custom module");

            module
                .start(ctx)
                .await
                .with_context(|| format!("failed to start custom module '{}'", module.name()))?;
        }

        Ok(())
    }

    /// Stop custom modules first (reverse order)
    pub async fn stop_custom_modules(&self) -> anyhow::Result<()> {
        tracing::info!("stopping {} custom modules", self.custom_modules.len());

        for module in self.custom_modules.iter().rev() {
            tracing::info!(module = module.name(), "stopping custom module");

            module
                .stop()
                .await
                .with_context(|| format!("failed to stop custom module '{}'", module.name()))?;
        }

        Ok(())
    }

    /// Stop core modules in reverse order
    pub async fn stop_core_modules(&self) -> anyhow::Result<()> {
        tracing::info!("stopping core modules in reverse order");

        // Stop core modules in reverse order of CORE_MODULE_ORDER
        for &module_name in CORE_MODULE_ORDER.iter().rev() {
            if let Some(module) = self.core_modules.iter().find(|m| m.name() == module_name) {
                tracing::info!(module = module.name(), "stopping core module");

                module
                    .stop()
                    .await
                    .with_context(|| format!("failed to stop core module '{}'", module.name()))?;
            }
        }

        Ok(())
    }

    /// Collect all migrations from all modules (core + custom)
    pub fn collect_migrations(&self) -> Vec<(String, crate::module::Migration)> {
        let mut migrations = Vec::new();

        // Collect from core modules first
        for module in &self.core_modules {
            for migration in module.migrations() {
                migrations.push((module.name().to_string(), migration));
            }
        }

        // Then collect from custom modules
        for module in &self.custom_modules {
            for migration in module.migrations() {
                migrations.push((module.name().to_string(), migration));
            }
        }

        // Sort by module name and migration ID for deterministic ordering
        migrations.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.id.cmp(b.1.id)));

        migrations
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module::Migration;
    use crate::settings::Settings;

    struct TestModule {
        name: &'static str,
    }

    #[async_trait::async_trait]
    impl Module for TestModule {
        fn name(&self) -> &'static str {
            self.name
        }

        fn migrations(&self) -> Vec<Migration> {
            vec![Migration {
                id: "001_init",
                up: "CREATE TABLE test;",
            }]
        }
    }

    #[test]
    fn test_module_registry_creation() {
        let registry = ModuleRegistry::new();
        assert!(registry.modules().is_empty()); // No modules registered yet
    }

    #[test]
    fn test_migration_collection() {
        let registry = ModuleRegistry::new();
        let migrations = registry.collect_migrations();
        assert!(migrations.is_empty()); // No modules registered yet
    }

    #[tokio::test]
    async fn test_module_lifecycle() {
        let mut registry = ModuleRegistry::new();
        let settings = Settings::default();
        let ctx = InitCtx {
            settings: &settings,
        };

        // Register a test module
        let test_module = Arc::new(TestModule { name: "test" });
        registry.register_custom(test_module);

        // These should not fail with the test module
        registry.init_core_modules(&ctx).await.unwrap();
        registry.init_custom_modules(&ctx).await.unwrap();
        registry.start_core_modules(&ctx).await.unwrap();
        registry.start_custom_modules(&ctx).await.unwrap();
        registry.stop_custom_modules().await.unwrap();
        registry.stop_core_modules().await.unwrap();
    }
}
