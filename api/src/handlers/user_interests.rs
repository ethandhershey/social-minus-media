use axum::{Json, Router, extract::State, routing::get};
use domain::{ports::UserInterestsRepository, user::User, user_interests::UserInterests};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    error::ApiError,
    state::{AppServices, AppState, LlmState, UserInterestsRepoState},
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
    let interests = repo.find_by_user(user.id).await?;
    Ok(Json(interests))
}

async fn update_interests<S: AppServices>(
    user: User,
    State(repo): State<UserInterestsRepoState<S>>,
    State(llm): State<LlmState<S>>,
    Json(UpdateInterestsBody { messages }): Json<UpdateInterestsBody>,
) -> Result<Json<UserInterests>, ApiError> {
    let interests = domain::user_interests::crud::update_interests(
        repo.as_ref(),
        llm.as_ref(),
        user.id,
        messages,
    )
    .await?;
    Ok(Json(interests))
}
