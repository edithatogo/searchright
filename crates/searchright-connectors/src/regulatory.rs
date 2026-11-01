//! Exact-ID adapters for archived openFDA application pages and EMA medicines snapshots.
use serde_json::{Value, json};

use super::{BibliographicRecord, RecordIdentifiers, RecordKind};

/// Largest accepted publisher response, including the complete EMA medicine catalogue.
pub const MAX_RESPONSE_BYTES: usize = 64 * 1024 * 1024;

/// Map one identified application or human medicine. Preserve its original submission or
/// indication metadata, publisher timestamp and exact source-byte digest.
///
/// # Errors
/// Returns an error for oversized, incomplete, ambiguous or unrecognised source data.
pub fn regulatory_record(
    provider: &str,
    id: &str,
    bytes: &[u8],
) -> Result<BibliographicRecord, String> {
    if bytes.len() > MAX_RESPONSE_BYTES || id.trim().is_empty() {
        return Err("response too large or identifier empty".into());
    }
    let root: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    let (items, key, publisher) = match provider {
        "drugsfda" => (
            "results",
            "application_number",
            "US Food and Drug Administration",
        ),
        "ema-medicines" => ("data", "ema_product_number", "European Medicines Agency"),
        "ema-epar" => ("data", "id", "European Medicines Agency"),
        _ => return Err("expected drugsfda, ema-medicines or ema-epar".into()),
    };
    let rows = root
        .get(items)
        .and_then(Value::as_array)
        .ok_or("missing publisher records")?;
    let total = if provider == "drugsfda" {
        root.pointer("/meta/results/total")
    } else {
        root.pointer("/meta/total_records")
    };
    if total.and_then(Value::as_u64) != Some(rows.len() as u64) {
        return Err("incomplete publisher response".into());
    }
    let mut matches = rows.iter().filter(|r| r[key].as_str() == Some(id));
    let row = matches.next().ok_or("identifier absent from response")?;
    if matches.next().is_some() {
        return Err("ambiguous publisher identifier".into());
    }
    let (title, urls) = if provider == "ema-epar" {
        let title = row["name"].as_str().ok_or("missing EPAR document title")?;
        let url = row["document_url"]
            .as_str()
            .ok_or("missing EPAR document URL")?;
        (title.to_owned(), vec![url.to_owned()])
    } else if provider == "ema-medicines" {
        if row["category"] != "Human" {
            return Err("expected a human medicine".into());
        }
        (
            row["name_of_medicine"]
                .as_str()
                .ok_or("missing medicine name")?
                .to_owned(),
            row["medicine_url"]
                .as_str()
                .map(str::to_owned)
                .into_iter()
                .collect(),
        )
    } else {
        if !row["submissions"].is_array() || !row["products"].is_array() {
            return Err("missing application history or products".into());
        }
        (
            format!("Drugs@FDA application {id}"),
            vec![format!(
                "https://api.fda.gov/drug/drugsfda.json?search=application_number%3A%22{id}%22"
            )],
        )
    };
    Ok(BibliographicRecord {
        schema_version: evidence_search_contracts::BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION.to_owned(),
        record_id: format!("{provider}:{id}"),
        source_receipt_id: format!("{provider}:blake3:{}", blake3::hash(bytes).to_hex()),
        native_id: id.to_owned(),
        kind: RecordKind::Report,
        identifiers: RecordIdentifiers::default(),
        title,
        abstract_text: None,
        authors: Vec::new(),
        container_title: Some(publisher.into()),
        publication_year: None,
        publication_date: None,
        languages: Vec::new(),
        subjects: Vec::new(),
        urls,
        provider_metadata: json!({"publisher_meta":root.get("meta"),"record":row}),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fda_keeps_indication_evidence_within_its_application() -> Result<(), String> {
        let application = json!({"application_number":"BLA123456","products":[],
            "submissions":[{"submission_type":"SUPPL","submission_number":"2"}]});
        let raw = json!({"meta":{"results":{"total":1}},"results":[application]}).to_string();
        let record = regulatory_record("drugsfda", "BLA123456", raw.as_bytes())?;
        assert_eq!(record.provider_metadata.get("record"), Some(&application));
        assert!(regulatory_record("drugsfda", "BLA000001", raw.as_bytes()).is_err());
        Ok(())
    }

    #[test]
    fn epar_preserves_document_identity_and_product_linkage() -> Result<(), String> {
        let raw = json!({"meta":{"total_records":1},"data":[{"id":"1","name":"Assessment",
            "ema_product_number":"EMEA/H/C/000001","document_url":"https://www.ema.europa.eu/example.pdf"}]});
        let record = regulatory_record("ema-epar", "1", raw.to_string().as_bytes())?;
        assert_eq!(record.native_id, "1");
        assert_eq!(
            record
                .provider_metadata
                .pointer("/record/ema_product_number")
                .and_then(Value::as_str),
            Some("EMEA/H/C/000001")
        );
        assert_eq!(record.urls, ["https://www.ema.europa.eu/example.pdf"]);
        Ok(())
    }

    #[test]
    fn ema_checks_snapshot_completeness_and_identity() -> Result<(), String> {
        let row = json!({"category":"Human","ema_product_number":"EMEA/H/C/000001",
            "name_of_medicine":"Example","medicine_status":"Authorised"});
        let payload = json!({"meta":{"total_records":1},"data":[row]});
        let parse = |p: &Value| {
            regulatory_record("ema-medicines", "EMEA/H/C/000001", p.to_string().as_bytes())
        };
        assert_eq!(parse(&payload)?.title, "Example");
        let payload = json!({"meta":{"total_records":2},"data":[row]});
        assert!(parse(&payload).is_err());
        let payload = json!({"meta":{"total_records":2},"data":[row,row]});
        assert!(parse(&payload).is_err());
        Ok(())
    }
}
