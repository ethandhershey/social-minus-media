use anyhow::anyhow;
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    error::DomainError,
    ports::{LlmService, UserInterestsRepository},
    user_interests::UserInterests,
};

pub async fn get_interests(
    repo: &impl UserInterestsRepository,
    user_id: Uuid,
) -> Result<Option<UserInterests>, DomainError> {
    repo.find_by_user(user_id).await
}

pub async fn update_interests(
    repo: &impl UserInterestsRepository,
    llm: &impl LlmService,
    user_id: Uuid,
    messages: Value,
    summary_model: &str,
    embed_model: &str,
) -> Result<UserInterests, DomainError> {
    let messages_str = serde_json::to_string(&messages)
        .map_err(|e| DomainError::Internal(anyhow!("failed to serialize messages: {e}")))?;

    let summary = llm
        .get_simple_response(
            summary_model,
            "Summarize the user's interests and preferences based on this conversation. Be concise and focus on topics, hobbies, and activities they care about.",
            &messages_str,
        )
        .await?;

    let embedding = llm.embed(embed_model, &summary).await?;

    let existing = repo.find_by_user(user_id).await?;
    let now = OffsetDateTime::now_utc();
    let (id, created_at) = match existing {
        Some(e) => (e.id, e.created_at),
        None => (Uuid::now_v7(), now),
    };

    let interests = UserInterests {
        id,
        user_id,
        messages,
        summary: Some(summary),
        embedding: Some(embedding),
        created_at,
        updated_at: now,
    };

    repo.upsert(&interests).await
}
