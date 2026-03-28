use crate::{
    error::ApiError,
    state::{AppServices, AppState},
};
use axum::{Json, Router, response::IntoResponse, routing::get};
use domain::user::User;

pub fn router<S: AppServices>() -> Router<AppState<S>> {
    Router::new().route("/profile", get(get_profile))
}

async fn get_profile(user: User) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(user))
}
