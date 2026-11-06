//! Dedup config and normalisation edge cases.

use searchright_contracts::{
    BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION, BibliographicRecord, RecordIdentifiers, RecordKind,
};
use searchright_dedup::{
    DedupConfig, DedupError, Deduplicator, normalise_doi, normalise_isbn, normalise_unicode_text,
};

fn record(id: &str, title: &str) -> BibliographicRecord {
    BibliographicRecord {
        schema_version: BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION.to_string(),
        record_id: id.to_string(),
        source_receipt_id: "receipt-1".to_string(),
        native_id: id.to_string(),
        kind: RecordKind::JournalArticle,
        identifiers: RecordIdentifiers::default(),
        title: title.to_string(),
        abstract_text: None,
        authors: vec![],
        container_title: None,
        publication_year: None,
        publication_date: None,
        languages: vec![],
        subjects: vec![],
        urls: vec![],
        provider_metadata: serde_json::Value::Null,
    }
}

#[test]
fn normalise_doi_handles_all_resolver_prefixes() {
    for raw in [
        "https://doi.org/10.1000/x",
        "http://doi.org/10.1000/x",
        "https://dx.doi.org/10.1000/x",
        "http://dx.doi.org/10.1000/x",
        "doi.org/10.1000/x",
        "doi:10.1000/x",
        "10.1000/x",
    ] {
        assert_eq!(normalise_doi(raw), "10.1000/x", "for {raw:?}");
    }
}

#[test]
fn normalise_doi_lowercases_and_trims_trailing_punct() {
    assert_eq!(normalise_doi("  HTTPS://DOI.ORG/10.1000/X. "), "10.1000/x");
    assert_eq!(normalise_doi("10.1000/x;"), "10.1000/x");
    assert_eq!(normalise_doi("10.1000/x,"), "10.1000/x");
}

#[test]
fn normalise_isbn_strips_prefixes_and_hyphens() {
    assert_eq!(
        normalise_isbn("ISBN-13: 978-3-16-148410-0"),
        "9783161484100"
    );
    assert_eq!(normalise_isbn("ISBN-10: 3-16-148410-0"), "3161484100");
    assert_eq!(normalise_isbn("isbn:9783161484100"), "9783161484100");
}

#[test]
fn normalise_unicode_maps_diacritics_and_ligatures() {
    assert_eq!(normalise_unicode_text("café"), "cafe");
    assert_eq!(normalise_unicode_text("naïve"), "naive");
    assert_eq!(normalise_unicode_text("œuvre"), "oeuvre");
    // fi ligature is not in the documented map; it is preserved verbatim.
    assert_eq!(normalise_unicode_text("ﬁle"), "ﬁle");
    assert_eq!(normalise_unicode_text("œ"), "oe");
}

#[test]
fn dedup_config_rejects_nan_threshold() {
    assert!(matches!(
        Deduplicator::new(DedupConfig {
            title_similarity_threshold: f64::NAN,
            year_tolerance: 1,
            require_first_author_match: true,
        }),
        Err(DedupError::InvalidTitleThreshold)
    ));
}

#[test]
fn dedup_config_rejects_zero_and_over_one_threshold() {
    for threshold in [0.0, 1.1, -0.5, f64::INFINITY] {
        assert!(matches!(
            Deduplicator::new(DedupConfig {
                title_similarity_threshold: threshold,
                year_tolerance: 0,
                require_first_author_match: false,
            }),
            Err(DedupError::InvalidTitleThreshold)
        ));
    }
}

#[test]
fn dedup_config_rejects_negative_year_tolerance() {
    assert!(matches!(
        Deduplicator::new(DedupConfig {
            title_similarity_threshold: 0.9,
            year_tolerance: -1,
            require_first_author_match: true,
        }),
        Err(DedupError::InvalidYearTolerance)
    ));
}

#[test]
fn dedup_config_accepts_boundary_thresholds() {
    assert!(
        Deduplicator::new(DedupConfig {
            title_similarity_threshold: 1.0,
            year_tolerance: 0,
            require_first_author_match: false,
        })
        .is_ok()
    );
    assert!(
        Deduplicator::new(DedupConfig {
            title_similarity_threshold: 0.01,
            year_tolerance: 0,
            require_first_author_match: false,
        })
        .is_ok()
    );
}

#[test]
fn cluster_empty_records_yields_empty_result() {
    let dedup =
        Deduplicator::new(DedupConfig::default()).unwrap_or_else(|_| panic!("expected dedup"));
    let result = dedup
        .cluster(&[])
        .unwrap_or_else(|_| panic!("expected result"));
    assert_eq!(result.clusters.len(), 0);
    assert_eq!(result.retained_record_ids.len(), 0);
    assert_eq!(result.comparisons, 0);
}

#[test]
fn cluster_single_record_is_retained() {
    let dedup =
        Deduplicator::new(DedupConfig::default()).unwrap_or_else(|_| panic!("expected dedup"));
    let result = dedup
        .cluster(&[record("r1", "A unique title")])
        .unwrap_or_else(|_| panic!("expected result"));
    assert_eq!(result.clusters.len(), 0);
    assert_eq!(result.retained_record_ids, vec!["r1"]);
    assert_eq!(result.comparisons, 0);
}

#[test]
fn cluster_duplicate_id_is_rejected() {
    let dedup =
        Deduplicator::new(DedupConfig::default()).unwrap_or_else(|_| panic!("expected dedup"));
    let rec = record("r1", "Title");
    assert!(matches!(
        dedup.cluster(&[rec.clone(), rec]),
        Err(DedupError::DuplicateRecordId(_))
    ));
}

#[test]
fn cluster_empty_id_is_rejected() {
    let dedup =
        Deduplicator::new(DedupConfig::default()).unwrap_or_else(|_| panic!("expected dedup"));
    let mut rec = record("r1", "Title");
    rec.record_id = "  ".to_string();
    assert!(matches!(
        dedup.cluster(&[rec]),
        Err(DedupError::EmptyRecordId)
    ));
}

#[test]
fn identical_records_cluster_with_review_evidence() {
    let dedup =
        Deduplicator::new(DedupConfig::default()).unwrap_or_else(|_| panic!("expected dedup"));
    let result = dedup
        .cluster(&[
            record("r1", "Machine learning for screening"),
            record("r2", "Machine learning for screening"),
        ])
        .unwrap_or_else(|_| panic!("expected result"));
    assert_eq!(result.clusters.len(), 1);
    assert_eq!(result.proposed_duplicate_count, 1);
    let cluster = result
        .clusters
        .first()
        .unwrap_or_else(|| panic!("expected cluster"));
    assert!(cluster.record_ids.contains(&"r1".to_string()));
    assert!(cluster.record_ids.contains(&"r2".to_string()));
}
