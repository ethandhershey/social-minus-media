use crate::{
    error::ApiError,
    state::{AppServices, AppState, UserInterestsRepoState, UserRepoState},
};
use axum::{
    Json, Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
};
use domain::{error::DomainError, user::{Tier, User}};
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
        .route("/profile", get(get_profile))
        .route("/users/nearby", get(nearby_users))
        .route("/users/nearby-by-interests", get(nearby_users_by_interests))
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn get_profile(user: User) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(PrivateUserResponse::from(user)))
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
) -> Result<Json<Vec<PublicUserResponse>>, ApiError> {
    use domain::ports::UserRepository;
    let users = user_repo
        .find_nearby(params.lat, params.lon, params.radius_meters)
        .await?;
    Ok(Json(users.into_iter().map(PublicUserResponse::from).collect()))
}

async fn nearby_users_by_interests<S: AppServices>(
    user: User,
    State(user_repo): State<UserRepoState<S>>,
    State(interests_repo): State<UserInterestsRepoState<S>>,
    Query(params): Query<NearbyQuery>,
) -> Result<Json<Vec<PublicUserResponse>>, ApiError> {
    use domain::ports::{UserInterestsRepository, UserRepository};

    let embedding = interests_repo
        .find_by_user(user.id)
        .await?
        .and_then(|i| i.embedding)
        .ok_or_else(|| DomainError::InvalidInput("no interest embedding found".into()))?;

    let users = user_repo
        .find_nearby_by_interests(params.lat, params.lon, params.radius_meters, &embedding)
        .await?;

    Ok(Json(users.into_iter().map(PublicUserResponse::from).collect()))
}
