use axum::{Json, Router, extract::State, routing::get};
use domain::{ports::InterestsService, user::User, user_interests::UserInterests};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    error::ApiError,
    state::{AppServices, AppState, InterestsServiceState, LlmState, UserInterestsRepoState},
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
    State(svc): State<InterestsServiceState<S>>,
    State(repo): State<UserInterestsRepoState<S>>,
) -> Result<Json<Option<UserInterests>>, ApiError> {
    let interests = svc.get_interests(repo.as_ref(), user.id).await?;
    Ok(Json(interests))
}

async fn update_interests<S: AppServices>(
    user: User,
    State(svc): State<InterestsServiceState<S>>,
    State(repo): State<UserInterestsRepoState<S>>,
    State(llm): State<LlmState<S>>,
    Json(body): Json<UpdateInterestsBody>,
) -> Result<Json<UserInterests>, ApiError> {
    let interests = svc.update_interests(repo.as_ref(), llm.as_ref(), user.id, body.messages).await?;
    Ok(Json(interests))
}
