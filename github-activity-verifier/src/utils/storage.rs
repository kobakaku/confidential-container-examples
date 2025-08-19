use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, info};

use crate::api::types::VerificationResult;

#[derive(Debug, Clone)]
pub struct ProofStorage {
    proofs: Arc<RwLock<HashMap<String, StoredProof>>>,
}

#[derive(Debug, Clone)]
struct StoredProof {
    verification_result: VerificationResult,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

impl ProofStorage {
    pub fn new() -> Self {
        Self {
            proofs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn store_proof(&self, proof_hash: String, result: VerificationResult) {
        let expires_at = Utc::now() + Duration::hours(24);
        let stored_proof = StoredProof {
            verification_result: result,
            created_at: Utc::now(),
            expires_at,
        };

        {
            let mut proofs = self.proofs.write().unwrap();
            proofs.insert(proof_hash.clone(), stored_proof);

            // Cleanup expired proofs while we have the write lock
            self.cleanup_expired(&mut proofs);
        }

        info!(
            "Stored proof with hash: {} (expires at: {})",
            proof_hash, expires_at
        );
    }

    pub async fn get_proof(&self, proof_hash: &str) -> Option<VerificationResult> {
        let mut proofs = self.proofs.write().unwrap();

        if let Some(stored_proof) = proofs.get(proof_hash) {
            if stored_proof.expires_at > Utc::now() {
                debug!("Retrieved valid proof for hash: {}", proof_hash);
                return Some(stored_proof.verification_result.clone());
            } else {
                debug!("Proof expired for hash: {}, removing", proof_hash);
                proofs.remove(proof_hash);
            }
        }

        debug!("Proof not found for hash: {}", proof_hash);
        None
    }

    fn cleanup_expired(&self, proofs: &mut HashMap<String, StoredProof>) {
        let now = Utc::now();
        let before_count = proofs.len();

        proofs.retain(|_, proof| proof.expires_at > now);

        let after_count = proofs.len();
        if before_count != after_count {
            debug!("Cleaned up {} expired proofs", before_count - after_count);
        }
    }

    pub async fn get_storage_stats(&self) -> StorageStats {
        let proofs = self.proofs.read().unwrap();
        let now = Utc::now();

        let mut valid_count = 0;
        let mut expired_count = 0;

        for proof in proofs.values() {
            if proof.expires_at > now {
                valid_count += 1;
            } else {
                expired_count += 1;
            }
        }

        StorageStats {
            total_proofs: proofs.len(),
            valid_proofs: valid_count,
            expired_proofs: expired_count,
        }
    }
}

#[derive(Debug)]
pub struct StorageStats {
    pub total_proofs: usize,
    pub valid_proofs: usize,
    pub expired_proofs: usize,
}
