use std::collections::BTreeMap;

use evidence_search_contracts::{
    NATIVE_SEARCH_STRATEGY_SCHEMA_VERSION, NativeNormalisationState, NativeParseDiagnostic,
    NativeParseSeverity, NativeQueryLine, NativeQueryLineKind, NativeSearchStrategy,
    NativeSourceSpan, QueryExpr, SEARCH_STRATEGY_SCHEMA_VERSION, SearchDialect, SearchField,
    SearchLimit, SearchStrategy, SearchTerm, Validate,
};

/// Native parser version recorded in every source-preserving parse.
pub const NATIVE_PARSER_VERSION: &str =
    concat!("evidence-search-native/", env!("CARGO_PKG_VERSION"));

/// Parse native text into a source-preserving representation and attempt semantic normalization.
///
/// Exact text, byte spans, line identifiers, comments and limit commands are always preserved.
/// When the native syntax belongs to a recognized dialect subset, a normalized
/// `SearchStrategy` AST is extracted and validated.
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

    let has_unsupported = diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.ends_with(".unsupported_syntax"));

    let (semantic_strategy, normalisation_state) = if has_unsupported || raw_text.is_empty() {
        (None, NativeNormalisationState::RawOnly)
    } else {
        match parse_native_semantic_strategy(&strategy_id, &dialect, &raw_text) {
            Ok(strategy) if diagnostics.is_empty() => {
                (Some(strategy), NativeNormalisationState::Complete)
            }
            Ok(strategy) => (Some(strategy), NativeNormalisationState::Partial),
            Err(_) => (None, NativeNormalisationState::RawOnly),
        }
    };

    NativeSearchStrategy {
        schema_version: NATIVE_SEARCH_STRATEGY_SCHEMA_VERSION.to_owned(),
        strategy_id,
        dialect,
        raw_text,
        lines,
        semantic_strategy,
        normalisation_state,
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

const fn dialect_code(dialect: &SearchDialect) -> &'static str {
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
            (value.starts_with('\'') && (lower.contains("'/exp") || lower.contains("'/de")))
                || has_colon_field_suffix(&lower, &["ti", "ab", "kw", "all"])
        }
        SearchDialect::CinahlEbsco => {
            lower.starts_with("(mh \"")
                || lower.starts_with("mh \"")
                || lower.starts_with("ti ")
                || lower.starts_with("ab ")
        }
        SearchDialect::Scopus => {
            (lower.starts_with("title-abs-key(")
                || lower.starts_with("title(")
                || lower.starts_with("abs("))
                && value.ends_with(')')
        }
        SearchDialect::WebOfScience => {
            (lower.starts_with("ts=(") || lower.starts_with("ti=(")) && value.ends_with(')')
        }
        SearchDialect::GenericBoolean => !value.trim().is_empty(),
        SearchDialect::EuropePmc
        | SearchDialect::Crossref
        | SearchDialect::OpenAlex
        | SearchDialect::ClinicalTrialsGov
        | SearchDialect::Custom(_) => false,
    }
}

fn has_ovid_field_suffix(value: &str, supported_fields: &[&str]) -> bool {
    let clean = value.trim_end_matches(')');
    let Some(without_final_dot) = clean.strip_suffix('.') else {
        return false;
    };
    let Some(suffix_start) = without_final_dot.rfind('.') else {
        return false;
    };
    without_final_dot[suffix_start + 1..]
        .split(',')
        .all(|field| supported_fields.contains(&field.trim()))
}

fn has_colon_field_suffix(value: &str, supported_fields: &[&str]) -> bool {
    let clean = value.trim_end_matches(')');
    let Some((_, fields)) = clean.rsplit_once(':') else {
        return false;
    };
    let fields_clean = fields.trim_end_matches(')');
    fields_clean
        .split(',')
        .all(|field| supported_fields.contains(&field.trim()))
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

/// Attempt to parse source-native search strategy text into a structured semantic `SearchStrategy`.
pub fn parse_native_semantic_strategy(
    strategy_id: &str,
    dialect: &SearchDialect,
    raw_text: &str,
) -> Result<SearchStrategy, String> {
    if raw_text.trim().is_empty() {
        return Err("native strategy is empty".to_owned());
    }
    let mut set_env: BTreeMap<String, QueryExpr> = BTreeMap::new();
    let mut last_expr: Option<QueryExpr> = None;
    let mut notes = Vec::new();
    let mut limits = SearchLimit::default();

    for raw_line in raw_text.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let (native_set_id, body) = split_native_set_id(dialect, trimmed);
        let lower = body.to_ascii_lowercase();

        if (trimmed.starts_with('#') && native_set_id.is_none()) || trimmed.starts_with("//") {
            notes.push(trimmed.to_owned());
            continue;
        }

        if lower.starts_with("limit ") || lower.starts_with("limits:") {
            parse_limit_line(&lower, &mut limits);
            continue;
        }

        if body.is_empty() {
            continue;
        }

        let expr = parse_expression_body(dialect, body, &set_env)?;
        if let Some(set_id) = &native_set_id {
            set_env.insert(set_id.clone(), expr.clone());
            let stripped = set_id.trim_start_matches(['#', 'S', 's']);
            if stripped != set_id {
                set_env.insert(stripped.to_owned(), expr.clone());
            }
        }
        last_expr = Some(expr);
    }

    let query = last_expr.ok_or_else(|| "no searchable expression found".to_owned())?;
    query.validate().map_err(|err| err.to_string())?;

    let strategy = SearchStrategy {
        schema_version: SEARCH_STRATEGY_SCHEMA_VERSION.to_owned(),
        strategy_id: strategy_id.to_owned(),
        review_id: format!("review-{strategy_id}"),
        source_id: dialect_code(dialect).to_owned(),
        dialect: dialect.clone(),
        query,
        limits,
        translated_from: None,
        notes,
    };
    strategy.validate().map_err(|err| err.to_string())?;
    Ok(strategy)
}

fn parse_limit_line(lower: &str, limits: &mut SearchLimit) {
    if lower.contains("english") {
        if !limits
            .languages
            .iter()
            .any(|lang| lang.eq_ignore_ascii_case("english"))
        {
            limits.languages.push("English".to_owned());
        }
        if limits.rationale.is_empty() {
            limits
                .rationale
                .push("Strategy language restriction".to_owned());
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    LParen,
    LParenWithFields(Vec<SearchField>),
    RParen(Option<Vec<SearchField>>),
    And,
    Or,
    Not,
    Proximity { distance: u16, ordered: bool },
    SetRef(String),
    Term(SearchTerm),
}

fn parse_expression_body(
    dialect: &SearchDialect,
    input: &str,
    set_env: &BTreeMap<String, QueryExpr>,
) -> Result<QueryExpr, String> {
    let tokens = tokenize_expression(dialect, input, set_env)?;
    let mut cursor = 0;
    let expr = parse_or_expr(&tokens, &mut cursor, set_env)?;
    if cursor < tokens.len() {
        return Err(format!(
            "unexpected token after expression at index {cursor}"
        ));
    }
    Ok(expr)
}

fn parse_or_expr(
    tokens: &[Token],
    cursor: &mut usize,
    set_env: &BTreeMap<String, QueryExpr>,
) -> Result<QueryExpr, String> {
    let mut children = vec![parse_and_expr(tokens, cursor, set_env)?];
    while *cursor < tokens.len() && tokens.get(*cursor) == Some(&Token::Or) {
        *cursor += 1;
        children.push(parse_and_expr(tokens, cursor, set_env)?);
    }
    if children.len() == 1 {
        children
            .into_iter()
            .next()
            .ok_or_else(|| "empty expression".to_owned())
    } else {
        Ok(QueryExpr::Or { children })
    }
}

fn parse_and_expr(
    tokens: &[Token],
    cursor: &mut usize,
    set_env: &BTreeMap<String, QueryExpr>,
) -> Result<QueryExpr, String> {
    let mut expr = parse_not_or_proximity(tokens, cursor, set_env)?;
    while *cursor < tokens.len() && tokens.get(*cursor) == Some(&Token::And) {
        *cursor += 1;
        let right = parse_not_or_proximity(tokens, cursor, set_env)?;
        expr = match expr {
            QueryExpr::And { mut children } => {
                children.push(right);
                QueryExpr::And { children }
            }
            left => QueryExpr::And {
                children: vec![left, right],
            },
        };
    }
    Ok(expr)
}

fn parse_not_or_proximity(
    tokens: &[Token],
    cursor: &mut usize,
    set_env: &BTreeMap<String, QueryExpr>,
) -> Result<QueryExpr, String> {
    let mut left = parse_primary(tokens, cursor, set_env)?;
    while *cursor < tokens.len() {
        match tokens.get(*cursor) {
            Some(Token::Not) => {
                *cursor += 1;
                let right = parse_primary(tokens, cursor, set_env)?;
                left = QueryExpr::Not {
                    include: Box::new(left),
                    exclude: Box::new(right),
                };
            }
            Some(Token::Proximity { distance, ordered }) => {
                let (d, o) = (*distance, *ordered);
                *cursor += 1;
                let right = parse_primary(tokens, cursor, set_env)?;
                left = QueryExpr::Proximity {
                    left: Box::new(left),
                    right: Box::new(right),
                    distance: d,
                    ordered: o,
                };
            }
            _ => break,
        }
    }
    Ok(left)
}

fn parse_primary(
    tokens: &[Token],
    cursor: &mut usize,
    set_env: &BTreeMap<String, QueryExpr>,
) -> Result<QueryExpr, String> {
    let Some(token) = tokens.get(*cursor) else {
        return Err("unexpected end of query tokens".to_owned());
    };
    match token {
        Token::LParen => {
            *cursor += 1;
            let mut expr = parse_or_expr(tokens, cursor, set_env)?;
            let Some(closing) = tokens.get(*cursor) else {
                return Err("missing closing parenthesis".to_owned());
            };
            if let Token::RParen(suffix_fields) = closing {
                if let Some(fields) = suffix_fields {
                    apply_fields_to_expr(&mut expr, fields);
                }
                *cursor += 1;
                Ok(expr)
            } else {
                Err("expected closing parenthesis".to_owned())
            }
        }
        Token::LParenWithFields(fields) => {
            let fields = fields.clone();
            *cursor += 1;
            let mut expr = parse_or_expr(tokens, cursor, set_env)?;
            let Some(closing) = tokens.get(*cursor) else {
                return Err("missing closing parenthesis for prefixed field".to_owned());
            };
            if let Token::RParen(suffix_fields) = closing {
                apply_fields_to_expr(&mut expr, &fields);
                if let Some(extra) = suffix_fields {
                    apply_fields_to_expr(&mut expr, extra);
                }
                *cursor += 1;
                Ok(expr)
            } else {
                Err("expected closing parenthesis for prefixed field".to_owned())
            }
        }
        Token::SetRef(id) => {
            *cursor += 1;
            set_env
                .get(id)
                .cloned()
                .ok_or_else(|| format!("referenced set `{id}` is not defined"))
        }
        Token::Term(term) => {
            *cursor += 1;
            Ok(QueryExpr::Term { term: term.clone() })
        }
        other => Err(format!("unexpected token in primary expression: {other:?}")),
    }
}

fn apply_fields_to_expr(expr: &mut QueryExpr, fields: &[SearchField]) {
    match expr {
        QueryExpr::Term { term } => {
            if term.vocabulary.is_none() {
                term.fields = fields.to_vec();
            }
        }
        QueryExpr::And { children } | QueryExpr::Or { children } => {
            for child in children {
                apply_fields_to_expr(child, fields);
            }
        }
        QueryExpr::Not { include, exclude } => {
            apply_fields_to_expr(include, fields);
            apply_fields_to_expr(exclude, fields);
        }
        QueryExpr::Proximity { left, right, .. } => {
            apply_fields_to_expr(left, fields);
            apply_fields_to_expr(right, fields);
        }
    }
}

fn tokenize_expression(
    dialect: &SearchDialect,
    input: &str,
    set_env: &BTreeMap<String, QueryExpr>,
) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars.get(i).copied().is_some_and(char::is_whitespace) {
            i += 1;
            continue;
        }

        let slice = &input[char_byte_offset(input, i)..];
        let lower = slice.to_ascii_lowercase();

        // Check Scopus/WoS prefixed parenthesis groups: TITLE-ABS-KEY(...), TS=(...), etc.
        if lower.starts_with("title-abs-key(") {
            tokens.push(Token::LParenWithFields(vec![SearchField::TitleAbstract]));
            i += "title-abs-key(".len();
            continue;
        }
        if lower.starts_with("title(") {
            tokens.push(Token::LParenWithFields(vec![SearchField::Title]));
            i += "title(".len();
            continue;
        }
        if lower.starts_with("abs(") {
            tokens.push(Token::LParenWithFields(vec![SearchField::Abstract]));
            i += "abs(".len();
            continue;
        }
        if lower.starts_with("auth(") {
            tokens.push(Token::LParenWithFields(vec![SearchField::Author]));
            i += "auth(".len();
            continue;
        }
        if lower.starts_with("srctitle(") {
            tokens.push(Token::LParenWithFields(vec![SearchField::Journal]));
            i += "srctitle(".len();
            continue;
        }
        if lower.starts_with("ts=(") {
            tokens.push(Token::LParenWithFields(vec![SearchField::All]));
            i += "ts=(".len();
            continue;
        }
        if lower.starts_with("ti=(") {
            tokens.push(Token::LParenWithFields(vec![SearchField::Title]));
            i += "ti=(".len();
            continue;
        }
        if lower.starts_with("au=(") {
            tokens.push(Token::LParenWithFields(vec![SearchField::Author]));
            i += "au=(".len();
            continue;
        }
        if lower.starts_with("so=(") {
            tokens.push(Token::LParenWithFields(vec![SearchField::Journal]));
            i += "so=(".len();
            continue;
        }

        // Check CINAHL TI / AB / AU / JN prefixes
        if lower.starts_with("ti ") {
            i += 3;
            let term = parse_single_term(
                dialect,
                &input[char_byte_offset(input, i)..],
                Some(vec![SearchField::Title]),
            )?;
            i += term_length(&input[char_byte_offset(input, i)..]);
            tokens.push(Token::Term(term));
            continue;
        }
        if lower.starts_with("ab ") {
            i += 3;
            let term = parse_single_term(
                dialect,
                &input[char_byte_offset(input, i)..],
                Some(vec![SearchField::Abstract]),
            )?;
            i += term_length(&input[char_byte_offset(input, i)..]);
            tokens.push(Token::Term(term));
            continue;
        }
        if lower.starts_with("au ") {
            i += 3;
            let term = parse_single_term(
                dialect,
                &input[char_byte_offset(input, i)..],
                Some(vec![SearchField::Author]),
            )?;
            i += term_length(&input[char_byte_offset(input, i)..]);
            tokens.push(Token::Term(term));
            continue;
        }
        if lower.starts_with("jn ") {
            i += 3;
            let term = parse_single_term(
                dialect,
                &input[char_byte_offset(input, i)..],
                Some(vec![SearchField::Journal]),
            )?;
            i += term_length(&input[char_byte_offset(input, i)..]);
            tokens.push(Token::Term(term));
            continue;
        }

        // Parentheses
        if chars.get(i) == Some(&'(') {
            // Check CINAHL (MH "heading+")
            if lower.starts_with("(mh \"") || lower.starts_with("(mh ") {
                let end = slice
                    .find(')')
                    .ok_or_else(|| "unclosed CINAHL heading parenthesis".to_owned())?;
                let heading_slice = &slice[1..end];
                let term = parse_cinahl_heading(heading_slice)?;
                tokens.push(Token::Term(term));
                i += end + 1;
                continue;
            }
            tokens.push(Token::LParen);
            i += 1;
            continue;
        }

        if chars.get(i) == Some(&')') {
            let after_paren = &slice[1..];
            let suffix_fields = extract_field_suffix(dialect, after_paren);
            let advance = 1 + suffix_fields.as_ref().map_or(0, |(_, len)| *len);
            let fields = suffix_fields.map(|(f, _)| f);
            tokens.push(Token::RParen(fields));
            i += advance;
            continue;
        }

        // Boolean operators
        if lower.starts_with("and ") || lower.starts_with("and\t") || lower == "and" {
            tokens.push(Token::And);
            i += 3;
            continue;
        }
        if lower.starts_with("or ") || lower.starts_with("or\t") || lower == "or" {
            tokens.push(Token::Or);
            i += 2;
            continue;
        }
        if lower.starts_with("not ") || lower.starts_with("not\t") || lower == "not" {
            tokens.push(Token::Not);
            i += 3;
            continue;
        }

        // Proximity operators: adj\d+, NEAR/\d+, PRE/\d+, W/\d+, W\d+, N\d+
        if let Some((distance, ordered, len)) = parse_proximity_operator(&lower) {
            tokens.push(Token::Proximity { distance, ordered });
            i += len;
            continue;
        }

        // Headings: exp Heading/ or Heading/ in Ovid
        if matches!(
            dialect,
            SearchDialect::OvidMedline | SearchDialect::PsycInfoOvid
        ) {
            if lower.starts_with("exp ") && slice.contains('/') {
                let slash_idx = slice
                    .find('/')
                    .ok_or_else(|| "missing closing slash on exploded heading".to_owned())?;
                let heading_text = slice[4..slash_idx].trim();
                let vocab = if matches!(dialect, SearchDialect::PsycInfoOvid) {
                    Some("apa thesaurus".to_owned())
                } else {
                    Some("MeSH".to_owned())
                };
                tokens.push(Token::Term(SearchTerm {
                    text: heading_text.to_owned(),
                    fields: vec![SearchField::SubjectHeading],
                    vocabulary: vocab,
                    explode: true,
                    phrase: false,
                    truncation: false,
                }));
                i += slash_idx + 1;
                continue;
            }
            if !slice.starts_with('(') && slice.contains('/') {
                let slash_idx = slice
                    .find('/')
                    .ok_or_else(|| "missing closing slash on heading".to_owned())?;
                let potential_heading = slice[..slash_idx].trim();
                if !potential_heading.is_empty()
                    && !potential_heading.contains(' ')
                    && !potential_heading.contains('(')
                    && !potential_heading.contains(')')
                {
                    let vocab = if matches!(dialect, SearchDialect::PsycInfoOvid) {
                        Some("apa thesaurus".to_owned())
                    } else {
                        Some("MeSH".to_owned())
                    };
                    tokens.push(Token::Term(SearchTerm {
                        text: potential_heading.to_owned(),
                        fields: vec![SearchField::SubjectHeading],
                        vocabulary: vocab,
                        explode: false,
                        phrase: false,
                        truncation: false,
                    }));
                    i += slash_idx + 1;
                    continue;
                }
            }
        }

        // Embase headings: 'heading'/exp or 'heading'/de
        if matches!(dialect, SearchDialect::Embase)
            && slice.starts_with('\'')
            && let Some(quote_end) = slice[1..].find('\'')
        {
            let heading_text = &slice[1..=quote_end];
            let remainder = &slice[quote_end + 2..];
            if remainder.starts_with("/exp") {
                tokens.push(Token::Term(SearchTerm {
                    text: heading_text.to_owned(),
                    fields: vec![SearchField::SubjectHeading],
                    vocabulary: Some("Emtree".to_owned()),
                    explode: true,
                    phrase: true,
                    truncation: false,
                }));
                i += quote_end + 2 + 4;
                continue;
            }
            if remainder.starts_with("/de") {
                tokens.push(Token::Term(SearchTerm {
                    text: heading_text.to_owned(),
                    fields: vec![SearchField::SubjectHeading],
                    vocabulary: Some("Emtree".to_owned()),
                    explode: false,
                    phrase: true,
                    truncation: false,
                }));
                i += quote_end + 2 + 3;
                continue;
            }
        }

        // CINAHL MH "heading+" or MH "heading"
        if matches!(dialect, SearchDialect::CinahlEbsco)
            && (lower.starts_with("mh \"") || lower.starts_with("mh "))
        {
            let term = parse_cinahl_heading(slice)?;
            let len = if let Some(end) = slice.find('"') {
                slice[end + 1..]
                    .find('"')
                    .map_or(slice.len(), |e2| end + 1 + e2 + 1)
            } else {
                slice.find(' ').unwrap_or(slice.len())
            };
            tokens.push(Token::Term(term));
            i += len;
            continue;
        }

        // Set references: e.g. 1, #1, S1
        let (set_ref, ref_len) = extract_set_reference(slice, set_env);
        if let Some(id) = set_ref {
            tokens.push(Token::SetRef(id));
            i += ref_len;
            continue;
        }

        // Regular search term
        let term = parse_single_term(dialect, slice, None)?;
        let t_len = term_length(slice);
        tokens.push(Token::Term(term));
        i += t_len;
    }

    Ok(tokens)
}

fn parse_proximity_operator(lower: &str) -> Option<(u16, bool, usize)> {
    if let Some(stripped) = lower.strip_prefix("adj") {
        let digits = stripped
            .chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>();
        if !digits.is_empty() {
            let d = digits.parse::<u16>().ok()?;
            return Some((d, false, 3 + digits.len()));
        }
    }
    if let Some(stripped) = lower.strip_prefix("near/") {
        let digits = stripped
            .chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>();
        if !digits.is_empty() {
            let d = digits.parse::<u16>().ok()?;
            return Some((d, false, 5 + digits.len()));
        }
    }
    if let Some(stripped) = lower.strip_prefix("pre/") {
        let digits = stripped
            .chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>();
        if !digits.is_empty() {
            let d = digits.parse::<u16>().ok()?;
            return Some((d, true, 4 + digits.len()));
        }
    }
    if let Some(stripped) = lower.strip_prefix("w/") {
        let digits = stripped
            .chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>();
        if !digits.is_empty() {
            let d = digits.parse::<u16>().ok()?;
            return Some((d, true, 2 + digits.len()));
        }
    }
    if let Some(stripped) = lower.strip_prefix('w')
        && stripped.chars().next().is_some_and(|c| c.is_ascii_digit())
    {
        let digits = stripped
            .chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>();
        if !digits.is_empty() {
            let d = digits.parse::<u16>().ok()?;
            return Some((d, true, 1 + digits.len()));
        }
    }
    if let Some(stripped) = lower.strip_prefix('n')
        && stripped.chars().next().is_some_and(|c| c.is_ascii_digit())
    {
        let digits = stripped
            .chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>();
        if !digits.is_empty() {
            let d = digits.parse::<u16>().ok()?;
            return Some((d, false, 1 + digits.len()));
        }
    }
    None
}

fn extract_field_suffix(
    dialect: &SearchDialect,
    after_paren: &str,
) -> Option<(Vec<SearchField>, usize)> {
    if matches!(
        dialect,
        SearchDialect::OvidMedline | SearchDialect::PsycInfoOvid
    ) && after_paren.starts_with('.')
    {
        let end = after_paren[1..].find('.')?;
        let suffix = &after_paren[1..=end];
        let fields = parse_ovid_fields(suffix);
        if !fields.is_empty() {
            return Some((fields, 1 + end + 1));
        }
    }
    if matches!(dialect, SearchDialect::Embase) && after_paren.starts_with(':') {
        let len = after_paren[1..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == ',')
            .count();
        let suffix = &after_paren[1..=len];
        let fields = parse_embase_fields(suffix);
        if !fields.is_empty() {
            return Some((fields, 1 + len));
        }
    }
    None
}

fn parse_ovid_fields(suffix: &str) -> Vec<SearchField> {
    let mut fields = Vec::new();
    for part in suffix.split(',') {
        match part.trim() {
            "ti" => fields.push(SearchField::Title),
            "ab" => fields.push(SearchField::Abstract),
            "ti,ab" => fields.push(SearchField::TitleAbstract),
            "kf" | "kw" | "id" => fields.push(SearchField::Keyword),
            "au" => fields.push(SearchField::Author),
            "jn" => fields.push(SearchField::Journal),
            _ => {}
        }
    }
    if fields.contains(&SearchField::Title) && fields.contains(&SearchField::Abstract) {
        fields.retain(|f| !matches!(f, SearchField::Title | SearchField::Abstract));
        fields.insert(0, SearchField::TitleAbstract);
    }
    if fields.is_empty() {
        vec![SearchField::TitleAbstract]
    } else {
        fields
    }
}

fn parse_embase_fields(suffix: &str) -> Vec<SearchField> {
    let mut fields = Vec::new();
    for part in suffix.split(',') {
        match part.trim() {
            "ti" => fields.push(SearchField::Title),
            "ab" => fields.push(SearchField::Abstract),
            "kw" => fields.push(SearchField::Keyword),
            "au" => fields.push(SearchField::Author),
            "jt" => fields.push(SearchField::Journal),
            "all" => fields.push(SearchField::All),
            _ => {}
        }
    }
    if fields.contains(&SearchField::Title) && fields.contains(&SearchField::Abstract) {
        fields.retain(|f| !matches!(f, SearchField::Title | SearchField::Abstract));
        fields.insert(0, SearchField::TitleAbstract);
    }
    if fields.is_empty() {
        vec![SearchField::TitleAbstract]
    } else {
        fields
    }
}

fn parse_cinahl_heading(input: &str) -> Result<SearchTerm, String> {
    let unquoted = input
        .trim_start_matches("mh")
        .trim_start_matches('(')
        .trim();
    let text_part = unquoted.trim_matches(['"', '(', ')', ' ']);
    let explode = text_part.ends_with('+');
    let heading = text_part.trim_end_matches('+').trim();
    Ok(SearchTerm {
        text: heading.to_owned(),
        fields: vec![SearchField::SubjectHeading],
        vocabulary: None,
        explode,
        phrase: true,
        truncation: false,
    })
}

fn extract_set_reference(
    slice: &str,
    set_env: &BTreeMap<String, QueryExpr>,
) -> (Option<String>, usize) {
    if slice.starts_with(['#', 'S', 's']) {
        let prefix = slice.chars().next().unwrap_or('#');
        let digits = slice[1..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>();
        if !digits.is_empty() {
            let id = format!("{prefix}{digits}");
            let after = &slice[1 + digits.len()..];
            if (after.is_empty()
                || after
                    .starts_with(|c: char| c.is_whitespace() || c == ')' || c == ',' || c == ';'))
                && (set_env.contains_key(&id) || set_env.contains_key(&digits))
            {
                return (Some(id), 1 + digits.len());
            }
        }
    }
    let digits = slice
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>();
    if !digits.is_empty() {
        let after = &slice[digits.len()..];
        if (after.is_empty()
            || after.starts_with(|c: char| c.is_whitespace() || c == ')' || c == ',' || c == ';'))
            && set_env.contains_key(&digits)
        {
            return (Some(digits.clone()), digits.len());
        }
    }
    (None, 0)
}

fn term_length(slice: &str) -> usize {
    if let Some(stripped) = slice.strip_prefix('"')
        && let Some(end) = stripped.find('"')
    {
        let after = &slice[1 + end + 1..];
        if let Some(bracket_stripped) = after.strip_prefix('[')
            && let Some(bracket_end) = bracket_stripped.find(']')
        {
            return 1 + end + 1 + 1 + bracket_end + 1;
        }
        return 1 + end + 1;
    }
    if let Some(stripped) = slice.strip_prefix('\'')
        && let Some(end) = stripped.find('\'')
    {
        return 1 + end + 1;
    }
    slice
        .find(|c: char| c.is_whitespace() || c == ')' || c == '(')
        .unwrap_or(slice.len())
}

fn parse_single_term(
    dialect: &SearchDialect,
    slice: &str,
    default_fields: Option<Vec<SearchField>>,
) -> Result<SearchTerm, String> {
    let t_len = term_length(slice);
    let raw = &slice[..t_len];

    // Check PubMed bracket tag: "term"[tag] or term[tag]
    if matches!(dialect, SearchDialect::PubMed) && raw.contains('[') && raw.ends_with(']') {
        let bracket_start = raw
            .find('[')
            .ok_or_else(|| "missing opening bracket".to_owned())?;
        let text_part = raw[..bracket_start].trim_matches('"');
        let tag = &raw[bracket_start + 1..raw.len() - 1];
        let tag_lower = tag.to_ascii_lowercase();

        if tag_lower.contains("mesh terms:noexp") || tag_lower.contains("mesh:noexp") {
            return Ok(SearchTerm {
                text: text_part.to_owned(),
                fields: vec![SearchField::SubjectHeading],
                vocabulary: Some("MeSH".to_owned()),
                explode: false,
                phrase: true,
                truncation: false,
            });
        }
        if tag_lower.contains("mesh terms") || tag_lower.contains("mesh") {
            return Ok(SearchTerm {
                text: text_part.to_owned(),
                fields: vec![SearchField::SubjectHeading],
                vocabulary: Some("MeSH".to_owned()),
                explode: true,
                phrase: true,
                truncation: false,
            });
        }
        let fields = match tag_lower.as_str() {
            "title/abstract" | "tiab" => vec![SearchField::TitleAbstract],
            "title" | "ti" => vec![SearchField::Title],
            "abstract" | "ab" => vec![SearchField::Abstract],
            "author" | "au" => vec![SearchField::Author],
            "journal" | "ta" => vec![SearchField::Journal],
            _ => vec![SearchField::All],
        };
        let truncation = text_part.ends_with('*');
        let text = text_part.trim_end_matches('*').to_owned();
        return Ok(SearchTerm {
            text,
            fields,
            vocabulary: None,
            explode: false,
            phrase: text_part.contains(' '),
            truncation,
        });
    }

    // Check Ovid dot tag: term.ti,ab,kf.
    if matches!(
        dialect,
        SearchDialect::OvidMedline | SearchDialect::PsycInfoOvid
    ) && raw.contains('.')
        && let Some(dot_idx) = raw.find('.')
    {
        let text_part = &raw[..dot_idx];
        let suffix = &raw[dot_idx + 1..];
        let fields = parse_ovid_fields(suffix.trim_end_matches('.'));
        let truncation = text_part.ends_with('*');
        let text = text_part.trim_end_matches('*').to_owned();
        return Ok(SearchTerm {
            text,
            fields,
            vocabulary: None,
            explode: false,
            phrase: text_part.contains(' '),
            truncation,
        });
    }

    // Check Embase colon tag: term:ti,ab,kw
    if matches!(dialect, SearchDialect::Embase)
        && raw.contains(':')
        && let Some(colon_idx) = raw.find(':')
    {
        let text_part = &raw[..colon_idx];
        let suffix = &raw[colon_idx + 1..];
        let fields = parse_embase_fields(suffix);
        let truncation = text_part.ends_with('*');
        let text = text_part.trim_end_matches('*').to_owned();
        return Ok(SearchTerm {
            text,
            fields,
            vocabulary: None,
            explode: false,
            phrase: text_part.contains(' '),
            truncation,
        });
    }

    // Generic unadorned or quoted term
    let is_quoted = raw.starts_with('"') && raw.ends_with('"');
    let unquoted = raw.trim_matches('"');
    let truncation = unquoted.ends_with('*');
    let text = unquoted.trim_end_matches('*').to_owned();
    let fields = default_fields.unwrap_or_else(|| vec![SearchField::All]);

    Ok(SearchTerm {
        text,
        fields,
        vocabulary: None,
        explode: false,
        phrase: is_quoted || unquoted.contains(' '),
        truncation,
    })
}

fn char_byte_offset(s: &str, char_index: usize) -> usize {
    s.char_indices()
        .nth(char_index)
        .map_or(s.len(), |(offset, _)| offset)
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
            let parsed = parse_native_strategy(format!("fixture-{index}"), dialect.clone(), text);
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
            assert!(
                parsed.semantic_strategy.is_some(),
                "checked-in {dialect:?} fixture must produce a semantic strategy"
            );
            assert_eq!(
                parsed.normalisation_state,
                NativeNormalisationState::Complete,
                "checked-in {dialect:?} fixture must achieve complete normalization"
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

    #[test]
    fn semantic_strategy_parses_nested_boolean_and_limits() {
        let text = "1. exp Genomics/\n2. (genome* or genomic*).ti,ab.\n3. 1 or 2\nlimit 3 to english language\n";
        let parsed = parse_native_strategy("s1", SearchDialect::OvidMedline, text);
        assert!(parsed.validate().is_ok());
        assert_eq!(
            parsed.normalisation_state,
            NativeNormalisationState::Complete
        );
        let semantic = parsed.semantic_strategy.as_ref();
        assert!(semantic.is_some());
        if let Some(strategy) = semantic {
            assert_eq!(strategy.dialect, SearchDialect::OvidMedline);
            assert!(strategy.limits.languages.contains(&"English".to_owned()));
            assert!(!strategy.limits.rationale.is_empty());
            if let QueryExpr::Or { children } = &strategy.query {
                assert_eq!(children.len(), 2);
            } else {
                panic!("expected Or query expression");
            }
        }
    }

    #[test]
    fn proximity_expressions_parse_in_native_strategies() {
        let ovid = parse_native_strategy(
            "p1",
            SearchDialect::OvidMedline,
            "1. (genom* adj3 screen*).ti,ab.\n",
        );
        assert!(ovid.validate().is_ok());
        assert_eq!(ovid.normalisation_state, NativeNormalisationState::Complete);
        if let Some(s) = &ovid.semantic_strategy {
            if let QueryExpr::Proximity {
                distance, ordered, ..
            } = &s.query
            {
                assert_eq!(*distance, 3);
                assert!(!*ordered);
            } else {
                panic!("expected Proximity query expression in Ovid");
            }
        }

        let embase = parse_native_strategy(
            "p2",
            SearchDialect::Embase,
            "#1 (genom*:ti NEAR/3 screen*:ti)\n",
        );
        assert!(embase.validate().is_ok());
        assert_eq!(
            embase.normalisation_state,
            NativeNormalisationState::Complete
        );

        let scopus = parse_native_strategy(
            "p3",
            SearchDialect::Scopus,
            "TITLE-ABS-KEY(genom* PRE/3 screen*)\n",
        );
        assert!(scopus.validate().is_ok());
        assert_eq!(
            scopus.normalisation_state,
            NativeNormalisationState::Complete
        );
        if let Some(s) = &scopus.semantic_strategy {
            if let QueryExpr::Proximity {
                distance, ordered, ..
            } = &s.query
            {
                assert_eq!(*distance, 3);
                assert!(*ordered);
            } else {
                panic!("expected ordered Proximity query expression in Scopus");
            }
        }
    }
}
