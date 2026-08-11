use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    BENCHMARK_REPORT_SCHEMA_VERSION, ContractError, Validate, require_schema_version, require_text,
};

/// One named metric in a reproducible benchmark report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BenchmarkMetric {
    /// Stable metric name.
    pub name: String,
    /// Observed point estimate.
    pub value: f64,
    /// Unit such as proportion, seconds or `records_per_second`.
    pub unit: String,
    /// Optional lower uncertainty bound.
    pub lower_bound: Option<f64>,
    /// Optional upper uncertainty bound.
    pub upper_bound: Option<f64>,
    /// Number of evaluated items.
    pub sample_size: u64,
}

/// Reproducible and claim-bounded benchmark report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BenchmarkReport {
    /// Contract identifier.
    pub schema_version: String,
    /// Stable benchmark-run identifier.
    pub benchmark_id: String,
    /// Corpus identifier.
    pub corpus_id: String,
    /// Corpus version or digest.
    pub corpus_version: String,
    /// Corpus licence or rights basis.
    pub rights_basis: String,
    /// Tool/component version.
    pub implementation_version: String,
    /// Deterministic configuration digest.
    pub configuration_digest: String,
    /// Leakage controls used for train/calibration/test separation.
    pub leakage_controls: Vec<String>,
    /// Metrics in stable name order.
    pub metrics: Vec<BenchmarkMetric>,
    /// Environment and execution notes.
    pub environment: Vec<String>,
    /// RFC 3339 generation timestamp.
    pub generated_at: String,
    /// Explicit boundary on claims supported by this report.
    pub claim_boundary: String,
}

impl Validate for BenchmarkMetric {
    fn validate(&self) -> Result<(), ContractError> {
        require_text(&self.name, "benchmark.metric.name")?;
        require_text(&self.unit, "benchmark.metric.unit")?;
        if !self.value.is_finite()
            || self.lower_bound.is_some_and(|value| !value.is_finite())
            || self.upper_bound.is_some_and(|value| !value.is_finite())
        {
            return Err(ContractError::Invariant(
                "benchmark metric values must be finite".to_owned(),
            ));
        }
        if let (Some(lower), Some(upper)) = (self.lower_bound, self.upper_bound)
            && lower > upper
        {
            return Err(ContractError::Invariant(
                "benchmark metric lower bound must not exceed the upper bound".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Validate for BenchmarkReport {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            BENCHMARK_REPORT_SCHEMA_VERSION,
            "benchmark.schema_version",
        )?;
        for (field, value) in [
            ("benchmark.benchmark_id", self.benchmark_id.as_str()),
            ("benchmark.corpus_id", self.corpus_id.as_str()),
            ("benchmark.corpus_version", self.corpus_version.as_str()),
            ("benchmark.rights_basis", self.rights_basis.as_str()),
            (
                "benchmark.implementation_version",
                self.implementation_version.as_str(),
            ),
            (
                "benchmark.configuration_digest",
                self.configuration_digest.as_str(),
            ),
            ("benchmark.generated_at", self.generated_at.as_str()),
            ("benchmark.claim_boundary", self.claim_boundary.as_str()),
        ] {
            require_text(value, field)?;
        }
        if self.configuration_digest.len() != 64
            || !self
                .configuration_digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(ContractError::Invariant(
                "benchmark configuration digest must be canonical BLAKE3 hexadecimal".to_owned(),
            ));
        }
        if self.metrics.is_empty() {
            return Err(ContractError::EmptyCollection("benchmark.metrics"));
        }
        let mut names = BTreeSet::new();
        for metric in &self.metrics {
            metric.validate()?;
            if !names.insert(metric.name.as_str()) {
                return Err(ContractError::Invariant(format!(
                    "benchmark metric name `{}` is duplicated",
                    metric.name
                )));
            }
        }
        if self.leakage_controls.is_empty()
            || self
                .leakage_controls
                .iter()
                .any(|value| value.trim().is_empty())
        {
            return Err(ContractError::EmptyCollection("benchmark.leakage_controls"));
        }
        Ok(())
    }
}
