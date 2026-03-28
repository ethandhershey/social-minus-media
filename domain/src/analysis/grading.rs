use uuid::Uuid;

use crate::user::User;
use crate::{analysis::ProductGrade, error::DomainError};

use crate::ports::{EntitlementService, LlmService, ProductRepository};

pub async fn grade_product(
    user: &User,
    ai: &impl LlmService,
    product_repo: &impl ProductRepository,
    entitlment: &impl EntitlementService,
    product_id: Uuid,
    query_model: &str,
    grader_model: &str,
) -> Result<ProductGrade, DomainError> {
    const QUERY_SYSTEM_PROMPT: &str = "You are a helpful assistant."; // TODO
    const GRADER_SYSTEM_PROMPT: &str = "Rate the user's view of the product on a scale from 1 to 10 where 1 is negative and 10 is positive. Explain your answer in one paragraph."; // TODO

    entitlment.check_tier(user, &crate::user::Tier::Essential)?;

    let product = product_repo.find(product_id).await?;

    let product_query = format!("What do you think of this product: {}", product.name); // TODO

    let query_response = ai
        .get_simple_response(query_model, QUERY_SYSTEM_PROMPT, &product_query)
        .await?;

    let grader_response = ai
        .get_structured_response(grader_model, GRADER_SYSTEM_PROMPT, &query_response)
        .await?;

    Ok(grader_response)
}
