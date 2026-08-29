//! Deterministic coverage for checked-in PRISMA and PRESS reporting fixtures.

use std::collections::BTreeSet;
use std::error::Error;

use searchright_contracts::{
    CompiledStrategy, PressElement, PrismaFlow, SearchStrategy, SearchValidationReport,
    SourceReceipt, StandardFamily, StandardPack, Validate,
};
use searchright_prisma::{
    PressEvidenceState, press_evidence_state, render_docx_table, render_markdown,
    render_strategy_appendix_markdown, render_svg, render_typst, validate_flow,
};
use serde_json::Value;

#[test]
fn canonical_prisma_flow_fixture_satisfies_arithmetic() -> Result<(), Box<dyn Error>> {
    let flow: PrismaFlow =
        serde_json::from_str(include_str!("../../../contracts/examples/prisma-flow.json"))?;
    validate_flow(&flow)?;
    Ok(())
}

#[test]
fn canonical_press_fixture_remains_explicitly_incomplete() -> Result<(), Box<dyn Error>> {
    let report: SearchValidationReport = serde_yaml::from_str(include_str!(
        "../../../contracts/examples/search-validation.yaml"
    ))?;
    report.validate()?;
    assert_eq!(
        press_evidence_state(&report.press_reviews),
        PressEvidenceState::IncompleteDomainEvidence,
        "one finding cannot be promoted to evidence for all six PRESS domains"
    );
    Ok(())
}

#[test]
fn standards_pack_fixtures_preserve_reporting_and_conduct_boundaries() -> Result<(), Box<dyn Error>>
{
    let scoping: StandardPack = serde_yaml::from_str(include_str!(
        "../../../contracts/standards/packs/prisma-scr-2018.yaml"
    ))?;
    let living: StandardPack = serde_yaml::from_str(include_str!(
        "../../../contracts/standards/packs/prisma-lsr-2024.yaml"
    ))?;
    let press: StandardPack = serde_yaml::from_str(include_str!(
        "../../../contracts/standards/packs/press-2015.yaml"
    ))?;

    for pack in [&scoping, &living, &press] {
        pack.validate()?;
    }
    assert_eq!(scoping.family, StandardFamily::PrismaScR);
    assert!(scoping.items.iter().all(|item| item.scope == "reporting"));
    assert_eq!(living.family, StandardFamily::PrismaLsr);
    assert!(living.items.iter().all(|item| item.scope == "reporting"));
    assert_eq!(press.family, StandardFamily::Press2015);
    assert!(press.items.iter().all(|item| item.scope == "conduct"));

    let represented_domains = press
        .items
        .iter()
        .map(|item| item.item_id.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(represented_domains.len(), 6);
    assert!(represented_domains.contains("press-question"));
    assert!(represented_domains.contains("press-limits"));

    let contract_domains = [
        PressElement::TranslationOfQuestion,
        PressElement::BooleanAndProximity,
        PressElement::SubjectHeadings,
        PressElement::TextWords,
        PressElement::SpellingSyntaxAndLines,
        PressElement::LimitsAndFilters,
    ];
    assert_eq!(contract_domains.len(), represented_domains.len());
    Ok(())
}

#[test]
fn accessible_renderings_match_checked_in_snapshots() -> Result<(), Box<dyn Error>> {
    let flow: PrismaFlow =
        serde_json::from_str(include_str!("../../../contracts/examples/prisma-flow.json"))?;

    assert_eq!(
        render_markdown(&flow)?,
        include_str!("snapshots/prisma-flow.md")
    );
    assert_eq!(
        render_svg(&flow)?,
        include_str!("snapshots/prisma-flow.svg")
    );
    assert_eq!(
        render_typst(&flow)?,
        include_str!("snapshots/prisma-flow.typ")
    );
    assert_eq!(
        render_docx_table(&flow)?,
        include_str!("snapshots/prisma-flow.docx.html")
    );
    Ok(())
}

#[test]
fn prisma_jl_overlap_is_fixture_proven_with_explicit_losses() -> Result<(), Box<dyn Error>> {
    let flow: Value =
        serde_json::from_str(include_str!("../../../contracts/examples/prisma-flow.json"))?;
    let parity: Value = serde_json::from_str(include_str!(
        "../../../integration/fixtures/prisma-jl/flow-parity-v1.json"
    ))?;

    assert_eq!(
        parity.pointer("/source/revision").and_then(Value::as_str),
        Some("95d5aa46c6cda6fb2d014391209d666349b577dc")
    );
    let mappings = parity
        .get("mapped_counts")
        .and_then(Value::as_array)
        .ok_or("mapped_counts must be an array")?;
    assert!(
        mappings.len() >= 12,
        "the common flow projection is too narrow"
    );
    for mapping in mappings {
        let pointer = mapping
            .get("searchright_pointer")
            .and_then(Value::as_str)
            .ok_or("mapping pointer must be text")?;
        let expected = mapping
            .get("expected")
            .and_then(Value::as_u64)
            .ok_or("mapping expected count must be an unsigned integer")?;
        assert_eq!(
            flow.pointer(pointer).and_then(Value::as_u64),
            Some(expected)
        );
        assert!(
            mapping
                .get("prisma_jl_label")
                .and_then(Value::as_str)
                .is_some_and(|label| !label.trim().is_empty())
        );
    }
    let losses = parity
        .get("loss_report")
        .and_then(Value::as_array)
        .ok_or("loss_report must be an array")?;
    assert!(
        losses.len() >= 4,
        "known cross-model losses must stay explicit"
    );
    assert_eq!(
        parity
            .pointer("/ownership/arithmetic_authority")
            .and_then(Value::as_str),
        Some("searchright")
    );
    assert_eq!(
        parity
            .pointer("/ownership/prisma_jl_role")
            .and_then(Value::as_str),
        Some("optional_presentation_comparator")
    );
    Ok(())
}

#[test]
fn complete_major_database_appendix_matches_snapshot() -> Result<(), Box<dyn Error>> {
    let mut strategy: SearchStrategy = serde_yaml::from_str(include_str!(
        "../../../contracts/examples/search-strategy.yaml"
    ))?;
    let mut compiled: CompiledStrategy = serde_yaml::from_str(include_str!(
        "../../../contracts/examples/compiled-strategy.yaml"
    ))?;
    let mut receipt: SourceReceipt = serde_yaml::from_str(include_str!(
        "../../../contracts/examples/source-receipt.yaml"
    ))?;
    compiled.strategy_id.clone_from(&strategy.strategy_id);
    receipt.strategy_id.clone_from(&strategy.strategy_id);
    receipt.review_id.clone_from(&strategy.review_id);

    assert_eq!(
        render_strategy_appendix_markdown(&strategy, &compiled, &receipt)?,
        include_str!("snapshots/major-database-appendix.md")
    );

    strategy.strategy_id.push_str("-mismatch");
    assert!(render_strategy_appendix_markdown(&strategy, &compiled, &receipt).is_err());

    strategy.strategy_id.clone_from(&compiled.strategy_id);
    receipt.compiler_version.push_str("-mismatch");
    assert!(render_strategy_appendix_markdown(&strategy, &compiled, &receipt).is_err());
    Ok(())
}
