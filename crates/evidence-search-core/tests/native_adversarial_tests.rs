//! Isolated adversarial regressions: Complete must not silently discard source semantics.

use evidence_search_contracts::{
    NativeNormalisationState, NativeSearchStrategy, QueryExpr, SearchDialect, SearchField,
};
use evidence_search_core::{QueryCompiler, parse_native_semantic_strategy, parse_native_strategy};

fn assert_review_required(parsed: &NativeSearchStrategy) {
    assert_eq!(
        parsed.normalisation_state,
        NativeNormalisationState::RawOnly
    );
    assert!(parsed.semantic_strategy.is_none());
    assert!(
        parsed
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.review_required)
    );
}

#[test]
fn unsupported_native_limits_cannot_disappear_into_complete() {
    for suffix in [
        "humans",
        "english language and humans",
        "english or french language",
    ] {
        let raw = format!("1. cancer.ti.\nlimit 1 to {suffix}\n");
        let parsed = parse_native_strategy("limit", SearchDialect::OvidMedline, raw.clone());
        assert_eq!(parsed.raw_text, raw);
        assert_review_required(&parsed);
        assert!(
            parse_native_semantic_strategy("limit", &SearchDialect::OvidMedline, &raw).is_err()
        );
    }
}

#[test]
fn native_limit_must_use_its_referenced_set_or_fail_closed() {
    let raw = "1. cancer.ti.\n2. diabetes.ti.\nlimit 1 to english language\n";
    let parsed = parse_native_strategy("limit-target", SearchDialect::OvidMedline, raw);
    if parsed.normalisation_state == NativeNormalisationState::Complete {
        let Some(strategy) = parsed.semantic_strategy else {
            panic!("Complete requires a semantic strategy");
        };
        let QueryExpr::Term { term } = strategy.query else {
            panic!("limit 1 must select the first set");
        };
        assert_eq!(term.text, "cancer");
        assert_eq!(strategy.limits.languages, vec!["English"]);
    } else {
        assert_review_required(&parsed);
    }
}

#[test]
fn supported_pubmed_clause_cannot_admit_unknown_tags_or_foreign_tokens() {
    for unsupported in ["\"heart\"[nonsense]", "TITLE(heart)", "TI heart"] {
        let raw = format!("\"cancer\"[title/abstract] OR {unsupported}\n");
        let parsed = parse_native_strategy("mixed", SearchDialect::PubMed, raw.clone());
        assert_eq!(parsed.raw_text, raw);
        assert_review_required(&parsed);
        assert!(parse_native_semantic_strategy("mixed", &SearchDialect::PubMed, &raw).is_err());
    }
}

#[test]
fn undefined_native_set_reference_cannot_become_literal_term() {
    for (dialect, raw) in [
        (SearchDialect::OvidMedline, "1. cancer.ti.\n2. 1 OR 999\n"),
        (SearchDialect::Embase, "#1 cancer:ti\n#2 #1 OR #999\n"),
        (SearchDialect::CinahlEbsco, "S1 TI cancer\nS2 S1 OR S999\n"),
    ] {
        let parsed = parse_native_strategy("unresolved", dialect.clone(), raw);
        assert_eq!(parsed.raw_text, raw);
        assert_review_required(&parsed);
        assert!(parse_native_semantic_strategy("unresolved", &dialect, raw).is_err());
    }
}

#[test]
fn unicode_terms_cannot_swallow_following_boolean_clauses() {
    for (dialect, raw, expected_left) in [
        (
            SearchDialect::GenericBoolean,
            "ééééééééé AND x\n",
            "ééééééééé",
        ),
        (
            SearchDialect::CinahlEbsco,
            "TI éééééééééééééééééééé AND AB x\n",
            "éééééééééééééééééééé",
        ),
    ] {
        let parsed = parse_native_strategy("unicode", dialect, raw);
        assert_eq!(parsed.raw_text, raw);
        if parsed.normalisation_state == NativeNormalisationState::Complete {
            let Some(strategy) = parsed.semantic_strategy else {
                panic!("Complete requires a semantic strategy");
            };
            let QueryExpr::And { children } = strategy.query else {
                panic!("a complete parse must retain AND and both operands");
            };
            let [
                QueryExpr::Term { term: left },
                QueryExpr::Term { term: right },
            ] = children.as_slice()
            else {
                panic!("expected both original terms");
            };
            assert_eq!(left.text, expected_left);
            assert_eq!(right.text, "x");
        } else {
            assert_review_required(&parsed);
        }
    }
}

#[test]
fn scopus_proximity_order_survives_parse_compile_parse() -> Result<(), Box<dyn std::error::Error>> {
    for (operator, expected_order) in [("W/3", false), ("PRE/3", true)] {
        let raw = format!("TITLE(genome) {operator} TITLE(screen)\n");
        let parsed = parse_native_strategy("scopus-proximity", SearchDialect::Scopus, raw);
        assert_eq!(
            parsed.normalisation_state,
            NativeNormalisationState::Complete
        );
        let strategy = parsed
            .semantic_strategy
            .ok_or("missing semantic strategy")?;
        let QueryExpr::Proximity {
            distance, ordered, ..
        } = &strategy.query
        else {
            panic!("expected native proximity expression");
        };
        assert_eq!(*distance, 3);
        assert_eq!(*ordered, expected_order);
        let compiled = QueryCompiler::compile(&strategy, SearchDialect::Scopus)?;
        let reparsed =
            parse_native_semantic_strategy("roundtrip", &SearchDialect::Scopus, &compiled.query)?;
        assert_eq!(reparsed.query, strategy.query);
    }
    Ok(())
}

#[test]
fn cinahl_heading_case_does_not_change_heading_text() {
    for raw in ["(MH \"Genomics+\")\n", "(mh \"Genomics+\")\n"] {
        let parsed = parse_native_strategy("heading", SearchDialect::CinahlEbsco, raw);
        assert_eq!(
            parsed.normalisation_state,
            NativeNormalisationState::Complete
        );
        let Some(strategy) = parsed.semantic_strategy else {
            panic!("Complete requires a semantic strategy");
        };
        let QueryExpr::Term { term } = strategy.query else {
            panic!("expected subject heading");
        };
        assert_eq!(term.text, "Genomics");
        assert_eq!(term.fields, vec![SearchField::SubjectHeading]);
        assert!(term.explode);
    }
}

#[test]
fn cinahl_heading_token_stops_before_following_boolean_clause() {
    for raw in [
        "MH \"Genomics+\" OR TI cancer\n",
        "(MH \"Genomics+\") OR TI cancer\n",
        "(MH \"Genomics+\" ) OR TI cancer\n",
    ] {
        let parsed = parse_native_strategy("heading-tail", SearchDialect::CinahlEbsco, raw);
        assert_eq!(
            parsed.normalisation_state,
            NativeNormalisationState::Complete
        );
        let Some(strategy) = parsed.semantic_strategy else {
            panic!("Complete requires a semantic strategy");
        };
        let QueryExpr::Or { children } = strategy.query else {
            panic!("expected the original disjunction");
        };
        let [
            QueryExpr::Term { term: heading },
            QueryExpr::Term { term: title },
        ] = children.as_slice()
        else {
            panic!("expected heading and title as separate operands");
        };
        assert_eq!(heading.text, "Genomics");
        assert!(heading.explode);
        assert_eq!(heading.fields, vec![SearchField::SubjectHeading]);
        assert_eq!(title.text, "cancer");
        assert_eq!(title.fields, vec![SearchField::Title]);
    }
}

#[test]
fn cinahl_quoted_heading_preserves_delimiters_inside_its_text() {
    for raw in [
        "MH \"Genomics (methods)+\" OR TI cancer\n",
        "(MH \"Genomics (methods)+\") OR TI cancer\n",
    ] {
        let parsed = parse_native_strategy("quoted-delimiters", SearchDialect::CinahlEbsco, raw);
        assert_eq!(
            parsed.normalisation_state,
            NativeNormalisationState::Complete
        );
        let Some(strategy) = parsed.semantic_strategy else {
            panic!("Complete requires a semantic strategy");
        };
        let QueryExpr::Or { children } = strategy.query else {
            panic!("expected separate heading and title");
        };
        let Some(QueryExpr::Term { term }) = children.first() else {
            panic!("expected a heading term");
        };
        assert_eq!(term.text, "Genomics (methods)");
        assert!(term.explode);
    }
}

#[test]
fn malformed_cinahl_heading_delimiters_fail_closed() {
    for raw in [
        "MH \"Genomics+ OR TI cancer\n",
        "(MH \"Genomics+ OR TI cancer)\n",
        "(MH \"Genomics+\" OR TI cancer\n",
        "(MH \"Genomics (methods)+\" OR TI cancer\n",
        "MH Genomics+ OR TI cancer\n",
    ] {
        let parsed = parse_native_strategy("malformed-heading", SearchDialect::CinahlEbsco, raw);
        assert_eq!(parsed.raw_text, raw);
        assert_review_required(&parsed);
        assert!(
            parse_native_semantic_strategy("malformed-heading", &SearchDialect::CinahlEbsco, raw)
                .is_err()
        );
    }
}

#[test]
fn unmatched_leading_term_quotes_cannot_normalize_completely() {
    for (dialect, raw) in [
        (SearchDialect::PubMed, "\"cancer[Title]\n"),
        (SearchDialect::CinahlEbsco, "TI \"cancer\n"),
        (SearchDialect::Embase, "'cancer:ti\n"),
    ] {
        let parsed = parse_native_strategy("unmatched-term-quote", dialect.clone(), raw);
        assert_eq!(parsed.raw_text, raw);
        assert_review_required(&parsed);
        assert!(parse_native_semantic_strategy("unmatched-term-quote", &dialect, raw).is_err());
    }
}

#[test]
fn ordinary_apostrophe_inside_unquoted_term_is_retained() {
    let parsed = parse_native_strategy("apostrophe", SearchDialect::CinahlEbsco, "TI Crohn's\n");
    assert_eq!(
        parsed.normalisation_state,
        NativeNormalisationState::Complete
    );
    let Some(strategy) = parsed.semantic_strategy else {
        panic!("Complete requires a semantic strategy");
    };
    let QueryExpr::Term { term } = strategy.query else {
        panic!("expected an unquoted title term");
    };
    assert_eq!(term.text, "Crohn's");
}

#[test]
fn valid_ovid_set_combination_preserves_both_terms_and_english_limit() {
    let raw = "1. cancer.ti.\n2. diabetes.ab.\n3. 1 OR 2\nlimit 3 to english language\n";
    let parsed = parse_native_strategy("valid-ovid", SearchDialect::OvidMedline, raw);
    assert_eq!(
        parsed.normalisation_state,
        NativeNormalisationState::Complete
    );
    let Some(strategy) = parsed.semantic_strategy else {
        panic!("Complete requires a semantic strategy");
    };
    let QueryExpr::Or { children } = strategy.query else {
        panic!("expected the selected disjunction");
    };
    let [
        QueryExpr::Term { term: left },
        QueryExpr::Term { term: right },
    ] = children.as_slice()
    else {
        panic!("expected the two referenced terms");
    };
    assert_eq!(left.text, "cancer");
    assert_eq!(left.fields, vec![SearchField::Title]);
    assert_eq!(right.text, "diabetes");
    assert_eq!(right.fields, vec![SearchField::Abstract]);
    assert_eq!(strategy.limits.languages, vec!["English"]);
}
