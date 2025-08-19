use base64::{engine::general_purpose, Engine as _};
use reqwest::Client;
use serde_json;
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, error, info};

#[derive(Error, Debug)]
pub enum MAAError {
    #[error("SKR Sidecar not available: {0}")]
    SidecarUnavailable(String),

    #[error("MAA endpoint not configured")]
    EndpointNotConfigured,

    #[error("JWT token invalid: {0}")]
    InvalidToken(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Base64 decoding error: {0}")]
    Base64Error(#[from] base64::DecodeError),

    #[error("Certificate generation error: {0}")]
    CertificateError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub struct MAAClient {
    pub endpoint: String,
    client: Client,
    skr_endpoint: String,
}

impl MAAClient {
    pub fn new(maa_endpoint: String) -> Self {
        let skr_port = std::env::var("SKR_PORT").unwrap_or_else(|_| "8080".to_string());
        let skr_endpoint = format!("http://localhost:{}/attest/maa", skr_port);

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            endpoint: maa_endpoint,
            client,
            skr_endpoint,
        }
    }

    pub async fn get_attestation_token(&self, runtime_data: &str) -> Result<String, MAAError> {
        if self.endpoint.is_empty() {
            return Err(MAAError::EndpointNotConfigured);
        }

        info!("Requesting MAA attestation token");

        let runtime_data_json = serde_json::json!({
            "proof_data_hash": runtime_data
        });

        let runtime_data_base64 =
            general_purpose::STANDARD.encode(runtime_data_json.to_string().as_bytes());

        let maa_request = serde_json::json!({
            "maa_endpoint": self.endpoint,
            "runtime_data": runtime_data_base64
        });

        debug!("Calling SKR Sidecar at: {}", self.skr_endpoint);
        debug!("MAA Request payload: {}", maa_request);

        // Call SKR Sidecar
        let response = self
            .client
            .post(&self.skr_endpoint)
            .header("Content-Type", "application/json")
            .json(&maa_request)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to connect to SKR Sidecar: {}", e);
                MAAError::SidecarUnavailable(format!(
                    "Failed to connect to SKR Sidecar at {}: {}",
                    self.skr_endpoint, e
                ))
            })?;

        let status = response.status();
        let response_text = response.text().await?;

        debug!("SKR Sidecar response status: {}", status);
        debug!("SKR Sidecar response body: {}", response_text);

        if !status.is_success() {
            return Err(MAAError::SidecarUnavailable(format!(
                "SKR Sidecar returned error {}: {}",
                status, response_text
            )));
        }

        // Parse response
        self.parse_attestation_response(&response_text)
    }

    fn parse_attestation_response(&self, response_text: &str) -> Result<String, MAAError> {
        // Try to parse as JSON first
        if let Ok(json_response) = serde_json::from_str::<serde_json::Value>(response_text) {
            // Check for token field
            if let Some(token) = json_response.get("token") {
                if let Some(token_str) = token.as_str() {
                    return Ok(token_str.to_string());
                }
            }

            // Check for attestation_token field
            if let Some(token) = json_response.get("attestation_token") {
                if let Some(token_str) = token.as_str() {
                    return Ok(token_str.to_string());
                }
            }

            return Err(MAAError::InvalidToken(format!(
                "SKR response contains JSON but no recognizable token field: {}",
                response_text
            )));
        }

        // If not JSON, treat entire response as token
        let token = response_text.trim();
        if token.is_empty() {
            return Err(MAAError::InvalidToken(
                "Empty token received from SKR Sidecar".to_string(),
            ));
        }

        // Basic JWT format validation
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(MAAError::InvalidToken(format!(
                "Invalid JWT format: expected 3 parts, got {}",
                parts.len()
            )));
        }

        info!("Successfully obtained MAA attestation token");
        Ok(token.to_string())
    }

    pub fn parse_jwt_claims(&self, token: &str) -> Result<serde_json::Value, MAAError> {
        // Split JWT into parts
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(MAAError::InvalidToken(format!(
                "Invalid JWT format: expected 3 parts, got {}",
                parts.len()
            )));
        }

        // Decode the payload (second part)
        let payload_part = parts[1];

        // JWT uses base64url encoding, add padding if needed
        let payload_padded = self.add_base64_padding(payload_part);

        let payload_bytes = general_purpose::URL_SAFE_NO_PAD
            .decode(&payload_padded)
            .or_else(|_| general_purpose::STANDARD.decode(&payload_padded))
            .map_err(|e| MAAError::Base64Error(e))?;

        let payload_str = String::from_utf8(payload_bytes)
            .map_err(|e| MAAError::InvalidToken(format!("Invalid UTF-8 in JWT payload: {}", e)))?;

        let claims: serde_json::Value = serde_json::from_str(&payload_str)?;

        Ok(claims)
    }

    // Helper function to add padding to base64 strings if needed
    fn add_base64_padding(&self, input: &str) -> String {
        let mut padded = input.to_string();
        while padded.len() % 4 != 0 {
            padded.push('=');
        }
        padded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_maa_client_creation() {
        let client = MAAClient::new("https://test.attest.azure.net".to_string());
        assert_eq!(client.endpoint, "https://test.attest.azure.net");
        assert!(client.skr_endpoint.contains("/attest/maa"));
    }

    #[test]
    fn test_parse_jwt_response() {
        let client = MAAClient::new("https://test.attest.azure.net".to_string());

        // Test JWT token response
        let jwt_token =
            "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.signature";
        let result = client.parse_attestation_response(jwt_token);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), jwt_token);

        // Test JSON response with token field
        let json_response = r#"{"token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.signature"}"#;
        let result = client.parse_attestation_response(json_response);
        assert!(result.is_ok());

        // Test empty response
        let result = client.parse_attestation_response("");
        assert!(result.is_err());

        // Test invalid JWT
        let result = client.parse_attestation_response("invalid.jwt");
        assert!(result.is_err());
    }
}
