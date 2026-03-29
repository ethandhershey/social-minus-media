use axum::extract::FromRef;
use derive_more::Deref;
use domain::ports::{
    Authenticator, BillingService, EntitlementService, EventRepository, LlmService, MailService,
    ProductRepository, RsvpRepository, UserInterestsRepository, UserRepository,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct InterestsConfig {
    pub summary_model: String,
    pub embed_model: String,
}

// ── Service bundle ────────────────────────────────────────────────────────────

pub trait AppServices: Clone + Send + Sync + 'static {
    type Auth: Authenticator;
    type UserRepo: UserRepository;
    type ProductRepo: ProductRepository;
    type EventRepo: EventRepository;
    type RsvpRepo: RsvpRepository;
    type UserInterestsRepo: UserInterestsRepository;
    type Llm: LlmService;
    type Billing: BillingService;
    type Mail: MailService;
    type Entitlement: EntitlementService;
}

// ── Named newtypes + FromRef via macro ────────────────────────────────────────

macro_rules! state_wrapper {
    ($name:ident, $assoc:ident, $field:ident) => {
        #[derive(Deref)]
        pub(crate) struct $name<S: AppServices>(Arc<S::$assoc>);

        impl<S: AppServices> $name<S> {
            pub fn new(inner: S::$assoc) -> Self {
                Self(Arc::new(inner))
            }
        }

        impl<S: AppServices> Clone for $name<S> {
            fn clone(&self) -> Self {
                Self(Arc::clone(&self.0))
            }
        }

        impl<S: AppServices> FromRef<AppState<S>> for $name<S> {
            fn from_ref(state: &AppState<S>) -> Self {
                state.$field.clone()
            }
        }
    };
}

state_wrapper!(AuthenticatorState, Auth, auth);
state_wrapper!(UserRepoState, UserRepo, user_repo);
state_wrapper!(ProductRepoState, ProductRepo, product_repo);
state_wrapper!(EventRepoState, EventRepo, event_repo);
state_wrapper!(RsvpRepoState, RsvpRepo, rsvp_repo);
state_wrapper!(
    UserInterestsRepoState,
    UserInterestsRepo,
    user_interests_repo
);
state_wrapper!(LlmState, Llm, llm);
state_wrapper!(BillingState, Billing, billing);
state_wrapper!(MailState, Mail, mail);
state_wrapper!(EntitlementState, Entitlement, entitlement);

// ── AppState ──────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct AppState<S: AppServices> {
    auth: AuthenticatorState<S>,
    user_repo: UserRepoState<S>,
    product_repo: ProductRepoState<S>,
    event_repo: EventRepoState<S>,
    rsvp_repo: RsvpRepoState<S>,
    user_interests_repo: UserInterestsRepoState<S>,
    llm: LlmState<S>,
    billing: BillingState<S>,
    mail: MailState<S>,
    entitlement: EntitlementState<S>,
    interests_config: Arc<InterestsConfig>,
}

impl<S: AppServices> FromRef<AppState<S>> for Arc<InterestsConfig> {
    fn from_ref(state: &AppState<S>) -> Self {
        Arc::clone(&state.interests_config)
    }
}

impl<S: AppServices> AppState<S> {
    pub fn new(
        auth: S::Auth,
        user_repo: S::UserRepo,
        product_repo: S::ProductRepo,
        event_repo: S::EventRepo,
        rsvp_repo: S::RsvpRepo,
        user_interests_repo: S::UserInterestsRepo,
        ai: S::Llm,
        billing: S::Billing,
        mail: S::Mail,
        entitlement: S::Entitlement,
        interests_config: InterestsConfig,
    ) -> Self {
        Self {
            auth: AuthenticatorState::new(auth),
            user_repo: UserRepoState::new(user_repo),
            product_repo: ProductRepoState::new(product_repo),
            event_repo: EventRepoState::new(event_repo),
            rsvp_repo: RsvpRepoState::new(rsvp_repo),
            user_interests_repo: UserInterestsRepoState::new(user_interests_repo),
            llm: LlmState::new(ai),
            billing: BillingState::new(billing),
            mail: MailState::new(mail),
            entitlement: EntitlementState::new(entitlement),
            interests_config: Arc::new(interests_config),
        }
    }
}
