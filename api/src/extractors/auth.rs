use axum::{
    RequestPartsExt,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use domain::ports::Authenticator;
use domain::{error::DomainError, user::AuthUser};

use crate::{
    error::ApiError,
    state::{AppServices, AppState, AuthenticatorState},
};

impl<S: AppServices> FromRequestParts<AppState<S>> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState<S>,
    ) -> Result<Self, Self::Rejection> {
        // Return cached result if already extracted
        if let Some(auth) = parts.extensions.get::<AuthUser>() {
            return Ok(auth.clone());
        }

        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| DomainError::Unauthorized)?;

        let validator = AuthenticatorState::from_ref(state);
        let auth = validator.validate(bearer.token()).await?;

        // Cache it for subsequent extractors
        parts.extensions.insert(auth.clone());
        Ok(auth)
    }
}
