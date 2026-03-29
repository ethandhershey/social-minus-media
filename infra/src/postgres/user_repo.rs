use domain::ports::UserRepository;
use domain::{
    error::DomainError,
    user::{Tier, User},
};
use sqlx::PgPool;
use uuid::Uuid;

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl UserRepository for PgUserRepository {
    async fn find_by_id(&self, user_id: Uuid) -> Result<User, DomainError> {
        sqlx::query_as!(
            User,
            r#"SELECT id as "id: Uuid", sub, email, display_name,
                      avatar_url, bio, city, latitude, longitude,
                      tier as "tier: Tier",
                      api_usage,
                      storage_usage,
                      billing_customer_id,
                      billing_period_start as "billing_period_start: time::OffsetDateTime",
                      created_at           as "created_at: time::OffsetDateTime",
                      updated_at           as "updated_at: time::OffsetDateTime",
                      archived_at          as "archived_at: time::OffsetDateTime"
               FROM users WHERE id = $1"#,
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
                      avatar_url, bio, city, latitude, longitude,
                      tier as "tier: Tier",
                      api_usage,
                      storage_usage,
                      billing_customer_id,
                      billing_period_start as "billing_period_start: time::OffsetDateTime",
                      created_at           as "created_at: time::OffsetDateTime",
                      updated_at           as "updated_at: time::OffsetDateTime",
                      archived_at          as "archived_at: time::OffsetDateTime"
               FROM users WHERE sub = $1"#,
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
               VALUES ($1, $2, $3, $4)
               ON CONFLICT(sub) DO UPDATE SET
                   email        = EXCLUDED.email,
                   display_name = EXCLUDED.display_name,
                   updated_at   = NOW()
               RETURNING id as "id: Uuid", sub, email, display_name,
                         avatar_url, bio, city, latitude, longitude,
                         tier as "tier: Tier",
                         api_usage,
                         storage_usage,
                         billing_customer_id,
                         billing_period_start as "billing_period_start: time::OffsetDateTime",
                         created_at           as "created_at: time::OffsetDateTime",
                         updated_at           as "updated_at: time::OffsetDateTime",
                         archived_at          as "archived_at: time::OffsetDateTime""#,
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
                      avatar_url, bio, city, latitude, longitude,
                      tier as "tier: Tier",
                      api_usage,
                      storage_usage,
                      billing_customer_id,
                      billing_period_start as "billing_period_start: time::OffsetDateTime",
                      created_at           as "created_at: time::OffsetDateTime",
                      updated_at           as "updated_at: time::OffsetDateTime",
                      archived_at          as "archived_at: time::OffsetDateTime"
               FROM users WHERE billing_customer_id = $1"#,
            customer_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| DomainError::NotFound)
    }

    async fn find_nearby(
        &self,
        lat: f64,
        lon: f64,
        radius_meters: f64,
    ) -> Result<Vec<User>, DomainError> {
        sqlx::query_as!(
            User,
            r#"SELECT id as "id: Uuid", sub, email, display_name,
                      avatar_url, bio, city, latitude, longitude,
                      tier as "tier: Tier",
                      api_usage,
                      storage_usage,
                      billing_customer_id,
                      billing_period_start as "billing_period_start: time::OffsetDateTime",
                      created_at           as "created_at: time::OffsetDateTime",
                      updated_at           as "updated_at: time::OffsetDateTime",
                      archived_at          as "archived_at: time::OffsetDateTime"
               FROM users
               WHERE latitude IS NOT NULL AND longitude IS NOT NULL
                 AND earth_box(ll_to_earth($1, $2), $3) @> ll_to_earth(latitude, longitude)"#,
            lat,
            lon,
            radius_meters,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| DomainError::Internal(anyhow::anyhow!("failed to query nearby users")))
    }

    async fn update_profile(
        &self,
        user_id: Uuid,
        avatar_url: Option<String>,
        bio: Option<String>,
        city: Option<String>,
        latitude: Option<f64>,
        longitude: Option<f64>,
    ) -> Result<User, DomainError> {
        sqlx::query_as!(
            User,
            r#"UPDATE users
               SET avatar_url  = $2,
                   bio         = $3,
                   city        = $4,
                   latitude    = $5,
                   longitude   = $6,
                   updated_at  = NOW()
               WHERE id = $1
               RETURNING id as "id: Uuid", sub, email, display_name,
                         avatar_url, bio, city, latitude, longitude,
                         tier as "tier: Tier",
                         api_usage,
                         storage_usage,
                         billing_customer_id,
                         billing_period_start as "billing_period_start: time::OffsetDateTime",
                         created_at           as "created_at: time::OffsetDateTime",
                         updated_at           as "updated_at: time::OffsetDateTime",
                         archived_at          as "archived_at: time::OffsetDateTime""#,
            user_id,
            avatar_url,
            bio,
            city,
            latitude,
            longitude,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| DomainError::NotFound)
    }

    async fn find_nearby_by_interests(
        &self,
        lat: f64,
        lon: f64,
        radius_meters: f64,
        embedding: &[f32],
    ) -> Result<Vec<User>, DomainError> {
        let vec = pgvector::Vector::from(embedding.to_vec());
        sqlx::query_as!(
            User,
            r#"SELECT u.id           as "id: Uuid",
                      u.sub,
                      u.email,
                      u.display_name,
                      u.avatar_url,
                      u.bio,
                      u.city,
                      u.latitude,
                      u.longitude,
                      u.tier          as "tier: Tier",
                      u.api_usage,
                      u.storage_usage,
                      u.billing_customer_id,
                      u.billing_period_start as "billing_period_start: time::OffsetDateTime",
                      u.created_at          as "created_at: time::OffsetDateTime",
                      u.updated_at          as "updated_at: time::OffsetDateTime",
                      u.archived_at         as "archived_at: time::OffsetDateTime"
               FROM users u
               JOIN user_interests ui ON ui.user_id = u.id
               WHERE u.latitude IS NOT NULL AND u.longitude IS NOT NULL
                 AND earth_box(ll_to_earth($1, $2), $3) @> ll_to_earth(u.latitude, u.longitude)
                 AND ui.embedding IS NOT NULL
               ORDER BY ui.embedding <=> $4"#,
            lat,
            lon,
            radius_meters,
            vec as pgvector::Vector,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| {
            DomainError::Internal(anyhow::anyhow!("failed to query nearby users by interests"))
        })
    }

    async fn update_subscription(
        &self,
        user_id: Uuid,
        tier: Tier,
        stripe_customer_id: Option<String>,
    ) -> Result<(), DomainError> {
        let result = sqlx::query!(
            "UPDATE users
             SET tier = $1,
                 billing_customer_id = COALESCE($2, billing_customer_id),
                 updated_at = NOW()
             WHERE id = $3",
            tier as Tier,
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
