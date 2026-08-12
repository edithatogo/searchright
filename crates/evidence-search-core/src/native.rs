use evidence_search_contracts::{
    NATIVE_SEARCH_STRATEGY_SCHEMA_VERSION, NativeNormalisationState, NativeParseDiagnostic,
    NativeParseSeverity, NativeQueryLine, NativeQueryLineKind, NativeSearchStrategy,
    NativeSourceSpan, SearchDialect,
};

/// Native parser version recorded in every source-preserving parse.
pub const NATIVE_PARSER_VERSION: &str =
    concat!("evidence-search-native/", env!("CARGO_PKG_VERSION"));

/// Parse native text without pretending that a portable semantic model is complete.
///
/// The first implementation is deliberately lexical: it preserves exact text,
/// line identifiers, comments, limit commands and common set-combination forms.
/// Dialect-specific semantic parsers can refine the result without replacing
/// or rewriting the immutable native representation.
#[must_use]
pub fn parse_native_strategy(
    strategy_id: impl Into<String>,
    dialect: SearchDialect,
    raw_text: impl Into<String>,
) -> NativeSearchStrategy {
    let strategy_id = strategy_id.into();
    let raw_text = raw_text.into();
    let mut lines = Vec::new();
    let mut diagnostics = Vec::new();
    let mut byte_offset = 0_u64;

    for (index, raw_line) in raw_text.split_inclusive('\n').enumerate() {
        let text = raw_line.strip_suffix('\n').unwrap_or(raw_line);
        let text = text.strip_suffix('\r').unwrap_or(text);
        let start = byte_offset;
        let end = start.saturating_add(u64::try_from(text.len()).unwrap_or(u64::MAX));
        byte_offset = byte_offset.saturating_add(u64::try_from(raw_line.len()).unwrap_or(u64::MAX));
        let trimmed = text.trim();
        let (native_set_id, body) = split_native_set_id(&dialect, trimmed);
        let lower = body.to_ascii_lowercase();
        let kind = if trimmed.is_empty() {
            NativeQueryLineKind::Blank
        } else if (trimmed.starts_with('#') && native_set_id.is_none()) || trimmed.starts_with("//")
        {
            NativeQueryLineKind::Comment
        } else if lower.starts_with("limit ") || lower.starts_with("limits:") {
            NativeQueryLineKind::Limit
        } else if looks_like_set_combination(&lower) {
            NativeQueryLineKind::SetCombination
        } else if body.is_empty() {
            diagnostics.push(NativeParseDiagnostic {
                code: "native.empty_numbered_line".to_owned(),
                severity: NativeParseSeverity::Warning,
                message: "numbered native line has no expression".to_owned(),
                span: Some(NativeSourceSpan {
                    start_byte: start,
                    end_byte: end,
                }),
                review_required: true,
            });
            NativeQueryLineKind::Unknown
        } else {
            NativeQueryLineKind::Expression
        };
        lines.push(NativeQueryLine {
            line_id: format!("line-{:04}", index + 1),
            native_set_id,
            text: text.to_owned(),
            kind,
            span: NativeSourceSpan {
                start_byte: start,
                end_byte: end,
            },
        });
        if kind == NativeQueryLineKind::Expression && !is_supported_expression(&dialect, body) {
            diagnostics.push(NativeParseDiagnostic {
                code: format!("native.{}.unsupported_syntax", dialect_code(&dialect)),
                severity: NativeParseSeverity::Warning,
                message: format!(
                    "line is outside the declared {} native syntax subset; exact source was preserved",
                    dialect_code(&dialect)
                ),
                span: Some(NativeSourceSpan {
                    start_byte: start,
                    end_byte: end,
                }),
                review_required: true,
            });
        }
    }
    if raw_text.is_empty() {
        diagnostics.push(NativeParseDiagnostic {
            code: "native.empty_strategy".to_owned(),
            severity: NativeParseSeverity::Error,
            message: "native search strategy is empty".to_owned(),
            span: None,
            review_required: true,
        });
    }
    if !raw_text.is_empty() && !raw_text.ends_with('\n') && lines.is_empty() {
        // `split_inclusive` yields a line for non-empty input, so this is an
        // explicit defensive invariant rather than a reachable normal path.
        diagnostics.push(NativeParseDiagnostic {
            code: "native.parser_invariant".to_owned(),
            severity: NativeParseSeverity::Error,
            message: "parser did not preserve a non-empty final line".to_owned(),
            span: None,
            review_required: true,
        });
    }

    NativeSearchStrategy {
        schema_version: NATIVE_SEARCH_STRATEGY_SCHEMA_VERSION.to_owned(),
        strategy_id,
        dialect,
        raw_text,
        lines,
        semantic_strategy: None,
        normalisation_state: NativeNormalisationState::RawOnly,
        diagnostics,
        parser_version: NATIVE_PARSER_VERSION.to_owned(),
    }
}

fn split_native_set_id<'a>(dialect: &SearchDialect, value: &'a str) -> (Option<String>, &'a str) {
    if matches!(dialect, SearchDialect::Embase)
        && let Some((identifier, body)) = split_prefixed_set_id(value, '#')
    {
        return (Some(identifier), body);
    }
    if matches!(dialect, SearchDialect::CinahlEbsco)
        && let Some((identifier, body)) = split_prefixed_set_id(value, 'S')
    {
        return (Some(identifier), body);
    }
    let digits = value.bytes().take_while(u8::is_ascii_digit).count();
    if digits == 0 {
        return (None, value);
    }
    let remainder = &value[digits..];
    let separator = remainder
        .strip_prefix('.')
        .or_else(|| remainder.strip_prefix(':'))
        .unwrap_or(remainder);
    let begins_with_whitespace = remainder.chars().next().is_some_and(char::is_whitespace);
    if separator.len() == remainder.len() && !begins_with_whitespace {
        return (None, value);
    }
    (Some(value[..digits].to_owned()), separator.trim_start())
}

fn split_prefixed_set_id(value: &str, prefix: char) -> Option<(String, &str)> {
    let remainder = value.strip_prefix(prefix)?;
    let digits = remainder.bytes().take_while(u8::is_ascii_digit).count();
    if digits == 0 {
        return None;
    }
    let after_digits = &remainder[digits..];
    if !after_digits.chars().next().is_some_and(char::is_whitespace) {
        return None;
    }
    let identifier_len = prefix.len_utf8().saturating_add(digits);
    Some((
        value[..identifier_len].to_owned(),
        after_digits.trim_start(),
    ))
}

fn dialect_code(dialect: &SearchDialect) -> &'static str {
    match dialect {
        SearchDialect::PubMed => "pubmed",
        SearchDialect::OvidMedline => "ovid_medline",
        SearchDialect::Embase => "embase",
        SearchDialect::CinahlEbsco => "cinahl_ebsco",
        SearchDialect::PsycInfoOvid => "psycinfo_ovid",
        SearchDialect::Scopus => "scopus",
        SearchDialect::WebOfScience => "web_of_science",
        SearchDialect::EuropePmc => "europe_pmc",
        SearchDialect::Crossref => "crossref",
        SearchDialect::OpenAlex => "openalex",
        SearchDialect::ClinicalTrialsGov => "clinical_trials_gov",
        SearchDialect::GenericBoolean => "generic_boolean",
        SearchDialect::Custom(_) => "custom",
    }
}

fn is_supported_expression(dialect: &SearchDialect, value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    match dialect {
        SearchDialect::PubMed => {
            value.contains('[')
                && value.contains(']')
                && (lower.contains("[mesh terms]") || lower.contains("[title/abstract]"))
        }
        SearchDialect::OvidMedline => {
            (lower.starts_with("exp ") && value.ends_with('/'))
                || has_ovid_field_suffix(&lower, &["ti", "ab", "kf"])
        }
        SearchDialect::PsycInfoOvid => {
            value.ends_with('/') || has_ovid_field_suffix(&lower, &["ti", "ab", "id"])
        }
        SearchDialect::Embase => {
            (value.starts_with('\'') && lower.contains("'/exp"))
                || has_colon_field_suffix(&lower, &["ti", "ab", "kw"])
        }
        SearchDialect::CinahlEbsco => {
            lower.starts_with("(mh \"")
                || lower.starts_with("mh \"")
                || lower.starts_with("ti ")
                || lower.starts_with("ab ")
        }
        SearchDialect::Scopus => lower.starts_with("title-abs-key(") && value.ends_with(')'),
        SearchDialect::WebOfScience => lower.starts_with("ts=(") && value.ends_with(')'),
        SearchDialect::GenericBoolean => !value.trim().is_empty(),
        SearchDialect::EuropePmc
        | SearchDialect::Crossref
        | SearchDialect::OpenAlex
        | SearchDialect::ClinicalTrialsGov
        | SearchDialect::Custom(_) => false,
    }
}

fn has_ovid_field_suffix(value: &str, supported_fields: &[&str]) -> bool {
    let Some(without_final_dot) = value.strip_suffix('.') else {
        return false;
    };
    let Some(suffix_start) = without_final_dot.rfind('.') else {
        return false;
    };
    without_final_dot[suffix_start + 1..]
        .split(',')
        .all(|field| supported_fields.contains(&field))
}

fn has_colon_field_suffix(value: &str, supported_fields: &[&str]) -> bool {
    let Some((_, fields)) = value.rsplit_once(':') else {
        return false;
    };
    fields
        .split(',')
        .all(|field| supported_fields.contains(&field))
}

fn looks_like_set_combination(value: &str) -> bool {
    let starts = ["and ", "or ", "not ", "adj", "near/", "w/"];
    let has_set_reference = value.bytes().any(|byte| byte.is_ascii_digit());
    has_set_reference
        && (starts.iter().any(|prefix| value.starts_with(prefix))
            || value.contains(" and ")
            || value.contains(" or ")
            || value.contains(" not "))
}

#[cfg(test)]
mod tests {
    use evidence_search_contracts::{NativeQueryLineKind, Validate};

    use super::*;

    #[test]
    fn preserves_numbered_ovid_lines_and_set_combinations() {
        let parsed = parse_native_strategy(
            "strategy-1",
            SearchDialect::OvidMedline,
            "1. exp Genomics/\n2. genome*.ti,ab.\n3. 1 or 2\nlimit 3 to english language\n",
        );
        assert!(parsed.validate().is_ok());
        assert_eq!(parsed.lines.len(), 4);
        assert_eq!(
            parsed
                .lines
                .first()
                .and_then(|line| line.native_set_id.as_deref()),
            Some("1")
        );
        assert_eq!(
            parsed.lines.get(2).map(|line| line.kind),
            Some(NativeQueryLineKind::SetCombination)
        );
        assert_eq!(
            parsed.lines.get(3).map(|line| line.kind),
            Some(NativeQueryLineKind::Limit)
        );
    }

    #[test]
    fn checked_in_native_corpus_is_source_preserving() {
        let fixtures = [
            (
                SearchDialect::PubMed,
                include_str!("../../../contracts/query-corpus/pubmed.txt"),
            ),
            (
                SearchDialect::OvidMedline,
                include_str!("../../../contracts/query-corpus/ovid-medline.txt"),
            ),
            (
                SearchDialect::Embase,
                include_str!("../../../contracts/query-corpus/embase.txt"),
            ),
            (
                SearchDialect::CinahlEbsco,
                include_str!("../../../contracts/query-corpus/cinahl-ebsco.txt"),
            ),
            (
                SearchDialect::PsycInfoOvid,
                include_str!("../../../contracts/query-corpus/psycinfo-ovid.txt"),
            ),
            (
                SearchDialect::Scopus,
                include_str!("../../../contracts/query-corpus/scopus.txt"),
            ),
            (
                SearchDialect::WebOfScience,
                include_str!("../../../contracts/query-corpus/web-of-science.txt"),
            ),
        ];
        for (index, (dialect, text)) in fixtures.into_iter().enumerate() {
            let parsed = parse_native_strategy(format!("fixture-{index}"), dialect, text);
            assert_eq!(parsed.raw_text, text);
            assert!(!parsed.lines.is_empty());
            assert!(parsed.validate().is_ok());
            assert!(
                !parsed
                    .diagnostics
                    .iter()
                    .any(|item| item.code.ends_with(".unsupported_syntax")),
                "checked-in {dialect:?} fixture must stay inside its declared subset"
            );
        }
    }

    #[test]
    fn parses_embase_and_cinahl_set_identifiers_as_queries_not_comments() {
        let embase = parse_native_strategy(
            "embase",
            SearchDialect::Embase,
            "#1 'genomics'/exp OR genome*:ti,ab,kw\n#2 #1 AND #1\n",
        );
        assert_eq!(
            embase
                .lines
                .first()
                .and_then(|line| line.native_set_id.as_deref()),
            Some("#1")
        );
        assert_eq!(
            embase.lines.first().map(|line| line.kind),
            Some(NativeQueryLineKind::Expression)
        );
        assert_eq!(
            embase.lines.get(1).map(|line| line.kind),
            Some(NativeQueryLineKind::SetCombination)
        );

        let cinahl = parse_native_strategy(
            "cinahl",
            SearchDialect::CinahlEbsco,
            "S1 (MH \"Genomics+\") OR TI genome*\nS2 S1 AND S1\n",
        );
        assert_eq!(
            cinahl
                .lines
                .first()
                .and_then(|line| line.native_set_id.as_deref()),
            Some("S1")
        );
        assert_eq!(
            cinahl.lines.get(1).map(|line| line.kind),
            Some(NativeQueryLineKind::SetCombination)
        );
    }

    #[test]
    fn unsupported_dialect_syntax_emits_stable_review_diagnostic() {
        let parsed = parse_native_strategy(
            "unsupported",
            SearchDialect::PubMed,
            "TITLE-ABS-KEY(genome*)\n",
        );
        let diagnostic = parsed
            .diagnostics
            .iter()
            .find(|item| item.code == "native.pubmed.unsupported_syntax");
        assert!(diagnostic.is_some());
        assert!(diagnostic.is_some_and(|item| item.review_required));
        assert!(diagnostic.and_then(|item| item.span).is_some());
        assert_eq!(parsed.raw_text, "TITLE-ABS-KEY(genome*)\n");
    }

    #[test]
    fn every_declared_corpus_dialect_has_a_stable_unsupported_code() {
        let dialects = [
            (SearchDialect::PubMed, "pubmed"),
            (SearchDialect::OvidMedline, "ovid_medline"),
            (SearchDialect::Embase, "embase"),
            (SearchDialect::CinahlEbsco, "cinahl_ebsco"),
            (SearchDialect::PsycInfoOvid, "psycinfo_ovid"),
            (SearchDialect::Scopus, "scopus"),
            (SearchDialect::WebOfScience, "web_of_science"),
        ];
        for (dialect, code) in dialects {
            let parsed = parse_native_strategy("unsupported", dialect, "unsupported-native-node\n");
            assert!(parsed.diagnostics.iter().any(|item| {
                item.code == format!("native.{code}.unsupported_syntax")
                    && item.review_required
                    && item.span.is_some()
            }));
        }
    }

    #[test]
    fn empty_native_strategy_requires_review() {
        let parsed = parse_native_strategy("empty", SearchDialect::GenericBoolean, "");
        assert!(parsed.diagnostics.iter().any(|item| item.review_required));
        assert!(parsed.validate().is_err());
    }

    #[test]
    fn validation_rejects_line_text_that_does_not_match_its_source_span() {
        let mut parsed = parse_native_strategy(
            "tampered",
            SearchDialect::OvidMedline,
            "1. exp Genomics/\n2. genome*.ti,ab.\n",
        );
        assert!(!parsed.lines.is_empty());
        if let Some(line) = parsed.lines.first_mut() {
            line.text = "1. exp Other/".to_owned();
        }
        assert!(parsed.validate().is_err());
    }

    #[test]
    fn validation_rejects_duplicate_native_set_identifiers() {
        let mut parsed = parse_native_strategy(
            "duplicate",
            SearchDialect::OvidMedline,
            "1. exp Genomics/\n2. genome*.ti,ab.\n",
        );
        assert!(parsed.lines.len() >= 2);
        if let Some(line) = parsed.lines.get_mut(1) {
            line.native_set_id = Some("1".to_owned());
        }
        assert!(parsed.validate().is_err());
    }
}
