use crate::{
    error::ApiError,
    state::{AppServices, AppState, UserRepoState},
};
use axum::{Json, Router, extract::{Query, State}, response::IntoResponse, routing::get};
use domain::user::User;
use serde::Deserialize;

pub fn router<S: AppServices>() -> Router<AppState<S>> {
    Router::new()
        .route("/profile", get(get_profile))
        .route("/users/nearby", get(nearby_users))
}

async fn get_profile(user: User) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(user))
}

#[derive(Deserialize)]
struct NearbyQuery {
    lat: f64,
    lon: f64,
    radius_meters: f64,
}

async fn nearby_users<S: AppServices>(
    _user: User,
    State(user_repo): State<UserRepoState<S>>,
    Query(params): Query<NearbyQuery>,
) -> Result<Json<Vec<User>>, ApiError> {
    use domain::ports::UserRepository;
    let users = user_repo
        .find_nearby(params.lat, params.lon, params.radius_meters)
        .await?;
    Ok(Json(users))
}
