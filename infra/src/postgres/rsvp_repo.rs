use domain::error::DomainError;
use domain::ports::RsvpRepository;
use domain::rsvp::{Rsvp, RsvpStatus};
use sqlx::PgPool;
use uuid::Uuid;

pub struct PgRsvpRepository {
    pool: PgPool,
}

impl PgRsvpRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl RsvpRepository for PgRsvpRepository {
    async fn find(&self, user_id: Uuid, event_id: Uuid) -> Result<Option<Rsvp>, DomainError> {
        sqlx::query_as!(
            Rsvp,
            r#"SELECT user_id    as "user_id: Uuid",
                      event_id   as "event_id: Uuid",
                      status     as "status: RsvpStatus",
                      created_at as "created_at: time::OffsetDateTime"
               FROM rsvps WHERE user_id = $1 AND event_id = $2"#,
            user_id,
            event_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| DomainError::Internal(anyhow::anyhow!("failed to fetch rsvp")))
    }

    async fn find_by_event(&self, event_id: Uuid) -> Result<Vec<Rsvp>, DomainError> {
        sqlx::query_as!(
            Rsvp,
            r#"SELECT user_id    as "user_id: Uuid",
                      event_id   as "event_id: Uuid",
                      status     as "status: RsvpStatus",
                      created_at as "created_at: time::OffsetDateTime"
               FROM rsvps WHERE event_id = $1"#,
            event_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| DomainError::Internal(anyhow::anyhow!("failed to list rsvps for event")))
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Rsvp>, DomainError> {
        sqlx::query_as!(
            Rsvp,
            r#"SELECT user_id    as "user_id: Uuid",
                      event_id   as "event_id: Uuid",
                      status     as "status: RsvpStatus",
                      created_at as "created_at: time::OffsetDateTime"
               FROM rsvps WHERE user_id = $1"#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| DomainError::Internal(anyhow::anyhow!("failed to list rsvps for user")))
    }

    async fn upsert(&self, rsvp: &Rsvp) -> Result<Rsvp, DomainError> {
        sqlx::query_as!(
            Rsvp,
            r#"INSERT INTO rsvps (user_id, event_id, status)
               VALUES ($1, $2, $3)
               ON CONFLICT(user_id, event_id) DO UPDATE SET
                   status = EXCLUDED.status
               RETURNING user_id    as "user_id: Uuid",
                         event_id   as "event_id: Uuid",
                         status     as "status: RsvpStatus",
                         created_at as "created_at: time::OffsetDateTime""#,
            rsvp.user_id,
            rsvp.event_id,
            rsvp.status.clone() as RsvpStatus,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| DomainError::Internal(anyhow::anyhow!("failed to upsert rsvp")))
    }

    async fn delete(&self, user_id: Uuid, event_id: Uuid) -> Result<(), DomainError> {
        let result = sqlx::query!(
            "DELETE FROM rsvps WHERE user_id = $1 AND event_id = $2",
            user_id,
            event_id
        )
        .execute(&self.pool)
        .await
        .map_err(|_| DomainError::Internal(anyhow::anyhow!("failed to delete rsvp")))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound);
        }
        Ok(())
    }
}
