use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use domain::error::DomainError;

pub struct ApiError(pub DomainError);

impl From<DomainError> for ApiError {
    fn from(e: DomainError) -> Self {
        Self(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.0 {
            DomainError::Unauthorized => StatusCode::UNAUTHORIZED,
            DomainError::InsufficientTier => StatusCode::FORBIDDEN,
            DomainError::NotFound => StatusCode::NOT_FOUND,
            DomainError::AiServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            DomainError::BillingServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            DomainError::MailServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            DomainError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            DomainError::LimitExceeded => StatusCode::FORBIDDEN,
            DomainError::Parse => StatusCode::INTERNAL_SERVER_ERROR,
            DomainError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        status.into_response()
    }
}
