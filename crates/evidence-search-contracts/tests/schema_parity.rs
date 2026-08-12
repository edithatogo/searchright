//! Compiler-backed parity checks for explicitly Rust-owned canonical schemas.

use std::{collections::BTreeSet, fs, path::Path};

use evidence_search_contracts::{rust_owned_schemas, rust_schema_parity_scope};
use serde_json::Value;

#[test]
fn rust_owned_root_fields_match_canonical_schemas() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let entries = rust_owned_schemas();
    assert_eq!(
        entries.len(),
        10,
        "registry changes require an intentional parity review"
    );

    let catalogue: Value = serde_json::from_str(&fs::read_to_string(
        workspace.join("contracts/schema-catalog.json"),
    )?)?;
    let catalogue_entries = catalogue
        .get("entries")
        .and_then(Value::as_array)
        .ok_or("schema catalogue entries must be an array")?;
    let mut registry_ids = BTreeSet::new();
    let mut registry_paths = BTreeSet::new();

    for entry in entries {
        assert!(
            registry_ids.insert(entry.catalogue_id),
            "duplicate registry id"
        );
        assert!(
            registry_paths.insert(entry.canonical_path),
            "duplicate registry path"
        );
        assert!(
            catalogue_entries.iter().any(|item| {
                item.get("id").and_then(Value::as_str) == Some(entry.catalogue_id)
                    && item.get("schema").and_then(Value::as_str) == Some(entry.canonical_path)
            }),
            "registry id/path is not canonical for {}",
            entry.catalogue_id
        );
        let canonical: Value =
            serde_json::from_str(&fs::read_to_string(workspace.join(entry.canonical_path))?)?;
        let generated = serde_json::to_value(&entry.generated)?;

        let generated_properties = property_names(&generated);
        let canonical_properties = property_names(&canonical);
        assert!(
            generated_properties.is_some(),
            "generated root must have properties"
        );
        assert!(
            canonical_properties.is_some(),
            "canonical root must have properties"
        );
        assert_eq!(
            generated_properties, canonical_properties,
            "root wire fields drifted for {}",
            entry.catalogue_id,
        );
    }
    Ok(())
}

#[test]
fn semantic_parity_scope_remains_fail_closed() -> Result<(), Box<dyn std::error::Error>> {
    let scope = rust_schema_parity_scope();
    assert!(!scope.exact_parity);
    assert_eq!(scope.rust_owned_roots, 10);
    assert_eq!(scope.known_losses.len(), 9);

    let encoded = serde_json::to_value(scope)?;
    assert_eq!(
        encoded.get("schema_version").and_then(Value::as_str),
        Some("org.searchright.rust-schema-parity.v1")
    );
    Ok(())
}

fn property_names(schema: &Value) -> Option<BTreeSet<&str>> {
    let properties = schema.get("properties")?.as_object()?;
    if properties.is_empty() {
        return None;
    }
    Some(properties.keys().map(String::as_str).collect())
}
