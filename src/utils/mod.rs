//! Project-specific utilities live here.

/// Formats a shared log prefix for project logs.
pub fn log_prefix(module: &str) -> String {
    format!("project::{module}")
}
