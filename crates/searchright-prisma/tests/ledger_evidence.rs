//! Evidence presence is not proved by a nonempty collection of blank strings.
use searchright_contracts::{PrismaSItem, PrismaSItemStatus};
use searchright_prisma::{SearchReportingEvidence, prisma_s_ledger};

#[test]
fn all_collection_items_fail_closed_for_blank_and_mixed_evidence() {
    for (values, expected) in [
        (Vec::new(), PrismaSItemStatus::Missing),
        (vec![String::new()], PrismaSItemStatus::Missing),
        (
            vec![" \t\r\n\u{2003}".to_owned()],
            PrismaSItemStatus::Missing,
        ),
        (
            vec!["  receipt:1  ".to_owned(), "\t".to_owned()],
            PrismaSItemStatus::Partial,
        ),
        (
            vec!["\t".to_owned(), "  receipt:1  ".to_owned()],
            PrismaSItemStatus::Partial,
        ),
        (
            vec!["  receipt:1  ".to_owned(), "receipt:2".to_owned()],
            PrismaSItemStatus::Complete,
        ),
    ] {
        let evidence = SearchReportingEvidence {
            database_names: values.clone(),
            registries: values.clone(),
            full_strategy_evidence: values.clone(),
            search_date_evidence: values.clone(),
            peer_review_evidence: values.clone(),
            total_record_evidence: values.clone(),
            deduplication_evidence: values.clone(),
            ..SearchReportingEvidence::default()
        };
        let ledger = prisma_s_ledger(&evidence);
        let mut checked = 0;
        for row in ledger {
            if matches!(
                row.item,
                PrismaSItem::DatabaseName
                    | PrismaSItem::StudyRegistries
                    | PrismaSItem::FullSearchStrategies
                    | PrismaSItem::DatesOfSearches
                    | PrismaSItem::PeerReview
                    | PrismaSItem::TotalRecords
                    | PrismaSItem::Deduplication
            ) {
                assert_eq!(row.status, expected, "{:?}: {values:?}", row.item);
                if expected == PrismaSItemStatus::Partial {
                    assert!(row.note.contains("blank"));
                    assert!(row.note.contains("incomplete"));
                }
                assert_eq!(
                    row.evidence, values,
                    "provided bytes must not be silently rewritten"
                );
                checked += 1;
            }
        }
        assert_eq!(checked, 7);
    }
}
