use super::EnvOr;
use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawTierConfig {
    free: RawTierFeatures,
    essential: RawTierFeatures,
    pro: RawTierFeatures,
    enterprise: RawTierFeatures,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTierFeatures {
    max_projects: EnvOr<u32>,
    max_storage_gb: EnvOr<u32>,
    can_export: EnvOr<bool>,
    api_calls_per_day: EnvOr<u32>,
}

impl RawTierConfig {
    pub(super) fn into_config(self) -> Result<TierConfig> {
        Ok(TierConfig {
            free: self.free.into_features()?,
            essential: self.essential.into_features()?,
            pro: self.pro.into_features()?,
            enterprise: self.enterprise.into_features()?,
        })
    }
}

impl RawTierFeatures {
    fn into_features(self) -> Result<TierFeatures> {
        Ok(TierFeatures {
            max_projects: self.max_projects.resolve()?,
            max_storage_gb: self.max_storage_gb.resolve()?,
            can_export: self.can_export.resolve()?,
            api_calls_per_day: self.api_calls_per_day.resolve()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TierConfig {
    pub free: TierFeatures,
    pub essential: TierFeatures,
    pub pro: TierFeatures,
    pub enterprise: TierFeatures,
}

#[derive(Debug, Clone)]
pub struct TierFeatures {
    pub max_projects: u32,
    pub max_storage_gb: u32,
    pub can_export: bool,
    pub api_calls_per_day: u32,
}

impl From<TierConfig> for domain::user::TierSettings {
    fn from(c: TierConfig) -> Self {
        Self {
            free: c.free.into(),
            essential: c.essential.into(),
            pro: c.pro.into(),
            enterprise: c.enterprise.into(),
        }
    }
}

impl From<TierFeatures> for domain::user::TierFeatures {
    fn from(f: TierFeatures) -> Self {
        Self {
            max_projects: f.max_projects,
            max_storage_gb: f.max_storage_gb,
            can_export: f.can_export,
            api_calls_per_day: f.api_calls_per_day,
        }
    }
}
