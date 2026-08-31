//! Local sibling handoff admission, without discovery or package invocation.
//! The supplied approval adapter may atomically consume one-use approvals.
//!
//! Pins must come from independently trusted owner policy, not from the candidate.
//! Matching bytes proves integrity only, not provenance, licence clearance, host
//! compatibility or permission to execute a package. Returned bytes are retained
//! so a caller need not reopen a mutable path after checking it.

use crate::{
    AgentHandoff, HandoffApprovalAuthority, HandoffExecutionMode, ProposedOperation,
    VerifiedHandoffArtifact, digest_hex, lowercase_sha256,
};
use searchright_contracts::ContractError;
use sha2::{Digest, Sha256};
use std::path::Path;

const MAX_COMPONENT_BYTES: usize = 8 * 1024 * 1024;
const MAX_HANDOFF_BYTES: usize = 64 * 1024;
const HANDOFF_SCHEMA: &[u8] =
    include_bytes!("../../../contracts/json-schema/agent-handoff.v1.schema.json");

/// Independently reviewed exact-byte pins; never deserialize these from a candidate.
#[derive(Debug, Clone)]
pub struct SiblingAdmissionPins {
    /// Expected Searchright package version, compatible with this compiled verifier.
    pub package_version: String,
    /// Trusted lowercase Git revision annotation. Byte equality alone does not
    /// establish that a snapshot was produced by this commit.
    pub source_revision: String,
    /// SHA-256 of the exact package bytes (not a candidate-declared digest).
    pub package_sha256: String,
    /// SHA-256 of the exact source snapshot bytes supplied for inspection.
    pub source_sha256: String,
    /// SHA-256 of the exact governed handoff JSON schema bytes.
    pub schema_sha256: String,
}

/// Candidate bytes already obtained through an independently authorized local route.
#[derive(Debug, Clone, Copy)]
pub struct SiblingAdmissionInput<'a> {
    /// Explicit user handoff, supplied by the trusted host rather than retrieved text.
    pub explicit_user_handoff: bool,
    /// Actual package bytes; this verifier does not unpack or execute them.
    pub package: &'a [u8],
    /// Actual source snapshot bytes; source provenance remains a separate gate.
    pub source: &'a [u8],
    /// Actual JSON schema bytes.
    pub schema: &'a [u8],
    /// Exact JSON envelope to parse and validate using the compiled handoff contract.
    pub handoff: &'a [u8],
}

/// Successful local integrity check, deliberately not an execution capability.
#[derive(Debug)]
pub struct AdmittedSiblingHandoff {
    /// Parsed envelope whose bytes and artifact-bound approvals were checked.
    pub handoff: AgentHandoff,
    /// SHA-256 of the original handoff JSON, retaining whitespace-sensitive identity.
    pub handoff_sha256: String,
    /// Verified, retained artifact bytes from the existing handoff boundary.
    pub artifacts: Vec<VerifiedHandoffArtifact>,
    /// Exact verified package bytes, never executed here.
    pub package: Vec<u8>,
    /// Exact verified source snapshot bytes, never executed here.
    pub source: Vec<u8>,
    /// Exact governed schema bytes.
    pub schema: Vec<u8>,
}

fn reject(message: &str) -> ContractError {
    ContractError::Invariant(message.to_owned())
}

fn verify_component(bytes: &[u8], expected: &str) -> Result<(), ContractError> {
    if bytes.is_empty() || bytes.len() > MAX_COMPONENT_BYTES {
        return Err(reject(
            "sibling component bytes are missing or exceed the byte budget",
        ));
    }
    if !lowercase_sha256(expected) || digest_hex(&Sha256::digest(bytes)) != expected {
        return Err(reject(
            "sibling component bytes do not match the trusted digest",
        ));
    }
    Ok(())
}

/// Verify exact pinned bytes and admit an explicitly requested local advisory handoff.
///
/// No source is downloaded, unpacked or invoked. Live, canonical-write, publication,
/// final-exclusion and protocol-amendment operations are unconditionally refused.
/// All structural and byte checks precede the existing atomic approval adapter.
/// Success cannot enable automated invocation or satisfy external Track 11 gates.
pub fn admit_sibling_handoff(
    pins: &SiblingAdmissionPins,
    input: SiblingAdmissionInput<'_>,
    approved_root: &Path,
    operation: ProposedOperation,
    authority: &mut impl HandoffApprovalAuthority,
) -> Result<AdmittedSiblingHandoff, ContractError> {
    if !input.explicit_user_handoff {
        return Err(reject(
            "sibling admission requires an explicit user handoff",
        ));
    }
    if !matches!(
        operation,
        ProposedOperation::Draft | ProposedOperation::FixtureReplay
    ) {
        return Err(reject(
            "sibling admission cannot grant consequential operation authority",
        ));
    }
    if pins.package_version != env!("CARGO_PKG_VERSION")
        || pins.source_revision.len() != 40
        || !pins
            .source_revision
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(reject(
            "sibling package version or source revision is incompatible",
        ));
    }
    verify_component(input.package, &pins.package_sha256)?;
    verify_component(input.source, &pins.source_sha256)?;
    verify_component(input.schema, &pins.schema_sha256)?;
    if input.schema != HANDOFF_SCHEMA {
        return Err(reject(
            "sibling schema is not the exact compiled governed contract",
        ));
    }
    if input.handoff.is_empty() || input.handoff.len() > MAX_HANDOFF_BYTES {
        return Err(reject(
            "sibling handoff bytes are missing or exceed the byte budget",
        ));
    }
    let handoff: AgentHandoff = serde_json::from_slice(input.handoff)
        .map_err(|_| reject("sibling handoff is not a valid handoff JSON envelope"))?;
    // Serde treats a missing Option as None, but the governed schema requires
    // an explicit execution_mode key (null outside the execution transition).
    let envelope: serde_json::Value = serde_json::from_slice(input.handoff)
        .map_err(|_| reject("sibling handoff is not a valid handoff JSON envelope"))?;
    if envelope.get("execution_mode").is_none() {
        return Err(reject(
            "sibling handoff must declare execution_mode explicitly",
        ));
    }
    if handoff.execution_mode == Some(HandoffExecutionMode::Live) {
        return Err(reject(
            "sibling admission does not permit live execution handoffs",
        ));
    }
    let artifacts = handoff.verify_and_authorize(approved_root, authority)?;
    Ok(AdmittedSiblingHandoff {
        handoff,
        handoff_sha256: digest_hex(&Sha256::digest(input.handoff)),
        artifacts,
        package: input.package.to_vec(),
        source: input.source.to_vec(),
        schema: input.schema.to_vec(),
    })
}
