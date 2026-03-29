use std::str::FromStr;

use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};

pub mod event_repo;
pub mod product_repo;
pub mod rsvp_repo;
pub mod user_interests_repo;
pub mod user_repo;

pub async fn create(url: &str, max_connections: u32) -> Result<PgPool, sqlx::Error> {
    let options = PgConnectOptions::from_str(url)?;

    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect_with(options)
        .await?;

    Ok(pool)
}

pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("../migrations").run(pool).await
}
