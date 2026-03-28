use std::collections::HashMap;
use std::sync::RwLock;

use uuid::Uuid;

use crate::{error::DomainError, ports::ProductRepository, product::Product};

pub struct FakeProductRepository {
    products: RwLock<HashMap<Uuid, Product>>,
}

impl FakeProductRepository {
    pub fn new() -> Self {
        Self {
            products: RwLock::new(HashMap::new()),
        }
    }

    pub fn with_product(self, product: Product) -> Self {
        self.products.write().unwrap().insert(product.id, product);
        self
    }
}

impl ProductRepository for FakeProductRepository {
    async fn find(&self, product_id: Uuid) -> Result<Product, DomainError> {
        self.products
            .read()
            .unwrap()
            .get(&product_id)
            .cloned()
            .ok_or(DomainError::NotFound)
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Product>, DomainError> {
        Ok(self
            .products
            .read()
            .unwrap()
            .values()
            .filter(|p| p.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn upsert(&self, product: &Product) -> Result<(), DomainError> {
        self.products
            .write()
            .unwrap()
            .insert(product.id, product.clone());
        Ok(())
    }

    async fn delete(&self, product_id: Uuid) -> Result<(), DomainError> {
        if self.products.write().unwrap().remove(&product_id).is_none() {
            return Err(DomainError::NotFound);
        }
        Ok(())
    }
}
