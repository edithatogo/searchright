use std::{fs, path::Path};

use evidence_search_contracts::{
    FilterApplicability, FilterChecksum, FilterRights, FilterSourceCitation, FilterValidation,
    FilterValidationState, NamedFilterPack, NamedFilterRecord, RedistributionDecision,
    SearchDialect, Validate,
};

fn record() -> NamedFilterRecord {
    NamedFilterRecord {
        filter_id: "synthetic-rct-pubmed".to_owned(),
        version: "1.0.0".to_owned(),
        name: "Synthetic RCT publication-type filter".to_owned(),
        dialect: SearchDialect::PubMed,
        expression: "randomized controlled trial[pt]".to_owned(),
        checksum: FilterChecksum {
            algorithm: "sha256".to_owned(),
            digest: "18599c423f896ed99d33b15840c39b16ff63abb88e5194ffd66a33c336ec1f16".to_owned(),
        },
        source: FilterSourceCitation {
            title: "Searchright synthetic contract fixture".to_owned(),
            citation: "Searchright contributors. Synthetic contract fixture. 2026.".to_owned(),
            source_version: "1.0.0".to_owned(),
            source_uri: None,
        },
        applicability: FilterApplicability {
            source_ids: vec!["synthetic-pubmed-fixture".to_owned()],
            platform_versions: vec!["synthetic-2026".to_owned()],
            intended_use: "Schema and semantic-validation testing only.".to_owned(),
            limitations: vec![
                "Not a validated methodological filter and not suitable for live retrieval claims."
                    .to_owned(),
            ],
        },
        validation: FilterValidation {
            state: FilterValidationState::StructuralOnly,
            reviewer_id: "searchright-contract-test".to_owned(),
            reviewer_role: "automated structural validator".to_owned(),
            method:
                "JSON Schema validation, checksum verification, and Rust semantic invariant tests."
                    .to_owned(),
            evidence_reference: "contracts/named-filters/synthetic-validation-evidence.txt"
                .to_owned(),
            evidence_sha256: "b1ef5ede813af5602aa6b8e95b7825a01460bf27e9887c0ce859e3ea1860a057"
                .to_owned(),
        },
        rights: FilterRights {
            basis: "Repository-authored synthetic expression under the repository licence."
                .to_owned(),
            redistribution: RedistributionDecision::Permitted,
            decided_by: "searchright repository policy".to_owned(),
            evidence_reference: "contracts/named-filters/synthetic-validation-evidence.txt"
                .to_owned(),
        },
        effective_from: "2026-08-13".to_owned(),
        expires_on: "2027-08-13".to_owned(),
    }
}

fn pack() -> NamedFilterPack {
    NamedFilterPack {
        schema_version: "org.searchright.named-filter-pack.v1".to_owned(),
        pack_id: "synthetic-contract-fixture".to_owned(),
        version: "1.0.0".to_owned(),
        title: "Synthetic named-filter contract fixture".to_owned(),
        validated_on: "2026-08-13".to_owned(),
        expires_on: "2027-08-13".to_owned(),
        filters: vec![record()],
    }
}

#[test]
fn canonical_example_deserialises_and_validates() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let encoded = fs::read_to_string(workspace.join("contracts/examples/named-filter-pack.json"))?;
    let parsed: NamedFilterPack = serde_json::from_str(&encoded)?;

    parsed.validate()?;
    assert_eq!(parsed, pack());
    Ok(())
}

#[test]
fn checksum_contract_is_fail_closed() {
    let mut candidate = pack();
    let Some(filter) = candidate.filters.first_mut() else {
        panic!("fixture must contain one filter");
    };
    filter.checksum.algorithm = "md5".to_owned();
    assert!(candidate.validate().is_err());

    let Some(filter) = candidate.filters.first_mut() else {
        panic!("fixture must contain one filter");
    };
    filter.checksum.algorithm = "sha256".to_owned();
    filter.checksum.digest = "A".repeat(64);
    assert!(candidate.validate().is_err());

    let Some(filter) = candidate.filters.first_mut() else {
        panic!("fixture must contain one filter");
    };
    filter.checksum.digest = "0".repeat(64);
    assert!(candidate.validate().is_err());
}

#[test]
fn pack_rejects_duplicate_record_identity() {
    let mut candidate = pack();
    candidate.filters.push(record());
    assert!(candidate.validate().is_err());
}

#[test]
fn pack_rejects_records_outside_validation_window() {
    let mut candidate = pack();
    let Some(filter) = candidate.filters.first_mut() else {
        panic!("fixture must contain one filter");
    };
    filter.expires_on = "2026-08-12".to_owned();
    assert!(candidate.validate().is_err());

    let Some(filter) = candidate.filters.first_mut() else {
        panic!("fixture must contain one filter");
    };
    filter.expires_on = "2027-08-12".to_owned();
    assert!(candidate.validate().is_err());
}

#[test]
fn pack_rejects_exact_text_without_redistribution_permission() {
    let mut candidate = pack();
    let Some(filter) = candidate.filters.first_mut() else {
        panic!("fixture must contain one filter");
    };
    filter.rights.redistribution = RedistributionDecision::ReviewRequired;
    assert!(candidate.validate().is_err());
}

#[test]
fn calendar_dates_are_validated_semantically() {
    let mut candidate = pack();
    let Some(filter) = candidate.filters.first_mut() else {
        panic!("fixture must contain one filter");
    };
    filter.effective_from = "2026-02-29".to_owned();
    assert!(candidate.validate().is_err());

    let Some(filter) = candidate.filters.first_mut() else {
        panic!("fixture must contain one filter");
    };
    filter.effective_from = "2024-02-29".to_owned();
    assert!(candidate.validate().is_ok());
}
