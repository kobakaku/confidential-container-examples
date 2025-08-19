use actix_web::{http::StatusCode, HttpResponse};
use thiserror::Error;

use crate::api::types::ApiError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("GitHub API error: {0}")]
    GitHub(#[from] crate::github::GitHubError),

    #[error("MAA error: {0}")]
    Maa(#[from] crate::attestation::MAAError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<AppError> for HttpResponse {
    fn from(error: AppError) -> Self {
        let (status, error_code, message) = match error {
            AppError::GitHub(ref gh_err) => match gh_err {
                crate::github::GitHubError::UserNotFound(username) => (
                    StatusCode::NOT_FOUND,
                    "USER_NOT_FOUND",
                    format!("GitHub user '{}' not found", username),
                ),
                crate::github::GitHubError::RateLimit => (
                    StatusCode::TOO_MANY_REQUESTS,
                    "RATE_LIMIT_EXCEEDED",
                    "GitHub API rate limit exceeded. Please try again later.".to_string(),
                ),
                crate::github::GitHubError::ApiError { status, message } => (
                    StatusCode::BAD_GATEWAY,
                    "GITHUB_API_ERROR",
                    format!("GitHub API error {}: {}", status, message),
                ),
                crate::github::GitHubError::Network(_) => (
                    StatusCode::BAD_GATEWAY,
                    "NETWORK_ERROR",
                    "Failed to connect to GitHub API".to_string(),
                ),
                crate::github::GitHubError::Json(_) => (
                    StatusCode::BAD_GATEWAY,
                    "JSON_PARSE_ERROR",
                    "Failed to parse GitHub API response".to_string(),
                ),
            },
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "An unexpected error occurred".to_string(),
            ),
        };

        HttpResponse::build(status).json(ApiError {
            error: message,
            error_code: error_code.to_string(),
            details: None,
        })
    }
}
