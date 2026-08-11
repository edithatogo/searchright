//! Deterministic coverage for checked-in PRISMA and PRESS reporting fixtures.

use std::collections::BTreeSet;
use std::error::Error;

use searchright_contracts::{
    PressElement, PrismaFlow, SearchValidationReport, StandardFamily, StandardPack, Validate,
};
use searchright_prisma::{PressEvidenceState, press_evidence_state, validate_flow};

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
