use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, serde::rfc3339};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(type_name = "TEXT", rename_all = "snake_case"))]
pub enum RsvpStatus {
    Going,
    Maybe,
    Declined,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rsvp {
    pub user_id: Uuid,
    pub event_id: Uuid,
    pub status: RsvpStatus,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
}
