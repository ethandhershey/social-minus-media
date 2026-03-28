use super::EnvOr;
use anyhow::Result;
use secrecy::SecretString;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawResendConfig {
    api_key: EnvOr<SecretString>,
    sender_email: EnvOr<String>,
    sender_name: EnvOr<String>,
}

impl RawResendConfig {
    pub(super) fn into_config(self) -> Result<ResendConfig> {
        Ok(ResendConfig {
            api_key: self.api_key.resolve_secret()?,
            sender_email: self.sender_email.resolve()?,
            sender_name: self.sender_name.resolve()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ResendConfig {
    pub api_key: SecretString,
    pub sender_email: String,
    pub sender_name: String,
}
