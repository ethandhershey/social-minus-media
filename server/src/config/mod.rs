pub mod app;
mod database;
mod entitlment;
mod llm;
mod resend;
mod server;
mod stripe;
mod zitadel;

pub use app::AppConfig;

// ── EnvOr ─────────────────────────────────────────────────────────────────────

use anyhow::Result;
use secrecy::SecretString;
use serde::{Deserialize, Deserializer, de::DeserializeOwned};
use std::str::FromStr;

/// A config value that is either an inline literal or a reference to an
/// environment variable.
///
/// ```toml
/// # literal
/// port = 8080
///
/// # env-var reference (resolved and parsed at startup)
/// port = { env = "APP_PORT" }
/// ```
#[derive(Debug, Clone)]
pub(super) enum EnvOr<T> {
    Value(T),
    Env { var: String },
}

impl<T> EnvOr<T>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    pub(super) fn resolve(self) -> Result<T> {
        match self {
            EnvOr::Value(v) => Ok(v),
            EnvOr::Env { var } => {
                let raw = std::env::var(&var)
                    .map_err(|_| anyhow::anyhow!("env var '{}' is not set", var))?;
                raw.parse()
                    .map_err(|e| anyhow::anyhow!("env var '{}' could not be parsed: {}", var, e))
            }
        }
    }
}

impl EnvOr<SecretString> {
    pub(super) fn resolve_secret(self) -> Result<SecretString> {
        match self {
            EnvOr::Value(v) => Ok(v),
            EnvOr::Env { var } => std::env::var(&var)
                .map(SecretString::from)
                .map_err(|_| anyhow::anyhow!("env var '{}' is not set", var)),
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum EnvOrRaw<T> {
    EnvRef { env: String },
    Literal(T),
}

impl<'de, T: DeserializeOwned> Deserialize<'de> for EnvOr<T> {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        match EnvOrRaw::<T>::deserialize(de)? {
            EnvOrRaw::EnvRef { env } => Ok(EnvOr::Env { var: env }),
            EnvOrRaw::Literal(v) => Ok(EnvOr::Value(v)),
        }
    }
}
