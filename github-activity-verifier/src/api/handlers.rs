use actix_web::{web, HttpResponse, Result};
use chrono::Utc;
use sha2::{Digest, Sha256};
use tracing::{error, info};

use crate::api::types::{ApiError, VerificationRequest, VerificationResult};
use crate::verification::engine::VerificationEngine;
use crate::{utils::errors::AppError, AppState};

pub async fn verify(
    app_state: AppState,
    req: web::Json<VerificationRequest>,
) -> Result<HttpResponse> {
    info!("Verification request for user: {}", req.github_username);

    match verify_internal(app_state, req.into_inner()).await {
        Ok(result) => {
            info!(
                "Verification completed successfully for user: {}",
                result.username
            );
            Ok(HttpResponse::Ok().json(result))
        }
        Err(err) => {
            error!("Verification failed: {}", err);
            Ok(err.into())
        }
    }
}

async fn verify_internal(
    app_state: AppState,
    req: VerificationRequest,
) -> Result<VerificationResult, AppError> {
    // 1. Input validation
    crate::utils::validation::validate_github_username(&req.github_username)?;

    let threshold = req
        .threshold
        .unwrap_or_else(|| req.verification_type.default_threshold());
    if threshold == 0 || threshold > 10000 {
        return Err(AppError::Validation(
            "Threshold must be between 1 and 10000".to_string(),
        ));
    }

    // 2. GitHub API calls
    let github_client = &app_state.github_client;
    let events = github_client
        .fetch_user_events(&req.github_username)
        .await?;

    // 3. Verification logic
    let engine = VerificationEngine::new();
    let meets_criteria = engine
        .verify_criteria(&events, req.verification_type, threshold)
        .await?;

    let verified_at = Utc::now();

    // 4. Generate proof only if verification succeeds
    let (attestation_token, attestation_claims, proof_hash) = if meets_criteria {
        let proof_data = format!(
            "{}:{}:{}:{}",
            req.github_username,
            serde_json::to_string(&req.verification_type).unwrap(),
            meets_criteria,
            verified_at.timestamp()
        );
        let hash = format!("{:x}", Sha256::digest(proof_data.as_bytes()));

        // MAA attestation for successful verification
        let (token, claims) = if !app_state.maa_client.endpoint.is_empty() {
            match app_state.maa_client.get_attestation_token(&hash).await {
                Ok(jwt_token) => {
                    // JWT claimsも解析
                    let parsed_claims = app_state
                        .maa_client
                        .parse_jwt_claims(&jwt_token)
                        .map_err(|err| {
                            error!("Failed to parse JWT claims: {}", err);
                            err
                        })
                        .ok();
                    (Some(jwt_token), parsed_claims)
                }
                Err(err) => {
                    error!("MAA attestation failed: {}", err);
                    (Some("MAA_UNAVAILABLE".to_string()), None)
                }
            }
        } else {
            (Some("MAA_NOT_CONFIGURED".to_string()), None)
        };

        (token, claims, Some(hash))
    } else {
        info!(
            "Verification failed - no proof generated for user: {}",
            req.github_username
        );
        (None, None, None)
    };

    // 5. Create result
    let result = VerificationResult {
        username: req.github_username,
        verification_type: req.verification_type,
        threshold,
        meets_criteria,
        attestation_token,
        attestation_claims,
        verified_at,
        proof_hash: proof_hash.clone(),
    };

    // 6. Store proof only if verification succeeded
    if let Some(hash) = proof_hash {
        app_state
            .proof_storage
            .store_proof(hash, result.clone())
            .await;
    }

    Ok(result)
}

pub async fn get_proof(app_state: AppState, path: web::Path<String>) -> Result<HttpResponse> {
    let proof_hash = path.into_inner();

    // Validate proof hash format
    if !proof_hash.chars().all(|c| c.is_ascii_hexdigit()) || proof_hash.len() != 64 {
        return Ok(HttpResponse::BadRequest().json(ApiError {
            error: "Invalid proof hash format".to_string(),
            error_code: "INVALID_PROOF_HASH".to_string(),
            details: None,
        }));
    }

    match app_state.proof_storage.get_proof(&proof_hash).await {
        Some(result) => {
            info!("Proof retrieved for hash: {}", proof_hash);
            Ok(HttpResponse::Ok().json(result))
        }
        None => {
            info!("Proof not found for hash: {}", proof_hash);
            Ok(HttpResponse::NotFound().json(ApiError {
                error: "Proof not found".to_string(),
                error_code: "PROOF_NOT_FOUND".to_string(),
                details: Some("The proof may have expired or never existed".to_string()),
            }))
        }
    }
}
