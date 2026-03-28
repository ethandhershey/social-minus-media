use domain::{
    error::DomainError,
    ports::EntitlementService,
    user::{Tier, TierSettings, User},
};

pub struct ConfigEntitlementService {
    config: TierSettings,
}

impl ConfigEntitlementService {
    pub fn new(config: TierSettings) -> Self {
        Self { config }
    }
}

impl EntitlementService for ConfigEntitlementService {
    fn check_tier(&self, user: &User, required: &Tier) -> Result<(), DomainError> {
        if &user.tier >= required {
            Ok(())
        } else {
            Err(DomainError::InsufficientTier)
        }
    }

    fn check_api_limit(&self, user: &User, usage: u32) -> Result<(), DomainError> {
        self.config.for_tier(&user.tier).check_api_limit(usage)
    }

    fn check_project_limit(&self, user: &User, count: u32) -> Result<(), DomainError> {
        self.config.for_tier(&user.tier).check_project_limit(count)
    }

    fn check_storage_limit(&self, user: &User, used_gb: u32) -> Result<(), DomainError> {
        self.config
            .for_tier(&user.tier)
            .check_storage_limit(used_gb)
    }

    fn can_export(&self, user: &User) -> Result<(), DomainError> {
        self.config.for_tier(&user.tier).check_can_export()
    }
}
