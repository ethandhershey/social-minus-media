pub mod crud;

use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, serde::rfc3339};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub host_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub address: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    #[serde(with = "rfc3339")]
    pub start_time: OffsetDateTime,
    pub max_capacity: Option<i32>,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
}
