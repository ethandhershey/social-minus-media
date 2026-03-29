use crate::{
    error::ApiError,
    state::{AppServices, AppState, UserInterestsRepoState, UserRepoState},
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::get,
};
use domain::user::{Tier, User};
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, serde::rfc3339};
use uuid::Uuid;

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct PrivateUserResponse {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub tier: Tier,
    pub api_usage: i64,
    pub storage_usage: i64,
    pub billing_customer_id: Option<String>,
    #[serde(with = "rfc3339")]
    pub billing_period_start: OffsetDateTime,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "rfc3339::option")]
    pub archived_at: Option<OffsetDateTime>,
}

impl From<User> for PrivateUserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            email: u.email,
            display_name: u.display_name,
            avatar_url: u.avatar_url,
            bio: u.bio,
            city: u.city,
            latitude: u.latitude,
            longitude: u.longitude,
            tier: u.tier,
            api_usage: u.api_usage.into(),
            storage_usage: u.storage_usage.into(),
            billing_customer_id: u.billing_customer_id,
            billing_period_start: u.billing_period_start,
            created_at: u.created_at,
            updated_at: u.updated_at,
            archived_at: u.archived_at,
        }
    }
}

#[derive(Serialize)]
pub struct PublicUserResponse {
    pub id: Uuid,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
}

impl From<User> for PublicUserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            display_name: u.display_name,
            avatar_url: u.avatar_url,
            bio: u.bio,
            city: u.city,
            latitude: u.latitude,
            longitude: u.longitude,
            created_at: u.created_at,
        }
    }
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router<S: AppServices>() -> Router<AppState<S>> {
    Router::new()
        .route("/profile", get(get_profile).put(update_profile))
        .route("/users/{id}", get(get_user))
        .route("/users/nearby", get(nearby_users))
        .route("/users/nearby-by-interests", get(nearby_users_by_interests))
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn get_profile(user: User) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(PrivateUserResponse::from(user)))
}

#[derive(Deserialize)]
struct UpdateProfileBody {
    avatar_url: Option<String>,
    bio: Option<String>,
    city: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

async fn update_profile<S: AppServices>(
    user: User,
    State(user_repo): State<UserRepoState<S>>,
    Json(body): Json<UpdateProfileBody>,
) -> Result<Json<PrivateUserResponse>, ApiError> {
    let updated = domain::user::crud::update_profile(
        user_repo.as_ref(),
        user.id,
        body.avatar_url,
        body.bio,
        body.city,
        body.latitude,
        body.longitude,
    )
    .await?;
    Ok(Json(PrivateUserResponse::from(updated)))
}

async fn get_user<S: AppServices>(
    _user: User,
    State(user_repo): State<UserRepoState<S>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<PublicUserResponse>, ApiError> {
    let user = domain::user::crud::get_user(user_repo.as_ref(), user_id).await?;
    Ok(Json(PublicUserResponse::from(user)))
}

#[derive(Deserialize)]
struct NearbyQuery {
    lat: f64,
    lon: f64,
    radius_meters: f64,
}

async fn nearby_users<S: AppServices>(
    user: User,
    State(user_repo): State<UserRepoState<S>>,
    Query(params): Query<NearbyQuery>,
) -> Result<Json<Vec<PublicUserResponse>>, ApiError> {
    let users = domain::user::crud::find_nearby_users(
        user_repo.as_ref(),
        user.id,
        params.lat,
        params.lon,
        params.radius_meters,
    )
    .await?;
    Ok(Json(
        users.into_iter().map(PublicUserResponse::from).collect(),
    ))
}

async fn nearby_users_by_interests<S: AppServices>(
    user: User,
    State(user_repo): State<UserRepoState<S>>,
    State(interests_repo): State<UserInterestsRepoState<S>>,
    Query(params): Query<NearbyQuery>,
) -> Result<Json<Vec<PublicUserResponse>>, ApiError> {
    let users = domain::user::crud::find_nearby_by_interests(
        user_repo.as_ref(),
        interests_repo.as_ref(),
        user.id,
        params.lat,
        params.lon,
        params.radius_meters,
    )
    .await?;

    Ok(Json(
        users.into_iter().map(PublicUserResponse::from).collect(),
    ))
}
