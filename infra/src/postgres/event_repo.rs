use domain::error::DomainError;
use domain::event::Event;
use domain::ports::EventRepository;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PgEventRepository {
    pool: PgPool,
}

impl PgEventRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl EventRepository for PgEventRepository {
    async fn find(&self, event_id: Uuid) -> Result<Event, DomainError> {
        sqlx::query_as!(
            Event,
            r#"SELECT id          as "id: Uuid",
                      host_id     as "host_id: Uuid",
                      title,
                      description,
                      address,
                      latitude,
                      longitude,
                      start_time  as "start_time: time::OffsetDateTime",
                      max_capacity,
                      created_at  as "created_at: time::OffsetDateTime"
               FROM events WHERE id = $1"#,
            event_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| DomainError::NotFound)
    }

    async fn find_upcoming(&self, limit: i64) -> Result<Vec<Event>, DomainError> {
        sqlx::query_as!(
            Event,
            r#"SELECT id          as "id: Uuid",
                      host_id     as "host_id: Uuid",
                      title,
                      description,
                      address,
                      latitude,
                      longitude,
                      start_time  as "start_time: time::OffsetDateTime",
                      max_capacity,
                      created_at  as "created_at: time::OffsetDateTime"
               FROM events WHERE start_time > NOW()
               ORDER BY start_time ASC
               LIMIT $1"#,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| DomainError::Internal(anyhow::anyhow!("failed to list upcoming events")))
    }

    async fn find_by_host(&self, host_id: Uuid) -> Result<Vec<Event>, DomainError> {
        sqlx::query_as!(
            Event,
            r#"SELECT id          as "id: Uuid",
                      host_id     as "host_id: Uuid",
                      title,
                      description,
                      address,
                      latitude,
                      longitude,
                      start_time  as "start_time: time::OffsetDateTime",
                      max_capacity,
                      created_at  as "created_at: time::OffsetDateTime"
               FROM events WHERE host_id = $1
               ORDER BY start_time ASC"#,
            host_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| DomainError::Internal(anyhow::anyhow!("failed to list events")))
    }

    async fn upsert(&self, event: &Event) -> Result<Event, DomainError> {
        sqlx::query_as!(
            Event,
            r#"INSERT INTO events (id, host_id, title, description, address, latitude, longitude, start_time, max_capacity)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
               ON CONFLICT(id) DO UPDATE SET
                   title        = EXCLUDED.title,
                   description  = EXCLUDED.description,
                   address      = EXCLUDED.address,
                   latitude     = EXCLUDED.latitude,
                   longitude    = EXCLUDED.longitude,
                   start_time   = EXCLUDED.start_time,
                   max_capacity = EXCLUDED.max_capacity
               RETURNING id          as "id: Uuid",
                         host_id     as "host_id: Uuid",
                         title,
                         description,
                         address,
                         latitude,
                         longitude,
                         start_time  as "start_time: time::OffsetDateTime",
                         max_capacity,
                         created_at  as "created_at: time::OffsetDateTime""#,
            event.id,
            event.host_id,
            event.title,
            event.description,
            event.address,
            event.latitude,
            event.longitude,
            event.start_time,
            event.max_capacity,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| DomainError::Internal(anyhow::anyhow!("failed to upsert event")))
    }

    async fn delete(&self, event_id: Uuid) -> Result<(), DomainError> {
        let result = sqlx::query!("DELETE FROM events WHERE id = $1", event_id)
            .execute(&self.pool)
            .await
            .map_err(|_| DomainError::Internal(anyhow::anyhow!("failed to delete event")))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound);
        }
        Ok(())
    }
}
