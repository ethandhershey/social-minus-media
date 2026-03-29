use axum::{Json, Router, extract::State, routing::get};
use domain::{user::User, user_interests::UserInterests};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::{OffsetDateTime, serde::rfc3339};
use uuid::Uuid;

use crate::{
    error::ApiError,
    state::{AppServices, AppState, LlmState, UserInterestsRepoState},
};

#[derive(Serialize)]
pub struct UserInterestsResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub messages: Value,
    pub summary: Option<String>,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl From<UserInterests> for UserInterestsResponse {
    fn from(i: UserInterests) -> Self {
        Self {
            id: i.id,
            user_id: i.user_id,
            messages: i.messages,
            summary: i.summary,
            created_at: i.created_at,
            updated_at: i.updated_at,
        }
    }
}

pub fn router<S: AppServices>() -> Router<AppState<S>> {
    Router::new().route("/me/interests", get(get_interests).put(update_interests))
}

#[derive(Deserialize)]
struct UpdateInterestsBody {
    messages: Value,
}

async fn get_interests<S: AppServices>(
    user: User,
    State(repo): State<UserInterestsRepoState<S>>,
) -> Result<Json<Option<UserInterestsResponse>>, ApiError> {
    use domain::ports::UserInterestsRepository;
    let interests: Option<_> = repo.find_by_user(user.id).await?;
    Ok(Json(interests.map(UserInterestsResponse::from)))
}

async fn update_interests<S: AppServices>(
    user: User,
    State(repo): State<UserInterestsRepoState<S>>,
    State(llm): State<LlmState<S>>,
    Json(UpdateInterestsBody { messages }): Json<UpdateInterestsBody>,
) -> Result<Json<UserInterestsResponse>, ApiError> {
    let interests = domain::user_interests::crud::update_interests(
        repo.as_ref(),
        llm.as_ref(),
        user.id,
        messages,
    )
    .await?;
    Ok(Json(UserInterestsResponse::from(interests)))
}
