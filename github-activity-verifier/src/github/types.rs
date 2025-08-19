use chrono::{DateTime, Utc};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitHubError {
    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("API request failed: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub actor: GitHubActor,
    pub repo: GitHubRepo,
    pub created_at: DateTime<Utc>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubActor {
    pub id: u64,
    pub login: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRepo {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
    pub public_repos: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubUserRepo {
    pub id: u64,
    pub name: String,
    pub stargazers_count: u32,
    pub created_at: DateTime<Utc>,
}
