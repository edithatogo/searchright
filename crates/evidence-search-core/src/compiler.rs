use searchright_contracts::{
    CompiledStrategy, ContractError, QueryExpr, SearchDialect, SearchField, SearchStrategy,
    SearchTerm, StrategyWarning, TranslationFidelity, Validate,
};
use serde_json::json;

use crate::audit::canonical_json;

/// Version embedded in compilation receipts and snapshots.
pub const COMPILER_VERSION: &str = "evidence-search-compiler/0.1.0";

/// Deterministic compiler from the portable query AST to provider syntax.
#[derive(Debug, Clone, Copy, Default)]
pub struct QueryCompiler;

impl QueryCompiler {
    /// Compile a strategy into the requested dialect.
    pub fn compile(
        strategy: &SearchStrategy,
        dialect: SearchDialect,
    ) -> Result<CompiledStrategy, CompileError> {
        strategy.validate()?;
        let mut warnings = Vec::new();
        if strategy.dialect != dialect {
            warnings.push(warning(
                "translation.target_differs_from_declared_dialect",
                "The requested target differs from the strategy's declared dialect; a human must review the translation.",
                true,
            ));
        }
        let mut rendered = render_expr(&strategy.query, &dialect, &mut warnings)?;
        append_limits(&mut rendered, strategy, &dialect, &mut warnings);

        let hash_input = canonical_json(&json!({
            "compiler": COMPILER_VERSION,
            "strategy": strategy,
            "target_dialect": dialect,
        }));
        let hash_bytes = serde_json::to_vec(&hash_input)?;

        let review_required = warnings.iter().any(|item| item.review_required);
        let loss_codes = warnings
            .iter()
            .filter(|item| warning_represents_loss(&item.code))
            .map(|item| item.code.clone())
            .collect::<Vec<_>>();
        let fidelity = if warnings.is_empty() {
            TranslationFidelity::Exact
        } else if !loss_codes.is_empty() {
            TranslationFidelity::Degraded
        } else if review_required {
            TranslationFidelity::Approximate
        } else {
            TranslationFidelity::SourceEquivalent
        };

        Ok(CompiledStrategy {
            schema_version: searchright_contracts::COMPILED_STRATEGY_SCHEMA_VERSION.to_owned(),
            strategy_id: strategy.strategy_id.clone(),
            dialect,
            query: rendered,
            warnings,
            fidelity,
            review_required,
            loss_codes,
            compilation_hash: blake3::hash(&hash_bytes).to_hex().to_string(),
            compiler_version: COMPILER_VERSION.to_owned(),
        })
    }
}

/// Query compilation error.
#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    /// Input contract was invalid.
    #[error(transparent)]
    Contract(#[from] ContractError),
    /// Serialisation for deterministic hashing failed.
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
    /// A query shape cannot be represented safely.
    #[error("unsupported query construct for {dialect:?}: {detail}")]
    Unsupported {
        dialect: SearchDialect,
        detail: String,
    },
}

fn render_expr(
    expression: &QueryExpr,
    dialect: &SearchDialect,
    warnings: &mut Vec<StrategyWarning>,
) -> Result<String, CompileError> {
    match expression {
        QueryExpr::Term { term } => Ok(render_term(term, dialect, warnings)),
        QueryExpr::And { children } => render_children(children, "AND", dialect, warnings),
        QueryExpr::Or { children } => render_children(children, "OR", dialect, warnings),
        QueryExpr::Not { include, exclude } => Ok(format!(
            "({}) NOT ({})",
            render_expr(include, dialect, warnings)?,
            render_expr(exclude, dialect, warnings)?
        )),
        QueryExpr::Proximity {
            left,
            right,
            distance,
            ordered,
        } => render_proximity(left, right, *distance, *ordered, dialect, warnings),
    }
}

fn render_children(
    children: &[QueryExpr],
    operator: &str,
    dialect: &SearchDialect,
    warnings: &mut Vec<StrategyWarning>,
) -> Result<String, CompileError> {
    let rendered = children
        .iter()
        .map(|child| render_expr(child, dialect, warnings))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(format!("({})", rendered.join(&format!(" {operator} "))))
}

fn render_proximity(
    left: &QueryExpr,
    right: &QueryExpr,
    distance: u16,
    ordered: bool,
    dialect: &SearchDialect,
    warnings: &mut Vec<StrategyWarning>,
) -> Result<String, CompileError> {
    let left = render_expr(left, dialect, warnings)?;
    let right = render_expr(right, dialect, warnings)?;
    let rendered = match dialect {
        SearchDialect::OvidMedline | SearchDialect::PsycInfoOvid => {
            if ordered {
                warnings.push(warning(
                    "translation.proximity.order_degraded",
                    "The Ovid adjacency operator does not preserve the requested order; review the rendered strategy.",
                    true,
                ));
            }
            format!("({left} adj{distance} {right})")
        }
        SearchDialect::Embase => {
            if ordered {
                warnings.push(warning(
                    "translation.proximity.order_degraded",
                    "The emitted Embase NEAR operator does not preserve the requested order; review source-specific NEXT syntax.",
                    true,
                ));
            }
            format!("({left} NEAR/{distance} {right})")
        },
        SearchDialect::Scopus => {
            let operator = if ordered { "PRE" } else { "W" };
            format!("({left} {operator}/{distance} {right})")
        }
        SearchDialect::WebOfScience => {
            if ordered {
                warnings.push(warning(
                    "translation.proximity.order_degraded",
                    "Web of Science NEAR/x is not order-preserving; review the requested ordered relationship.",
                    true,
                ));
            }
            format!("({left} NEAR/{distance} {right})")
        }
        SearchDialect::GenericBoolean => {
            warnings.push(warning(
                "translation.proximity.generic",
                "Generic Boolean proximity syntax is not portable and requires target-specific review.",
                true,
            ));
            format!("({left} NEAR/{distance} {right})")
        }
        SearchDialect::PubMed => {
            warnings.push(warning(
                "translation.pubmed.proximity_review",
                "PubMed proximity syntax is field/phrase dependent; emitted an AND fallback that requires human review.",
                true,
            ));
            format!("({left} AND {right})")
        }
        SearchDialect::EuropePmc => {
            warnings.push(warning(
                "translation.europe_pmc.proximity_degraded",
                "Europe PMC proximity semantics were not emitted automatically; an AND fallback requires human review.",
                true,
            ));
            format!("({left} AND {right})")
        }
        SearchDialect::CinahlEbsco => {
            let operator = if ordered { "W" } else { "N" };
            format!("({left} {operator}{distance} {right})")
        }
        SearchDialect::Crossref
        | SearchDialect::OpenAlex
        | SearchDialect::ClinicalTrialsGov
        | SearchDialect::Custom(_) => {
            warnings.push(warning(
                "translation.proximity.lossy",
                "Target source does not expose an equivalent portable proximity operator; emitted AND.",
                true,
            ));
            format!("({left} AND {right})")
        }
    };
    Ok(rendered)
}

fn render_term(
    term: &SearchTerm,
    dialect: &SearchDialect,
    warnings: &mut Vec<StrategyWarning>,
) -> String {
    if term.vocabulary.is_some() || term.fields.contains(&SearchField::SubjectHeading) {
        return render_subject_heading(term, dialect, warnings);
    }

    let literal = text_literal(term, dialect);
    let fields = if term.fields.is_empty() {
        vec![SearchField::All]
    } else {
        term.fields.clone()
    };
    let rendered_fields: Vec<String> = fields
        .iter()
        .map(|field| apply_field(&literal, field, dialect, warnings))
        .collect();
    match rendered_fields.as_slice() {
        [single] => single.clone(),
        _ => format!("({})", rendered_fields.join(" OR ")),
    }
}

fn render_subject_heading(
    term: &SearchTerm,
    dialect: &SearchDialect,
    warnings: &mut Vec<StrategyWarning>,
) -> String {
    let escaped = escape_quotes(&term.text);
    warn_on_vocabulary_mismatch(term, dialect, warnings);
    match dialect {
        SearchDialect::PubMed if term.explode => format!("\"{escaped}\"[MeSH Terms]"),
        SearchDialect::PubMed => format!("\"{escaped}\"[MeSH Terms:noexp]"),
        SearchDialect::OvidMedline | SearchDialect::PsycInfoOvid => {
            if term.explode {
                format!("exp {escaped}/")
            } else {
                format!("{escaped}/")
            }
        }
        SearchDialect::Embase => {
            if term.explode {
                format!("'{escaped}'/exp")
            } else {
                format!("'{escaped}'/de")
            }
        }
        SearchDialect::CinahlEbsco => format!("MH \"{escaped}{}\"", if term.explode { "+" } else { "" }),
        SearchDialect::EuropePmc => format!("MESH:\"{escaped}\""),
        _ => {
            warnings.push(warning(
                "translation.controlled_vocabulary.degraded",
                "The target source does not share this controlled vocabulary; rendered the heading as a phrase.",
                true,
            ));
            format!("\"{escaped}\"")
        }
    }
}

fn warn_on_vocabulary_mismatch(
    term: &SearchTerm,
    dialect: &SearchDialect,
    warnings: &mut Vec<StrategyWarning>,
) {
    let Some(vocabulary) = term.vocabulary.as_deref() else {
        return;
    };
    let expected = match dialect {
        SearchDialect::PubMed | SearchDialect::OvidMedline => Some("mesh"),
        SearchDialect::Embase => Some("emtree"),
        SearchDialect::PsycInfoOvid => Some("apa thesaurus"),
        _ => None,
    };
    if expected.is_some_and(|expected| !vocabulary.trim().eq_ignore_ascii_case(expected)) {
        warnings.push(warning(
            "translation.controlled_vocabulary.system_mismatch",
            "The declared controlled vocabulary differs from the target database vocabulary; map and validate the heading manually.",
            true,
        ));
    }
}

fn text_literal(term: &SearchTerm, dialect: &SearchDialect) -> String {
    let mut escaped = escape_quotes(&term.text);
    if term.truncation && !escaped.ends_with('*') {
        escaped.push('*');
    }
    if term.phrase {
        match dialect {
            SearchDialect::Embase => format!("'{escaped}'"),
            _ => format!("\"{escaped}\""),
        }
    } else {
        escaped
    }
}

fn apply_field(
    literal: &str,
    field: &SearchField,
    dialect: &SearchDialect,
    warnings: &mut Vec<StrategyWarning>,
) -> String {
    match dialect {
        SearchDialect::PubMed => match field {
            SearchField::All => literal.to_owned(),
            SearchField::Title => format!("{literal}[Title]"),
            SearchField::Abstract => format!("{literal}[Abstract]"),
            SearchField::TitleAbstract => format!("{literal}[Title/Abstract]"),
            SearchField::Author => format!("{literal}[Author]"),
            SearchField::Journal => format!("{literal}[Journal]"),
            SearchField::Identifier => format!("{literal}[AID]"),
            SearchField::Keyword => format!("{literal}[Other Term]"),
            SearchField::SubjectHeading => format!("{literal}[MeSH Terms]"),
            SearchField::Custom(field) => format!("{literal}[{field}]"),
        },
        SearchDialect::OvidMedline | SearchDialect::PsycInfoOvid => match field {
            SearchField::All => literal.to_owned(),
            SearchField::Title => format!("{literal}.ti."),
            SearchField::Abstract => format!("{literal}.ab."),
            SearchField::TitleAbstract => format!("{literal}.ti,ab."),
            SearchField::Author => format!("{literal}.au."),
            SearchField::Journal => format!("{literal}.jn."),
            SearchField::Identifier => format!("{literal}.ui."),
            SearchField::Keyword => format!("{literal}.kw."),
            SearchField::SubjectHeading => format!("{literal}/"),
            SearchField::Custom(field) => format!("{literal}.{field}."),
        },
        SearchDialect::Embase => match field {
            SearchField::All => format!("{literal}:all"),
            SearchField::Title => format!("{literal}:ti"),
            SearchField::Abstract => format!("{literal}:ab"),
            SearchField::TitleAbstract => format!("{literal}:ti,ab"),
            SearchField::Author => format!("{literal}:au"),
            SearchField::Journal => format!("{literal}:jt"),
            SearchField::Identifier => format!("{literal}:dn"),
            SearchField::Keyword => format!("{literal}:kw"),
            SearchField::SubjectHeading => format!("{literal}/de"),
            SearchField::Custom(field) => format!("{literal}:{field}"),
        },
        SearchDialect::EuropePmc => match field {
            SearchField::Title => format!("TITLE:{literal}"),
            SearchField::Abstract => format!("ABSTRACT:{literal}"),
            SearchField::TitleAbstract => format!("(TITLE:{literal} OR ABSTRACT:{literal})"),
            SearchField::Author => format!("AUTH:{literal}"),
            SearchField::Journal => format!("JOURNAL:{literal}"),
            SearchField::Identifier => format!("EXT_ID:{literal}"),
            SearchField::SubjectHeading => format!("MESH:{literal}"),
            _ => literal.to_owned(),
        },
        SearchDialect::CinahlEbsco => match field {
            SearchField::Title => format!("TI {literal}"),
            SearchField::Abstract => format!("AB {literal}"),
            SearchField::TitleAbstract => format!("(TI {literal} OR AB {literal})"),
            SearchField::Author => format!("AU {literal}"),
            SearchField::Journal => format!("JN {literal}"),
            SearchField::SubjectHeading => format!("MH {literal}"),
            _ => literal.to_owned(),
        },
        SearchDialect::Scopus => match field {
            SearchField::Title => format!("TITLE({literal})"),
            SearchField::Abstract => format!("ABS({literal})"),
            SearchField::TitleAbstract | SearchField::Keyword => format!("TITLE-ABS-KEY({literal})"),
            SearchField::Author => format!("AUTH({literal})"),
            SearchField::Journal => format!("SRCTITLE({literal})"),
            _ => literal.to_owned(),
        },
        SearchDialect::WebOfScience => match field {
            SearchField::Title => format!("TI=({literal})"),
            SearchField::Author => format!("AU=({literal})"),
            SearchField::Journal => format!("SO=({literal})"),
            _ => format!("TS=({literal})"),
        },
        SearchDialect::Crossref
        | SearchDialect::OpenAlex
        | SearchDialect::ClinicalTrialsGov
        | SearchDialect::GenericBoolean
        | SearchDialect::Custom(_) => {
            if !matches!(field, SearchField::All | SearchField::TitleAbstract) {
                warnings.push(warning(
                    "translation.field.degraded",
                    "The target source cannot express the requested portable field exactly.",
                    true,
                ));
            }
            literal.to_owned()
        }
    }
}

fn append_limits(
    query: &mut String,
    strategy: &SearchStrategy,
    dialect: &SearchDialect,
    warnings: &mut Vec<StrategyWarning>,
) {
    if let Some(date) = &strategy.limits.publication_date {
        match dialect {
            SearchDialect::PubMed => {
                let from = date
                    .from_year
                    .map_or_else(|| "0001".to_owned(), |year| year.to_string());
                let to = date
                    .to_year
                    .map_or_else(|| "3000".to_owned(), |year| year.to_string());
                query.push_str(&format!(
                    " AND (\"{from}\"[Date - Publication] : \"{to}\"[Date - Publication])"
                ));
            }
            _ => warnings.push(warning(
                "translation.date_limit.manual",
                "Publication-date limit is retained in the contract but requires target-specific review.",
                true,
            )),
        }
    }
    if !strategy.limits.languages.is_empty() || !strategy.limits.publication_types.is_empty() {
        warnings.push(warning(
            "translation.restrictions.manual",
            "Language/publication-type restrictions require source-specific validation and were not silently appended.",
            true,
        ));
    }
    if !strategy.limits.filters.is_empty() {
        warnings.push(warning(
            "translation.filters.review",
            "Named filters are recorded but require a versioned, source-specific filter contract.",
            true,
        ));
    }
}


fn warning_represents_loss(code: &str) -> bool {
    [
        ".lossy",
        ".degraded",
        ".generic",
        ".manual",
        ".fallback",
        ".review",
        "target_differs",
    ]
    .iter()
    .any(|marker| code.contains(marker))
}

fn escape_quotes(value: &str) -> String {
    value.replace('"', "\\\"")
}

fn warning(code: &str, message: &str, review_required: bool) -> StrategyWarning {
    StrategyWarning {
        code: code.to_owned(),
        message: message.to_owned(),
        review_required,
    }
}

#[cfg(test)]
mod tests {
    use searchright_contracts::{SearchLimit, SearchTerm};

    use super::*;

    fn strategy() -> SearchStrategy {
        SearchStrategy {
            schema_version: "org.searchright.search-strategy.v1".to_owned(),
            strategy_id: "s1".to_owned(),
            review_id: "r1".to_owned(),
            source_id: "pubmed".to_owned(),
            dialect: SearchDialect::PubMed,
            query: QueryExpr::And {
                children: vec![
                    QueryExpr::Term {
                        term: SearchTerm {
                            text: "rare disease".to_owned(),
                            fields: vec![SearchField::TitleAbstract],
                            vocabulary: None,
                            explode: false,
                            phrase: true,
                            truncation: false,
                        },
                    },
                    QueryExpr::Term {
                        term: SearchTerm {
                            text: "Genetic Testing".to_owned(),
                            fields: vec![SearchField::SubjectHeading],
                            vocabulary: Some("MeSH".to_owned()),
                            explode: true,
                            phrase: true,
                            truncation: false,
                        },
                    },
                ],
            },
            limits: SearchLimit::default(),
            translated_from: None,
            notes: Vec::new(),
        }
    }

    #[test]
    fn pubmed_compilation_is_deterministic() {
        let left = QueryCompiler::compile(&strategy(), SearchDialect::PubMed);
        let right = QueryCompiler::compile(&strategy(), SearchDialect::PubMed);
        assert!(left.is_ok());
        assert!(right.is_ok());
        if let (Ok(left), Ok(right)) = (left, right) {
            assert_eq!(left, right);
            assert!(left.query.contains("[Title/Abstract]"));
            assert!(left.query.contains("[MeSH Terms]"));
            assert_eq!(left.fidelity, TranslationFidelity::Exact);
            assert!(!left.review_required);
            assert!(left.loss_codes.is_empty());
        }
    }

    #[test]
    fn lossy_target_requires_review_and_exposes_codes() {
        let result = QueryCompiler::compile(&strategy(), SearchDialect::Crossref);
        assert!(result.is_ok());
        if let Ok(result) = result {
            assert_eq!(result.fidelity, TranslationFidelity::Degraded);
            assert!(result.review_required);
            assert!(!result.loss_codes.is_empty());
        }
    }
}
