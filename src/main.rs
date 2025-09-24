mod modules;
mod utils;

use anyhow::Context;
use atlas_kernel::settings::Settings;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::try_init().ok();

    let settings = Settings::load().with_context(|| "failed to load ATLAS settings")?;

    tracing::info!(
        env = ?settings.environment,
        db = %settings.database.endpoint,
        "atlas-app bootstrap starting"
    );

    modules::register_all();

    tracing::info!("atlas-app bootstrap complete");
    Ok(())
}
