use super::EnvOr;
use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawZitadelConfig {
    issuer: EnvOr<String>,
    client_id: EnvOr<String>,
    jwks_url: EnvOr<String>,
}

impl RawZitadelConfig {
    pub(super) fn into_config(self) -> Result<ZitadelConfig> {
        Ok(ZitadelConfig {
            issuer: self.issuer.resolve()?,
            client_id: self.client_id.resolve()?,
            jwks_url: self.jwks_url.resolve()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ZitadelConfig {
    pub issuer: String,
    pub client_id: String,
    pub jwks_url: String,
}
