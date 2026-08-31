//! Emit an original synthetic reporting fixture for an opt-in local Typst smoke.
use searchright_contracts::PrismaFlow;
use searchright_prisma::{FlowFormat, PrismaFlowVariant, render_flow};
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut flow: PrismaFlow =
        serde_json::from_str(include_str!("../../../contracts/examples/prisma-flow.json"))?;
    if std::env::args().skip(1).any(|arg| arg == "--hostile") {
        flow.review_id = "synthetic \"quote\" \\ slash #panic(\"inert\") <script>".into();
    }
    let source = render_flow(&flow, PrismaFlowVariant::NewReview, None, FlowFormat::Typst)?;
    std::io::stdout().lock().write_all(source.as_bytes())?;
    Ok(())
}
