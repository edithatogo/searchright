//! Generate original normal/hostile fixtures for explicit local renderer checks.
use searchright_contracts::PrismaFlow;
use searchright_prisma::{FlowFormat, PrismaFlowVariant, render_flow};
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let format = match args.next().as_deref() {
        Some("typst") => FlowFormat::Typst,
        Some("html") => FlowFormat::DocxFriendlyHtml,
        Some("svg") => FlowFormat::Svg,
        Some("json") => FlowFormat::Json,
        _ => return Err("expected typst, html, svg or json format".into()),
    };
    let hostile = match args.next().as_deref() {
        None => false,
        Some("--hostile") => true,
        _ => return Err("expected optional --hostile flag".into()),
    };
    if args.next().is_some() {
        return Err("unexpected argument".into());
    }
    let mut flow: PrismaFlow =
        serde_json::from_str(include_str!("../../../contracts/examples/prisma-flow.json"))?;
    if hostile {
        flow.review_id = "synthetic \"quote\" \\ slash #panic(\"inert\") <script>".into();
        flow.full_text_exclusions
            .first_mut()
            .ok_or("fixture has no exclusion")?
            .label = "Synthetic | exclusion <script> #panic(\"inert\") & quoted text".into();
    }
    let output = render_flow(
        &flow,
        PrismaFlowVariant::UpdatedReview,
        Some("prior-fixture"),
        format,
    )?;
    std::io::stdout().lock().write_all(output.as_bytes())?;
    Ok(())
}
