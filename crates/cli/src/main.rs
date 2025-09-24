use anyhow::Context;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::try_init().ok();

    let settings = atlas_kernel::settings::Settings::load()
        .with_context(|| "failed to load ATLAS settings")?;

    tracing::info!(
        env = ?settings.environment,
        "ATLAS CLI bootstrap pending implementation"
    );

    Ok(())
}
