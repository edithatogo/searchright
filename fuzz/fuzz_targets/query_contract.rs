#![no_main]

use evidence_search_core::QueryCompiler;
use libfuzzer_sys::fuzz_target;
use searchright_contracts::{SearchDialect, SearchStrategy, Validate};

fuzz_target!(|data: &[u8]| {
    let Ok(strategy) = serde_json::from_slice::<SearchStrategy>(data) else {
        return;
    };
    if strategy.validate().is_err() {
        return;
    }
    let dialect = match data.first().copied().unwrap_or_default() % 4 {
        0 => SearchDialect::GenericBoolean,
        1 => SearchDialect::PubMed,
        2 => SearchDialect::Embase,
        _ => SearchDialect::EuropePmc,
    };
    let _result = QueryCompiler::compile(&strategy, dialect);
});
