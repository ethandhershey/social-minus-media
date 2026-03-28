use axum::{Json, Router, extract::State, routing::post};
use domain::{analysis::ProductGrade, user::User};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::ApiError,
    state::{AppServices, AppState, EntitlementState, LlmState, ProductRepoState},
};

pub fn router<S: AppServices>() -> Router<AppState<S>> {
    Router::new().route("/analysis/grade_product", post(grade_product))
}

#[derive(Deserialize)]
struct GradeQuery {
    product: Uuid,
    query_model: String,
    grader_model: String,
}

async fn grade_product<S: AppServices>(
    user: User,
    State(product_repo): State<ProductRepoState<S>>,
    State(ai): State<LlmState<S>>,
    State(entitlment): State<EntitlementState<S>>,
    Json(GradeQuery {
        product,
        query_model,
        grader_model,
    }): Json<GradeQuery>,
) -> Result<Json<ProductGrade>, ApiError> {
    let grade = domain::analysis::grading::grade_product(
        &user,
        ai.as_ref(),
        product_repo.as_ref(),
        entitlment.as_ref(),
        product,
        &query_model,
        &grader_model,
    )
    .await?;
    Ok(Json(grade))
}
