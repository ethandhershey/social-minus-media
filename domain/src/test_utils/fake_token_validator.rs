use crate::ports::Authenticator;
use crate::{error::DomainError, user::AuthUser};

#[derive(Clone)]
pub enum FakeTokenBehaviour {
    Valid {
        sub: String,
        email: String,
        display_name: String,
    },
    Invalid,
}

#[derive(Clone)]
pub struct FakeTokenValidator {
    behaviour: FakeTokenBehaviour,
}

impl FakeTokenValidator {
    pub fn valid(sub: &str, email: &str, display_name: &str) -> Self {
        Self {
            behaviour: FakeTokenBehaviour::Valid {
                sub: sub.to_string(),
                email: email.to_string(),
                display_name: display_name.to_string(),
            },
        }
    }

    pub fn invalid() -> Self {
        Self {
            behaviour: FakeTokenBehaviour::Invalid,
        }
    }
}

impl Authenticator for FakeTokenValidator {
    async fn validate(&self, _token: &str) -> Result<AuthUser, DomainError> {
        match &self.behaviour {
            FakeTokenBehaviour::Valid {
                sub,
                email,
                display_name,
            } => Ok(AuthUser {
                sub: sub.clone(),
                email: email.clone(),
                display_name: display_name.clone(),
            }),
            FakeTokenBehaviour::Invalid => Err(DomainError::Unauthorized),
        }
    }
}
