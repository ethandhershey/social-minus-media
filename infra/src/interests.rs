use domain::{
    error::DomainError,
    ports::{InterestsService, LlmService, UserInterestsRepository},
    user_interests::UserInterests,
};
use serde_json::Value;
use uuid::Uuid;

pub struct LlmInterestsService {
    summary_model: String,
    embed_model: String,
}

impl LlmInterestsService {
    pub fn new(summary_model: String, embed_model: String) -> Self {
        Self { summary_model, embed_model }
    }
}

impl InterestsService for LlmInterestsService {
    async fn get_interests(
        &self,
        repo: &impl UserInterestsRepository,
        user_id: Uuid,
    ) -> Result<Option<UserInterests>, DomainError> {
        domain::user_interests::crud::get_interests(repo, user_id).await
    }

    async fn update_interests(
        &self,
        repo: &impl UserInterestsRepository,
        llm: &impl LlmService,
        user_id: Uuid,
        messages: Value,
    ) -> Result<UserInterests, DomainError> {
        domain::user_interests::crud::update_interests(
            repo,
            llm,
            user_id,
            messages,
            &self.summary_model,
            &self.embed_model,
        )
        .await
    }
}
