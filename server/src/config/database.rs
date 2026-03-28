use super::EnvOr;
use anyhow::Result;
use secrecy::SecretString;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawDatabaseConfig {
    url: EnvOr<SecretString>,
    max_connections: EnvOr<u32>,
}

impl RawDatabaseConfig {
    pub(super) fn into_config(self) -> Result<DatabaseConfig> {
        Ok(DatabaseConfig {
            url: self.url.resolve_secret()?,
            max_connections: self.max_connections.resolve()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: SecretString,
    pub max_connections: u32,
}
