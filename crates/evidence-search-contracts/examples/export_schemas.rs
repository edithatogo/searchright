//! Export the explicitly registered Rust-owned JSON Schemas as one deterministic document.

use std::collections::BTreeMap;
use std::io::{self, Write};

use evidence_search_contracts::rust_owned_schemas;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schemas = rust_owned_schemas()
        .into_iter()
        .map(|entry| (entry.catalogue_id, entry.generated))
        .collect::<BTreeMap<_, _>>();
    let mut output = io::BufWriter::new(io::stdout().lock());
    serde_json::to_writer_pretty(&mut output, &schemas)?;
    output.write_all(b"\n")?;
    Ok(())
}
