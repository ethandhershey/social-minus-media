use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{any, get},
};
use serde::Serialize;

use axum::extract::FromRef;

pub fn root_router<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/health", any(health_check))
        .route("/healthz", any(health_check))
        .route("/status", any(health_check))
}

pub fn router<S: Clone + Send + Sync + 'static>() -> Router<S>
where
    crate::state::PublicConfig: FromRef<S>,
{
    Router::new()
        .route("/health", any(health_check))
        .route("/version", get(version))
        .route("/config", get(public_config))
}

#[cfg(debug_assertions)]
pub fn debug_router<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new().route("/health", any(health_check))
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

#[derive(Serialize)]
struct VersionResponse {
    version: &'static str,
}

async fn version(State(public_config): State<crate::state::PublicConfig>) -> impl IntoResponse {
    Json(VersionResponse {
        version: public_config.version,
    })
}

async fn public_config(
    State(public_config): State<crate::state::PublicConfig>,
) -> impl IntoResponse {
    Json(public_config)
}
