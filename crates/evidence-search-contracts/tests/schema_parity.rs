//! Compiler-backed parity checks for explicitly Rust-owned canonical schemas.

use std::{collections::BTreeSet, fs, path::Path};

use evidence_search_contracts::{rust_owned_schemas, rust_schema_parity_report};
use serde_json::Value;

#[test]
fn rust_owned_root_fields_match_canonical_schemas() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let entries = rust_owned_schemas();
    assert_eq!(
        entries.len(),
        9,
        "registry changes require an intentional parity review"
    );

    for entry in entries {
        let canonical: Value =
            serde_json::from_str(&fs::read_to_string(workspace.join(entry.canonical_path))?)?;
        let generated = serde_json::to_value(&entry.generated)?;

        assert_eq!(
            property_names(&generated),
            property_names(&canonical),
            "root wire fields drifted for {}",
            entry.catalogue_id,
        );
    }
    Ok(())
}

#[test]
fn semantic_parity_report_remains_fail_closed() -> Result<(), Box<dyn std::error::Error>> {
    let report = rust_schema_parity_report();
    assert!(!report.exact_parity);
    assert_eq!(report.rust_owned_roots, 9);
    assert_eq!(report.root_field_parity, 9);
    assert_eq!(report.exact_parity_roots, 0);
    assert_eq!(report.known_losses.len(), 5);

    let encoded = serde_json::to_value(report)?;
    assert_eq!(
        encoded.get("schema_version").and_then(Value::as_str),
        Some("org.searchright.rust-schema-parity.v1")
    );
    Ok(())
}

fn property_names(schema: &Value) -> BTreeSet<&str> {
    schema
        .get("properties")
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|properties| properties.keys().map(String::as_str))
        .collect()
}
