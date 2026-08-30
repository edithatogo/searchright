use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ContractError, INTERCHANGE_RECEIPT_SCHEMA_VERSION, Validate, require_schema_version,
    require_text,
};

/// Supported import or export format.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InterchangeFormat {
    /// Searchright canonical JSON array.
    SearchrightJson,
    /// JSON Lines.
    JsonLines,
    /// Citation Style Language JSON.
    CslJson,
    /// RIS.
    Ris,
    /// PubMed/NLM tagged text.
    Nbib,
    /// PubMed XML.
    PubmedXml,
    /// `BibTeX`.
    Bibtex,
    /// `EndNote` XML.
    EndnoteXml,
    /// UTF-8 CSV.
    Csv,
    /// Parquet analytical export.
    Parquet,
    /// A named custom format.
    Custom(String),
}

/// Result of one deterministic import/export operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct InterchangeReceipt {
    /// Contract identifier.
    pub schema_version: String,
    /// Stable operation identifier.
    pub operation_id: String,
    /// Review identifier.
    pub review_id: String,
    /// Input format.
    pub input_format: InterchangeFormat,
    /// Output format.
    pub output_format: InterchangeFormat,
    /// SHA-256 or BLAKE3 digest of the input bytes.
    pub input_digest: String,
    /// Digest of the output bytes.
    pub output_digest: String,
    /// Number of records read.
    pub records_read: u64,
    /// Number of records written.
    pub records_written: u64,
    /// Non-fatal conversion warnings.
    #[serde(default)]
    pub warnings: Vec<String>,
    /// Whether conversion was lossless for the canonical fields represented.
    pub lossless: bool,
}

impl Validate for InterchangeReceipt {
    fn validate(&self) -> Result<(), ContractError> {
        require_schema_version(
            &self.schema_version,
            INTERCHANGE_RECEIPT_SCHEMA_VERSION,
            "interchange.schema_version",
        )?;
        require_text(&self.operation_id, "interchange.operation_id")?;
        require_text(&self.review_id, "interchange.review_id")?;
        require_text(&self.input_digest, "interchange.input_digest")?;
        require_text(&self.output_digest, "interchange.output_digest")?;
        if self.records_written > self.records_read
            && !self.warnings.iter().any(|item| item.contains("expanded"))
        {
            return Err(ContractError::Invariant(
                "interchange output count exceeds input count without an expansion warning"
                    .to_owned(),
            ));
        }
        Ok(())
    }
}
