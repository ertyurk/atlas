use anyhow::Context;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "atlas")]
#[command(about = "ATLAS CLI - Core SaaS Framework")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the HTTP server
    Server,
    /// Migration commands
    Migrate {
        #[command(subcommand)]
        command: MigrateCommands,
    },
}

#[derive(Subcommand)]
enum MigrateCommands {
    /// Plan migrations (show what would be applied)
    Plan,
    /// Apply migrations
    Up,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::try_init().ok();

    let cli = Cli::parse();

    let settings = atlas_kernel::settings::Settings::load()
        .with_context(|| "failed to load ATLAS settings")?;

    match cli.command {
        Commands::Server => {
            tracing::info!(
                env = ?settings.environment,
                "starting ATLAS server"
            );

            // Create module registry
            let mut registry = atlas_kernel::registry::ModuleRegistry::new();

            // Register core modules first (excluding HTTP router)
            // TODO: Register core modules like telemetry, db, authz, events

            // Register custom modules
            atlas_app::modules::register_all(&mut registry);

            // Initialize all modules in proper order
            let init_ctx = atlas_kernel::module::InitCtx {
                settings: &settings,
            };

            // Initialize core modules first (excluding HTTP)
            registry
                .init_core_modules(&init_ctx)
                .await
                .context("failed to initialize core modules")?;

            // Initialize custom modules
            registry
                .init_custom_modules(&init_ctx)
                .await
                .context("failed to initialize custom modules")?;

            // Start core modules (excluding HTTP)
            registry
                .start_core_modules(&init_ctx)
                .await
                .context("failed to start core modules")?;

            // Start custom modules
            registry
                .start_custom_modules(&init_ctx)
                .await
                .context("failed to start custom modules")?;

            // Now start HTTP server with fully initialized modules
            atlas_http::start_server(&registry, &settings).await?;
        }
        Commands::Migrate { command } => match command {
            MigrateCommands::Plan => {
                tracing::info!("migration planning not yet implemented");
            }
            MigrateCommands::Up => {
                tracing::info!("migration execution not yet implemented");
            }
        },
    }

    Ok(())
}
