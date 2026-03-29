use std::{path::Path, time::Duration};

use crate::{
    handlers,
    state::{AppServices, AppState},
};
use axum::{
    Router,
    http::{HeaderValue, StatusCode},
};
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    services::{ServeDir, ServeFile},
};

pub fn create_router<S: AppServices>(
    state: AppState<S>,
    build_id: &'static str,
    frontend_dir: impl AsRef<Path>,
    allowed_origins: Vec<HeaderValue>,
    max_upload_size: usize,
) -> Router {
    let frontend_path = frontend_dir.as_ref();
    let static_frontend = ServeDir::new(frontend_path)
        .append_index_html_on_directories(true)
        .precompressed_gzip()
        .fallback(ServeFile::new(frontend_path.join("404.html")));

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed_origins))
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ])
        .allow_credentials(true);

    let api = handlers::simple::router(build_id)
        .merge(handlers::user::router())
        .merge(handlers::product::router())
        .merge(handlers::event::router())
        .merge(handlers::user_interests::router())
        .merge(handlers::analysis::router())
        .merge(handlers::billing::router());

    let router = handlers::simple::root_router().nest("/api", api);

    #[cfg(debug_assertions)]
    let router = router.nest("/test", handlers::simple::debug_router());

    router
        .with_state(state)
        .route_layer(tower_http::timeout::TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
        .fallback_service(static_frontend)
        .layer((
            tower_http::trace::TraceLayer::new_for_http(),
            tower_http::decompression::RequestDecompressionLayer::new(),
            tower_http::compression::CompressionLayer::new(),
            axum::extract::DefaultBodyLimit::max(max_upload_size),
            cors,
        ))
}
