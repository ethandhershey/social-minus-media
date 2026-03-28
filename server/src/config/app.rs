use anyhow::Result;
use serde::Deserialize;

use super::{
    database::{DatabaseConfig, RawDatabaseConfig},
    entitlment::{RawTierConfig, TierConfig},
    llm::{LlmConfig, RawLlmConfig},
    resend::{RawResendConfig, ResendConfig},
    server::{RawServerConfig, ServerConfig},
    stripe::{RawStripeConfig, StripeConfig},
    zitadel::{RawZitadelConfig, ZitadelConfig},
};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub llm: LlmConfig,
    pub zitadel: ZitadelConfig,
    pub tiers: TierConfig,
    pub stripe: StripeConfig,
    pub resend: ResendConfig,
}

impl AppConfig {
    pub fn new(configs: &[String]) -> Result<Self> {
        let raw: Raw = configs
            .iter()
            .fold(::config::Config::builder(), |builder, file| {
                builder.add_source(::config::File::with_name(&format!("config/{file}")))
            })
            .add_source(::config::Environment::with_prefix("APP").separator("__"))
            .build()?
            .try_deserialize()?;

        Ok(Self {
            server: raw.server.into_config()?,
            database: raw.database.into_config()?,
            llm: raw.llm.into_config()?,
            zitadel: raw.zitadel.into_config()?,
            tiers: raw.tiers.into_config()?,
            stripe: raw.stripe.into_config()?,
            resend: raw.resend.into_config()?,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    server: RawServerConfig,
    database: RawDatabaseConfig,
    llm: RawLlmConfig,
    zitadel: RawZitadelConfig,
    tiers: RawTierConfig,
    stripe: RawStripeConfig,
    resend: RawResendConfig,
}
