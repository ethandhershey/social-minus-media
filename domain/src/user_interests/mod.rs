pub mod crud;

use serde::Deserialize;
use serde_json::Value;
use time::{OffsetDateTime, serde::rfc3339};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct UserInterests {
    pub id: Uuid,
    pub user_id: Uuid,
    pub messages: Value,
    pub summary: Option<String>,
    pub embedding: Option<Vec<f32>>,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "rfc3339")]
    pub updated_at: OffsetDateTime,
}
