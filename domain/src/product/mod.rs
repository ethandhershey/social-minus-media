pub mod crud;

use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, serde::rfc3339};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub brand: Option<String>,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "rfc3339::option")]
    pub archived_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductInsight {
    pub id: Uuid,
    pub product_id: Uuid,
    pub content: serde_json::Value,
    #[serde(with = "rfc3339")]
    pub generated_at: OffsetDateTime,
}
