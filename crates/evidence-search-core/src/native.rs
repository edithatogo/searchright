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
        let (native_set_id, body) = split_native_set_id(trimmed);
        let lower = body.to_ascii_lowercase();
        let kind = if trimmed.is_empty() {
            NativeQueryLineKind::Blank
        } else if trimmed.starts_with('#') || trimmed.starts_with("//") {
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

fn split_native_set_id(value: &str) -> (Option<String>, &str) {
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
