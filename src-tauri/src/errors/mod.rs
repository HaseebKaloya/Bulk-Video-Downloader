use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize, Clone)]
#[serde(tag = "category", content = "details")]
pub enum AppError {
    #[error("Validation error: {message}")]
    ValidationError {
        code: String,
        message: String,
        action: String,
    },

    #[error("Provider error: {message}")]
    ProviderError {
        code: String,
        message: String,
        action: String,
    },

    #[error("Authorization error: {message}")]
    AuthorizationError {
        code: String,
        message: String,
        action: String,
    },

    #[error("Unsupported source error: {message}")]
    UnsupportedSourceError {
        code: String,
        message: String,
        action: String,
    },

    #[error("Network error: {message}")]
    NetworkError {
        code: String,
        message: String,
        action: String,
        retryable: bool,
    },

    #[error("Rate limit error: {message}")]
    RateLimitError {
        code: String,
        message: String,
        action: String,
        retry_after_seconds: Option<u64>,
    },

    #[error("Disk space error: {message}")]
    DiskSpaceError {
        code: String,
        message: String,
        action: String,
    },

    #[error("Filesystem error: {message}")]
    FilesystemError {
        code: String,
        message: String,
        action: String,
    },

    #[error("Database error: {message}")]
    DatabaseError {
        code: String,
        message: String,
        action: String,
    },

    #[error("Media processing error: {message}")]
    MediaProcessingError {
        code: String,
        message: String,
        action: String,
    },

    #[error("Cancellation error: {message}")]
    CancellationError {
        code: String,
        message: String,
        action: String,
    },

    #[error("Recovery error: {message}")]
    RecoveryError {
        code: String,
        message: String,
        action: String,
    },

    #[error("Configuration error: {message}")]
    ConfigurationError {
        code: String,
        message: String,
        action: String,
    },

    #[error("Internal error: {message}")]
    InternalError {
        code: String,
        message: String,
        action: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErrorResponse {
    pub title: String,
    pub message: String,
    pub action: String,
    pub code: String,
    pub retryable: bool,
}

impl From<AppError> for ErrorResponse {
    fn from(err: AppError) -> Self {
        match err {
            AppError::ValidationError { code, message, action } => ErrorResponse {
                title: "Validation Error".into(),
                message,
                action,
                code,
                retryable: false,
            },
            AppError::ProviderError { code, message, action } => ErrorResponse {
                title: "Provider Error".into(),
                message,
                action,
                code,
                retryable: false,
            },
            AppError::AuthorizationError { code, message, action } => ErrorResponse {
                title: "Authorization Error".into(),
                message,
                action,
                code,
                retryable: false,
            },
            AppError::UnsupportedSourceError { code, message, action } => ErrorResponse {
                title: "Unsupported Source".into(),
                message,
                action,
                code,
                retryable: false,
            },
            AppError::NetworkError { code, message, action, retryable } => ErrorResponse {
                title: "Network Error".into(),
                message,
                action,
                code,
                retryable,
            },
            AppError::RateLimitError { code, message, action, .. } => ErrorResponse {
                title: "Rate Limit Exceeded".into(),
                message,
                action,
                code,
                retryable: true,
            },
            AppError::DiskSpaceError { code, message, action } => ErrorResponse {
                title: "Disk Space Error".into(),
                message,
                action,
                code,
                retryable: false,
            },
            AppError::FilesystemError { code, message, action } => ErrorResponse {
                title: "Filesystem Error".into(),
                message,
                action,
                code,
                retryable: false,
            },
            AppError::DatabaseError { code, message, action } => ErrorResponse {
                title: "Database Error".into(),
                message,
                action,
                code,
                retryable: false,
            },
            AppError::MediaProcessingError { code, message, action } => ErrorResponse {
                title: "Media Processing Error".into(),
                message,
                action,
                code,
                retryable: false,
            },
            AppError::CancellationError { code, message, action } => ErrorResponse {
                title: "Operation Cancelled".into(),
                message,
                action,
                code,
                retryable: false,
            },
            AppError::RecoveryError { code, message, action } => ErrorResponse {
                title: "Recovery Error".into(),
                message,
                action,
                code,
                retryable: false,
            },
            AppError::ConfigurationError { code, message, action } => ErrorResponse {
                title: "Configuration Error".into(),
                message,
                action,
                code,
                retryable: false,
            },
            AppError::InternalError { code, message, action } => ErrorResponse {
                title: "Internal Error".into(),
                message,
                action,
                code,
                retryable: false,
            },
        }
    }
}
