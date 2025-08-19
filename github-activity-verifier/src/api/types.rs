use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct VerificationRequest {
    pub github_username: String,
    pub verification_type: VerificationType,
    pub threshold: Option<u32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum VerificationType {
    YearlyCommits,
    ConsecutiveDays,
    TotalStars,
    PublicRepos,
}

impl VerificationType {
    pub fn default_threshold(&self) -> u32 {
        match self {
            VerificationType::YearlyCommits => 365,
            VerificationType::ConsecutiveDays => 100,
            VerificationType::TotalStars => 1000,
            VerificationType::PublicRepos => 10,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct VerificationResult {
    pub username: String,
    pub verification_type: VerificationType,
    pub threshold: u32,
    pub meets_criteria: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attestation_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attestation_claims: Option<serde_json::Value>,
    pub verified_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_hash: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
    pub error_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}
