use crate::{
    error::DomainError,
    user::{AuthUser, User},
};

use crate::ports::UserRepository;

/// Gets the user profile, upserting on first access or when identity data changes.
pub async fn get_or_upsert_user(
    repo: &impl UserRepository,
    user: &AuthUser,
) -> Result<User, DomainError> {
    match repo.find_by_sub(&user.sub).await {
        Ok(profile) => {
            if profile.email == user.email && profile.display_name == user.display_name {
                Ok(profile)
            } else {
                repo.upsert(user).await
            }
        }
        Err(DomainError::NotFound) => repo.upsert(user).await,
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::fake_user_profile_repository::FakeUserProfileRepository;
    use crate::user::{AuthUser, Tier};
    use time::OffsetDateTime;
    use uuid::Uuid;

    #[tokio::test]
    async fn returns_existing_profile_without_upsert() {
        let user = AuthUser {
            sub: "123".to_string(),
            email: "test@example.com".to_string(),
            display_name: "Test User".to_string(),
        };
        let now = OffsetDateTime::now_utc();
        let profile = crate::user::User {
            id: Uuid::now_v7(),
            sub: user.sub.clone(),
            email: user.email.clone(),
            display_name: user.display_name.clone(),
            avatar_url: None,
            bio: None,
            city: None,
            latitude: None,
            longitude: None,
            tier: Tier::Free,
            api_usage: 0i64.into(),
            storage_usage: 0i64.into(),
            billing_customer_id: None,
            billing_period_start: now,
            created_at: now,
            updated_at: now,
            archived_at: None,
        };
        let repo = FakeUserProfileRepository::new().with_profile(profile.clone());

        let result = get_or_upsert_user(&repo, &user).await.unwrap();

        assert_eq!(result.sub, user.sub);
        assert_eq!(result.email, user.email);
    }

    #[tokio::test]
    async fn upserts_on_first_access() {
        let user = AuthUser {
            sub: "123".to_string(),
            email: "new@example.com".to_string(),
            display_name: "New User".to_string(),
        };
        let repo = FakeUserProfileRepository::new(); // empty

        let result = get_or_upsert_user(&repo, &user).await.unwrap();

        assert_eq!(result.email, "new@example.com");
    }

    #[tokio::test]
    async fn upserts_when_email_changes() {
        let user = AuthUser {
            sub: "123456789".to_string(),
            email: "new@example.com".to_string(),
            display_name: "Test User".to_string(),
        };
        let now = OffsetDateTime::now_utc();
        let old_profile = crate::user::User {
            id: Uuid::now_v7(),
            sub: user.sub.clone(),
            email: "old@example.com".to_string(),
            display_name: "Test User".to_string(),
            avatar_url: None,
            bio: None,
            city: None,
            latitude: None,
            longitude: None,
            tier: Tier::Free,
            api_usage: 0i64.into(),
            storage_usage: 0i64.into(),
            billing_customer_id: None,
            billing_period_start: now,
            created_at: now,
            updated_at: now,
            archived_at: None,
        };
        let repo = FakeUserProfileRepository::new().with_profile(old_profile);

        let result = get_or_upsert_user(&repo, &user).await.unwrap();

        assert_eq!(result.email, "new@example.com");
    }
}
