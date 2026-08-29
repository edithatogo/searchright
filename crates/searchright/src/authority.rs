//! Fail-closed authority boundary for consequential product operations.

use std::time::{SystemTime, UNIX_EPOCH};

const MAX_AUTHORITY_LIFETIME_SECONDS: u64 = 300;

/// Exact binding presented to a trusted host authority verifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectAuthorityRequest {
    /// Consequential facade operation being authorised.
    pub tool_name: String,
    /// Base64-encoded SHA-256 binding the exact submitted operation arguments.
    pub request_digest: String,
    /// Review or PRESS record identifier in the bounded operation.
    pub review_id: String,
    /// Caller-provided identity hint that the trusted verifier must authenticate.
    pub principal_hint: String,
    /// Optional policy digest for policy-governed operations.
    pub policy_digest: Option<String>,
    /// Digest of the store state against which the authority was granted.
    pub store_state_digest: String,
}

/// Verifier response validated before it becomes an opaque grant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectAuthorityAttestation {
    /// Exact consequential facade operation admitted by the host.
    pub tool_name: String,
    /// Exact request digest verified by the host.
    pub request_digest: String,
    /// Exact review or PRESS record identifier verified by the host.
    pub review_id: String,
    /// Authenticated principal, never inferred by the facade from a role label.
    pub principal: String,
    /// Optional exact policy digest verified by the host.
    pub policy_digest: Option<String>,
    /// Exact store-state digest verified by the host.
    pub store_state_digest: String,
    /// Bounded verifier-issued replay nonce.
    pub nonce: String,
    /// Inclusive Unix issuance time.
    pub issued_at_unix_seconds: u64,
    /// Exclusive Unix expiry time, at most five minutes after issuance.
    pub expires_at_unix_seconds: u64,
}

/// Trusted host boundary for human identity and approval state.
pub trait EffectAuthorityVerifier: Send + Sync {
    /// Authenticate and bind a principal to the exact request and store state.
    fn verify(
        &self,
        request: &EffectAuthorityRequest,
    ) -> Result<EffectAuthorityAttestation, EffectAuthorityError>;
}

/// Deliberately non-sensitive authority failure.
#[derive(Debug, thiserror::Error)]
#[error("effect authority denied")]
pub struct EffectAuthorityError;

/// Opaque proof required by consequential shared-facade operations.
#[derive(Debug)]
pub struct VerifiedEffectAuthority {
    attestation: EffectAuthorityAttestation,
}

impl VerifiedEffectAuthority {
    pub(crate) fn permits(&self, tool: &str, review_id: &str, principal: &str) -> bool {
        self.attestation.tool_name == tool
            && self.attestation.review_id == review_id
            && self.attestation.principal == principal
    }

    /// Single-use nonce for replay protection in the invoking adapter.
    #[must_use]
    pub fn nonce(&self) -> &str {
        &self.attestation.nonce
    }
}

/// Validate a host attestation and mint an opaque, bounded facade grant.
pub fn verify_effect_authority(
    verifier: &dyn EffectAuthorityVerifier,
    request: &EffectAuthorityRequest,
) -> Result<VerifiedEffectAuthority, EffectAuthorityError> {
    let attestation = verifier.verify(request)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| EffectAuthorityError)?
        .as_secs();
    let principal_matches = attestation.principal == request.principal_hint;
    let fields_match = attestation.tool_name == request.tool_name
        && attestation.request_digest == request.request_digest
        && attestation.review_id == request.review_id
        && principal_matches
        && attestation.policy_digest == request.policy_digest
        && attestation.store_state_digest == request.store_state_digest;
    let lifetime_valid = attestation.issued_at_unix_seconds <= now
        && now < attestation.expires_at_unix_seconds
        && attestation.expires_at_unix_seconds >= attestation.issued_at_unix_seconds
        && attestation.expires_at_unix_seconds - attestation.issued_at_unix_seconds
            <= MAX_AUTHORITY_LIFETIME_SECONDS;
    if !fields_match || !lifetime_valid || !bounded_nonce(&attestation.nonce) {
        return Err(EffectAuthorityError);
    }
    Ok(VerifiedEffectAuthority { attestation })
}

fn bounded_nonce(value: &str) -> bool {
    (16..=128).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}
