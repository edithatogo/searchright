//! Property-based testing for query AST compilation, determinism, and semantic invariants.

use evidence_search_contracts::{
    QueryExpr, SearchDialect, SearchField, SearchLimit, SearchStrategy, SearchTerm, Validate,
};
use evidence_search_core::{QueryCompiler, parse_native_strategy};
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

fn arb_native_boolean_expr() -> impl Strategy<Value = QueryExpr> {
    // Every leaf contains Unicode; the ASCII prefix avoids reserved operator words.
    let leaf = (
        "[a-zα中]{1,6}",
        prop_oneof![Just(SearchField::Title), Just(SearchField::TitleAbstract)],
        any::<bool>(),
    )
        .prop_map(|(suffix, field, truncation)| QueryExpr::Term {
            term: SearchTerm {
                text: format!("qé{suffix}"),
                fields: vec![field],
                vocabulary: None,
                explode: false,
                phrase: false,
                truncation,
            },
        });
    leaf.prop_recursive(3, 12, 3, |inner| {
        prop_oneof![
            proptest::collection::vec(inner.clone(), 2..=3)
                .prop_map(|children| QueryExpr::And { children }),
            proptest::collection::vec(inner.clone(), 2..=3)
                .prop_map(|children| QueryExpr::Or { children }),
            (inner.clone(), inner).prop_map(|(include, exclude)| QueryExpr::Not {
                include: Box::new(include),
                exclude: Box::new(exclude),
            }),
        ]
    })
}

// Normalize only Boolean associativity, not fields, term text, order, or negation.
// This avoids treating redundant grouping as a semantic change.
fn canonical_boolean_grouping(expr: QueryExpr) -> QueryExpr {
    match expr {
        QueryExpr::And { children } => QueryExpr::And {
            children: children
                .into_iter()
                .map(canonical_boolean_grouping)
                .flat_map(|child| match child {
                    QueryExpr::And { children } => children,
                    other => vec![other],
                })
                .collect(),
        },
        QueryExpr::Or { children } => QueryExpr::Or {
            children: children
                .into_iter()
                .map(canonical_boolean_grouping)
                .flat_map(|child| match child {
                    QueryExpr::Or { children } => children,
                    other => vec![other],
                })
                .collect(),
        },
        QueryExpr::Not { include, exclude } => QueryExpr::Not {
            include: Box::new(canonical_boolean_grouping(*include)),
            exclude: Box::new(canonical_boolean_grouping(*exclude)),
        },
        other => other,
    }
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

    #[test]
    fn prop_declared_dialect_subset_compile_parse_compile_is_stable(
        left_text in "[a-z]{2,10}",
        right_text in "[a-z]{2,10}",
        left_truncation in any::<bool>(),
        disjunction in any::<bool>(),
        dialect in prop_oneof![
            Just(SearchDialect::PubMed),
            Just(SearchDialect::OvidMedline),
            Just(SearchDialect::Embase),
            Just(SearchDialect::CinahlEbsco),
            Just(SearchDialect::PsycInfoOvid),
            Just(SearchDialect::Scopus),
            Just(SearchDialect::WebOfScience),
        ],
    ) {
        let children = vec![
            QueryExpr::Term {
                term: SearchTerm {
                    text: left_text,
                    fields: vec![SearchField::TitleAbstract],
                    vocabulary: None,
                    explode: false,
                    phrase: false,
                    truncation: left_truncation,
                },
            },
            QueryExpr::Term {
                term: SearchTerm {
                    text: right_text,
                    fields: vec![SearchField::Title],
                    vocabulary: None,
                    explode: false,
                    phrase: false,
                    truncation: false,
                },
            },
        ];
        let query = if disjunction {
            QueryExpr::Or { children }
        } else {
            QueryExpr::And { children }
        };
        let strategy = SearchStrategy {
            schema_version: "org.searchright.search-strategy.v1".to_owned(),
            strategy_id: "prop-native-round-trip".to_owned(),
            review_id: "prop-review".to_owned(),
            source_id: "prop-source".to_owned(),
            dialect: dialect.clone(),
            query,
            limits: SearchLimit::default(),
            translated_from: None,
            notes: Vec::new(),
        };
        let compiled = QueryCompiler::compile(&strategy, dialect.clone());
        prop_assert!(compiled.is_ok());
        if let Ok(compiled) = compiled {
            let parsed = parse_native_strategy(
                "prop-native-round-trip",
                dialect.clone(),
                format!("{}\n", compiled.query),
            );
            prop_assert_eq!(
                &parsed.normalisation_state,
                &evidence_search_contracts::NativeNormalisationState::Complete
            );
            prop_assert!(parsed.semantic_strategy.is_some());
            if let Some(reparsed) = parsed.semantic_strategy {
                let recompiled = QueryCompiler::compile(&reparsed, dialect);
                prop_assert!(recompiled.is_ok());
                if let Ok(recompiled) = recompiled {
                    prop_assert_eq!(recompiled.query, compiled.query);
                }
            }
        }
    }

    #[test]
    fn prop_recursive_unicode_boolean_subset_roundtrips_within_each_dialect(
        query in arb_native_boolean_expr(),
    ) {
        let query = canonical_boolean_grouping(query);
        for dialect in [
            SearchDialect::PubMed,
            SearchDialect::OvidMedline,
            SearchDialect::Embase,
            SearchDialect::CinahlEbsco,
            SearchDialect::PsycInfoOvid,
            SearchDialect::Scopus,
            SearchDialect::WebOfScience,
        ] {
            let strategy = SearchStrategy {
                schema_version: "org.searchright.search-strategy.v1".to_owned(),
                strategy_id: "recursive-unicode".to_owned(),
                review_id: "property-review".to_owned(),
                source_id: "synthetic-fixture".to_owned(),
                dialect: dialect.clone(),
                query: query.clone(),
                limits: SearchLimit::default(),
                translated_from: None,
                notes: Vec::new(),
            };
            let compiled = QueryCompiler::compile(&strategy, dialect.clone())?;
            let parsed = parse_native_strategy("recursive-unicode", dialect.clone(), compiled.query.clone());
            prop_assert_eq!(
                parsed.normalisation_state,
                evidence_search_contracts::NativeNormalisationState::Complete,
                "dialect {:?}, query {:?}, diagnostics {:?}", dialect, compiled.query, parsed.diagnostics
            );
            let reparsed = parsed.semantic_strategy.ok_or_else(||
                TestCaseError::fail("Complete parse omitted semantic strategy")
            )?;
            // CINAHL expands TitleAbstract into two fields. WoS explicitly reports
            // field degradation to Topic. Neither representation promises source-AST identity.
            if !matches!(dialect, SearchDialect::CinahlEbsco | SearchDialect::WebOfScience) {
                prop_assert_eq!(canonical_boolean_grouping(reparsed.query.clone()), query.clone());
            }
            let recompiled = QueryCompiler::compile(&reparsed, dialect.clone())?;
            prop_assert_eq!(&recompiled.query, &compiled.query, "dialect {:?}", dialect);
            let second = parse_native_strategy("recursive-unicode-second", dialect, recompiled.query);
            prop_assert_eq!(second.normalisation_state, evidence_search_contracts::NativeNormalisationState::Complete);
            let second_strategy = second.semantic_strategy.ok_or_else(||
                TestCaseError::fail("second Complete parse omitted semantic strategy")
            )?;
            prop_assert_eq!(second_strategy.query, reparsed.query);
        }
    }
}
