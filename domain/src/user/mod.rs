pub mod crud;

use crate::error::DomainError;
use derive_more::{Add, Display, From, Into, Sub};
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, serde::rfc3339};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub sub: String,
    pub email: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub sub: String,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub tier: Tier,
    pub api_usage: ApiUsage,
    pub storage_usage: StorageUsage,
    pub billing_customer_id: Option<String>,
    #[serde(with = "rfc3339")]
    pub billing_period_start: OffsetDateTime,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "rfc3339::option")]
    pub archived_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(type_name = "TEXT", rename_all = "PascalCase"))]
pub enum Tier {
    Free,
    Essential,
    Pro,
    Enterprise,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Add,
    Sub,
    From,
    Into,
    Display,
)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(transparent))]
pub struct ApiUsage(pub i64);

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Add,
    Sub,
    From,
    Into,
    Display,
)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(transparent))]
pub struct StorageUsage(pub i64);

/// Parsed Stripe webhook events relevant to subscription management.
#[derive(Debug)]
pub enum BillingEvent {
    /// A checkout session completed — link Stripe customer to user and set tier.
    CheckoutCompleted {
        user_id: Uuid,
        customer_id: String,
        tier: Tier,
    },
    /// A subscription was updated (plan change, renewal status change).
    SubscriptionUpdated { customer_id: String, tier: Tier },
    /// A subscription was cancelled or expired — downgrade to Free.
    SubscriptionDeleted { customer_id: String },
    /// Any other event type — ignore.
    Other,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TierSettings {
    pub free: TierFeatures,
    pub essential: TierFeatures,
    pub pro: TierFeatures,
    pub enterprise: TierFeatures,
}

impl TierSettings {
    pub fn for_tier(&self, tier: &Tier) -> &TierFeatures {
        match tier {
            Tier::Free => &self.free,
            Tier::Essential => &self.essential,
            Tier::Pro => &self.pro,
            Tier::Enterprise => &self.enterprise,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TierFeatures {
    pub max_projects: u32,
    pub max_storage_gb: u32,
    pub can_export: bool,
    pub api_calls_per_day: u32,
}

impl TierFeatures {
    pub fn check_api_limit(&self, usage: u32) -> Result<(), DomainError> {
        if usage >= self.api_calls_per_day {
            return Err(DomainError::LimitExceeded);
        }
        Ok(())
    }

    pub fn check_project_limit(&self, count: u32) -> Result<(), DomainError> {
        if count >= self.max_projects {
            return Err(DomainError::LimitExceeded);
        }
        Ok(())
    }

    pub fn check_storage_limit(&self, used_gb: u32) -> Result<(), DomainError> {
        if used_gb >= self.max_storage_gb {
            return Err(DomainError::LimitExceeded);
        }
        Ok(())
    }

    pub fn check_can_export(&self) -> Result<(), DomainError> {
        if !self.can_export {
            return Err(DomainError::InsufficientTier);
        }
        Ok(())
    }
}
