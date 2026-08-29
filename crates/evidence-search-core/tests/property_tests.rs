//! Property-based testing for query AST compilation, determinism, and semantic invariants.

use evidence_search_contracts::{
    QueryExpr, SearchDialect, SearchField, SearchLimit, SearchStrategy, SearchTerm, Validate,
};
use evidence_search_core::QueryCompiler;
use proptest::prelude::*;

fn arb_search_field() -> impl Strategy<Value = SearchField> {
    prop_oneof![
        Just(SearchField::All),
        Just(SearchField::Title),
        Just(SearchField::Abstract),
        Just(SearchField::TitleAbstract),
        Just(SearchField::Author),
        Just(SearchField::Journal),
        Just(SearchField::SubjectHeading),
        Just(SearchField::Keyword),
    ]
}

fn arb_search_term() -> impl Strategy<Value = SearchTerm> {
    (
        "[a-zA-Z0-9]{2,12}",
        proptest::collection::vec(arb_search_field(), 0..3),
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(|(text, fields, phrase, truncation)| {
            let mut unique_fields = Vec::new();
            for f in fields {
                if !unique_fields.contains(&f) {
                    unique_fields.push(f);
                }
            }
            SearchTerm {
                text,
                fields: unique_fields,
                vocabulary: None,
                explode: false,
                phrase,
                truncation,
            }
        })
}

fn arb_query_expr() -> impl Strategy<Value = QueryExpr> {
    let leaf = arb_search_term().prop_map(|term| QueryExpr::Term { term });
    leaf.prop_recursive(3, 16, 4, |inner| {
        prop_oneof![
            proptest::collection::vec(inner.clone(), 2..4)
                .prop_map(|children| QueryExpr::And { children }),
            proptest::collection::vec(inner.clone(), 2..4)
                .prop_map(|children| QueryExpr::Or { children }),
            (inner.clone(), inner.clone()).prop_map(|(include, exclude)| QueryExpr::Not {
                include: Box::new(include),
                exclude: Box::new(exclude),
            }),
            (inner.clone(), inner, 1..10_u16, any::<bool>()).prop_map(
                |(left, right, distance, ordered)| QueryExpr::Proximity {
                    left: Box::new(left),
                    right: Box::new(right),
                    distance,
                    ordered,
                }
            ),
        ]
    })
}

fn arb_dialect() -> impl Strategy<Value = SearchDialect> {
    prop_oneof![
        Just(SearchDialect::PubMed),
        Just(SearchDialect::OvidMedline),
        Just(SearchDialect::Embase),
        Just(SearchDialect::CinahlEbsco),
        Just(SearchDialect::PsycInfoOvid),
        Just(SearchDialect::Scopus),
        Just(SearchDialect::WebOfScience),
        Just(SearchDialect::EuropePmc),
        Just(SearchDialect::Crossref),
        Just(SearchDialect::OpenAlex),
        Just(SearchDialect::ClinicalTrialsGov),
        Just(SearchDialect::GenericBoolean),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_compilation_is_deterministic(
        expr in arb_query_expr(),
        dialect in arb_dialect(),
    ) {
        let strategy = SearchStrategy {
            schema_version: "org.searchright.search-strategy.v1".to_owned(),
            strategy_id: "prop-test".to_owned(),
            review_id: "prop-review".to_owned(),
            source_id: "prop-source".to_owned(),
            dialect: dialect.clone(),
            query: expr,
            limits: SearchLimit::default(),
            translated_from: None,
            notes: Vec::new(),
        };
        if strategy.validate().is_ok() {
            let left = QueryCompiler::compile(&strategy, dialect.clone());
            let right = QueryCompiler::compile(&strategy, dialect);
            prop_assert_eq!(&left.is_ok(), &right.is_ok());
            if let (Ok(left), Ok(right)) = (left, right) {
                prop_assert_eq!(left.query, right.query);
                prop_assert_eq!(left.fidelity, right.fidelity);
                prop_assert_eq!(left.warnings, right.warnings);
                prop_assert_eq!(left.loss_codes, right.loss_codes);
                prop_assert_eq!(left.compilation_hash, right.compilation_hash);
            }
        }
    }

    #[test]
    fn prop_canonical_json_roundtrip_preserves_compilation(
        expr in arb_query_expr(),
        dialect in arb_dialect(),
    ) {
        let strategy = SearchStrategy {
            schema_version: "org.searchright.search-strategy.v1".to_owned(),
            strategy_id: "prop-json".to_owned(),
            review_id: "prop-review".to_owned(),
            source_id: "prop-source".to_owned(),
            dialect: dialect.clone(),
            query: expr,
            limits: SearchLimit::default(),
            translated_from: None,
            notes: Vec::new(),
        };
        if strategy.validate().is_ok() {
            let serialized = serde_json::to_vec(&strategy);
            prop_assert!(serialized.is_ok());
            if let Ok(bytes) = serialized {
                let deserialized: Result<SearchStrategy, _> = serde_json::from_slice(&bytes);
                prop_assert!(deserialized.is_ok());
                if let Ok(decoded) = deserialized {
                    let compiled_orig = QueryCompiler::compile(&strategy, dialect.clone());
                    let compiled_decoded = QueryCompiler::compile(&decoded, dialect);
                    if let (Ok(c1), Ok(c2)) = (compiled_orig, compiled_decoded) {
                        prop_assert_eq!(c1, c2);
                    }
                }
            }
        }
    }

    #[test]
    fn prop_proximity_on_lossy_dialects_requires_review(
        expr in arb_query_expr(),
    ) {
        let prox = QueryExpr::Proximity {
            left: Box::new(expr.clone()),
            right: Box::new(expr),
            distance: 3,
            ordered: false,
        };
        let strategy = SearchStrategy {
            schema_version: "org.searchright.search-strategy.v1".to_owned(),
            strategy_id: "prop-prox".to_owned(),
            review_id: "prop-review".to_owned(),
            source_id: "prop-source".to_owned(),
            dialect: SearchDialect::Crossref,
            query: prox,
            limits: SearchLimit::default(),
            translated_from: None,
            notes: Vec::new(),
        };
        if strategy.validate().is_ok() {
            let compiled = QueryCompiler::compile(&strategy, SearchDialect::Crossref);
            if let Ok(res) = compiled {
                prop_assert!(res.review_required);
                prop_assert!(!res.loss_codes.is_empty());
            }
        }
    }
}
