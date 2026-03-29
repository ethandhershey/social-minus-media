use axum::{
    Json, Router,
    extract::State,
    routing::get,
};
use domain::{user::User, user_interests::UserInterests};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

use crate::{
    error::ApiError,
    state::{AppServices, AppState, InterestsConfig, LlmState, UserInterestsRepoState},
};

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
) -> Result<Json<Option<UserInterests>>, ApiError> {
    let interests = domain::user_interests::crud::get_interests(repo.as_ref(), user.id).await?;
    Ok(Json(interests))
}

async fn update_interests<S: AppServices>(
    user: User,
    State(repo): State<UserInterestsRepoState<S>>,
    State(llm): State<LlmState<S>>,
    State(config): State<Arc<InterestsConfig>>,
    Json(body): Json<UpdateInterestsBody>,
) -> Result<Json<UserInterests>, ApiError> {
    let interests = domain::user_interests::crud::update_interests(
        repo.as_ref(),
        llm.as_ref(),
        user.id,
        body.messages,
        &config.summary_model,
        &config.embed_model,
    )
    .await?;
    Ok(Json(interests))
}
