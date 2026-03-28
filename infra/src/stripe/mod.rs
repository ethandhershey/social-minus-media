use domain::{
    error::DomainError,
    ports::BillingService,
    user::{BillingEvent, Tier},
};
use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, SecretString};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use uuid::Uuid;

use crate::http::HttpClient;

// Reject webhooks whose timestamp deviates from now by more than this.
// Prevents replayed payloads from being accepted indefinitely.
const WEBHOOK_TIMESTAMP_TOLERANCE_SECS: i64 = 300;

pub struct StripeClient {
    http: reqwest::Client,
    secret_key: SecretString,
    webhook_secret: SecretString,
    essential_price_id: String,
    pro_price_id: String,
    success_url: String,
    cancel_url: String,
    portal_return_url: String,
}

impl StripeClient {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        http: HttpClient,
        secret_key: SecretString,
        webhook_secret: SecretString,
        essential_price_id: String,
        pro_price_id: String,
        success_url: String,
        cancel_url: String,
        portal_return_url: String,
    ) -> Self {
        Self {
            http: http.into(),
            secret_key,
            webhook_secret,
            essential_price_id,
            pro_price_id,
            success_url,
            cancel_url,
            portal_return_url,
        }
    }

    fn price_id_for_tier<'a>(&'a self, tier: &Tier) -> Option<&'a str> {
        match tier {
            Tier::Essential => Some(&self.essential_price_id),
            Tier::Pro => Some(&self.pro_price_id),
            _ => None,
        }
    }

    fn price_id_to_tier(&self, price_id: &str) -> Option<Tier> {
        if price_id == self.essential_price_id {
            Some(Tier::Essential)
        } else if price_id == self.pro_price_id {
            Some(Tier::Pro)
        } else {
            None
        }
    }
}

impl BillingService for StripeClient {
    async fn create_checkout_session(
        &self,
        user_id: Uuid,
        email: &str,
        tier: &Tier,
        existing_customer_id: Option<&str>,
    ) -> Result<String, DomainError> {
        let price_id = self.price_id_for_tier(tier).ok_or_else(|| {
            tracing::warn!(?tier, "checkout attempted for non-self-serve tier");
            DomainError::InvalidInput("tier is not available for self-serve checkout".into())
        })?;
        let user_id_str = user_id.to_string();

        // Keyed on user + tier so that double-clicks or retries within Stripe's
        // 30-minute idempotency window return the same session rather than
        // creating duplicates.
        let idempotency_key = format!("checkout-{user_id_str}-{}", tier_to_metadata_name(tier));

        let mut params: Vec<(&str, &str)> = vec![
            ("mode", "subscription"),
            ("line_items[0][price]", price_id),
            ("line_items[0][quantity]", "1"),
            ("client_reference_id", &user_id_str),
            ("success_url", &self.success_url),
            ("cancel_url", &self.cancel_url),
            ("metadata[tier]", tier_to_metadata_name(tier)),
        ];

        if let Some(cid) = existing_customer_id {
            params.push(("customer", cid));
        } else {
            params.push(("customer_email", email));
        }

        let resp = self
            .http
            .post("https://api.stripe.com/v1/checkout/sessions")
            .basic_auth(self.secret_key.expose_secret(), Some(""))
            .header("Stripe-Version", "2024-04-10")
            .header("Idempotency-Key", &idempotency_key)
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Stripe checkout request failed: {e}");
                DomainError::BillingServiceUnavailable
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            tracing::error!("Stripe checkout error {status}: {body}");
            return Err(DomainError::BillingServiceUnavailable);
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|_| DomainError::BillingServiceUnavailable)?;

        body["url"]
            .as_str()
            .map(|s| s.to_owned())
            .ok_or(DomainError::BillingServiceUnavailable)
    }

    async fn create_portal_session(&self, customer_id: &str) -> Result<String, DomainError> {
        // Keyed on customer ID so retries within the idempotency window return
        // the same portal session rather than creating duplicates.
        let idempotency_key = format!("portal-{customer_id}");

        let params = [
            ("customer", customer_id),
            ("return_url", self.portal_return_url.as_str()),
        ];

        let resp = self
            .http
            .post("https://api.stripe.com/v1/billing_portal/sessions")
            .basic_auth(self.secret_key.expose_secret(), Some(""))
            .header("Stripe-Version", "2024-04-10")
            .header("Idempotency-Key", &idempotency_key)
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Stripe portal request failed: {e}");
                DomainError::BillingServiceUnavailable
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            tracing::error!("Stripe portal error {status}: {body}");
            return Err(DomainError::BillingServiceUnavailable);
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|_| DomainError::BillingServiceUnavailable)?;

        body["url"]
            .as_str()
            .map(|s| s.to_owned())
            .ok_or(DomainError::BillingServiceUnavailable)
    }

    fn verify_webhook(
        &self,
        payload: &[u8],
        sig_header: &str,
    ) -> Result<BillingEvent, DomainError> {
        // Parse Stripe-Signature header: t=<timestamp>,v1=<sig>[,v1=<sig>...]
        let mut timestamp: Option<&str> = None;
        let mut signatures: Vec<&str> = Vec::new();

        for part in sig_header.split(',') {
            if let Some(ts) = part.strip_prefix("t=") {
                timestamp = Some(ts);
            } else if let Some(sig) = part.strip_prefix("v1=") {
                signatures.push(sig);
            }
        }

        let ts = timestamp.ok_or(DomainError::Unauthorized)?;

        // Reject payloads outside the tolerance window to prevent replay attacks.
        let ts_secs: i64 = ts.parse().map_err(|_| {
            tracing::warn!("Stripe webhook has non-numeric timestamp");
            DomainError::Unauthorized
        })?;
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        if (now_secs - ts_secs).abs() > WEBHOOK_TIMESTAMP_TOLERANCE_SECS {
            tracing::warn!(
                ts_secs,
                now_secs,
                "Stripe webhook timestamp outside tolerance window — possible replay attack"
            );
            return Err(DomainError::Unauthorized);
        }

        let body_str = std::str::from_utf8(payload).map_err(|_| DomainError::Unauthorized)?;
        let signed_payload = format!("{ts}.{body_str}");

        // A key error here indicates a misconfigured WEBHOOK_SECRET, not a bad request.
        let mut mac =
            Hmac::<Sha256>::new_from_slice(self.webhook_secret.expose_secret().as_bytes())
                .map_err(|e| {
                    tracing::error!(
                        "Stripe webhook HMAC key invalid — check WEBHOOK_SECRET config: {e}"
                    );
                    DomainError::BillingServiceUnavailable
                })?;
        mac.update(signed_payload.as_bytes());
        let mac_bytes = mac.finalize().into_bytes();

        // Decode each v1 hex signature and compare in constant time to prevent
        // timing side-channel attacks.
        let verified = signatures.iter().any(|sig| {
            hex::decode(sig)
                .map(|decoded| mac_bytes.ct_eq(decoded.as_slice()).into())
                .unwrap_or(false)
        });

        if !verified {
            tracing::warn!("Stripe webhook signature mismatch");
            return Err(DomainError::Unauthorized);
        }

        let event: serde_json::Value = serde_json::from_slice(payload)
            .map_err(|_| DomainError::InvalidInput("malformed webhook json".into()))?;

        let result = parse_event(&event, self)?;

        tracing::info!(
            event_type = event["type"].as_str().unwrap_or("unknown"),
            "Stripe webhook processed successfully"
        );

        Ok(result)
    }
}

fn parse_event(
    event: &serde_json::Value,
    client: &StripeClient,
) -> Result<BillingEvent, DomainError> {
    // Ignore test-mode events to prevent them from modifying production data.
    if event["livemode"].as_bool() == Some(false) {
        tracing::warn!("ignoring Stripe test-mode webhook");
        return Ok(BillingEvent::Other);
    }

    let event_type = event["type"].as_str().unwrap_or("");
    let obj = &event["data"]["object"];

    match event_type {
        "checkout.session.completed" => {
            // Only process sessions where payment actually succeeded.
            let payment_status = obj["payment_status"].as_str().unwrap_or("");
            if !matches!(payment_status, "paid" | "no_payment_required") {
                tracing::warn!(
                    payment_status,
                    "checkout.session.completed with unexpected payment_status — ignoring"
                );
                return Ok(BillingEvent::Other);
            }

            let user_id_str = obj["client_reference_id"]
                .as_str()
                .ok_or_else(|| DomainError::InvalidInput("missing client_reference_id".into()))?;
            let user_id = user_id_str.parse::<Uuid>().map_err(|_| {
                DomainError::InvalidInput("invalid uuid in client_reference_id".into())
            })?;
            let customer_id = obj["customer"]
                .as_str()
                .ok_or_else(|| DomainError::InvalidInput("missing customer".into()))?
                .to_owned();

            // metadata["tier"] is written by us at session creation time, and the
            // verified HMAC above guarantees Stripe delivered it unmodified, so it
            // is safe to use as the source of truth for the purchased tier.
            let tier_name = obj["metadata"]["tier"].as_str().unwrap_or("");
            let tier = tier_from_metadata_name(tier_name)?;

            Ok(BillingEvent::CheckoutCompleted {
                user_id,
                customer_id,
                tier,
            })
        }

        "customer.subscription.updated" => {
            let customer_id = obj["customer"]
                .as_str()
                .ok_or_else(|| DomainError::InvalidInput("missing customer".into()))?
                .to_owned();

            // Treat non-active/trialing subscriptions as effectively deleted
            let status = obj["status"].as_str().unwrap_or("");
            if !matches!(status, "active" | "trialing") {
                tracing::info!(
                    customer_id,
                    status,
                    "Treating non-active subscription as deleted"
                );
                return Ok(BillingEvent::SubscriptionDeleted { customer_id });
            }

            let price_id = obj["items"]["data"][0]["price"]["id"]
                .as_str()
                .ok_or_else(|| {
                    tracing::error!(customer_id, "subscription.updated has no items[0].price.id");
                    DomainError::InvalidInput("missing price id".into())
                })?;

            let tier = client.price_id_to_tier(price_id).ok_or_else(|| {
                tracing::error!(price_id, "unrecognised price id in subscription.updated");
                DomainError::InvalidInput(format!("unknown price id: {price_id}"))
            })?;

            Ok(BillingEvent::SubscriptionUpdated { customer_id, tier })
        }

        "customer.subscription.deleted" => {
            let customer_id = obj["customer"]
                .as_str()
                .ok_or_else(|| DomainError::InvalidInput("missing customer".into()))?
                .to_owned();
            Ok(BillingEvent::SubscriptionDeleted { customer_id })
        }

        other => {
            tracing::debug!(event_type = other, "ignoring unhandled Stripe event type");
            Ok(BillingEvent::Other)
        }
    }
}

fn tier_to_metadata_name(tier: &Tier) -> &'static str {
    match tier {
        Tier::Essential => "essential",
        Tier::Pro => "pro",
        // price_id_for_tier returns None for any other tier, so checkout is
        // rejected before this point is ever reached.
        _ => unreachable!("tier_to_metadata_name called with non-self-serve tier"),
    }
}

fn tier_from_metadata_name(name: &str) -> Result<Tier, DomainError> {
    match name {
        "essential" => Ok(Tier::Essential),
        "pro" => Ok(Tier::Pro),
        _ => Err(DomainError::InvalidInput(format!(
            "unknown tier in metadata: {name}"
        ))),
    }
}
