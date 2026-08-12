//! Deterministic, evidence-bearing bibliographic interchange.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use searchright_contracts::{
    BibliographicRecord, INTERCHANGE_RECEIPT_SCHEMA_VERSION, InterchangeFormat, InterchangeReceipt,
    RecordIdentifiers, RecordKind, Validate,
};
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::{Value, json};

/// Result of importing bibliographic records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ImportResult {
    /// Canonical records.
    pub records: Vec<BibliographicRecord>,
    /// Non-fatal warnings.
    pub warnings: Vec<String>,
}

/// Import records from a supported text format.
pub fn import_records(
    input: &str,
    format: InterchangeFormat,
    source_receipt_id: &str,
) -> Result<ImportResult, InterchangeError> {
    match format {
        InterchangeFormat::SearchrightJson => import_searchright_json(input),
        InterchangeFormat::JsonLines => import_json_lines(input),
        InterchangeFormat::CslJson => import_csl_json(input, source_receipt_id),
        InterchangeFormat::Ris => import_tagged(input, source_receipt_id, TaggedFormat::Ris),
        InterchangeFormat::Nbib => import_tagged(input, source_receipt_id, TaggedFormat::Nbib),
        other => Err(InterchangeError::UnsupportedImport(other)),
    }
}

/// Export records to a supported text format.
pub fn export_records(
    records: &[BibliographicRecord],
    format: InterchangeFormat,
) -> Result<String, InterchangeError> {
    match format {
        InterchangeFormat::SearchrightJson => Ok(serde_json::to_string_pretty(records)?),
        InterchangeFormat::JsonLines => export_json_lines(records),
        InterchangeFormat::CslJson => export_csl_json(records),
        InterchangeFormat::Ris => Ok(export_ris(records)),
        InterchangeFormat::Nbib => Ok(export_nbib(records)),
        InterchangeFormat::Csv => Ok(export_csv(records)),
        other => Err(InterchangeError::UnsupportedExport(other)),
    }
}

/// Produce a validated conversion receipt.
pub fn conversion_receipt(
    review_id: &str,
    input_format: InterchangeFormat,
    output_format: InterchangeFormat,
    input: &[u8],
    output: &[u8],
    records_read: usize,
    records_written: usize,
    warnings: Vec<String>,
    lossless: bool,
) -> Result<InterchangeReceipt, InterchangeError> {
    let receipt = InterchangeReceipt {
        schema_version: INTERCHANGE_RECEIPT_SCHEMA_VERSION.to_owned(),
        operation_id: uuid::Uuid::now_v7().to_string(),
        review_id: review_id.to_owned(),
        input_format,
        output_format,
        input_digest: blake3::hash(input).to_hex().to_string(),
        output_digest: blake3::hash(output).to_hex().to_string(),
        records_read: usize_to_u64(records_read),
        records_written: usize_to_u64(records_written),
        warnings,
        lossless,
    };
    receipt.validate()?;
    Ok(receipt)
}

fn import_searchright_json(input: &str) -> Result<ImportResult, InterchangeError> {
    let records: Vec<BibliographicRecord> = serde_json::from_str(input)?;
    validate_records(&records)?;
    Ok(ImportResult {
        records,
        warnings: Vec::new(),
    })
}

fn import_json_lines(input: &str) -> Result<ImportResult, InterchangeError> {
    let mut records = Vec::new();
    for (line_number, line) in input.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let record = serde_json::from_str::<BibliographicRecord>(line).map_err(|source| {
            InterchangeError::MalformedLine {
                line: line_number.saturating_add(1),
                source,
            }
        })?;
        record.validate()?;
        records.push(record);
    }
    Ok(ImportResult {
        records,
        warnings: Vec::new(),
    })
}

fn import_csl_json(input: &str, source_receipt_id: &str) -> Result<ImportResult, InterchangeError> {
    let items: Vec<Value> = serde_json::from_str(input)?;
    let mut records = Vec::with_capacity(items.len());
    let mut warnings = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let title = item
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("Untitled imported record")
            .to_owned();
        let native_id = item
            .get("id")
            .and_then(Value::as_str)
            .map_or_else(|| format!("csl-{}", index.saturating_add(1)), str::to_owned);
        let authors =
            item.get("author")
                .and_then(Value::as_array)
                .map_or_else(Vec::new, |authors| {
                    authors
                        .iter()
                        .filter_map(|author| {
                            let family = author.get("family").and_then(Value::as_str).unwrap_or("");
                            let given = author.get("given").and_then(Value::as_str).unwrap_or("");
                            let rendered = format!("{family}, {given}")
                                .trim_matches([',', ' '])
                                .to_owned();
                            (!rendered.is_empty()).then_some(rendered)
                        })
                        .collect()
                });
        let publication_year = item
            .get("issued")
            .and_then(|issued| issued.get("date-parts"))
            .and_then(Value::as_array)
            .and_then(|parts| parts.first())
            .and_then(Value::as_array)
            .and_then(|parts| parts.first())
            .and_then(Value::as_i64)
            .and_then(|year| i32::try_from(year).ok());
        let record = BibliographicRecord {
            schema_version: searchright_contracts::BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION.to_owned(),
            record_id: stable_record_id("csl", &native_id),
            source_receipt_id: source_receipt_id.to_owned(),
            native_id,
            kind: csl_kind(item.get("type").and_then(Value::as_str)),
            identifiers: RecordIdentifiers {
                doi: item.get("DOI").and_then(Value::as_str).map(str::to_owned),
                isbn: item.get("ISBN").and_then(Value::as_str).map(str::to_owned),
                ..RecordIdentifiers::default()
            },
            title,
            abstract_text: item
                .get("abstract")
                .and_then(Value::as_str)
                .map(str::to_owned),
            authors,
            container_title: item
                .get("container-title")
                .and_then(Value::as_str)
                .map(str::to_owned),
            publication_year,
            publication_date: None,
            languages: item
                .get("language")
                .and_then(Value::as_str)
                .map_or_else(Vec::new, |value| vec![value.to_owned()]),
            subjects: Vec::new(),
            urls: item
                .get("URL")
                .and_then(Value::as_str)
                .map_or_else(Vec::new, |value| vec![value.to_owned()]),
            provider_metadata: item.clone(),
        };
        if record.title == "Untitled imported record" {
            warnings.push(format!("CSL item `{}` had no title", record.native_id));
        }
        record.validate()?;
        records.push(record);
    }
    Ok(ImportResult { records, warnings })
}

#[derive(Debug, Clone, Copy)]
enum TaggedFormat {
    Ris,
    Nbib,
}

fn import_tagged(
    input: &str,
    source_receipt_id: &str,
    format: TaggedFormat,
) -> Result<ImportResult, InterchangeError> {
    let mut blocks = Vec::new();
    let mut current = BTreeMap::<String, Vec<String>>::new();
    for line in input.lines() {
        if line.trim().is_empty() {
            if !current.is_empty() {
                blocks.push(std::mem::take(&mut current));
            }
            continue;
        }
        let parsed = match format {
            TaggedFormat::Ris => line
                .split_once("  - ")
                .map(|(tag, value)| (tag.trim().to_owned(), value.trim().to_owned())),
            TaggedFormat::Nbib => line
                .split_once("- ")
                .map(|(tag, value)| (tag.trim().to_owned(), value.trim().to_owned())),
        };
        if let Some((tag, value)) = parsed {
            current.entry(tag).or_default().push(value);
        }
        if matches!(format, TaggedFormat::Ris) && line.starts_with("ER  -") {
            blocks.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        blocks.push(current);
    }

    let mut records = Vec::with_capacity(blocks.len());
    let mut warnings = Vec::new();
    for (index, block) in blocks.iter().enumerate() {
        let native_id = first(block, &["ID", "AN", "PMID"]).map_or_else(
            || format!("tagged-{}", index.saturating_add(1)),
            str::to_owned,
        );
        let title = first(block, &["TI", "T1", "BT"])
            .unwrap_or("Untitled imported record")
            .to_owned();
        let doi = first(block, &["DO", "LID"])
            .and_then(extract_doi)
            .map(str::to_owned);
        let pmid = first(block, &["PMID", "AN"])
            .filter(|value| value.chars().all(|character| character.is_ascii_digit()))
            .map(str::to_owned);
        let year = first(block, &["PY", "DP"])
            .and_then(|value| value.get(..4))
            .and_then(|value| value.parse::<i32>().ok());
        let record = BibliographicRecord {
            schema_version: searchright_contracts::BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION.to_owned(),
            record_id: stable_record_id(
                match format {
                    TaggedFormat::Ris => "ris",
                    TaggedFormat::Nbib => "nbib",
                },
                &native_id,
            ),
            source_receipt_id: source_receipt_id.to_owned(),
            native_id,
            kind: tagged_kind(first(block, &["TY", "PT"])),
            identifiers: RecordIdentifiers {
                doi,
                pmid,
                ..RecordIdentifiers::default()
            },
            title,
            abstract_text: first(block, &["AB", "N2"]).map(str::to_owned),
            authors: values(block, &["AU", "A1", "FAU"]),
            container_title: first(block, &["JO", "JF", "JT", "T2"]).map(str::to_owned),
            publication_year: year,
            publication_date: first(block, &["DA", "DP", "Y1"]).map(str::to_owned),
            languages: values(block, &["LA"]),
            subjects: values(block, &["KW", "MH"]),
            urls: values(block, &["UR", "L2"]),
            provider_metadata: json!({"tags": block}),
        };
        if record.title == "Untitled imported record" {
            warnings.push(format!("tagged record `{}` had no title", record.native_id));
        }
        record.validate()?;
        records.push(record);
    }
    Ok(ImportResult { records, warnings })
}

fn export_json_lines(records: &[BibliographicRecord]) -> Result<String, InterchangeError> {
    let mut lines = Vec::with_capacity(records.len());
    for record in records {
        lines.push(serde_json::to_string(record)?);
    }
    Ok(format!("{}\n", lines.join("\n")))
}

fn export_csl_json(records: &[BibliographicRecord]) -> Result<String, InterchangeError> {
    let values: Vec<Value> = records
        .iter()
        .map(|record| {
            json!({
                "id": record.native_id,
                "type": csl_type(&record.kind),
                "title": record.title,
                "abstract": record.abstract_text,
                "author": record.authors.iter().map(|author| json!({"literal": author})).collect::<Vec<_>>(),
                "container-title": record.container_title,
                "issued": record.publication_year.map(|year| json!({"date-parts": [[year]]})),
                "DOI": record.identifiers.doi,
                "PMID": record.identifiers.pmid,
                "URL": record.urls.first()
            })
        })
        .collect();
    Ok(serde_json::to_string_pretty(&values)?)
}

fn export_ris(records: &[BibliographicRecord]) -> String {
    let mut output = String::new();
    for record in records {
        push_tag(&mut output, "TY", ris_type(&record.kind));
        push_tag(&mut output, "ID", &record.native_id);
        push_tag(&mut output, "TI", &record.title);
        for author in &record.authors {
            push_tag(&mut output, "AU", author);
        }
        if let Some(container) = &record.container_title {
            push_tag(&mut output, "JO", container);
        }
        if let Some(year) = record.publication_year {
            push_tag(&mut output, "PY", &year.to_string());
        }
        if let Some(abstract_text) = &record.abstract_text {
            push_tag(&mut output, "AB", abstract_text);
        }
        if let Some(doi) = &record.identifiers.doi {
            push_tag(&mut output, "DO", doi);
        }
        for url in &record.urls {
            push_tag(&mut output, "UR", url);
        }
        push_tag(&mut output, "ER", "");
        output.push('\n');
    }
    output
}

fn export_nbib(records: &[BibliographicRecord]) -> String {
    let mut output = String::new();
    for record in records {
        if let Some(pmid) = &record.identifiers.pmid {
            push_nbib(&mut output, "PMID", pmid);
        }
        push_nbib(&mut output, "TI", &record.title);
        for author in &record.authors {
            push_nbib(&mut output, "FAU", author);
        }
        if let Some(container) = &record.container_title {
            push_nbib(&mut output, "JT", container);
        }
        if let Some(year) = record.publication_year {
            push_nbib(&mut output, "DP", &year.to_string());
        }
        if let Some(abstract_text) = &record.abstract_text {
            push_nbib(&mut output, "AB", abstract_text);
        }
        if let Some(doi) = &record.identifiers.doi {
            push_nbib(&mut output, "LID", &format!("{doi} [doi]"));
        }
        output.push('\n');
    }
    output
}

fn export_csv(records: &[BibliographicRecord]) -> String {
    let mut output = "record_id,native_id,title,authors,year,doi,pmid,container_title\n".to_owned();
    for record in records {
        let fields = [
            record.record_id.clone(),
            record.native_id.clone(),
            record.title.clone(),
            record.authors.join("; "),
            record
                .publication_year
                .map(|year| year.to_string())
                .unwrap_or_default(),
            record.identifiers.doi.clone().unwrap_or_default(),
            record.identifiers.pmid.clone().unwrap_or_default(),
            record.container_title.clone().unwrap_or_default(),
        ];
        output.push_str(
            &fields
                .iter()
                .map(|field| csv_escape(field))
                .collect::<Vec<_>>()
                .join(","),
        );
        output.push('\n');
    }
    output
}

fn validate_records(records: &[BibliographicRecord]) -> Result<(), InterchangeError> {
    for record in records {
        record.validate()?;
    }
    Ok(())
}

fn first<'a>(block: &'a BTreeMap<String, Vec<String>>, tags: &[&str]) -> Option<&'a str> {
    tags.iter().find_map(|tag| {
        block
            .get(*tag)
            .and_then(|values| values.first())
            .map(String::as_str)
    })
}

fn values(block: &BTreeMap<String, Vec<String>>, tags: &[&str]) -> Vec<String> {
    tags.iter()
        .flat_map(|tag| block.get(*tag).into_iter().flatten().cloned())
        .collect()
}

fn extract_doi(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    let candidate = trimmed
        .strip_suffix(" [doi]")
        .unwrap_or(trimmed)
        .strip_prefix("https://doi.org/")
        .unwrap_or(trimmed);
    candidate.starts_with("10.").then_some(candidate)
}

fn tagged_kind(value: Option<&str>) -> RecordKind {
    match value.map(str::to_ascii_lowercase).as_deref() {
        Some("jour" | "journal article" | "article") => RecordKind::JournalArticle,
        Some("preprint") => RecordKind::Preprint,
        Some("conf" | "conference") => RecordKind::Conference,
        Some("thes" | "thesis") => RecordKind::Thesis,
        Some("rprt" | "report") => RecordKind::Report,
        Some(other) => RecordKind::Other(other.to_owned()),
        None => RecordKind::Other("imported".to_owned()),
    }
}

fn csl_kind(value: Option<&str>) -> RecordKind {
    match value {
        Some("article-journal") => RecordKind::JournalArticle,
        Some("paper-conference") => RecordKind::Conference,
        Some("thesis") => RecordKind::Thesis,
        Some("report") => RecordKind::Report,
        Some("dataset") => RecordKind::Dataset,
        Some(other) => RecordKind::Other(other.to_owned()),
        None => RecordKind::Other("csl".to_owned()),
    }
}

const fn csl_type(kind: &RecordKind) -> &str {
    match kind {
        RecordKind::JournalArticle => "article-journal",
        RecordKind::Conference => "paper-conference",
        RecordKind::Thesis => "thesis",
        RecordKind::Report => "report",
        RecordKind::Dataset => "dataset",
        RecordKind::Preprint | RecordKind::TrialRegistry | RecordKind::Other(_) => "article",
    }
}

const fn ris_type(kind: &RecordKind) -> &str {
    match kind {
        RecordKind::JournalArticle => "JOUR",
        RecordKind::Conference => "CONF",
        RecordKind::Thesis => "THES",
        RecordKind::Report => "RPRT",
        RecordKind::Dataset => "DATA",
        RecordKind::Preprint | RecordKind::TrialRegistry | RecordKind::Other(_) => "GEN",
    }
}

fn push_tag(output: &mut String, tag: &str, value: &str) {
    output.push_str(tag);
    output.push_str("  - ");
    output.push_str(value);
    output.push('\n');
}

fn push_nbib(output: &mut String, tag: &str, value: &str) {
    output.push_str(tag);
    output.push_str("- ");
    output.push_str(value);
    output.push('\n');
}

fn csv_escape(value: &str) -> String {
    let escaped = value.replace('"', "\"\"");
    format!("\"{escaped}\"")
}

fn stable_record_id(namespace: &str, native_id: &str) -> String {
    let hash = blake3::hash(format!("{namespace}:{native_id}").as_bytes());
    let suffix: String = hash.to_hex().chars().take(16).collect();
    format!("{namespace}-{suffix}")
}

fn usize_to_u64(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

/// Interchange operation error.
#[derive(Debug, thiserror::Error)]
pub enum InterchangeError {
    /// JSON parsing or serialisation failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// One JSONL line was malformed.
    #[error("JSONL line {line} is malformed: {source}")]
    MalformedLine {
        /// One-based line number of the malformed JSONL record.
        line: usize,
        /// JSON parser error reported for the malformed line.
        source: serde_json::Error,
    },
    /// Contract validation failed.
    #[error(transparent)]
    Contract(#[from] searchright_contracts::ContractError),
    /// Input format is not yet supported by the deterministic importer.
    #[error("import format is not supported: {0:?}")]
    UnsupportedImport(InterchangeFormat),
    /// Output format is not yet supported by the deterministic exporter.
    #[error("export format is not supported: {0:?}")]
    UnsupportedExport(InterchangeFormat),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ris_import_export_round_trip_preserves_title() {
        let input = "TY  - JOUR\nID  - 1\nTI  - Test article\nDO  - 10.1000/test\nER  - \n";
        let imported = import_records(input, InterchangeFormat::Ris, "receipt-1");
        assert!(imported.is_ok());
        if let Ok(imported) = imported {
            assert_eq!(imported.records.len(), 1);
            assert_eq!(
                imported.records.first().map(|record| record.title.as_str()),
                Some("Test article")
            );
            assert!(export_records(&imported.records, InterchangeFormat::Ris).is_ok());
        }
    }
}
