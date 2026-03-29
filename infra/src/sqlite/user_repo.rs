use domain::ports::UserRepository;
use domain::{
    error::DomainError,
    user::{Tier, User},
};
use sqlx::SqlitePool;
use uuid::Uuid;

pub struct SqliteUserRepository {
    pool: SqlitePool,
}

impl SqliteUserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl UserRepository for SqliteUserRepository {
    async fn find_by_id(&self, user_id: Uuid) -> Result<User, DomainError> {
        sqlx::query_as!(
            User,
            r#"SELECT id as "id: Uuid", sub, email, display_name,
                      tier as "tier: Tier",
                      api_usage as "api_usage: i64",
                      storage_usage as "storage_usage: i64",
                      billing_customer_id,
                      billing_period_start as "billing_period_start: time::OffsetDateTime",
                      created_at           as "created_at: time::OffsetDateTime",
                      updated_at           as "updated_at: time::OffsetDateTime",
                      archived_at          as "archived_at?: time::OffsetDateTime"
               FROM users WHERE id = ?"#,
            user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| DomainError::NotFound)
    }

    async fn find_by_sub(&self, sub: &str) -> Result<User, DomainError> {
        sqlx::query_as!(
            User,
            r#"SELECT id as "id: Uuid", sub, email, display_name,
                      tier as "tier: Tier",
                      api_usage as "api_usage: i64",
                      storage_usage as "storage_usage: i64",
                      billing_customer_id,
                      billing_period_start as "billing_period_start: time::OffsetDateTime",
                      created_at           as "created_at: time::OffsetDateTime",
                      updated_at           as "updated_at: time::OffsetDateTime",
                      archived_at          as "archived_at?: time::OffsetDateTime"
               FROM users WHERE sub = ?"#,
            sub
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| DomainError::NotFound)
    }

    async fn upsert(&self, user: &User) -> Result<User, DomainError> {
        let id = Uuid::now_v7();
        sqlx::query_as!(
            User,
            r#"INSERT INTO users (id, sub, email, display_name)
               VALUES (?, ?, ?, ?)
               ON CONFLICT(sub) DO UPDATE SET
                   email        = excluded.email,
                   display_name = excluded.display_name,
                   updated_at   = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
               RETURNING id as "id: Uuid", sub, email, display_name,
                         tier as "tier: Tier",
                         api_usage as "api_usage: i64",
                         storage_usage as "storage_usage: i64",
                         billing_customer_id,
                         billing_period_start as "billing_period_start: time::OffsetDateTime",
                         created_at           as "created_at: time::OffsetDateTime",
                         updated_at           as "updated_at: time::OffsetDateTime",
                         archived_at          as "archived_at?: time::OffsetDateTime""#,
            id,
            user.sub,
            user.email,
            user.display_name,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| DomainError::InvalidInput("failed to upsert profile".into()))
    }

    async fn find_by_billing_customer_id(&self, customer_id: &str) -> Result<User, DomainError> {
        sqlx::query_as!(
            User,
            r#"SELECT id as "id: Uuid", sub, email, display_name,
                      tier as "tier: Tier",
                      api_usage as "api_usage: i64",
                      storage_usage as "storage_usage: i64",
                      billing_customer_id,
                      billing_period_start as "billing_period_start: time::OffsetDateTime",
                      created_at           as "created_at: time::OffsetDateTime",
                      updated_at           as "updated_at: time::OffsetDateTime",
                      archived_at          as "archived_at?: time::OffsetDateTime"
               FROM users WHERE billing_customer_id = ?"#,
            customer_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| DomainError::NotFound)
    }

    async fn update_subscription(
        &self,
        user_id: Uuid,
        tier: Tier,
        stripe_customer_id: Option<String>,
    ) -> Result<(), DomainError> {
        let result = sqlx::query!(
            "UPDATE users
             SET tier = ?,
                 billing_customer_id = COALESCE(?, billing_customer_id),
                 updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
             WHERE id = ?",
            tier,
            stripe_customer_id,
            user_id,
        )
        .execute(&self.pool)
        .await
        .map_err(|_| DomainError::InvalidInput("failed to update subscription".into()))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound);
        }
        Ok(())
    }
}
