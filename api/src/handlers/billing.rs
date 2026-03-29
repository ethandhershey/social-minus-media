use axum::{
    Json, Router,
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
};
use domain::{
    error::DomainError,
    ports::{BillingService, UserRepository},
    user::{BillingEvent, Tier, User},
};
use serde::{Deserialize, Serialize};

use crate::{
    error::ApiError,
    state::{AppServices, AppState, BillingState, UserRepoState},
};

pub fn router<S: AppServices>() -> Router<AppState<S>> {
    Router::new()
        .route("/stripe/checkout", post(create_checkout))
        .route("/stripe/portal", post(create_portal))
        .route("/stripe/webhook", post(webhook))
}

// ── Request / response types ─────────────────────────────────────────────────

#[derive(Deserialize)]
struct CheckoutRequest {
    tier: TierParam,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum TierParam {
    Essential,
    Pro,
}

impl From<TierParam> for Tier {
    fn from(p: TierParam) -> Self {
        match p {
            TierParam::Essential => Tier::Essential,
            TierParam::Pro => Tier::Pro,
        }
    }
}

#[derive(Serialize)]
struct SessionUrlResponse {
    url: String,
}

// ── Handlers ─────────────────────────────────────────────────────────────────

async fn create_checkout<S: AppServices>(
    user: User,
    State(billing): State<BillingState<S>>,
    Json(body): Json<CheckoutRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let tier: Tier = body.tier.into();

    let url = billing
        .create_checkout_session(
            user.id,
            &user.email,
            &tier,
            user.billing_customer_id.as_deref(),
        )
        .await?;

    Ok(Json(SessionUrlResponse { url }))
}

async fn create_portal<S: AppServices>(
    user: User,
    State(billing): State<BillingState<S>>,
) -> Result<impl IntoResponse, ApiError> {
    let customer_id = user
        .billing_customer_id
        .as_deref()
        .ok_or(DomainError::NotFound)?;

    let url = billing.create_portal_session(customer_id).await?;

    Ok(Json(SessionUrlResponse { url }))
}

async fn webhook<S: AppServices>(
    State(repo): State<UserRepoState<S>>,
    State(billing): State<BillingState<S>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, ApiError> {
    let sig = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or(DomainError::Unauthorized)?;

    let event = billing.verify_webhook(&body, sig)?;

    match event {
        BillingEvent::CheckoutCompleted {
            user_id,
            customer_id,
            tier,
        } => {
            tracing::info!(%user_id, %customer_id, ?tier, "checkout completed");
            match repo
                .update_subscription(user_id, tier, Some(customer_id))
                .await
            {
                Ok(()) => {}
                Err(DomainError::NotFound) => {
                    tracing::warn!(%user_id, "checkout.session.completed for unknown user — ignoring");
                }
                Err(e) => return Err(e.into()),
            }
        }

        BillingEvent::SubscriptionUpdated { customer_id, tier } => {
            tracing::info!(%customer_id, ?tier, "subscription updated");
            match repo.find_by_billing_customer_id(&customer_id).await {
                Ok(user) => repo.update_subscription(user.id, tier, None).await?,
                Err(DomainError::NotFound) => {
                    tracing::warn!(%customer_id, "subscription.updated for unknown customer — ignoring");
                }
                Err(e) => return Err(e.into()),
            }
        }

        BillingEvent::SubscriptionDeleted { customer_id } => {
            tracing::info!(%customer_id, "subscription deleted — downgrading to Free");
            match repo.find_by_billing_customer_id(&customer_id).await {
                Ok(profile) => {
                    repo.update_subscription(profile.id, Tier::Free, None)
                        .await?
                }
                Err(DomainError::NotFound) => {
                    tracing::warn!(%customer_id, "subscription.deleted for unknown customer — ignoring");
                }
                Err(e) => return Err(e.into()),
            }
        }

        BillingEvent::Other => {
            // Unrecognised event — acknowledge to prevent Stripe retries
        }
    }

    Ok(StatusCode::OK)
}
