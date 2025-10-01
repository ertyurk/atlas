use std::path::PathBuf;

use anyhow::{anyhow, Context};
use serde::Deserialize;

const DEFAULT_ENV: &str = "local";
const ENV_VAR_NAME: &str = "ATLAS_ENV";
const CONFIG_DIR_ENV: &str = "ATLAS_CONFIG_DIR";

/// Deployment environment the application is running in.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    #[default]
    Local,
    Staging,
    Production,
}

/// Top-level configuration structure loaded from layered sources.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Settings {
    #[serde(default)]
    pub environment: Environment,
    #[serde(default)]
    pub server: ServerSettings,
    #[serde(default)]
    pub database: DatabaseSettings,
    #[serde(default)]
    pub telemetry: TelemetrySettings,
    #[serde(default)]
    pub auth: AuthSettings,
}

impl Settings {
    /// Load configuration by layering `.env`, base file, and environment overlay.
    pub fn load() -> anyhow::Result<Self> {
        // Allow missing `.env` files without failing.
        let _ = dotenvy::dotenv();

        let environment = std::env::var(ENV_VAR_NAME).unwrap_or_else(|_| DEFAULT_ENV.to_string());
        let config_dir = std::env::var(CONFIG_DIR_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                // Default to repo root `config` directory.
                std::env::current_dir()
                    .map(|cwd| cwd.join("config"))
                    .expect("unable to resolve current directory")
            });

        let base_path = config_dir.join("base.toml");
        let environment_filename = format!("{}.toml", environment);
        let environment_path = config_dir.join(environment_filename);

        let builder = config::Config::builder()
            .add_source(config::File::from(base_path).required(false))
            .add_source(config::File::from(environment_path).required(false))
            .add_source(config::Environment::with_prefix("ATLAS").separator("_"));

        let cfg = builder
            .build()
            .with_context(|| "failed to build configuration")?;

        let mut settings: Settings = cfg
            .try_deserialize()
            .with_context(|| "failed to deserialize configuration")?;

        // Override environment field with parsed enum variant.
        settings.environment = match environment.as_str() {
            "local" => Environment::Local,
            "staging" => Environment::Staging,
            "production" => Environment::Production,
            other => {
                return Err(anyhow!(
                    "unsupported environment '{}'; expected local/staging/production",
                    other
                ));
            }
        };

        Ok(settings)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerSettings {
    #[serde(default = "ServerSettings::default_host")]
    pub host: String,
    #[serde(default = "ServerSettings::default_port")]
    pub port: u16,
    #[serde(default = "ServerSettings::default_request_timeout_ms")]
    pub request_timeout_ms: u64,
}

impl ServerSettings {
    fn default_host() -> String {
        "0.0.0.0".to_string()
    }

    fn default_port() -> u16 {
        8080
    }

    fn default_request_timeout_ms() -> u64 {
        15000
    }
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            host: Self::default_host(),
            port: Self::default_port(),
            request_timeout_ms: Self::default_request_timeout_ms(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseSettings {
    #[serde(default = "DatabaseSettings::default_endpoint")]
    pub endpoint: String,
    #[serde(default = "DatabaseSettings::default_namespace")]
    pub namespace: String,
    #[serde(default = "DatabaseSettings::default_database")]
    pub database: String,
}

impl DatabaseSettings {
    fn default_endpoint() -> String {
        "ws://127.0.0.1:8000".to_string()
    }

    fn default_namespace() -> String {
        "atlas".to_string()
    }

    fn default_database() -> String {
        "core".to_string()
    }
}

impl Default for DatabaseSettings {
    fn default() -> Self {
        Self {
            endpoint: Self::default_endpoint(),
            namespace: Self::default_namespace(),
            database: Self::default_database(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TelemetrySettings {
    #[serde(default)]
    pub otlp_endpoint: Option<String>,
    #[serde(default)]
    pub prometheus_bind: Option<String>,
    #[serde(default)]
    pub log_format: LogFormat,
}

impl Default for TelemetrySettings {
    fn default() -> Self {
        Self {
            otlp_endpoint: None,
            prometheus_bind: Some("127.0.0.1:9000".to_string()),
            log_format: LogFormat::Pretty,
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    #[default]
    Pretty,
    Json,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthSettings {
    #[serde(default = "AuthSettings::default_model_path")]
    pub casbin_model_path: String,
    #[serde(default = "AuthSettings::default_policy_path")]
    pub casbin_policy_path: String,
}

impl AuthSettings {
    fn default_model_path() -> String {
        "config/auth/model.conf".to_string()
    }

    fn default_policy_path() -> String {
        "config/auth/policy.csv".to_string()
    }
}

impl Default for AuthSettings {
    fn default() -> Self {
        Self {
            casbin_model_path: Self::default_model_path(),
            casbin_policy_path: Self::default_policy_path(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_environment_is_local() {
        let settings = Settings::default();
        assert_eq!(settings.environment, Environment::Local);
    }

    #[test]
    fn default_database_endpoint_is_ws_localhost() {
        let settings = Settings::default();
        assert_eq!(settings.database.endpoint, "ws://127.0.0.1:8000");
    }
}
