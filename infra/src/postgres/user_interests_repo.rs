use anyhow::anyhow;
use domain::error::DomainError;
use domain::ports::UserInterestsRepository;
use domain::user_interests::UserInterests;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PgUserInterestsRepository {
    pool: PgPool,
    summary_model: String,
    embed_model: String,
}

impl PgUserInterestsRepository {
    pub fn new(pool: PgPool, summary_model: String, embed_model: String) -> Self {
        Self {
            pool,
            summary_model,
            embed_model,
        }
    }
}

struct UserInterestsRow {
    id: Uuid,
    user_id: Uuid,
    messages: serde_json::Value,
    summary: Option<String>,
    embedding: Option<pgvector::Vector>,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
}

impl From<UserInterestsRow> for UserInterests {
    fn from(row: UserInterestsRow) -> Self {
        UserInterests {
            id: row.id,
            user_id: row.user_id,
            messages: row.messages,
            summary: row.summary,
            embedding: row.embedding.map(|v| v.to_vec()),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl UserInterestsRepository for PgUserInterestsRepository {
    async fn find_by_user(&self, user_id: Uuid) -> Result<Option<UserInterests>, DomainError> {
        sqlx::query_as!(
            UserInterestsRow,
            r#"SELECT id         as "id: Uuid",
                      user_id    as "user_id: Uuid",
                      messages   as "messages: serde_json::Value",
                      summary,
                      embedding  as "embedding: pgvector::Vector",
                      created_at as "created_at: time::OffsetDateTime",
                      updated_at as "updated_at: time::OffsetDateTime"
               FROM user_interests WHERE user_id = $1"#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| DomainError::Internal(anyhow!("failed to fetch user interests")))
        .map(|opt| opt.map(Into::into))
    }

    async fn upsert(&self, interests: &UserInterests) -> Result<UserInterests, DomainError> {
        let embedding = interests
            .embedding
            .as_ref()
            .map(|v| pgvector::Vector::from(v.clone()));

        sqlx::query_as!(
            UserInterestsRow,
            r#"INSERT INTO user_interests (id, user_id, messages, summary, embedding)
               VALUES ($1, $2, $3, $4, $5)
               ON CONFLICT(id) DO UPDATE SET
                   messages   = EXCLUDED.messages,
                   summary    = EXCLUDED.summary,
                   embedding  = EXCLUDED.embedding,
                   updated_at = NOW()
               RETURNING id         as "id: Uuid",
                         user_id    as "user_id: Uuid",
                         messages   as "messages: serde_json::Value",
                         summary,
                         embedding  as "embedding: pgvector::Vector",
                         created_at as "created_at: time::OffsetDateTime",
                         updated_at as "updated_at: time::OffsetDateTime""#,
            interests.id,
            interests.user_id,
            interests.messages,
            interests.summary,
            embedding as Option<pgvector::Vector>,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| DomainError::Internal(anyhow!("failed to upsert user interests")))
        .map(Into::into)
    }

    fn get_summary_model(&self) -> &str {
        &self.summary_model
    }

    fn get_embed_model(&self) -> &str {
        &self.embed_model
    }
}
