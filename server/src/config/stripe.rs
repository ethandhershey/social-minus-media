use super::EnvOr;
use anyhow::Result;
use secrecy::SecretString;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawStripeConfig {
    secret_key: EnvOr<SecretString>,
    webhook_secret: EnvOr<SecretString>,
    essential_price_id: EnvOr<String>,
    pro_price_id: EnvOr<String>,
    success_url: EnvOr<String>,
    cancel_url: EnvOr<String>,
    portal_return_url: EnvOr<String>,
}

impl RawStripeConfig {
    pub(super) fn into_config(self) -> Result<StripeConfig> {
        Ok(StripeConfig {
            secret_key: self.secret_key.resolve_secret()?,
            webhook_secret: self.webhook_secret.resolve_secret()?,
            essential_price_id: self.essential_price_id.resolve()?,
            pro_price_id: self.pro_price_id.resolve()?,
            success_url: self.success_url.resolve()?,
            cancel_url: self.cancel_url.resolve()?,
            portal_return_url: self.portal_return_url.resolve()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct StripeConfig {
    pub secret_key: SecretString,
    pub webhook_secret: SecretString,
    pub essential_price_id: String,
    pub pro_price_id: String,
    pub success_url: String,
    pub cancel_url: String,
    pub portal_return_url: String,
}
