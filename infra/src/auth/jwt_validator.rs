use std::sync::Arc;

use domain::ports::Authenticator;
use domain::{error::DomainError, user::AuthUser};
use jsonwebtoken::{
    DecodingKey, Validation, decode, decode_header,
    jwk::{AlgorithmParameters, JwkSet},
};
use reqwest::IntoUrl;
use tokio::sync::RwLock;

use crate::http::AuthClient;

pub struct JwtTokenValidator {
    jwks: Arc<RwLock<JwkSet>>,
    issuer: String,
    client_id: String,
}

impl JwtTokenValidator {
    pub async fn new(
        issuer: impl Into<String>,
        client_id: impl Into<String>,
        jwks_url: impl Into<String>,
        http_client: AuthClient,
    ) -> Result<Self, anyhow::Error> {
        let jwks_url = jwks_url.into();
        let jwks = fetch_jwks(jwks_url.clone(), &http_client).await;
        let jwks = if cfg!(debug_assertions) {
            jwks.unwrap_or_else(|e| {
                tracing::warn!(error = %e, "failed to fetch JWKS, starting with empty keyset");
                JwkSet { keys: vec![] }
            })
        } else {
            jwks?
        };
        let shared = Arc::new(RwLock::new(jwks));

        spawn_jwks_refresh(jwks_url, Arc::clone(&shared), http_client.clone());

        Ok(Self {
            jwks: shared,
            issuer: issuer.into(),
            client_id: client_id.into(),
        })
    }
}

impl Authenticator for JwtTokenValidator {
    async fn validate(&self, token: &str) -> Result<AuthUser, DomainError> {
        let header = decode_header(token).map_err(|e| DomainError::InvalidInput(e.to_string()))?;

        let kid = header.kid.ok_or_else(|| DomainError::Unauthorized)?;

        let decoding_key = {
            let jwks_guard = self.jwks.read().await;
            let jwk = jwks_guard.find(&kid).ok_or(DomainError::Unauthorized)?;

            match &jwk.algorithm {
                AlgorithmParameters::RSA(rsa) => DecodingKey::from_rsa_components(&rsa.n, &rsa.e)
                    .map_err(|_| DomainError::Unauthorized)?,
                AlgorithmParameters::EllipticCurve(ec) => {
                    DecodingKey::from_ec_components(&ec.x, &ec.y)
                        .map_err(|_| DomainError::Unauthorized)?
                }
                _ => return Err(DomainError::Unauthorized),
            }
        };

        let mut validation = Validation::new(header.alg);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.client_id]);
        validation.set_required_spec_claims(&["exp", "iss", "aud"]);

        #[derive(serde::Deserialize)]
        struct ZitadelClaims {
            sub: String,
            email: String,
            name: String,
        }

        let claims = decode::<ZitadelClaims>(token, &decoding_key, &validation)
            .map_err(|e| {
                tracing::warn!("error decoding jwt: {e}");
                DomainError::Unauthorized
            })?
            .claims;

        Ok(AuthUser {
            sub: claims.sub,
            email: claims.email,
            display_name: claims.name,
        })
    }
}

async fn fetch_jwks(url: impl IntoUrl, client: &AuthClient) -> anyhow::Result<JwkSet> {
    let jwks = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json::<JwkSet>()
        .await?;
    Ok(jwks)
}

fn spawn_jwks_refresh(
    jwks_url: impl Into<String>,
    shared: Arc<RwLock<JwkSet>>,
    client: AuthClient,
) {
    let jwks_url = jwks_url.into();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
            match fetch_jwks(&jwks_url, &client).await {
                Ok(fresh) => {
                    *shared.write().await = fresh;
                    tracing::info!("JWKS refreshed");
                }
                Err(e) => {
                    tracing::error!("JWKS refresh failed, keeping existing keys: {e}");
                }
            }
        }
    });
}
