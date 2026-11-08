//! Recompute only synthetic example digests, never execute either provider runtime.
use searchright_contracts::parity_matrix::{ParityExecutionMatrixV2, ParityExpectedCatalogueV2};
use searchright_sourceright_compat::parity_matrix::{
    expected_catalogue_digest, observation_digest, validate_execution_matrix,
};
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let expected: ParityExpectedCatalogueV2 = serde_json::from_str(include_str!(
        "../../../contracts/examples/sourceright-parity-catalogue.v2.json"
    ))?;
    let mut matrix: ParityExecutionMatrixV2 = serde_json::from_str(include_str!(
        "../../../contracts/examples/sourceright-parity-matrix.v2.json"
    ))?;
    for cell in &mut matrix.cells {
        cell.legacy.digest = observation_digest(&cell.expected, &matrix.legacy, &cell.legacy)?;
        cell.shared.digest = observation_digest(&cell.expected, &matrix.shared, &cell.shared)?;
    }
    let pin = expected_catalogue_digest(&expected)?;
    validate_execution_matrix(&expected, &pin, &matrix)?;
    let output = serde_json::json!({"synthetic_only": true, "expected_pin": pin, "matrix": matrix});
    serde_json::to_writer_pretty(std::io::stdout().lock(), &output)?;
    std::io::stdout().lock().write_all(b"\n")?;
    Ok(())
}
