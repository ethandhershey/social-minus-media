use axum::{
    Json, Router,
    http::StatusCode,
    routing::{MethodRouter, any, get},
};
use serde::Serialize;

pub fn root_router<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/health", any(health_check))
        .route("/healthz", any(health_check))
        .route("/status", any(health_check))
}

pub fn router<S: Clone + Send + Sync + 'static>(build_id: &'static str) -> Router<S> {
    Router::new()
        .route("/health", any(health_check))
        .route("/version", version_handler(build_id))
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

fn version_handler<S>(build_id: &'static str) -> MethodRouter<S>
where
    S: Clone + Send + Sync + 'static,
{
    get(move || async move { Json(VersionResponse { version: build_id }) })
}
