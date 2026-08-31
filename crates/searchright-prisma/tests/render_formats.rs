//! Rights-clear synthetic flow rendering and injection regressions.
use searchright_contracts::PrismaFlow;
use searchright_prisma::{FlowFormat, PrismaFlowVariant, render_flow};

fn flow() -> Result<PrismaFlow, serde_json::Error> {
    serde_json::from_str(include_str!("../../../contracts/examples/prisma-flow.json"))
}

#[test]
fn formats_are_deterministic_and_reject_invalid_counts() -> Result<(), Box<dyn std::error::Error>> {
    let mut flow = flow()?;
    for format in [
        FlowFormat::Markdown,
        FlowFormat::Json,
        FlowFormat::Mermaid,
        FlowFormat::Svg,
        FlowFormat::Typst,
        FlowFormat::DocxFriendlyHtml,
    ] {
        let rendered = render_flow(&flow, PrismaFlowVariant::NewReview, None, format)?;
        assert_eq!(
            rendered,
            render_flow(&flow, PrismaFlowVariant::NewReview, None, format)?
        );
        assert!(rendered.contains("not methodological certification"));
        flow.records_screened += 1;
        assert!(render_flow(&flow, PrismaFlowVariant::NewReview, None, format).is_err());
        flow.records_screened -= 1;
    }
    Ok(())
}

#[test]
fn machine_projection_preserves_counts_and_update_lineage() -> Result<(), Box<dyn std::error::Error>>
{
    let flow = flow()?;
    let rendered = render_flow(
        &flow,
        PrismaFlowVariant::UpdatedReview,
        Some("prior-1"),
        FlowFormat::Json,
    )?;
    let value: serde_json::Value = serde_json::from_str(&rendered)?;
    assert_eq!(value.get("flow"), Some(&serde_json::to_value(&flow)?));
    assert_eq!(
        value
            .get("prior_review_id")
            .and_then(serde_json::Value::as_str),
        Some("prior-1")
    );
    assert_eq!(
        value.get("variant").and_then(serde_json::Value::as_str),
        Some("updated_review")
    );
    assert!(
        render_flow(
            &flow,
            PrismaFlowVariant::UpdatedReview,
            None,
            FlowFormat::Svg
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn presentation_escapes_untrusted_labels() -> Result<(), Box<dyn std::error::Error>> {
    let mut flow = flow()?;
    flow.review_id = "<script>evil()</script>".into();
    flow.full_text_exclusions
        .first_mut()
        .ok_or("fixture missing exclusion reason")?
        .label = "x | y\n#panic(\"bad\") <script>".into();
    for format in [FlowFormat::Svg, FlowFormat::DocxFriendlyHtml] {
        let rendered = render_flow(&flow, PrismaFlowVariant::NewReview, None, format)?;
        assert!(!rendered.contains("<script>"));
        assert!(rendered.contains("&lt;script&gt;"));
    }
    let markdown = render_flow(
        &flow,
        PrismaFlowVariant::NewReview,
        None,
        FlowFormat::Markdown,
    )?;
    assert!(!markdown.contains("x | y"));
    assert!(!markdown.contains("<script>"));
    let typst = render_flow(&flow, PrismaFlowVariant::NewReview, None, FlowFormat::Typst)?;
    assert!(typst.contains("\\\"bad\\\""));
    assert!(!typst.contains("\n#panic"));
    Ok(())
}

#[test]
fn every_presentation_format_contains_each_count_reason_and_lineage()
-> Result<(), Box<dyn std::error::Error>> {
    let flow = flow()?;
    let mut rows = vec![
        ("Review".to_owned(), flow.review_id.clone()),
        ("Variant".to_owned(), "updated_review".to_owned()),
        ("Prior review".to_owned(), "prior-1".to_owned()),
    ];
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
    for format in [
        FlowFormat::Markdown,
        FlowFormat::Mermaid,
        FlowFormat::Svg,
        FlowFormat::Typst,
        FlowFormat::DocxFriendlyHtml,
    ] {
        let rendered = render_flow(
            &flow,
            PrismaFlowVariant::UpdatedReview,
            Some("prior-1"),
            format,
        )?;
        for (label, value) in &rows {
            let expected = match format {
                FlowFormat::Markdown => format!("| {label} | {} |", value.replace('_', "&#95;")),
                FlowFormat::Mermaid => format!("%% {label}: {value}\n"),
                FlowFormat::Svg => format!(">{label}: {value}</text>"),
                FlowFormat::Typst => format!(
                    "text({}), text({}),",
                    serde_json::to_string(label)?,
                    serde_json::to_string(value)?
                ),
                FlowFormat::DocxFriendlyHtml => {
                    format!("<th scope=\"row\">{label}</th><td>{value}</td>")
                }
                FlowFormat::Json => {
                    return Err("JSON has its separate lossless-projection assertion".into());
                }
            };
            assert!(
                rendered.contains(&expected),
                "{format:?} omitted or changed {label}"
            );
        }
    }
    Ok(())
}
