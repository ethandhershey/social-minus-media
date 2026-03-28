use time::OffsetDateTime;
use uuid::Uuid;

use crate::{error::DomainError, ports::ProductRepository, product::Product};

pub async fn create_product(
    repo: &impl ProductRepository,
    user_id: Uuid,
    name: String,
    brand: Option<String>,
) -> Result<Product, DomainError> {
    let now = OffsetDateTime::now_utc();
    let product = Product {
        id: Uuid::now_v7(),
        user_id,
        name,
        brand,
        created_at: now,
        updated_at: now,
        archived_at: None,
    };
    repo.upsert(&product).await?;
    Ok(product)
}

/// Fetches a product by ID, returning `NotFound` if it doesn't exist or belongs to another user.
pub async fn get_product(
    repo: &impl ProductRepository,
    product_id: Uuid,
    user_id: Uuid,
) -> Result<Product, DomainError> {
    let product = repo.find(product_id).await?;
    if product.user_id != user_id {
        return Err(DomainError::NotFound);
    }
    Ok(product)
}

/// Lists all non-archived products for a user.
pub async fn list_products(
    repo: &impl ProductRepository,
    user_id: Uuid,
) -> Result<Vec<Product>, DomainError> {
    let products = repo.find_by_user(user_id).await?;
    Ok(products
        .into_iter()
        .filter(|p| p.archived_at.is_none())
        .collect())
}

/// Soft-deletes a product by setting `archived_at`. Returns the updated product.
pub async fn archive_product(
    repo: &impl ProductRepository,
    product_id: Uuid,
    user_id: Uuid,
) -> Result<Product, DomainError> {
    let mut product = repo.find(product_id).await?;
    if product.user_id != user_id {
        return Err(DomainError::NotFound);
    }
    if product.archived_at.is_some() {
        return Err(DomainError::InvalidInput(
            "product is already archived".into(),
        ));
    }
    product.archived_at = Some(OffsetDateTime::now_utc());
    repo.upsert(&product).await?;
    Ok(product)
}

/// Permanently deletes a product. Returns `NotFound` if it doesn't exist or belongs to another user.
pub async fn delete_product(
    repo: &impl ProductRepository,
    product_id: Uuid,
    user_id: Uuid,
) -> Result<(), DomainError> {
    let product = repo.find(product_id).await?;
    if product.user_id != user_id {
        return Err(DomainError::NotFound);
    }
    repo.delete(product_id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::fake_product_repository::FakeProductRepository;
    use time::OffsetDateTime;

    fn make_product(user_id: Uuid) -> Product {
        let now = OffsetDateTime::now_utc();
        Product {
            id: Uuid::now_v7(),
            user_id,
            name: "Widget".to_string(),
            brand: Some("Acme".to_string()),
            created_at: now,
            updated_at: now,
            archived_at: None,
        }
    }

    #[tokio::test]
    async fn create_product_sets_correct_fields() {
        let repo = FakeProductRepository::new();
        let user_id = Uuid::now_v7();

        let product = create_product(
            &repo,
            user_id,
            "Widget".to_string(),
            Some("Acme".to_string()),
        )
        .await
        .unwrap();

        assert_eq!(product.user_id, user_id);
        assert_eq!(product.name, "Widget");
        assert_eq!(product.brand, Some("Acme".to_string()));
        assert!(product.archived_at.is_none());
    }

    #[tokio::test]
    async fn get_product_returns_not_found_for_unknown_id() {
        let repo = FakeProductRepository::new();
        let user_id = Uuid::now_v7();

        let result = get_product(&repo, Uuid::now_v7(), user_id).await;

        assert!(matches!(result, Err(DomainError::NotFound)));
    }

    #[tokio::test]
    async fn get_product_returns_not_found_for_wrong_user() {
        let owner_id = Uuid::now_v7();
        let product = make_product(owner_id);
        let repo = FakeProductRepository::new().with_product(product.clone());

        let result = get_product(&repo, product.id, Uuid::now_v7()).await;

        assert!(matches!(result, Err(DomainError::NotFound)));
    }

    #[tokio::test]
    async fn list_products_excludes_archived() {
        let user_id = Uuid::now_v7();
        let mut archived = make_product(user_id);
        archived.archived_at = Some(OffsetDateTime::now_utc());
        let active = make_product(user_id);
        let repo = FakeProductRepository::new()
            .with_product(archived)
            .with_product(active.clone());

        let products = list_products(&repo, user_id).await.unwrap();

        assert_eq!(products.len(), 1);
        assert_eq!(products[0].id, active.id);
    }

    #[tokio::test]
    async fn list_products_returns_only_user_products() {
        let user_id = Uuid::now_v7();
        let other_user_id = Uuid::now_v7();
        let repo = FakeProductRepository::new()
            .with_product(make_product(user_id))
            .with_product(make_product(other_user_id));

        let products = list_products(&repo, user_id).await.unwrap();

        assert_eq!(products.len(), 1);
        assert_eq!(products[0].user_id, user_id);
    }

    #[tokio::test]
    async fn archive_product_sets_archived_at() {
        let user_id = Uuid::now_v7();
        let product = make_product(user_id);
        let repo = FakeProductRepository::new().with_product(product.clone());

        let result = archive_product(&repo, product.id, user_id).await.unwrap();

        assert!(result.archived_at.is_some());
    }

    #[tokio::test]
    async fn archive_product_returns_not_found_for_wrong_user() {
        let product = make_product(Uuid::now_v7());
        let repo = FakeProductRepository::new().with_product(product.clone());

        let result = archive_product(&repo, product.id, Uuid::now_v7()).await;

        assert!(matches!(result, Err(DomainError::NotFound)));
    }

    #[tokio::test]
    async fn archive_already_archived_product_returns_error() {
        let user_id = Uuid::now_v7();
        let mut product = make_product(user_id);
        product.archived_at = Some(OffsetDateTime::now_utc());
        let repo = FakeProductRepository::new().with_product(product.clone());

        let result = archive_product(&repo, product.id, user_id).await;

        assert!(matches!(result, Err(DomainError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn delete_product_removes_it() {
        let user_id = Uuid::now_v7();
        let product = make_product(user_id);
        let repo = FakeProductRepository::new().with_product(product.clone());

        delete_product(&repo, product.id, user_id).await.unwrap();

        let result = repo.find(product.id).await;
        assert!(matches!(result, Err(DomainError::NotFound)));
    }

    #[tokio::test]
    async fn delete_product_returns_not_found_for_wrong_user() {
        let product = make_product(Uuid::now_v7());
        let repo = FakeProductRepository::new().with_product(product.clone());

        let result = delete_product(&repo, product.id, Uuid::now_v7()).await;

        assert!(matches!(result, Err(DomainError::NotFound)));
    }
}
