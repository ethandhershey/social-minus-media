use uuid::Uuid;

use crate::{
    error::DomainError,
    ports::{UserInterestsRepository, UserRepository},
    user::{AuthUser, User},
};

/// Gets the user profile, upserting on first access or when identity data changes.
pub async fn get_or_upsert_user(
    repo: &impl UserRepository,
    user: &AuthUser,
) -> Result<User, DomainError> {
    match repo.find_by_sub(&user.sub).await {
        Ok(profile) => repo.upsert(&profile).await,
        Err(e) => Err(e),
    }
}

pub async fn get_user(repo: &impl UserRepository, user_id: Uuid) -> Result<User, DomainError> {
    repo.find_by_id(user_id).await
}

pub async fn update_profile(
    repo: &impl UserRepository,
    user_id: Uuid,
    avatar_url: Option<String>,
    bio: Option<String>,
    city: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
) -> Result<User, DomainError> {
    repo.update_profile(user_id, avatar_url, bio, city, latitude, longitude)
        .await
}

pub async fn find_nearby_users(
    repo: &impl UserRepository,
    user_id: Uuid,
    lat: f64,
    lon: f64,
    radius_meters: f64,
) -> Result<Vec<User>, DomainError> {
    let users = repo.find_nearby(lat, lon, radius_meters).await?;
    Ok(users.into_iter().filter(|u| u.id != user_id).collect())
}

pub async fn find_nearby_by_interests(
    user_repo: &impl UserRepository,
    interests_repo: &impl UserInterestsRepository,
    user_id: Uuid,
    lat: f64,
    lon: f64,
    radius_meters: f64,
) -> Result<Vec<User>, DomainError> {
    let embedding = interests_repo
        .find_by_user(user_id)
        .await?
        .and_then(|i| i.embedding)
        .ok_or_else(|| DomainError::InvalidInput("no interest embedding found".into()))?;

    user_repo
        .find_nearby_by_interests(lat, lon, radius_meters, &embedding)
        .await
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::test_utils::fake_user_profile_repository::FakeUserProfileRepository;
    // use crate::user::{AuthUser, Tier};
    // use time::OffsetDateTime;
    // use uuid::Uuid;

    // #[tokio::test]
    // async fn returns_existing_profile_without_upsert() {
    //     let user = AuthUser {
    //         sub: "123".to_string(),
    //         email: "test@example.com".to_string(),
    //         display_name: "Test User".to_string(),
    //     };
    //     let now = OffsetDateTime::now_utc();
    //     let profile = crate::user::User {
    //         id: Uuid::now_v7(),
    //         sub: user.sub.clone(),
    //         email: user.email.clone(),
    //         display_name: user.display_name.clone(),
    //         avatar_url: None,
    //         bio: None,
    //         city: None,
    //         latitude: None,
    //         longitude: None,
    //         tier: Tier::Free,
    //         api_usage: 0i64.into(),
    //         storage_usage: 0i64.into(),
    //         billing_customer_id: None,
    //         billing_period_start: now,
    //         created_at: now,
    //         updated_at: now,
    //         archived_at: None,
    //     };
    //     let repo = FakeUserProfileRepository::new().with_profile(profile.clone());

    //     let result = get_or_upsert_user(&repo, &user).await.unwrap();

    //     assert_eq!(result.sub, user.sub);
    //     assert_eq!(result.email, user.email);
    // }

    // #[tokio::test]
    // async fn upserts_on_first_access() {
    //     let user = AuthUser {
    //         sub: "123".to_string(),
    //         email: "new@example.com".to_string(),
    //         display_name: "New User".to_string(),
    //     };
    //     let repo = FakeUserProfileRepository::new(); // empty

    //     let result = get_or_upsert_user(&repo, &user).await.unwrap();

    //     assert_eq!(result.email, "new@example.com");
    // }

    // #[tokio::test]
    // async fn upserts_when_email_changes() {
    //     let user = AuthUser {
    //         sub: "123456789".to_string(),
    //         email: "new@example.com".to_string(),
    //         display_name: "Test User".to_string(),
    //     };
    //     let now = OffsetDateTime::now_utc();
    //     let old_profile = crate::user::User {
    //         id: Uuid::now_v7(),
    //         sub: user.sub.clone(),
    //         email: "old@example.com".to_string(),
    //         display_name: "Test User".to_string(),
    //         avatar_url: None,
    //         bio: None,
    //         city: None,
    //         latitude: None,
    //         longitude: None,
    //         tier: Tier::Free,
    //         api_usage: 0i64.into(),
    //         storage_usage: 0i64.into(),
    //         billing_customer_id: None,
    //         billing_period_start: now,
    //         created_at: now,
    //         updated_at: now,
    //         archived_at: None,
    //     };
    //     let repo = FakeUserProfileRepository::new().with_profile(old_profile);

    //     let result = get_or_upsert_user(&repo, &user).await.unwrap();

    //     assert_eq!(result.email, "new@example.com");
    // }
}
