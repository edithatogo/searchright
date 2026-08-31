//! Deterministic presentation adapters over the same validated arithmetic.
use searchright_contracts::PrismaFlow;

use crate::{PrismaError, PrismaFlowVariant, mermaid_flow, validate_flow_variant};

const BOUNDARY: &str = "Derived reporting evidence only; not methodological certification.";

/// Output representation; HTML tables are import-friendly, not DOCX packages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowFormat {
    /// Plain Markdown table.
    Markdown,
    /// Lossless JSON projection containing the original flow.
    Json,
    /// Mermaid diagram plus a plain-text equivalent in comments.
    Mermaid,
    /// Accessible SVG text table, not an official diagram template.
    Svg,
    /// Typst table source with escaped text literals.
    Typst,
    /// Semantic HTML table suitable for word-processor import.
    DocxFriendlyHtml,
}

/// Render validated counts without changing review state or claiming conduct adequacy.
///
/// Update lineage is retained, but historical and new-study counts cannot be
/// inferred from the v1 flow contract and are not fabricated.
pub fn render_flow(
    flow: &PrismaFlow,
    variant: PrismaFlowVariant,
    prior_review_id: Option<&str>,
    format: FlowFormat,
) -> Result<String, PrismaError> {
    validate_flow_variant(flow, variant, prior_review_id)?;
    let variant = match variant {
        PrismaFlowVariant::NewReview => "new_review",
        PrismaFlowVariant::UpdatedReview => "updated_review",
    };
    let mut rows = vec![
        ("Review".to_owned(), flow.review_id.clone()),
        ("Variant".to_owned(), variant.to_owned()),
    ];
    if let Some(prior) = prior_review_id {
        rows.push(("Prior review".to_owned(), prior.to_owned()));
    }
    for (label, count) in [
        ("Records from databases", flow.records_databases),
        ("Records from registers", flow.records_registers),
        ("Records from other sources", flow.records_other),
        ("Duplicates removed", flow.duplicates_removed),
        ("Automation removals", flow.automation_removed),
        ("Other removals", flow.other_removed),
        ("Records screened", flow.records_screened),
        ("Records excluded", flow.records_excluded),
        ("Reports sought", flow.reports_sought),
        ("Reports not retrieved", flow.reports_not_retrieved),
        ("Reports assessed", flow.reports_assessed),
        ("Reports included", flow.reports_included),
        ("Studies included", flow.studies_included),
    ] {
        rows.push((label.to_owned(), count.to_string()));
    }
    for reason in &flow.full_text_exclusions {
        rows.push((
            format!("Exclusion {}: {}", reason.reason_id, reason.label),
            reason.count.to_string(),
        ));
    }
    match format {
        FlowFormat::Json => serde_json::to_string_pretty(&serde_json::json!({
            "claim_boundary": BOUNDARY, "variant": variant,
            "prior_review_id": prior_review_id, "flow": flow,
        }))
        .map_err(|_| PrismaError::RenderEncoding),
        FlowFormat::Markdown => {
            let mut output = format!("{BOUNDARY}\n\n| Field | Value |\n| --- | --- |\n");
            for (label, value) in rows {
                output.push_str(&format!(
                    "| {} | {} |\n",
                    markdown(&label),
                    markdown(&value)
                ));
            }
            Ok(output)
        }
        FlowFormat::Mermaid => {
            let mut output = format!("%% {BOUNDARY}\n");
            for (label, value) in rows {
                output.push_str(&format!(
                    "%% {}: {}\n",
                    single_line(&label),
                    single_line(&value)
                ));
            }
            output.push_str(&mermaid_flow(flow)?);
            Ok(output)
        }
        FlowFormat::DocxFriendlyHtml => {
            let mut output = format!(
                "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>Derived reporting table</title></head><body><p>{BOUNDARY}</p><table><caption>PRISMA-style flow counts</caption><thead><tr><th scope=\"col\">Field</th><th scope=\"col\">Value</th></tr></thead><tbody>"
            );
            for (label, value) in rows {
                output.push_str(&format!(
                    "<tr><th scope=\"row\">{}</th><td>{}</td></tr>",
                    xml(&label),
                    xml(&value)
                ));
            }
            output.push_str("</tbody></table></body></html>\n");
            Ok(output)
        }
        FlowFormat::Typst => {
            let mut output = format!(
                "#text({})\n#table(columns: 2, table.header([Field], [Value]),\n",
                quoted(BOUNDARY)?
            );
            for (label, value) in rows {
                output.push_str(&format!(
                    "text({}), text({}),\n",
                    quoted(&label)?,
                    quoted(&value)?
                ));
            }
            output.push_str(")\n");
            Ok(output)
        }
        FlowFormat::Svg => {
            // One line per row; the title and description provide text equivalents.
            let height = 80 + rows.len() * 24;
            let width = rows.iter().fold(1200, |width, (label, value)| {
                width.max((label.chars().count() + value.chars().count() + 2) * 16 + 24)
            });
            let mut output = format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" role=\"img\" aria-labelledby=\"title desc\" viewBox=\"0 0 {width} {height}\"><title id=\"title\">Derived reporting table</title><desc id=\"desc\">{BOUNDARY}"
            );
            for (label, value) in &rows {
                output.push_str(&format!(" {}: {};", xml(label), xml(value)));
            }
            output.push_str("</desc><g font-family=\"sans-serif\" font-size=\"14\">");
            output.push_str(&format!("<text x=\"12\" y=\"24\">{BOUNDARY}</text>"));
            for (index, (label, value)) in rows.iter().enumerate() {
                output.push_str(&format!(
                    "<text x=\"12\" y=\"{}\">{}: {}</text>",
                    56 + index * 24,
                    xml(label),
                    xml(value)
                ));
            }
            output.push_str("</g></svg>\n");
            Ok(output)
        }
    }
}

fn single_line(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_control() || matches!(c, '\u{2028}' | '\u{2029}' | '\u{fffe}' | '\u{ffff}') {
                ' '
            } else {
                c
            }
        })
        .collect()
}

fn xml(value: &str) -> String {
    single_line(value)
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn markdown(value: &str) -> String {
    xml(value)
        .replace('\\', "&#92;")
        .replace('|', "&#124;")
        .replace('`', "&#96;")
        .replace('*', "&#42;")
        .replace('_', "&#95;")
        .replace('[', "&#91;")
        .replace(']', "&#93;")
}

fn quoted(value: &str) -> Result<String, PrismaError> {
    serde_json::to_string(&single_line(value)).map_err(|_| PrismaError::RenderEncoding)
}
