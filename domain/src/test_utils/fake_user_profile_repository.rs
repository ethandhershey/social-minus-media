use crate::ports::UserRepository;
use crate::{
    error::DomainError,
    user::{AuthUser, Tier, User},
};
use std::collections::HashMap;
use std::sync::RwLock;
use time::OffsetDateTime;
use uuid::Uuid;

pub struct FakeUserProfileRepository {
    profiles: RwLock<HashMap<Uuid, User>>,
}

impl FakeUserProfileRepository {
    pub fn new() -> Self {
        Self {
            profiles: RwLock::new(HashMap::new()),
        }
    }

    pub fn with_profile(self, profile: User) -> Self {
        self.profiles.write().unwrap().insert(profile.id, profile);
        self
    }
}

impl UserRepository for FakeUserProfileRepository {
    async fn find_by_id(&self, user_id: Uuid) -> Result<User, DomainError> {
        self.profiles
            .read()
            .unwrap()
            .get(&user_id)
            .cloned()
            .ok_or(DomainError::NotFound)
    }

    async fn find_by_sub(&self, sub: &str) -> Result<User, DomainError> {
        self.profiles
            .read()
            .unwrap()
            .values()
            .find(|p| p.sub == sub)
            .cloned()
            .ok_or(DomainError::NotFound)
    }

    async fn upsert(&self, user: &AuthUser) -> Result<User, DomainError> {
        let now = OffsetDateTime::now_utc();
        let mut profiles = self.profiles.write().unwrap();

        // keep existing id if user already exists
        let id = profiles
            .values()
            .find(|p| p.sub == user.sub)
            .map(|p| p.id)
            .unwrap_or_else(Uuid::now_v7);

        let profile = User {
            id,
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

        profiles.insert(profile.id, profile.clone());
        Ok(profile)
    }

    async fn find_nearby_by_interests(
        &self,
        _lat: f64,
        _lon: f64,
        _radius_meters: f64,
        _embedding: &[f32],
    ) -> Result<Vec<User>, DomainError> {
        Ok(vec![])
    }

    async fn find_nearby(&self, lat: f64, lon: f64, _radius_meters: f64) -> Result<Vec<User>, DomainError> {
        let profiles = self.profiles.read().unwrap();
        Ok(profiles
            .values()
            .filter(|p| p.latitude.is_some() && p.longitude.is_some())
            .cloned()
            .collect())
    }

    async fn find_by_billing_customer_id(&self, customer_id: &str) -> Result<User, DomainError> {
        self.profiles
            .read()
            .unwrap()
            .values()
            .find(|p| p.billing_customer_id.as_deref() == Some(customer_id))
            .cloned()
            .ok_or(DomainError::NotFound)
    }

    async fn update_subscription(
        &self,
        user_id: Uuid,
        tier: Tier,
        stripe_customer_id: Option<String>,
    ) -> Result<(), DomainError> {
        let mut profiles = self.profiles.write().unwrap();
        match profiles.get_mut(&user_id) {
            Some(profile) => {
                profile.tier = tier;
                if stripe_customer_id.is_some() {
                    profile.billing_customer_id = stripe_customer_id;
                }
                Ok(())
            }
            None => Err(DomainError::NotFound),
        }
    }
}
