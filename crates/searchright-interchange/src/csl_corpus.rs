//! Stable, lossless CSL-JSON bibliography normalisation for downstream consumers.
//!
//! This is a Searchright-scoped content identity, not RFC 8785/JCS and not a
//! digest of source observations, rights or provenance sidecars. Those have
//! separate immutable revisions in the search-run evidence manifest.

use std::collections::BTreeSet;

use serde_json::{Map, Value};

/// A canonically ordered CSL-JSON bibliography and its BLAKE3 content revision.
#[derive(Debug, Clone, PartialEq)]
pub struct CslBibliography {
    /// Items sorted by stable CSL reference id, with recursive object-key ordering.
    pub references: Vec<Value>,
    /// Searchright bibliography revision, prefixed to distinguish its hash domain.
    pub revision: String,
}

/// Failure to construct an unambiguous canonical bibliography.
#[derive(Debug, thiserror::Error)]
pub enum CslCorpusError {
    /// Malformed JSON.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Root must be an array.
    #[error("CSL-JSON bibliography must be an array")]
    NotArray,
    /// All entries must be CSL-JSON objects.
    #[error("CSL-JSON item at index {0} must be an object")]
    NotObject(usize),
    /// A missing or invalid CSL id cannot be silently replaced with a generated id.
    #[error("CSL-JSON item at index {0} needs a nonblank string id")]
    InvalidId(usize),
    /// All canonical reference ids are unique.
    #[error("duplicate CSL-JSON reference id: {0}")]
    DuplicateId(String),
    /// A CSL item must declare a nonblank type.
    #[error("CSL-JSON item at index {0} needs a nonblank string type")]
    InvalidType(usize),
}

/// Construct a stable, non-destructive bibliography projection from CSL-JSON.
///
/// The output is independent of JSON object key order and record order. This
/// function checks the reference identity boundary; full CSL schema conformance
/// is deliberately delegated to a pinned upstream CSL schema validator.
/// It does not create source identifiers, merge records or discard extra fields.
pub fn canonical_csl_bibliography(input: &str) -> Result<CslBibliography, CslCorpusError> {
    let document: Value = serde_json::from_str(input)?;
    let items = document.as_array().ok_or(CslCorpusError::NotArray)?;
    let mut ids = BTreeSet::new();
    let mut normalised = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let object = item.as_object().ok_or(CslCorpusError::NotObject(index))?;
        let id = object.get("id").and_then(Value::as_str)
            .filter(|id| !id.trim().is_empty())
            .ok_or(CslCorpusError::InvalidId(index))?;
        if !ids.insert(id.to_owned()) {
            return Err(CslCorpusError::DuplicateId(id.to_owned()));
        }
        if object.get("type").and_then(Value::as_str)
            .is_none_or(|kind| kind.trim().is_empty()) {
            return Err(CslCorpusError::InvalidType(index));
        }
        normalised.push((id.to_owned(), sort_json_keys(item)));
    }
    normalised.sort_by(|left, right| left.0.cmp(&right.0));
    let references: Vec<Value> = normalised.into_iter().map(|(_, item)| item).collect();
    let payload = serde_json::to_vec(&references)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"searchright/csl-bibliography/v1\n");
    hasher.update(&payload);
    let revision = format!("blake3:{}", hasher.finalize().to_hex());
    Ok(CslBibliography { references, revision })
}

fn sort_json_keys(value: &Value) -> Value {
    match value {
        Value::Object(fields) => {
            let mut pairs: Vec<_> = fields.iter().collect();
            pairs.sort_by(|(left, _), (right, _)| left.cmp(right));
            let mut sorted = Map::new();
            for (key, item) in pairs {
                sorted.insert(key.to_owned(), sort_json_keys(item));
            }
            Value::Object(sorted)
        }
        Value::Array(items) => Value::Array(items.iter().map(sort_json_keys).collect()),
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_order_and_object_order_do_not_change_revision() {
        let first = r#"[{"id":"z","type":"report","custom":{"b":2,"a":1}},{"type":"article-journal","id":"a","title":"Study"}]"#;
        let second = r#"[{"title":"Study","id":"a","type":"article-journal"},{"custom":{"a":1,"b":2},"type":"report","id":"z"}]"#;
        let a = canonical_csl_bibliography(first).expect("valid bibliography");
        let b = canonical_csl_bibliography(second).expect("valid bibliography");
        assert_eq!(a, b);
        assert_eq!(a.references[0]["id"], "a");
    }

    #[test]
    fn bibliographic_change_creates_new_revision() {
        let a = canonical_csl_bibliography(r#"[{"id":"a","type":"report","title":"Before"}]"#).expect("a");
        let b = canonical_csl_bibliography(r#"[{"id":"a","type":"report","title":"After"}]"#).expect("b");
        assert_ne!(a.revision, b.revision);
    }

    #[test]
    fn rejects_missing_blank_duplicate_and_non_object_ids() {
        assert!(matches!(
            canonical_csl_bibliography(r#"[{"type":"report"}]"#),
            Err(CslCorpusError::InvalidId(0))
        ));
        assert!(matches!(
            canonical_csl_bibliography(r#"[{"id":" ","type":"report"}]"#),
            Err(CslCorpusError::InvalidId(0))
        ));
        assert!(matches!(
            canonical_csl_bibliography(r#"[{"id":"a","type":"report"},{"id":"a","type":"report"}]"#),
            Err(CslCorpusError::DuplicateId(_))
        ));
        assert!(matches!(
            canonical_csl_bibliography("[null]"),
            Err(CslCorpusError::NotObject(0))
        ));
        assert!(matches!(
            canonical_csl_bibliography("{}"),
            Err(CslCorpusError::NotArray)
        ));
    }

    #[test]
    fn requires_type_and_preserves_source_fields() {
        assert!(matches!(
            canonical_csl_bibliography(r#"[{"id":"a"}]"#),
            Err(CslCorpusError::InvalidType(0))
        ));
        let a = canonical_csl_bibliography(r#"[{"id":"a","type":"article-journal","publisher":"Example","author":[{"given":"Jane","family":"Doe"}]}]"#).expect("valid");
        assert_eq!(a.references[0]["publisher"], "Example");
        assert_eq!(a.references[0]["author"][0]["family"], "Doe");
    }

    #[test]
    fn empty_bibliography_has_stable_revision() {
        let a = canonical_csl_bibliography("[]").expect("empty allowed");
        let b = canonical_csl_bibliography("[ ]").expect("empty allowed");
        assert_eq!(a.revision, b.revision);
    }
}
