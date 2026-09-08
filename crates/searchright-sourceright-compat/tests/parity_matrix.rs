//! Offline parity execution matrix v2 validation fixtures.
use searchright_contracts::parity_matrix::{
    ExpectedParityCell, FixtureProvenance, ObservedParityCell, PARITY_CATALOGUE_SCHEMA_V2,
    PARITY_COMPARATOR_V2, PARITY_MATRIX_SCHEMA_V2, ParityCellKey, ParityEvidenceKind,
    ParityExecutionMatrixV2, ParityExecutionObservation, ParityExpectedCatalogueV2, ParityOutcome,
    ParityRunBinding, ParityRunSide,
};
use searchright_sourceright_compat::parity_matrix::{
    MatrixError, expected_catalogue_digest, observation_digest, validate_execution_matrix,
};
use serde_json::json;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn fixture() -> Result<(ParityExpectedCatalogueV2, ParityExecutionMatrixV2, String), MatrixError> {
    let binding = ParityRunBinding {
        side: ParityRunSide::Legacy,
        repository: "synthetic/legacy".into(),
        revision: "a".repeat(40),
        config_sha256: "1".repeat(64),
        harness_sha256: "2".repeat(64),
    };
    let shared = ParityRunBinding {
        side: ParityRunSide::Shared,
        repository: "synthetic/shared".into(),
        revision: "b".repeat(40),
        ..binding.clone()
    };
    let cells = ["identifiers", "normalised fields"]
        .into_iter()
        .map(|dimension| ExpectedParityCell {
            key: ParityCellKey {
                provider_id: "pubmed".into(),
                fixture_sha256: "3".repeat(64),
                case_id: "fixture-identifiers".into(),
                dimension: dimension.into(),
            },
            provenance: FixtureProvenance {
                source_id: "synthetic:test".into(),
                rights_basis: "synthetic-original".into(),
            },
            comparator_id: PARITY_COMPARATOR_V2.into(),
        })
        .collect::<Vec<_>>();
    let expected = ParityExpectedCatalogueV2 {
        schema_version: PARITY_CATALOGUE_SCHEMA_V2.into(),
        evidence_kind: ParityEvidenceKind::Synthetic,
        legacy: binding,
        shared,
        cells,
    };
    let mut matrix = ParityExecutionMatrixV2 {
        schema_version: PARITY_MATRIX_SCHEMA_V2.into(),
        evidence_kind: ParityEvidenceKind::Synthetic,
        legacy: expected.legacy.clone(),
        shared: expected.shared.clone(),
        cells: Vec::new(),
    };
    for cell in &expected.cells {
        let observation = ParityExecutionObservation {
            status: ParityOutcome::Success,
            value: json!({"id":"1"}),
            execution_id: "synthetic-run".into(),
            evidence_sha256: "4".repeat(64),
            digest: String::new(),
        };
        let mut result = ObservedParityCell {
            expected: cell.clone(),
            legacy: observation.clone(),
            shared: observation,
            decision_references: Vec::new(),
        };
        result.legacy.digest =
            observation_digest(&result.expected, &matrix.legacy, &result.legacy)?;
        result.shared.digest =
            observation_digest(&result.expected, &matrix.shared, &result.shared)?;
        matrix.cells.push(result);
    }
    let pin = expected_catalogue_digest(&expected)?;
    Ok((expected, matrix, pin))
}

fn first<T>(items: &[T]) -> Result<&T, Box<dyn std::error::Error>> {
    items.first().ok_or_else(|| "missing test item".into())
}
fn first_mut<T>(items: &mut [T]) -> Result<&mut T, Box<dyn std::error::Error>> {
    items.first_mut().ok_or_else(|| "missing test item".into())
}
fn seal(matrix: &mut ParityExecutionMatrixV2) -> Result<(), MatrixError> {
    for cell in &mut matrix.cells {
        cell.legacy.digest = observation_digest(&cell.expected, &matrix.legacy, &cell.legacy)?;
        cell.shared.digest = observation_digest(&cell.expected, &matrix.shared, &cell.shared)?;
    }
    Ok(())
}

#[test]
fn complete_synthetic_matrix_is_only_a_comparison() -> TestResult {
    let (expected, matrix, pin) = fixture()?;
    let assessment = validate_execution_matrix(&expected, &pin, &matrix)?;
    assert_eq!(assessment.cells.len(), 2);
    assert!(
        assessment
            .cells
            .iter()
            .all(|cell| cell.equal && cell.executed)
    );
    assert_eq!(assessment.evidence_kind, ParityEvidenceKind::Synthetic);
    Ok(())
}

#[test]
fn missing_duplicate_unexpected_and_reassigned_cells_fail() -> TestResult {
    let (expected, matrix, pin) = fixture()?;
    let mut missing = matrix.clone();
    missing.cells.pop();
    assert!(validate_execution_matrix(&expected, &pin, &missing).is_err());
    let mut duplicate = matrix.clone();
    duplicate.cells.push(first(&matrix.cells)?.clone());
    assert!(validate_execution_matrix(&expected, &pin, &duplicate).is_err());
    let mut same_count_duplicate = matrix.clone();
    *same_count_duplicate
        .cells
        .last_mut()
        .ok_or("missing last cell")? = first(&matrix.cells)?.clone();
    assert_eq!(same_count_duplicate.cells.len(), expected.cells.len());
    assert!(validate_execution_matrix(&expected, &pin, &same_count_duplicate).is_err());
    for mutation in 0..4 {
        let mut changed = matrix.clone();
        let key = &mut first_mut(&mut changed.cells)?.expected.key;
        match mutation {
            0 => key.provider_id = "crossref".into(),
            1 => key.fixture_sha256 = "5".repeat(64),
            2 => key.case_id = "disabled-live".into(),
            _ => key.dimension = "receipt counts".into(),
        }
        assert!(validate_execution_matrix(&expected, &pin, &changed).is_err());
    }
    let mut smaller = expected.clone();
    smaller.cells.pop();
    assert!(validate_execution_matrix(&smaller, &pin, &missing).is_err());
    let mut duplicated = expected.clone();
    duplicated.cells.push(first(&expected.cells)?.clone());
    assert!(expected_catalogue_digest(&duplicated).is_err());
    Ok(())
}

#[test]
fn binding_provenance_comparator_and_observation_tampering_fail() -> TestResult {
    let (expected, matrix, pin) = fixture()?;
    for mutation in 0..12 {
        let mut changed = matrix.clone();
        let cell = first_mut(&mut changed.cells)?;
        match mutation {
            0 => changed.legacy.revision = "c".repeat(40),
            1 => changed.shared.config_sha256 = "c".repeat(64),
            2 => changed.shared.harness_sha256 = "c".repeat(64),
            3 => changed.shared.repository = "other/repository".into(),
            4 => cell.expected.provenance.source_id = "other-source".into(),
            5 => cell.expected.comparator_id = "invented".into(),
            6 => cell.legacy.value = json!("tampered"),
            7 => cell.legacy.status = ParityOutcome::Skipped,
            8 => cell.legacy.execution_id = "another-run".into(),
            9 => cell.legacy.evidence_sha256 = "c".repeat(64),
            10 => changed.evidence_kind = ParityEvidenceKind::DeclaredExecution,
            _ => cell.legacy.digest = "c".repeat(64),
        }
        assert!(validate_execution_matrix(&expected, &pin, &changed).is_err());
    }
    let mut transplanted = matrix.clone();
    let old_observation = first(&matrix.cells)?.legacy.clone();
    transplanted
        .cells
        .last_mut()
        .ok_or("missing last cell")?
        .legacy = old_observation;
    assert!(validate_execution_matrix(&expected, &pin, &transplanted).is_err());
    Ok(())
}

#[test]
fn swapped_outputs_fail_even_when_repository_revision_and_config_are_identical() -> TestResult {
    let (mut expected, mut matrix, _) = fixture()?;
    expected.shared = ParityRunBinding {
        side: ParityRunSide::Shared,
        ..expected.legacy.clone()
    };
    matrix.shared = expected.shared.clone();
    seal(&mut matrix)?;
    let pin = expected_catalogue_digest(&expected)?;
    validate_execution_matrix(&expected, &pin, &matrix)?;
    let cell = first_mut(&mut matrix.cells)?;
    std::mem::swap(&mut cell.legacy, &mut cell.shared);
    assert!(validate_execution_matrix(&expected, &pin, &matrix).is_err());
    Ok(())
}

#[test]
fn differences_provider_errors_and_nonexecution_remain_visible() -> TestResult {
    let (expected, original, pin) = fixture()?;
    for status in [
        ParityOutcome::ProviderError,
        ParityOutcome::HarnessFailure,
        ParityOutcome::Skipped,
    ] {
        let mut matrix = original.clone();
        let cell = first_mut(&mut matrix.cells)?;
        cell.legacy.status = status;
        cell.shared.status = status;
        seal(&mut matrix)?;
        let result = validate_execution_matrix(&expected, &pin, &matrix)?;
        let cell = first(&result.cells)?;
        assert!(cell.equal);
        assert_eq!(cell.executed, status == ParityOutcome::ProviderError);
        assert_eq!(cell.legacy_status, status);
    }
    let mut matrix = original;
    let cell = first_mut(&mut matrix.cells)?;
    cell.shared.value = json!("different");
    cell.decision_references
        .push("owner-reference-not-authenticated".into());
    seal(&mut matrix)?;
    let result = validate_execution_matrix(&expected, &pin, &matrix)?;
    assert!(!first(&result.cells)?.equal);
    assert_eq!(
        first(&result.cells)?.decision_references,
        first(&matrix.cells)?.decision_references
    );
    Ok(())
}

#[test]
fn order_is_irrelevant_except_inside_observed_arrays_and_numbers_are_not_coerced() -> TestResult {
    let (mut expected, mut matrix, pin) = fixture()?;
    expected.cells.reverse();
    matrix.cells.reverse();
    assert_eq!(expected_catalogue_digest(&expected)?, pin);
    validate_execution_matrix(&expected, &pin, &matrix)?;
    for (legacy, shared, equality) in [
        (r#"{"b":2,"a":1}"#, r#"{"a":1,"b":2}"#, true),
        ("[1,2]", "[2,1]", false),
        ("1", "1.0", false),
        ("0.0", "-0.0", false),
        (r#""value""#, r#"" value""#, false),
    ] {
        let cell = first_mut(&mut matrix.cells)?;
        cell.legacy.value = serde_json::from_str(legacy)?;
        cell.shared.value = serde_json::from_str(shared)?;
        seal(&mut matrix)?;
        let assessment = validate_execution_matrix(&expected, &pin, &matrix)?;
        assert_eq!(assessment.cells.iter().all(|cell| cell.equal), equality);
    }
    Ok(())
}

#[test]
fn malformed_metadata_unknown_fields_and_resource_limits_fail() -> TestResult {
    let (expected, matrix, pin) = fixture()?;
    let mut unknown = serde_json::to_value(&matrix)?;
    unknown
        .as_object_mut()
        .ok_or("object")?
        .insert("cutover_ready".into(), json!(true));
    assert!(serde_json::from_value::<ParityExecutionMatrixV2>(unknown).is_err());
    let mut nested = serde_json::to_value(first(&matrix.cells)?)?;
    nested
        .as_object_mut()
        .ok_or("object")?
        .insert("accepted".into(), json!(true));
    assert!(serde_json::from_value::<ObservedParityCell>(nested).is_err());
    let mut invalid = expected.clone();
    invalid.legacy.revision = "main".into();
    assert!(expected_catalogue_digest(&invalid).is_err());
    let mut invalid = expected.clone();
    first_mut(&mut invalid.cells)?.key.case_id = "disabled-live".into();
    assert!(expected_catalogue_digest(&invalid).is_err());
    let mut invalid = expected.clone();
    first_mut(&mut invalid.cells)?.provenance.rights_basis = " ".into();
    assert!(expected_catalogue_digest(&invalid).is_err());
    let mut invalid = matrix;
    first_mut(&mut invalid.cells)?.legacy.value = json!("x".repeat(65_537));
    assert!(validate_execution_matrix(&expected, &pin, &invalid).is_err());
    let mut value = json!(0);
    for _ in 0..34 {
        value = json!([value]);
    }
    first_mut(&mut invalid.cells)?.legacy.value = value;
    assert!(validate_execution_matrix(&expected, &pin, &invalid).is_err());
    let mut oversized = expected.clone();
    oversized.cells = vec![first(&expected.cells)?.clone(); 4097];
    assert!(expected_catalogue_digest(&oversized).is_err());
    Ok(())
}

#[test]
fn aggregate_observation_bytes_are_bounded() -> TestResult {
    let (mut expected, mut matrix, _) = fixture()?;
    let template = first(&matrix.cells)?.clone();
    expected.cells.clear();
    matrix.cells.clear();
    for number in 0..70 {
        let mut cell = template.clone();
        cell.expected.key.provider_id = format!("synthetic-{number}");
        cell.legacy.value = json!("x".repeat(60_000));
        cell.shared.value = json!("x".repeat(60_000));
        expected.cells.push(cell.expected.clone());
        matrix.cells.push(cell);
    }
    seal(&mut matrix)?;
    let pin = expected_catalogue_digest(&expected)?;
    assert_eq!(
        validate_execution_matrix(&expected, &pin, &matrix),
        Err(MatrixError::Limit)
    );
    Ok(())
}

#[test]
fn exact_observation_size_limit_and_declared_execution_label_are_preserved() -> TestResult {
    let (mut expected, mut matrix, _) = fixture()?;
    expected.evidence_kind = ParityEvidenceKind::DeclaredExecution;
    matrix.evidence_kind = ParityEvidenceKind::DeclaredExecution;
    first_mut(&mut matrix.cells)?.legacy.value = json!("x".repeat(65_534));
    assert_eq!(
        serde_json::to_vec(&first(&matrix.cells)?.legacy.value)?.len(),
        65_536
    );
    seal(&mut matrix)?;
    let pin = expected_catalogue_digest(&expected)?;
    let result = validate_execution_matrix(&expected, &pin, &matrix)?;
    assert_eq!(result.evidence_kind, ParityEvidenceKind::DeclaredExecution);
    assert!(!first(&result.cells)?.equal);
    assert!(first(&result.cells)?.executed);
    first_mut(&mut matrix.cells)?.legacy.value = json!("x".repeat(65_535));
    assert_eq!(seal(&mut matrix), Err(MatrixError::Limit));
    Ok(())
}

#[test]
fn catalogue_case_dimension_assignments_exactly_match_v1_source_catalogue() -> TestResult {
    let source: serde_json::Value = serde_json::from_str(include_str!(
        "../../../migration/sourceright/parity-cases.json"
    ))?;
    let cases = source
        .get("cases")
        .and_then(serde_json::Value::as_array)
        .ok_or("cases")?;
    let (mut expected, _, _) = fixture()?;
    expected.cells.truncate(1);
    let mut assignments = 0;
    for case in cases {
        let id = case
            .get("case_id")
            .and_then(serde_json::Value::as_str)
            .ok_or("case id")?;
        let dimensions = case
            .get("dimensions")
            .and_then(serde_json::Value::as_array)
            .ok_or("dimensions")?;
        for dimension in searchright_contracts::SOURCERIGHT_PARITY_DIMENSIONS {
            let cell = first_mut(&mut expected.cells)?;
            cell.key.case_id = id.into();
            cell.key.dimension = (*dimension).into();
            let declared = dimensions
                .iter()
                .any(|value| value.as_str() == Some(dimension));
            assert_eq!(expected_catalogue_digest(&expected).is_ok(), declared);
            assignments += usize::from(declared);
        }
    }
    assert_eq!(cases.len(), 7);
    assert_eq!(assignments, 20);
    Ok(())
}
