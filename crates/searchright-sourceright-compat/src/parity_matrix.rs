//! Validate caller-supplied evidence against independently pinned expected scope.
use searchright_contracts::parity_matrix::{
    ExpectedParityCell, PARITY_CATALOGUE_SCHEMA_V2, PARITY_COMPARATOR_V2, PARITY_MATRIX_SCHEMA_V2,
    ParityCellKey, ParityEvidenceKind, ParityExecutionMatrixV2, ParityExecutionObservation,
    ParityExpectedCatalogueV2, ParityOutcome, ParityRunBinding, ParityRunSide,
};
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

const MAX_CELLS: usize = 4096;
const MAX_OBSERVATION_BYTES: usize = 65_536;
const MAX_AGGREGATE_BYTES: usize = 8 * 1024 * 1024;
const MAX_DEPTH: usize = 32;

/// Redacted validation failures; never echo arbitrary observations or references.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum MatrixError {
    /// A declared field, schema, comparator or provenance is malformed.
    #[error("invalid parity matrix metadata or provenance")]
    Metadata,
    /// Required scope is missing, duplicated, unexpected or reassigned.
    #[error("parity matrix scope does not match expected cells")]
    Coverage,
    /// An externally supplied pin or observation digest disagrees.
    #[error("parity matrix digest mismatch")]
    Digest,
    /// Declared execution binding differs from expected scope.
    #[error("parity execution binding mismatch")]
    Binding,
    /// Bounded input work exceeded the declared limit.
    #[error("parity matrix resource limit exceeded")]
    Limit,
}

/// Advisory comparison for one cell; equality does not imply successful execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixCellAssessment {
    /// Exact comparison identity.
    pub key: ParityCellKey,
    /// Exact canonical status/value equality, ignoring execution references.
    pub equal: bool,
    /// Both sides declare a provider outcome, not harness failure or skipped work.
    pub executed: bool,
    /// Preserved legacy outcome classification.
    pub legacy_status: ParityOutcome,
    /// Preserved shared outcome classification.
    pub shared_status: ParityOutcome,
    /// Advisory only; never interpreted as approval or a waiver.
    pub decision_references: Vec<String>,
}

/// Structurally validated comparisons, not authenticated execution or authorization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixAssessment {
    /// Synthetic examples remain explicitly synthetic.
    pub evidence_kind: ParityEvidenceKind,
    /// Results in stable tuple order; differences and nonexecution are retained.
    pub cells: Vec<MatrixCellAssessment>,
}

fn text(value: &str) -> Result<(), MatrixError> {
    if value.is_empty()
        || value.len() > 512
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(MatrixError::Metadata);
    }
    Ok(())
}

fn hex(value: &str, length: usize) -> Result<(), MatrixError> {
    if value.len() != length
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(MatrixError::Metadata);
    }
    Ok(())
}

fn binding(value: &ParityRunBinding) -> Result<(), MatrixError> {
    text(&value.repository)?;
    hex(&value.revision, 40)?;
    hex(&value.config_sha256, 64)?;
    hex(&value.harness_sha256, 64)
}

fn expected_cell(cell: &ExpectedParityCell) -> Result<(), MatrixError> {
    text(&cell.key.provider_id)?;
    hex(&cell.key.fixture_sha256, 64)?;
    text(&cell.provenance.source_id)?;
    text(&cell.provenance.rights_basis)?;
    if cell.comparator_id != PARITY_COMPARATOR_V2 {
        return Err(MatrixError::Metadata);
    }
    let dimensions: &[&str] = match cell.key.case_id.as_str() {
        "disabled-live" => &[
            "execution mode",
            "error classification",
            "endpoint and secret redaction",
            "disabled-live negative behaviour",
        ],
        "fixture-identifiers" => &[
            "provider identity",
            "identifiers",
            "normalised fields",
            "fixture determinism",
        ],
        "bounded-retry" => &[
            "retry and rate behaviour",
            "timeout behaviour",
            "error classification",
        ],
        "cache-write-replay" => &[
            "execution mode",
            "replay and cache behaviour",
            "receipt counts",
        ],
        "malformed-payload" => &[
            "error classification",
            "malformed and adversarial response handling",
        ],
        "undeclared-host" => &["host policy", "malformed and adversarial response handling"],
        "secret-redaction" => &["endpoint and secret redaction", "cache key redaction"],
        _ => return Err(MatrixError::Coverage),
    };
    if !dimensions.contains(&cell.key.dimension.as_str()) {
        return Err(MatrixError::Coverage);
    }
    Ok(())
}

fn catalogue(value: &ParityExpectedCatalogueV2) -> Result<(), MatrixError> {
    if value.schema_version != PARITY_CATALOGUE_SCHEMA_V2 {
        return Err(MatrixError::Metadata);
    }
    binding(&value.legacy)?;
    binding(&value.shared)?;
    if value.legacy.side != ParityRunSide::Legacy || value.shared.side != ParityRunSide::Shared {
        return Err(MatrixError::Binding);
    }
    if value.cells.is_empty() {
        return Err(MatrixError::Coverage);
    }
    if value.cells.len() > MAX_CELLS {
        return Err(MatrixError::Limit);
    }
    let mut keys = BTreeSet::new();
    for cell in &value.cells {
        expected_cell(cell)?;
        if !keys.insert(&cell.key) {
            return Err(MatrixError::Coverage);
        }
    }
    Ok(())
}

// Iterative preflight bounds depth, breadth and strings before recursive canonicalisation.
fn value_size(value: &Value) -> Result<usize, MatrixError> {
    let mut stack = vec![(value, 0_usize)];
    let mut estimated = 0_usize;
    while let Some((value, depth)) = stack.pop() {
        if depth > MAX_DEPTH {
            return Err(MatrixError::Limit);
        }
        estimated = estimated.saturating_add(match value {
            Value::String(value) => value.len().saturating_add(2),
            Value::Array(items) => {
                if items.len() > MAX_OBSERVATION_BYTES {
                    return Err(MatrixError::Limit);
                }
                stack.extend(items.iter().map(|item| (item, depth + 1)));
                items.len().saturating_add(2)
            }
            Value::Object(fields) => {
                if fields.len() > MAX_OBSERVATION_BYTES {
                    return Err(MatrixError::Limit);
                }
                stack.extend(fields.values().map(|item| (item, depth + 1)));
                fields.keys().fold(2_usize, |total, key| {
                    total.saturating_add(key.len()).saturating_add(4)
                })
            }
            _ => 1,
        });
        if estimated > MAX_OBSERVATION_BYTES {
            return Err(MatrixError::Limit);
        }
    }
    let encoded = serde_json::to_vec(value).map_err(|_| MatrixError::Metadata)?;
    if encoded.len() > MAX_OBSERVATION_BYTES {
        return Err(MatrixError::Limit);
    }
    Ok(encoded.len())
}

fn hash<T: Serialize>(domain: &str, value: &T) -> Result<String, MatrixError> {
    let value = serde_json::to_value(value).map_err(|_| MatrixError::Metadata)?;
    let bytes =
        serde_json::to_vec(&super::canonicalise(&value)).map_err(|_| MatrixError::Metadata)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    hasher.update(&[0]);
    hasher.update(&bytes);
    Ok(hasher.finalize().to_hex().to_string())
}

/// Canonical pin for a caller-selected catalogue. Cell order is not significant.
/// The caller must obtain/select this pin independently of the untrusted report.
pub fn expected_catalogue_digest(value: &ParityExpectedCatalogueV2) -> Result<String, MatrixError> {
    catalogue(value)?;
    let mut sorted = value.clone();
    sorted.cells.sort_by(|left, right| left.key.cmp(&right.key));
    hash("searchright.parity-catalogue.blake3.v2", &sorted)
}

/// Hash an observation including exact cell, run, status and evidence declarations.
/// The existing `digest` field is excluded. This is integrity, not authentication.
pub fn observation_digest(
    cell: &ExpectedParityCell,
    run: &ParityRunBinding,
    observation: &ParityExecutionObservation,
) -> Result<String, MatrixError> {
    expected_cell(cell)?;
    binding(run)?;
    text(&observation.execution_id)?;
    hex(&observation.evidence_sha256, 64)?;
    value_size(&observation.value)?;
    hash(
        "searchright.parity-observation.blake3.v2",
        &(
            cell,
            run,
            observation.status,
            &observation.value,
            &observation.execution_id,
            &observation.evidence_sha256,
        ),
    )
}

/// Validate exact coverage against an independently selected/pinned catalogue.
///
/// No data is fetched, no fixture/harness execution is authenticated, and no
/// equality or decision reference authorizes migration. Inputs are already
/// allocated Rust structures, not a secure untrusted-byte deserialization API.
pub fn validate_execution_matrix(
    expected: &ParityExpectedCatalogueV2,
    expected_pin: &str,
    matrix: &ParityExecutionMatrixV2,
) -> Result<MatrixAssessment, MatrixError> {
    hex(expected_pin, 64)?;
    if expected_catalogue_digest(expected)? != expected_pin {
        return Err(MatrixError::Digest);
    }
    if matrix.schema_version != PARITY_MATRIX_SCHEMA_V2 {
        return Err(MatrixError::Metadata);
    }
    if matrix.evidence_kind != expected.evidence_kind
        || matrix.legacy != expected.legacy
        || matrix.shared != expected.shared
    {
        return Err(MatrixError::Binding);
    }
    if matrix.cells.len() > MAX_CELLS {
        return Err(MatrixError::Limit);
    }
    if matrix.cells.len() != expected.cells.len() {
        return Err(MatrixError::Coverage);
    }
    let required: BTreeMap<_, _> = expected
        .cells
        .iter()
        .map(|cell| (&cell.key, cell))
        .collect();
    let mut results = BTreeMap::new();
    let mut aggregate = 0_usize;
    for cell in &matrix.cells {
        if required.get(&cell.expected.key).copied() != Some(&cell.expected)
            || results.contains_key(&cell.expected.key)
        {
            return Err(MatrixError::Coverage);
        }
        if cell.decision_references.len() > 16 {
            return Err(MatrixError::Limit);
        }
        let mut references = BTreeSet::new();
        for reference in &cell.decision_references {
            text(reference)?;
            if !references.insert(reference) {
                return Err(MatrixError::Metadata);
            }
        }
        for (run, observation) in [
            (&matrix.legacy, &cell.legacy),
            (&matrix.shared, &cell.shared),
        ] {
            aggregate = aggregate.saturating_add(value_size(&observation.value)?);
            if aggregate > MAX_AGGREGATE_BYTES {
                return Err(MatrixError::Limit);
            }
            hex(&observation.digest, 64)?;
            if observation_digest(&cell.expected, run, observation)? != observation.digest {
                return Err(MatrixError::Digest);
            }
        }
        let equal = hash(
            "searchright.parity-comparison.blake3.v2",
            &(cell.legacy.status, &cell.legacy.value),
        )? == hash(
            "searchright.parity-comparison.blake3.v2",
            &(cell.shared.status, &cell.shared.value),
        )?;
        let executed = [cell.legacy.status, cell.shared.status]
            .iter()
            .all(|status| {
                matches!(
                    status,
                    ParityOutcome::Success | ParityOutcome::ProviderError
                )
            });
        results.insert(
            cell.expected.key.clone(),
            MatrixCellAssessment {
                key: cell.expected.key.clone(),
                equal,
                executed,
                legacy_status: cell.legacy.status,
                shared_status: cell.shared.status,
                decision_references: cell.decision_references.clone(),
            },
        );
    }
    Ok(MatrixAssessment {
        evidence_kind: matrix.evidence_kind,
        cells: results.into_values().collect(),
    })
}
