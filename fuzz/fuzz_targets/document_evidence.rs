#![no_main]

use libfuzzer_sys::fuzz_target;
use searchright_contracts::{DocumentEvidence, Validate};

fuzz_target!(|data: &[u8]| {
    if let Ok(evidence) = serde_json::from_slice::<DocumentEvidence>(data) {
        let _result = evidence.validate();
    }
});
