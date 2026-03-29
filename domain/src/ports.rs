use crate::{
    error::DomainError,
    event::Event,
    product::Product,
    rsvp::Rsvp,
    user::{AuthUser, BillingEvent, Tier, User},
    user_interests::UserInterests,
};
use uuid::Uuid;

#[trait_variant::make(Send)]
pub trait Authenticator: Send + Sync {
    async fn validate(&self, token: &str) -> Result<AuthUser, DomainError>;
}

#[trait_variant::make(Send)]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, user_id: Uuid) -> Result<User, DomainError>;
    async fn find_by_sub(&self, user_sub: &str) -> Result<User, DomainError>;
    async fn upsert(&self, user: &AuthUser) -> Result<User, DomainError>;
    async fn find_by_billing_customer_id(&self, customer_id: &str) -> Result<User, DomainError>;
    async fn find_nearby(&self, lat: f64, lon: f64, radius_meters: f64) -> Result<Vec<User>, DomainError>;
    async fn update_subscription(
        &self,
        user_id: Uuid,
        tier: Tier,
        stripe_customer_id: Option<String>,
    ) -> Result<(), DomainError>;
}

#[trait_variant::make(Send)]
pub trait BillingService: Send + Sync {
    /// Create a Checkout Session for the given tier and return the redirect URL.
    async fn create_checkout_session(
        &self,
        user_id: Uuid,
        email: &str,
        tier: &Tier,
        existing_customer_id: Option<&str>,
    ) -> Result<String, DomainError>;

    /// Create a Customer Portal Session and return the redirect URL.
    async fn create_portal_session(&self, customer_id: &str) -> Result<String, DomainError>;

    /// Verify a webhook signature and parse the event payload.
    fn verify_webhook(&self, payload: &[u8], sig_header: &str)
    -> Result<BillingEvent, DomainError>;
}

#[trait_variant::make(Send)]
pub trait LlmService: Send + Sync {
    fn available_models(&self) -> Vec<String>;
    async fn get_simple_response(
        &self,
        model: &str,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<String, DomainError>;
    async fn get_structured_response<T>(
        &self,
        model: &str,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<T, DomainError>
    where
        T: schemars::JsonSchema + serde::de::DeserializeOwned;
    async fn embed(&self, model: &str, input: &str) -> Result<Vec<f32>, DomainError>;
}

#[trait_variant::make(Send)]
pub trait ProductRepository: Send + Sync {
    async fn find(&self, product_id: Uuid) -> Result<Product, DomainError>;
    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Product>, DomainError>;
    async fn upsert(&self, product: &Product) -> Result<(), DomainError>;
    async fn delete(&self, product_id: Uuid) -> Result<(), DomainError>;
}

#[trait_variant::make(Send)]
pub trait MailService: Send + Sync {
    async fn send(
        &self,
        to_email: &str,
        to_name: &str,
        subject: &str,
        html_body: &str,
    ) -> Result<(), DomainError>;
}

#[trait_variant::make(Send)]
pub trait EventRepository: Send + Sync {
    async fn find(&self, event_id: Uuid) -> Result<Event, DomainError>;
    async fn find_by_host(&self, host_id: Uuid) -> Result<Vec<Event>, DomainError>;
    async fn find_upcoming(&self, limit: i64) -> Result<Vec<Event>, DomainError>;
    async fn upsert(&self, event: &Event) -> Result<Event, DomainError>;
    async fn delete(&self, event_id: Uuid) -> Result<(), DomainError>;
}

#[trait_variant::make(Send)]
pub trait RsvpRepository: Send + Sync {
    async fn find(&self, user_id: Uuid, event_id: Uuid) -> Result<Option<Rsvp>, DomainError>;
    async fn find_by_event(&self, event_id: Uuid) -> Result<Vec<Rsvp>, DomainError>;
    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Rsvp>, DomainError>;
    async fn upsert(&self, rsvp: &Rsvp) -> Result<Rsvp, DomainError>;
    async fn delete(&self, user_id: Uuid, event_id: Uuid) -> Result<(), DomainError>;
}

#[trait_variant::make(Send)]
pub trait UserInterestsRepository: Send + Sync {
    async fn find_by_user(&self, user_id: Uuid) -> Result<Option<UserInterests>, DomainError>;
    async fn upsert(&self, interests: &UserInterests) -> Result<UserInterests, DomainError>;
}

#[trait_variant::make(Send)]
pub trait EntitlementService: Send + Sync {
    fn check_tier(&self, user: &User, required: &Tier) -> Result<(), DomainError>;
    fn check_api_limit(&self, user: &User, usage: u32) -> Result<(), DomainError>;
    fn check_project_limit(&self, user: &User, count: u32) -> Result<(), DomainError>;
    fn check_storage_limit(&self, user: &User, used_gb: u32) -> Result<(), DomainError>;
    fn can_export(&self, user: &User) -> Result<(), DomainError>;
}
