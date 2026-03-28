use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("unauthorized")]
    Unauthorized,

    #[error("insufficient tier")]
    InsufficientTier,

    #[error("not found")]
    NotFound,

    #[error("AI service unavailable")]
    AiServiceUnavailable,

    #[error("billing service unavailable")]
    BillingServiceUnavailable,

    #[error("mail service unavailable")]
    MailServiceUnavailable,

    #[error("limit exceeded")]
    LimitExceeded,

    #[error("couldn't parse")]
    Parse,

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("internal error")]
    Internal(#[from] anyhow::Error),
}

pub type DomainResult<T> = Result<T, DomainError>;
