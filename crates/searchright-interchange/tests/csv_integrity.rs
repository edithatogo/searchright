//! CSV import compatibility and deterministic export regressions.

use searchright_contracts::InterchangeFormat;
use searchright_interchange::{export_records, import_csv};

#[test]
fn csv_preserves_multiline_fields_quotes_unicode_and_line_endings()
-> Result<(), Box<dyn std::error::Error>> {
    for newline in ["\n", "\r\n", "\r"] {
        let input = format!(
            "native_id,title,authors,year,abstract{newline}\
             id-1,\"A \"\"quoted\"\", title{newline}second line\",Müller; Smith,2025,\"First paragraph{newline}Second paragraph\"{newline}\
             id-2,Next article,Alice,2024,Single paragraph{newline}"
        );
        let result = import_csv(&input, "receipt-csv")?;
        assert_eq!(result.records.len(), 2);
        assert!(result.warnings.is_empty());
        assert!(result.quarantined.is_empty());
        let first = result.records.first().ok_or("missing first record")?;
        assert_eq!(
            first.title,
            format!("A \"quoted\", title{newline}second line")
        );
        assert_eq!(first.authors, ["Müller", "Smith"]);
        assert_eq!(
            first.abstract_text,
            Some(format!("First paragraph{newline}Second paragraph"))
        );
        assert_eq!(first.native_id, "id-1");
        assert_eq!(
            result
                .records
                .get(1)
                .map(|record| record.native_id.as_str()),
            Some("id-2")
        );
    }
    Ok(())
}

#[test]
fn csv_accepts_utf8_bom_and_preserves_header_mapping() -> Result<(), Box<dyn std::error::Error>> {
    let result = import_csv(
        "\u{feff}title,native_id\r\nBOM title,source-1\r\n",
        "receipt",
    )?;
    let record = result.records.first().ok_or("missing BOM record")?;
    assert_eq!(record.title, "BOM title");
    assert_eq!(record.native_id, "source-1");
    assert!(result.warnings.is_empty());
    Ok(())
}

#[test]
fn csv_preserves_variable_rows_and_ignores_blank_records() -> Result<(), Box<dyn std::error::Error>>
{
    let input =
        "\n \n,,\nnative_id,title,authors,year\nid-1,Short\nid-2,Long,Alice,2024,extra\n,,,\n";
    let result = import_csv(input, "receipt")?;
    assert_eq!(result.records.len(), 2);
    let short = result.records.first().ok_or("missing short record")?;
    assert_eq!(short.title, "Short");
    assert!(short.authors.is_empty());
    assert_eq!(short.publication_year, None);
    let long = result.records.get(1).ok_or("missing long record")?;
    assert_eq!(long.title, "Long");
    assert_eq!(
        long.provider_metadata
            .get("csv_row")
            .and_then(|row| row.get(4)),
        Some(&serde_json::json!("extra"))
    );
    Ok(())
}

#[test]
fn csv_empty_inputs_keep_the_existing_warning() -> Result<(), Box<dyn std::error::Error>> {
    for input in ["", "\n\r\n \n,,\n", "\u{feff}"] {
        let result = import_csv(input, "receipt")?;
        assert!(result.records.is_empty());
        assert!(result.quarantined.is_empty());
        assert_eq!(result.warnings, ["Empty CSV input"]);
    }
    Ok(())
}

#[test]
fn csv_validation_errors_remain_quarantined() -> Result<(), Box<dyn std::error::Error>> {
    let result = import_csv("native_id,title\nid-1,Valid title\n", "")?;
    assert!(result.records.is_empty());
    assert_eq!(result.quarantined.len(), 1);
    assert_eq!(result.warnings.len(), 1);
    let quarantined = result.quarantined.first().ok_or("missing quarantine")?;
    assert!(quarantined.error.contains("record.source_receipt_id"));
    assert_eq!(quarantined.index, 1);
    Ok(())
}

#[test]
fn csv_uses_upstream_permissive_quoting_without_discarding_literal_quotes()
-> Result<(), Box<dyn std::error::Error>> {
    let result = import_csv(
        "native_id,title\nid-1,An unquoted \"word\"\nid-2,\"Unclosed",
        "receipt",
    )?;
    assert_eq!(result.records.len(), 2);
    assert_eq!(
        result.records.first().map(|record| record.title.as_str()),
        Some("An unquoted \"word\"")
    );
    assert_eq!(
        result.records.get(1).map(|record| record.title.as_str()),
        Some("Unclosed")
    );
    Ok(())
}

#[test]
fn csv_export_preserves_exact_bytes_and_round_trips() -> Result<(), Box<dyn std::error::Error>> {
    let input = "native_id,title,authors,year,doi,pmid,container_title\n\
                 id-1,\"A \"\"quoted\"\", title\ncontinued\",Alice; Bob,2025,10.1234/test,123,\"Journal, examples\"\n";
    let imported = import_csv(input, "receipt")?;
    let record = imported.records.first().ok_or("missing export record")?;
    let exported = export_records(&imported.records, InterchangeFormat::Csv)?;
    let expected = format!(
        "record_id,native_id,title,authors,year,doi,pmid,container_title\n\
         \"{}\",\"id-1\",\"A \"\"quoted\"\", title\ncontinued\",\"Alice; Bob\",\"2025\",\"10.1234/test\",\"123\",\"Journal, examples\"\n",
        record.record_id
    );
    assert_eq!(exported, expected);
    let round_trip = import_csv(&exported, "receipt")?;
    let restored = round_trip
        .records
        .first()
        .ok_or("missing round-trip record")?;
    assert_eq!(restored.record_id, record.record_id);
    assert_eq!(restored.title, record.title);
    assert_eq!(restored.authors, record.authors);
    assert_eq!(restored.identifiers, record.identifiers);
    assert_eq!(restored.container_title, record.container_title);
    Ok(())
}

#[test]
fn csv_empty_export_remains_header_only() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        export_records(&[], InterchangeFormat::Csv)?,
        "record_id,native_id,title,authors,year,doi,pmid,container_title\n"
    );
    Ok(())
}
