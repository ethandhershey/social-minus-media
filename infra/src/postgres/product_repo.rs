use domain::error::DomainError;
use domain::ports::ProductRepository;
use domain::product::Product;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PgProductRepository {
    pool: PgPool,
}

impl PgProductRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ProductRepository for PgProductRepository {
    async fn find(&self, product_id: Uuid) -> Result<Product, DomainError> {
        sqlx::query_as!(
            Product,
            r#"SELECT id as "id: Uuid", user_id as "user_id: Uuid", name, brand,
                      created_at  as "created_at: time::OffsetDateTime",
                      updated_at  as "updated_at: time::OffsetDateTime",
                      archived_at as "archived_at: time::OffsetDateTime"
               FROM products WHERE id = $1"#,
            product_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| DomainError::NotFound)
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Product>, DomainError> {
        sqlx::query_as!(
            Product,
            r#"SELECT id as "id: Uuid", user_id as "user_id: Uuid", name, brand,
                      created_at  as "created_at: time::OffsetDateTime",
                      updated_at  as "updated_at: time::OffsetDateTime",
                      archived_at as "archived_at: time::OffsetDateTime"
               FROM products WHERE user_id = $1 ORDER BY created_at DESC"#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| DomainError::InvalidInput("failed to fetch products".into()))
    }

    async fn upsert(&self, product: &Product) -> Result<(), DomainError> {
        sqlx::query!(
            r#"INSERT INTO products (id, user_id, name, brand)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT(id) DO UPDATE SET
                   name        = EXCLUDED.name,
                   brand       = EXCLUDED.brand,
                   archived_at = EXCLUDED.archived_at,
                   updated_at  = NOW()"#,
            product.id,
            product.user_id,
            product.name,
            product.brand,
        )
        .execute(&self.pool)
        .await
        .map_err(|_| DomainError::InvalidInput("failed to upsert product".into()))?;
        Ok(())
    }

    async fn delete(&self, product_id: Uuid) -> Result<(), DomainError> {
        let result = sqlx::query!("DELETE FROM products WHERE id = $1", product_id)
            .execute(&self.pool)
            .await
            .map_err(|_| DomainError::InvalidInput("failed to delete product".into()))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound);
        }
        Ok(())
    }
}
