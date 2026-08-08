#![no_main]

use libfuzzer_sys::fuzz_target;
use searchright_contracts::{AuditEvent, Validate};

fuzz_target!(|data: &[u8]| {
    if let Ok(event) = serde_json::from_slice::<AuditEvent>(data) {
        let _result = event.validate();
    }
});
