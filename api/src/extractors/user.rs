use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use domain::user::AuthUser;
use domain::user::User;

use crate::{
    error::ApiError,
    state::{AppServices, AppState, UserRepoState},
};

impl<S: AppServices> FromRequestParts<AppState<S>> for User {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState<S>,
    ) -> Result<Self, Self::Rejection> {
        let auth = AuthUser::from_request_parts(parts, state).await?;
        let repo = UserRepoState::from_ref(state);
        Ok(domain::user::crud::get_or_upsert_user(repo.as_ref(), &auth).await?)
    }
}
