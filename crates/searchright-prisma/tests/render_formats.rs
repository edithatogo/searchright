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
