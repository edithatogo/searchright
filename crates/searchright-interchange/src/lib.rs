//! Deterministic, evidence-bearing bibliographic interchange.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use schemars::JsonSchema;
use searchright_contracts::{
    BibliographicRecord, INTERCHANGE_RECEIPT_SCHEMA_VERSION, InterchangeFormat, InterchangeReceipt,
    RecordIdentifiers, RecordKind, Validate,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// A record or line that failed import parsing or validation and was quarantined with its line range.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct QuarantinedRecord {
    /// 1-based sequence index within the imported batch.
    pub index: usize,
    /// Raw content of the quarantined block or line.
    pub raw_content: String,
    /// 1-based start line if available.
    pub start_line: usize,
    /// 1-based end line if available.
    pub end_line: usize,
    /// Explanation of why the record was quarantined.
    pub error: String,
}

/// Result of importing bibliographic records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ImportResult {
    /// Canonical records successfully parsed and validated.
    pub records: Vec<BibliographicRecord>,
    /// Non-fatal warnings emitted during conversion.
    pub warnings: Vec<String>,
    /// Malformed or invalid records isolated during conversion.
    #[serde(default)]
    pub quarantined: Vec<QuarantinedRecord>,
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
        InterchangeFormat::Csv => import_csv(input, source_receipt_id),
        InterchangeFormat::Bibtex => import_bibtex(input, source_receipt_id),
        InterchangeFormat::EndnoteXml => import_endnote_xml(input, source_receipt_id),
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
        InterchangeFormat::Bibtex => Ok(export_bibtex(records)),
        InterchangeFormat::EndnoteXml => Ok(export_endnote_xml(records)),
        other => Err(InterchangeError::UnsupportedExport(other)),
    }
}

/// Produce a validated conversion receipt.
#[allow(
    clippy::too_many_arguments,
    reason = "conversion receipt requires comprehensive parameters"
)]
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
    let raw_records: Vec<BibliographicRecord> = serde_json::from_str(input)?;
    let mut records = Vec::new();
    let mut quarantined = Vec::new();
    let mut warnings = Vec::new();
    for (index, record) in raw_records.into_iter().enumerate() {
        if let Err(err) = record.validate() {
            quarantined.push(QuarantinedRecord {
                index: index.saturating_add(1),
                raw_content: serde_json::to_string(&record).unwrap_or_default(),
                start_line: index.saturating_add(1),
                end_line: index.saturating_add(1),
                error: err.to_string(),
            });
            warnings.push(format!(
                "Record index {} failed contract validation",
                index.saturating_add(1)
            ));
        } else {
            records.push(record);
        }
    }
    Ok(ImportResult {
        records,
        warnings,
        quarantined,
    })
}

fn import_json_lines(input: &str) -> Result<ImportResult, InterchangeError> {
    let mut records = Vec::new();
    let mut quarantined = Vec::new();
    let mut warnings = Vec::new();
    for (line_number, line) in input.lines().enumerate() {
        let line_no = line_number.saturating_add(1);
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<BibliographicRecord>(line) {
            Ok(record) => {
                if let Err(err) = record.validate() {
                    quarantined.push(QuarantinedRecord {
                        index: line_no,
                        raw_content: line.to_owned(),
                        start_line: line_no,
                        end_line: line_no,
                        error: err.to_string(),
                    });
                    warnings.push(format!("JSONL line {line_no} failed validation: {err}"));
                } else {
                    records.push(record);
                }
            }
            Err(source) => {
                quarantined.push(QuarantinedRecord {
                    index: line_no,
                    raw_content: line.to_owned(),
                    start_line: line_no,
                    end_line: line_no,
                    error: source.to_string(),
                });
                warnings.push(format!("JSONL line {line_no} is malformed: {source}"));
            }
        }
    }
    Ok(ImportResult {
        records,
        warnings,
        quarantined,
    })
}

fn import_csl_json(input: &str, source_receipt_id: &str) -> Result<ImportResult, InterchangeError> {
    let items: Vec<Value> = serde_json::from_str(input)?;
    let mut records = Vec::with_capacity(items.len());
    let mut warnings = Vec::new();
    let mut quarantined = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let idx = index.saturating_add(1);
        let title = item
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("Untitled imported record")
            .to_owned();
        let native_id = item
            .get("id")
            .and_then(Value::as_str)
            .map_or_else(|| format!("csl-{idx}"), str::to_owned);
        let authors =
            item.get("author")
                .and_then(Value::as_array)
                .map_or_else(Vec::new, |authors| {
                    authors
                        .iter()
                        .filter_map(|author| {
                            let family = author.get("family").and_then(Value::as_str).unwrap_or("");
                            let given = author.get("given").and_then(Value::as_str).unwrap_or("");
                            let literal =
                                author.get("literal").and_then(Value::as_str).unwrap_or("");
                            if !literal.is_empty() {
                                return Some(literal.to_owned());
                            }
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
                pmid: item.get("PMID").and_then(Value::as_str).map(str::to_owned),
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
        if let Err(err) = record.validate() {
            quarantined.push(QuarantinedRecord {
                index: idx,
                raw_content: serde_json::to_string(item).unwrap_or_default(),
                start_line: idx,
                end_line: idx,
                error: err.to_string(),
            });
            warnings.push(format!("CSL item {idx} failed validation: {err}"));
        } else {
            records.push(record);
        }
    }
    Ok(ImportResult {
        records,
        warnings,
        quarantined,
    })
}

#[derive(Debug, Clone, Copy)]
enum TaggedFormat {
    Ris,
    Nbib,
}

struct TaggedBlock {
    fields: BTreeMap<String, Vec<String>>,
    raw: String,
    start_line: usize,
    end_line: usize,
}

fn import_tagged(
    input: &str,
    source_receipt_id: &str,
    format: TaggedFormat,
) -> Result<ImportResult, InterchangeError> {
    let mut blocks: Vec<TaggedBlock> = Vec::new();
    let mut current_fields = BTreeMap::<String, Vec<String>>::new();
    let mut current_raw = String::new();
    let mut block_start_line = 1_usize;
    let mut current_line_no = 0_usize;

    for (line_idx, line) in input.lines().enumerate() {
        current_line_no = line_idx.saturating_add(1);
        if line.trim().is_empty() {
            if !current_fields.is_empty() {
                blocks.push(TaggedBlock {
                    fields: std::mem::take(&mut current_fields),
                    raw: std::mem::take(&mut current_raw),
                    start_line: block_start_line,
                    end_line: current_line_no.saturating_sub(1),
                });
            }
            block_start_line = current_line_no.saturating_add(1);
            continue;
        }

        if current_fields.is_empty() {
            block_start_line = current_line_no;
        }
        current_raw.push_str(line);
        current_raw.push('\n');

        let parsed = match format {
            TaggedFormat::Ris => line
                .split_once("  - ")
                .map(|(tag, value)| (tag.trim().to_owned(), value.trim().to_owned())),
            TaggedFormat::Nbib => line
                .split_once("- ")
                .map(|(tag, value)| (tag.trim().to_owned(), value.trim().to_owned())),
        };
        if let Some((tag, value)) = parsed {
            current_fields.entry(tag).or_default().push(value);
        }
        if matches!(format, TaggedFormat::Ris) && line.starts_with("ER  -") {
            blocks.push(TaggedBlock {
                fields: std::mem::take(&mut current_fields),
                raw: std::mem::take(&mut current_raw),
                start_line: block_start_line,
                end_line: current_line_no,
            });
            block_start_line = current_line_no.saturating_add(1);
        }
    }
    if !current_fields.is_empty() {
        blocks.push(TaggedBlock {
            fields: current_fields,
            raw: current_raw,
            start_line: block_start_line,
            end_line: current_line_no,
        });
    }

    let mut records = Vec::with_capacity(blocks.len());
    let mut warnings = Vec::new();
    let mut quarantined = Vec::new();

    for (index, block) in blocks.into_iter().enumerate() {
        let idx = index.saturating_add(1);
        let native_id = first(&block.fields, &["ID", "AN", "PMID"])
            .map_or_else(|| format!("tagged-{idx}"), str::to_owned);
        let title = first(&block.fields, &["TI", "T1", "BT"])
            .unwrap_or("Untitled imported record")
            .to_owned();
        let doi = first(&block.fields, &["DO", "LID"])
            .and_then(extract_doi)
            .map(str::to_owned);
        let pmid = first(&block.fields, &["PMID", "AN"])
            .filter(|value| value.chars().all(|character| character.is_ascii_digit()))
            .map(str::to_owned);
        let year = first(&block.fields, &["PY", "DP"])
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
            kind: tagged_kind(first(&block.fields, &["TY", "PT"])),
            identifiers: RecordIdentifiers {
                doi,
                pmid,
                ..RecordIdentifiers::default()
            },
            title,
            abstract_text: first(&block.fields, &["AB", "N2"]).map(str::to_owned),
            authors: values(&block.fields, &["AU", "A1", "FAU"]),
            container_title: first(&block.fields, &["JO", "JF", "JT", "T2"]).map(str::to_owned),
            publication_year: year,
            publication_date: first(&block.fields, &["DA", "DP", "Y1"]).map(str::to_owned),
            languages: values(&block.fields, &["LA"]),
            subjects: values(&block.fields, &["KW", "MH"]),
            urls: values(&block.fields, &["UR", "L2"]),
            provider_metadata: json!({"tags": block.fields}),
        };
        if record.title == "Untitled imported record" {
            warnings.push(format!("tagged record `{}` had no title", record.native_id));
        }
        if let Err(err) = record.validate() {
            quarantined.push(QuarantinedRecord {
                index: idx,
                raw_content: block.raw,
                start_line: block.start_line,
                end_line: block.end_line,
                error: err.to_string(),
            });
            warnings.push(format!(
                "tagged block {idx} (lines {}-{}) failed validation: {err}",
                block.start_line, block.end_line
            ));
        } else {
            records.push(record);
        }
    }
    Ok(ImportResult {
        records,
        warnings,
        quarantined,
    })
}

/// Import bibliographic records from CSV text.
pub fn import_csv(input: &str, source_receipt_id: &str) -> Result<ImportResult, InterchangeError> {
    let rows = parse_csv_rows(input);
    if rows.is_empty() {
        return Ok(ImportResult {
            records: Vec::new(),
            warnings: vec!["Empty CSV input".to_owned()],
            quarantined: Vec::new(),
        });
    }

    let Some(header_row) = rows.first() else {
        return Ok(ImportResult {
            records: Vec::new(),
            warnings: Vec::new(),
            quarantined: Vec::new(),
        });
    };

    let col_map: BTreeMap<String, usize> = header_row
        .iter()
        .enumerate()
        .map(|(idx, header)| (header.trim().to_ascii_lowercase(), idx))
        .collect();

    let title_idx = col_map
        .get("title")
        .or_else(|| col_map.get("article_title"))
        .or_else(|| col_map.get("ti"))
        .copied();
    let authors_idx = col_map
        .get("authors")
        .or_else(|| col_map.get("author"))
        .or_else(|| col_map.get("au"))
        .copied();
    let year_idx = col_map
        .get("year")
        .or_else(|| col_map.get("publication_year"))
        .or_else(|| col_map.get("py"))
        .copied();
    let doi_idx = col_map.get("doi").copied();
    let pmid_idx = col_map.get("pmid").copied();
    let container_idx = col_map
        .get("container_title")
        .or_else(|| col_map.get("journal"))
        .or_else(|| col_map.get("source"))
        .copied();
    let abstract_idx = col_map
        .get("abstract")
        .or_else(|| col_map.get("abstract_text"))
        .or_else(|| col_map.get("ab"))
        .copied();
    let id_idx = col_map
        .get("native_id")
        .or_else(|| col_map.get("record_id"))
        .or_else(|| col_map.get("id"))
        .copied();

    let mut records = Vec::new();
    let mut warnings = Vec::new();
    let mut quarantined = Vec::new();

    for (row_idx, row) in rows.iter().skip(1).enumerate() {
        let record_num = row_idx.saturating_add(1);
        let line_num = record_num.saturating_add(1);
        let native_id = id_idx
            .and_then(|idx| row.get(idx))
            .map(|id| id.trim().to_owned())
            .filter(|id| !id.is_empty())
            .unwrap_or_else(|| format!("csv-{record_num}"));

        let title = title_idx
            .and_then(|idx| row.get(idx))
            .map(|t| t.trim().to_owned())
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| "Untitled imported record".to_owned());

        let authors: Vec<String> = authors_idx
            .and_then(|idx| row.get(idx))
            .map(|val| {
                val.split([';', '|'])
                    .map(|author| author.trim().to_owned())
                    .filter(|author| !author.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        let publication_year = year_idx
            .and_then(|idx| row.get(idx))
            .and_then(|val| val.trim().get(..4))
            .and_then(|val| val.parse::<i32>().ok());

        let doi = doi_idx
            .and_then(|idx| row.get(idx))
            .and_then(|val| extract_doi(val).map(str::to_owned));

        let pmid = pmid_idx
            .and_then(|idx| row.get(idx))
            .map(|val| val.trim().to_owned())
            .filter(|val| !val.is_empty() && val.chars().all(|ch| ch.is_ascii_digit()));

        let container_title = container_idx
            .and_then(|idx| row.get(idx))
            .map(|val| val.trim().to_owned())
            .filter(|val| !val.is_empty());

        let abstract_text = abstract_idx
            .and_then(|idx| row.get(idx))
            .map(|val| val.trim().to_owned())
            .filter(|val| !val.is_empty());

        let record = BibliographicRecord {
            schema_version: searchright_contracts::BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION.to_owned(),
            record_id: stable_record_id("csv", &native_id),
            source_receipt_id: source_receipt_id.to_owned(),
            native_id,
            kind: RecordKind::JournalArticle,
            identifiers: RecordIdentifiers {
                doi,
                pmid,
                ..RecordIdentifiers::default()
            },
            title,
            abstract_text,
            authors,
            container_title,
            publication_year,
            publication_date: None,
            languages: Vec::new(),
            subjects: Vec::new(),
            urls: Vec::new(),
            provider_metadata: json!({"csv_row": row}),
        };

        if record.title == "Untitled imported record" {
            warnings.push(format!("CSV row {line_num} had no title"));
        }

        if let Err(err) = record.validate() {
            quarantined.push(QuarantinedRecord {
                index: record_num,
                raw_content: row.join(","),
                start_line: line_num,
                end_line: line_num,
                error: err.to_string(),
            });
            warnings.push(format!("CSV row {line_num} failed validation: {err}"));
        } else {
            records.push(record);
        }
    }

    Ok(ImportResult {
        records,
        warnings,
        quarantined,
    })
}

fn parse_csv_rows(input: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut current_row = Vec::new();
    let mut current_field = String::new();
    let mut in_quotes = false;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if in_quotes {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    current_field.push('"');
                } else {
                    in_quotes = false;
                }
            } else {
                current_field.push(ch);
            }
        } else {
            match ch {
                '"' => in_quotes = true,
                ',' => {
                    current_row.push(std::mem::take(&mut current_field));
                }
                '\n' => {
                    current_row.push(std::mem::take(&mut current_field));
                    if current_row.iter().all(|f: &String| f.trim().is_empty()) {
                        current_row.clear();
                    } else {
                        rows.push(std::mem::take(&mut current_row));
                    }
                }
                '\r' => {
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    current_row.push(std::mem::take(&mut current_field));
                    if current_row.iter().all(|f: &String| f.trim().is_empty()) {
                        current_row.clear();
                    } else {
                        rows.push(std::mem::take(&mut current_row));
                    }
                }
                _ => current_field.push(ch),
            }
        }
    }
    if in_quotes || !current_field.is_empty() || !current_row.is_empty() {
        current_row.push(current_field);
        if !current_row.iter().all(|f: &String| f.trim().is_empty()) {
            rows.push(current_row);
        }
    }
    rows
}

/// Import bibliographic records from `BibTeX` text.
pub fn import_bibtex(
    input: &str,
    source_receipt_id: &str,
) -> Result<ImportResult, InterchangeError> {
    let mut records = Vec::new();
    let mut warnings = Vec::new();
    let mut quarantined = Vec::new();

    let mut current_entry = String::new();
    let mut in_entry = false;
    let mut brace_depth = 0_usize;
    let mut start_line = 1_usize;
    let mut current_line;

    for (line_idx, line) in input.lines().enumerate() {
        current_line = line_idx.saturating_add(1);
        let trimmed = line.trim();
        if !in_entry && trimmed.starts_with('@') {
            in_entry = true;
            start_line = current_line;
            current_entry.clear();
            brace_depth = 0;
        }

        if in_entry {
            current_entry.push_str(line);
            current_entry.push('\n');
            for ch in line.chars() {
                if ch == '{' {
                    brace_depth = brace_depth.saturating_add(1);
                } else if ch == '}' {
                    brace_depth = brace_depth.saturating_sub(1);
                    if brace_depth == 0 {
                        in_entry = false;
                        let entry_str = std::mem::take(&mut current_entry);
                        match parse_single_bibtex_entry(&entry_str, source_receipt_id) {
                            Ok(record) => {
                                if let Err(err) = record.validate() {
                                    quarantined.push(QuarantinedRecord {
                                        index: records.len().saturating_add(1),
                                        raw_content: entry_str,
                                        start_line,
                                        end_line: current_line,
                                        error: err.to_string(),
                                    });
                                    warnings.push(format!("BibTeX entry lines {start_line}-{current_line} failed validation: {err}"));
                                } else {
                                    records.push(record);
                                }
                            }
                            Err(err) => {
                                quarantined.push(QuarantinedRecord {
                                    index: records.len().saturating_add(1),
                                    raw_content: entry_str,
                                    start_line,
                                    end_line: current_line,
                                    error: err.clone(),
                                });
                                warnings.push(format!(
                                    "BibTeX parse error lines {start_line}-{current_line}: {err}"
                                ));
                            }
                        }
                        break;
                    }
                }
            }
        }
    }

    Ok(ImportResult {
        records,
        warnings,
        quarantined,
    })
}

fn parse_single_bibtex_entry(
    entry: &str,
    source_receipt_id: &str,
) -> Result<BibliographicRecord, String> {
    let trimmed = entry.trim();
    if !trimmed.starts_with('@') {
        return Err("Entry does not start with @".to_owned());
    }
    let open_brace = trimmed
        .find('{')
        .ok_or_else(|| "Missing opening brace".to_owned())?;
    let entry_type = trimmed
        .get(1..open_brace)
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    let body = trimmed
        .get(open_brace.saturating_add(1)..trimmed.len().saturating_sub(1))
        .unwrap_or("")
        .trim();

    let (cite_key, fields_str) = match body.split_once(',') {
        Some((key, rest)) => (key.trim().to_owned(), rest.trim()),
        None => (format!("bibtex-{}", uuid::Uuid::now_v7()), ""),
    };

    let fields = parse_bibtex_fields(fields_str);
    let title = fields.get("title").map_or_else(
        || "Untitled imported record".to_owned(),
        |t| clean_bibtex_value(t),
    );
    let authors = fields
        .get("author")
        .map(|a| {
            clean_bibtex_value(a)
                .split(" and ")
                .map(|author| author.trim().to_owned())
                .filter(|author| !author.is_empty())
                .collect()
        })
        .unwrap_or_default();
    let year = fields.get("year").and_then(|y| {
        clean_bibtex_value(y)
            .get(..4)
            .and_then(|val| val.parse::<i32>().ok())
    });
    let doi = fields
        .get("doi")
        .and_then(|d| extract_doi(&clean_bibtex_value(d)).map(str::to_owned));
    let pmid = fields
        .get("pmid")
        .map(|p| clean_bibtex_value(p))
        .filter(|p| !p.is_empty() && p.chars().all(|ch| ch.is_ascii_digit()));
    let container_title = fields
        .get("journal")
        .or_else(|| fields.get("booktitle"))
        .map(|c| clean_bibtex_value(c));
    let abstract_text = fields.get("abstract").map(|a| clean_bibtex_value(a));

    let kind = match entry_type.as_str() {
        "article" => RecordKind::JournalArticle,
        "inproceedings" | "conference" | "proceedings" => RecordKind::Conference,
        "phdthesis" | "mastersthesis" | "thesis" => RecordKind::Thesis,
        "techreport" | "report" => RecordKind::Report,
        other => RecordKind::Other(other.to_owned()),
    };

    Ok(BibliographicRecord {
        schema_version: searchright_contracts::BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION.to_owned(),
        record_id: stable_record_id("bibtex", &cite_key),
        source_receipt_id: source_receipt_id.to_owned(),
        native_id: cite_key,
        kind,
        identifiers: RecordIdentifiers {
            doi,
            pmid,
            ..RecordIdentifiers::default()
        },
        title,
        abstract_text,
        authors,
        container_title,
        publication_year: year,
        publication_date: None,
        languages: Vec::new(),
        subjects: Vec::new(),
        urls: fields
            .get("url")
            .map(|u| vec![clean_bibtex_value(u)])
            .unwrap_or_default(),
        provider_metadata: json!({"bibtex_type": entry_type}),
    })
}

fn parse_bibtex_fields(input: &str) -> BTreeMap<String, String> {
    let mut fields = BTreeMap::new();
    let mut key = String::new();
    let mut val = String::new();
    let mut reading_key = true;
    let mut in_braces = 0_usize;
    let mut in_quotes = false;

    for ch in input.chars() {
        if reading_key {
            if ch == '=' {
                reading_key = false;
            } else if ch == ',' {
                key.clear();
            } else if !ch.is_whitespace() {
                key.push(ch);
            }
        } else {
            if ch == '{' {
                in_braces = in_braces.saturating_add(1);
                val.push(ch);
            } else if ch == '}' {
                in_braces = in_braces.saturating_sub(1);
                val.push(ch);
            } else if ch == '"' && in_braces == 0 {
                in_quotes = !in_quotes;
                val.push(ch);
            } else if ch == ',' && in_braces == 0 && !in_quotes {
                let trimmed_key = key.trim().to_ascii_lowercase();
                if !trimmed_key.is_empty() {
                    fields.insert(trimmed_key, val.trim().to_owned());
                }
                key.clear();
                val.clear();
                reading_key = true;
            } else {
                val.push(ch);
            }
        }
    }
    if !reading_key && !key.trim().is_empty() {
        let trimmed_key = key.trim().to_ascii_lowercase();
        fields.insert(trimmed_key, val.trim().to_owned());
    }
    fields
}

fn clean_bibtex_value(val: &str) -> String {
    let trimmed = val.trim();
    let unwrapped = trimmed
        .strip_prefix('{')
        .and_then(|s| s.strip_suffix('}'))
        .or_else(|| trimmed.strip_prefix('"').and_then(|s| s.strip_suffix('"')))
        .unwrap_or(trimmed);
    unwrapped.trim().replace(['{', '}'], "")
}

/// Import bibliographic records from `EndNote` XML text.
pub fn import_endnote_xml(
    input: &str,
    source_receipt_id: &str,
) -> Result<ImportResult, InterchangeError> {
    let mut records = Vec::new();
    let mut warnings = Vec::new();
    let mut quarantined = Vec::new();

    let mut current_record = String::new();
    let mut in_record = false;
    let mut start_line = 1_usize;

    for (line_idx, line) in input.lines().enumerate() {
        let line_num = line_idx.saturating_add(1);
        if line.contains("<record>") {
            in_record = true;
            start_line = line_num;
            current_record.clear();
        }
        if in_record {
            current_record.push_str(line);
            current_record.push('\n');
            if line.contains("</record>") {
                in_record = false;
                let raw = std::mem::take(&mut current_record);
                let title = extract_xml_tag(&raw, "title")
                    .unwrap_or_else(|| "Untitled imported record".to_owned());
                let authors = extract_all_xml_tags(&raw, "author");
                let year = extract_xml_tag(&raw, "year")
                    .and_then(|y| y.get(..4).and_then(|val| val.parse::<i32>().ok()));
                let doi = extract_xml_tag(&raw, "electronic-resource-num")
                    .and_then(|d| extract_doi(&d).map(str::to_owned));
                let pmid = extract_xml_tag(&raw, "accession-num")
                    .filter(|p| !p.is_empty() && p.chars().all(|ch| ch.is_ascii_digit()));
                let container = extract_xml_tag(&raw, "secondary-title");
                let abstract_text = extract_xml_tag(&raw, "abstract");
                let native_id = extract_xml_tag(&raw, "rec-number")
                    .unwrap_or_else(|| format!("endnote-{}", records.len().saturating_add(1)));

                let record = BibliographicRecord {
                    schema_version: searchright_contracts::BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION
                        .to_owned(),
                    record_id: stable_record_id("endnote", &native_id),
                    source_receipt_id: source_receipt_id.to_owned(),
                    native_id,
                    kind: RecordKind::JournalArticle,
                    identifiers: RecordIdentifiers {
                        doi,
                        pmid,
                        ..RecordIdentifiers::default()
                    },
                    title,
                    abstract_text,
                    authors,
                    container_title: container,
                    publication_year: year,
                    publication_date: None,
                    languages: Vec::new(),
                    subjects: Vec::new(),
                    urls: Vec::new(),
                    provider_metadata: Value::Null,
                };

                if let Err(err) = record.validate() {
                    quarantined.push(QuarantinedRecord {
                        index: records.len().saturating_add(1),
                        raw_content: raw,
                        start_line,
                        end_line: line_num,
                        error: err.to_string(),
                    });
                    warnings.push(format!(
                        "EndNote record lines {start_line}-{line_num} failed validation: {err}"
                    ));
                } else {
                    records.push(record);
                }
            }
        }
    }

    Ok(ImportResult {
        records,
        warnings,
        quarantined,
    })
}

fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)?.saturating_add(open.len());
    let end = xml.find(&close)?;
    if start <= end {
        Some(
            xml.get(start..end)?
                .trim()
                .replace("&amp;", "&")
                .replace("&lt;", "<")
                .replace("&gt;", ">"),
        )
    } else {
        None
    }
}

fn extract_all_xml_tags(xml: &str, tag: &str) -> Vec<String> {
    let mut results = Vec::new();
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let mut cursor = 0_usize;
    while let Some(start_idx) = xml.get(cursor..).and_then(|sub| sub.find(&open)) {
        let abs_start = cursor.saturating_add(start_idx).saturating_add(open.len());
        if let Some(end_idx) = xml.get(abs_start..).and_then(|sub| sub.find(&close)) {
            let abs_end = abs_start.saturating_add(end_idx);
            if let Some(val) = xml.get(abs_start..abs_end) {
                let cleaned = val
                    .trim()
                    .replace("&amp;", "&")
                    .replace("&lt;", "<")
                    .replace("&gt;", ">");
                if !cleaned.is_empty() {
                    results.push(cleaned);
                }
            }
            cursor = abs_end.saturating_add(close.len());
        } else {
            break;
        }
    }
    results
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

fn export_bibtex(records: &[BibliographicRecord]) -> String {
    let mut output = String::new();
    for record in records {
        let entry_type = match &record.kind {
            RecordKind::JournalArticle => "article",
            RecordKind::Conference => "inproceedings",
            RecordKind::Thesis => "phdthesis",
            RecordKind::Report => "techreport",
            _ => "misc",
        };
        let cite_key = record.native_id.replace([' ', ',', '{', '}', '\\'], "_");
        output.push_str(&format!("@{entry_type}{{{cite_key},\n"));
        output.push_str(&format!("  title = {{{}}},\n", record.title));
        if !record.authors.is_empty() {
            output.push_str(&format!(
                "  author = {{{}}},\n",
                record.authors.join(" and ")
            ));
        }
        if let Some(year) = record.publication_year {
            output.push_str(&format!("  year = {{{year}}},\n"));
        }
        if let Some(container) = &record.container_title {
            output.push_str(&format!("  journal = {{{container}}},\n"));
        }
        if let Some(doi) = &record.identifiers.doi {
            output.push_str(&format!("  doi = {{{doi}}},\n"));
        }
        if let Some(pmid) = &record.identifiers.pmid {
            output.push_str(&format!("  pmid = {{{pmid}}},\n"));
        }
        if let Some(abstract_text) = &record.abstract_text {
            output.push_str(&format!("  abstract = {{{abstract_text}}},\n"));
        }
        output.push_str("}\n\n");
    }
    output
}

fn export_endnote_xml(records: &[BibliographicRecord]) -> String {
    let mut output = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<xml><records>\n".to_owned();
    for record in records {
        output.push_str("  <record>\n");
        output.push_str(&format!(
            "    <rec-number>{}</rec-number>\n",
            xml_escape(&record.native_id)
        ));
        output.push_str(&format!(
            "    <titles><title>{}</title>",
            xml_escape(&record.title)
        ));
        if let Some(container) = &record.container_title {
            output.push_str(&format!(
                "<secondary-title>{}</secondary-title>",
                xml_escape(container)
            ));
        }
        output.push_str("</titles>\n");
        if !record.authors.is_empty() {
            output.push_str("    <contributors><authors>\n");
            for author in &record.authors {
                output.push_str(&format!("      <author>{}</author>\n", xml_escape(author)));
            }
            output.push_str("    </authors></contributors>\n");
        }
        if let Some(year) = record.publication_year {
            output.push_str(&format!("    <dates><year>{year}</year></dates>\n"));
        }
        if let Some(doi) = &record.identifiers.doi {
            output.push_str(&format!(
                "    <electronic-resource-num>{}</electronic-resource-num>\n",
                xml_escape(doi)
            ));
        }
        if let Some(pmid) = &record.identifiers.pmid {
            output.push_str(&format!(
                "    <accession-num>{}</accession-num>\n",
                xml_escape(pmid)
            ));
        }
        if let Some(abstract_text) = &record.abstract_text {
            output.push_str(&format!(
                "    <abstract>{}</abstract>\n",
                xml_escape(abstract_text)
            ));
        }
        output.push_str("  </record>\n");
    }
    output.push_str("</records></xml>\n");
    output
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
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
#[allow(
    clippy::indexing_slicing,
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "test assertions"
)]
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

    #[test]
    fn csv_import_export_round_trip() -> Result<(), InterchangeError> {
        let csv = "title,authors,year,doi,pmid,container_title\n\"A Study on Search\",Smith; Jones,2025,10.1000/study,12345678,Lancet\n";
        let res = import_records(csv, InterchangeFormat::Csv, "receipt-csv")?;
        assert_eq!(res.records.len(), 1);
        if let Some(record) = res.records.first() {
            assert_eq!(record.title, "A Study on Search");
            assert_eq!(record.authors, vec!["Smith", "Jones"]);
            assert_eq!(record.publication_year, Some(2025));
            assert_eq!(record.identifiers.doi.as_deref(), Some("10.1000/study"));
            assert_eq!(record.identifiers.pmid.as_deref(), Some("12345678"));
        }

        let exported = export_records(&res.records, InterchangeFormat::Csv)?;
        assert!(exported.contains("A Study on Search"));
        Ok(())
    }

    #[test]
    fn bibtex_import_export_round_trip() -> Result<(), InterchangeError> {
        let bib = "@article{smith2025study,\n  title = {A Study on Search},\n  author = {Smith, John and Jones, Alice},\n  year = {2025},\n  journal = {Lancet},\n  doi = {10.1000/study},\n  pmid = {12345678}\n}\n";
        let res = import_records(bib, InterchangeFormat::Bibtex, "receipt-bib")?;
        assert_eq!(res.records.len(), 1);
        if let Some(record) = res.records.first() {
            assert_eq!(record.title, "A Study on Search");
            assert_eq!(record.authors, vec!["Smith, John", "Jones, Alice"]);
            assert_eq!(record.publication_year, Some(2025));
        }

        let exported = export_records(&res.records, InterchangeFormat::Bibtex)?;
        assert!(exported.contains("title = {A Study on Search}"));
        Ok(())
    }

    #[test]
    fn endnote_xml_import_export_round_trip() -> Result<(), InterchangeError> {
        let xml = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><xml><records><record><rec-number>1</rec-number><titles><title>A Study on Search</title><secondary-title>Lancet</secondary-title></titles><contributors><authors><author>Smith, J.</author></authors></contributors><dates><year>2025</year></dates><electronic-resource-num>10.1000/study</electronic-resource-num></record></records></xml>";
        let res = import_records(xml, InterchangeFormat::EndnoteXml, "receipt-xml")?;
        assert_eq!(res.records.len(), 1);
        if let Some(record) = res.records.first() {
            assert_eq!(record.title, "A Study on Search");
            assert_eq!(record.authors, vec!["Smith, J."]);
        }

        let exported = export_records(&res.records, InterchangeFormat::EndnoteXml)?;
        assert!(exported.contains("<title>A Study on Search</title>"));
        Ok(())
    }

    proptest::proptest! {
        #[test]
        fn roundtrip_json_lines_records(records in proptest::collection::vec(
            proptest::strategy::Just(BibliographicRecord {
                schema_version: searchright_contracts::BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION.to_owned(),
                record_id: "rec-1".to_owned(),
                source_receipt_id: "rcpt-1".to_owned(),
                native_id: "nat-1".to_owned(),
                kind: RecordKind::JournalArticle,
                identifiers: RecordIdentifiers::default(),
                title: "Test Title".to_owned(),
                abstract_text: None,
                authors: vec!["Author, A.".to_owned()],
                container_title: None,
                publication_year: Some(2026),
                publication_date: None,
                languages: Vec::new(),
                subjects: Vec::new(),
                urls: Vec::new(),
                provider_metadata: Value::Null,
            }),
            1..5
        )) {
            if let Ok(exported) = export_records(&records, InterchangeFormat::JsonLines) {
                if let Ok(imported) = import_records(&exported, InterchangeFormat::JsonLines, "rcpt-1") {
                    proptest::prop_assert_eq!(records, imported.records);
                }
            }
        }
    }
}
