use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use domain::{product::Product, user::User};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::ApiError,
    state::{AppServices, AppState, ProductRepoState},
};

pub fn router<S: AppServices>() -> Router<AppState<S>> {
    Router::new()
        .route("/products", get(list_products).post(create_product))
        .route("/products/{id}", get(get_product).delete(delete_product))
        .route("/products/{id}/archive", post(archive_product))
}

#[derive(Deserialize)]
struct CreateProductBody {
    name: String,
    brand: Option<String>,
}

async fn list_products<S: AppServices>(
    user: User,
    State(product_repo): State<ProductRepoState<S>>,
) -> Result<Json<Vec<Product>>, ApiError> {
    let products = domain::product::crud::list_products(product_repo.as_ref(), user.id).await?;
    Ok(Json(products))
}

async fn create_product<S: AppServices>(
    user: User,
    State(product_repo): State<ProductRepoState<S>>,
    Json(CreateProductBody { name, brand }): Json<CreateProductBody>,
) -> Result<Json<Product>, ApiError> {
    let product =
        domain::product::crud::create_product(product_repo.as_ref(), user.id, name, brand).await?;
    Ok(Json(product))
}

async fn get_product<S: AppServices>(
    user: User,
    State(product_repo): State<ProductRepoState<S>>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<Product>, ApiError> {
    let product =
        domain::product::crud::get_product(product_repo.as_ref(), product_id, user.id).await?;
    Ok(Json(product))
}

async fn archive_product<S: AppServices>(
    user: User,
    State(product_repo): State<ProductRepoState<S>>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<Product>, ApiError> {
    let product =
        domain::product::crud::archive_product(product_repo.as_ref(), product_id, user.id).await?;
    Ok(Json(product))
}

async fn delete_product<S: AppServices>(
    user: User,
    State(product_repo): State<ProductRepoState<S>>,
    Path(product_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    domain::product::crud::delete_product(product_repo.as_ref(), product_id, user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}
