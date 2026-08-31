//! Bounded offline PubMed citation/abstract XML, not full-text retrieval.

use evidence_search_contracts::{
    BibliographicRecord, ProviderPage, RecordIdentifiers, RecordKind, Validate,
};
use evidence_search_core::ProviderError;
use quick_xml::{Reader, XmlVersion, events::Event};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

const MAX_BYTES: usize = 8 * 1024 * 1024;
const MAX_DEPTH: usize = 64;
const MAX_RECORDS: usize = 1_000;
const MAX_TEXT: usize = 4 * 1024 * 1024;
const MAX_FIELD: usize = 64 * 1024;
const MAX_NODES: usize = 100_000;

fn invalid(message: &str) -> ProviderError {
    ProviderError::MalformedResponse {
        provider: "pubmed".to_owned(),
        format: "XML",
        message: message.to_owned(),
    }
}

fn requested_set(ids: &[String]) -> Result<BTreeSet<&str>, ProviderError> {
    if ids.is_empty() || ids.len() > MAX_RECORDS {
        return Err(invalid("requested PMID count is outside the offline limit"));
    }
    let mut result = BTreeSet::new();
    for id in ids {
        if id.is_empty()
            || id.len() > 32
            || !id.bytes().all(|byte| byte.is_ascii_digit())
            || !result.insert(id.as_str())
        {
            return Err(invalid(
                "requested PMIDs must be numeric, bounded and unique",
            ));
        }
    }
    Ok(result)
}

/// Offline `EFetch` endpoint description. Construction does not enable network access.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PubMedFetchRequest {
    /// Exact nonempty PMID batch, bounded to 1,000 distinct numeric identifiers.
    pub pmids: Vec<String>,
    /// Optional operator-supplied NCBI tool identifier.
    pub tool: Option<String>,
    /// Optional operator-supplied NCBI contact address.
    pub email: Option<String>,
}

impl PubMedFetchRequest {
    /// Build an HTTPS PubMed XML endpoint without retrieving or logging it.
    pub fn endpoint(&self) -> Result<url::Url, ProviderError> {
        requested_set(&self.pmids)?;
        for (value, maximum) in [(&self.tool, 256), (&self.email, 320)] {
            if value.as_ref().is_some_and(|text| {
                text.trim().is_empty() || text.len() > maximum || text.chars().any(char::is_control)
            }) {
                return Err(invalid(
                    "optional request identity is malformed or exceeds its limit",
                ));
            }
        }
        let mut endpoint =
            url::Url::parse("https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi")
                .map_err(|_| invalid("fixed EFetch endpoint could not be constructed"))?;
        {
            let mut query = endpoint.query_pairs_mut();
            query
                .append_pair("db", "pubmed")
                .append_pair("retmode", "xml")
                .append_pair("id", &self.pmids.join(","));
            if let Some(tool) = &self.tool {
                query.append_pair("tool", tool);
            }
            if let Some(email) = &self.email {
                query.append_pair("email", email);
            }
        }
        Ok(endpoint)
    }
}

#[derive(Default)]
struct Node {
    name: String,
    attributes: BTreeMap<String, String>,
    text: String,
    children: Vec<Self>,
    direct_nonwhitespace: bool,
}

impl Node {
    fn container(&self, allowed: &[&str], required: &str) -> Result<(), ProviderError> {
        if self.direct_nonwhitespace
            || self
                .children
                .iter()
                .any(|child| !allowed.contains(&child.name.as_str()))
            || self.all(required).next().is_none()
        {
            return Err(invalid(
                "mapped container has unsupported or missing child elements",
            ));
        }
        Ok(())
    }
    fn all(&self, name: &str) -> impl Iterator<Item = &Self> {
        let name = name.to_owned();
        self.children.iter().filter(move |node| node.name == name)
    }

    fn optional(&self, name: &str) -> Result<Option<&Self>, ProviderError> {
        let mut children = self.all(name);
        let first = children.next();
        if children.next().is_some() {
            return Err(invalid("duplicate mapped singleton element"));
        }
        Ok(first)
    }

    fn one(&self, name: &str) -> Result<&Self, ProviderError> {
        self.optional(name)?
            .ok_or_else(|| invalid("required mapped element is missing"))
    }

    fn content(&self) -> Result<String, ProviderError> {
        if self.text.len() > MAX_FIELD || self.text.trim().is_empty() {
            return Err(invalid("mapped text is blank or exceeds the field limit"));
        }
        Ok(self.text.trim().to_owned())
    }

    fn field(&self, name: &str) -> Result<Option<String>, ProviderError> {
        self.optional(name)?.map(Self::scalar).transpose()
    }

    fn scalar(&self) -> Result<String, ProviderError> {
        if !self.children.is_empty() {
            return Err(invalid("scalar mapped field contains nested elements"));
        }
        self.content()
    }

    fn mixed(&self) -> Result<String, ProviderError> {
        fn supported(node: &Node) -> bool {
            node.children.iter().all(|child| {
                matches!(
                    child.name.as_str(),
                    "i" | "b" | "u" | "sup" | "sub" | "italic" | "bold"
                ) && supported(child)
            })
        }
        if !supported(self) {
            return Err(invalid("unsupported inline markup in mapped text"));
        }
        self.content()
    }
}

fn valid_characters(text: &str) -> bool {
    text.chars().all(|ch| matches!(ch, '\t' | '\n' | '\r' | '\u{20}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}'))
}

fn declaration_is_valid(raw: &[u8]) -> bool {
    fn fields(raw: &[u8]) -> Option<Vec<(&str, &str)>> {
        if raw.len() > 512 {
            return None;
        }
        let mut rest = std::str::from_utf8(raw).ok()?.strip_prefix("xml")?;
        let mut result = Vec::new();
        while !rest.is_empty() {
            let trimmed = rest.trim_start_matches([' ', '\t', '\n', '\r']);
            if trimmed.len() == rest.len() {
                return None;
            }
            rest = trimmed;
            if rest.is_empty() {
                break;
            }
            if result.len() >= 3 {
                return None;
            }
            let end = rest.find(|ch: char| !ch.is_ascii_alphabetic())?;
            let name = rest.get(..end)?;
            rest = rest
                .get(end..)?
                .trim_start_matches([' ', '\t', '\n', '\r'])
                .strip_prefix('=')?
                .trim_start_matches([' ', '\t', '\n', '\r']);
            let quote = rest.chars().next()?;
            if !matches!(quote, '\'' | '"') {
                return None;
            }
            rest = rest.get(1..)?;
            let end = rest.find(quote)?;
            let value = rest.get(..end)?;
            result.push((name, value));
            rest = rest.get(end + 1..)?;
        }
        Some(result)
    }
    fields(raw).is_some_and(|items| match items.as_slice() {
        [("version", "1.0")] => true,
        [("version", "1.0"), ("encoding", encoding)] => encoding.eq_ignore_ascii_case("UTF-8"),
        [("version", "1.0"), ("standalone", standalone)] => matches!(*standalone, "yes" | "no"),
        [
            ("version", "1.0"),
            ("encoding", encoding),
            ("standalone", standalone),
        ] => encoding.eq_ignore_ascii_case("UTF-8") && matches!(*standalone, "yes" | "no"),
        _ => false,
    })
}

fn append_text(target: &mut String, text: &str, total: &mut usize) -> Result<(), ProviderError> {
    if !valid_characters(text) || text.len() > MAX_TEXT.saturating_sub(*total) {
        return Err(invalid(
            "decoded XML text is invalid or exceeds the aggregate limit",
        ));
    }
    *total += text.len();
    target.push_str(text);
    Ok(())
}

fn decode_tree(xml: &str) -> Result<Node, ProviderError> {
    if xml.len() > MAX_BYTES || !valid_characters(xml) {
        return Err(invalid(
            "XML exceeds the byte limit or contains invalid characters",
        ));
    }
    let mut reader = Reader::from_str(xml);
    reader.config_mut().check_comments = true;
    let mut stack: Vec<Node> = Vec::new();
    let mut root = None;
    let mut nodes = 0_usize;
    let mut total_text = 0_usize;
    let mut declaration = false;
    loop {
        let event_start = reader.buffer_position();
        let event = reader
            .read_event()
            .map_err(|_| invalid("malformed or truncated XML"))?;
        match event {
            Event::Start(ref start) | Event::Empty(ref start) => {
                if stack.len() >= MAX_DEPTH || nodes >= MAX_NODES {
                    return Err(invalid("XML depth or element limit exceeded"));
                }
                nodes += 1;
                let name = std::str::from_utf8(start.name().as_ref())
                    .map_err(|_| invalid("XML element name is not UTF-8"))?
                    .to_owned();
                if !valid_name(&name) {
                    return Err(invalid("unsupported XML element name"));
                }
                let mut node = Node {
                    name,
                    ..Node::default()
                };
                for attribute in start.attributes() {
                    let attribute =
                        attribute.map_err(|_| invalid("malformed or duplicate XML attribute"))?;
                    if attribute.value.len() > MAX_FIELD {
                        return Err(invalid("XML attribute exceeds the raw field limit"));
                    }
                    if node.attributes.len() >= 64 {
                        return Err(invalid("XML attribute count limit exceeded"));
                    }
                    let key = std::str::from_utf8(attribute.key.as_ref())
                        .map_err(|_| invalid("invalid XML attribute name"))?
                        .to_owned();
                    if !valid_name(&key) {
                        return Err(invalid("unsupported XML attribute name"));
                    }
                    let value = attribute
                        .decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())
                        .map_err(|_| invalid("unsupported XML attribute entity"))?;
                    if value.len() > MAX_FIELD
                        || !valid_characters(&value)
                        || attribute.value.contains(&b'<')
                    {
                        return Err(invalid("invalid or oversized XML attribute"));
                    }
                    node.attributes.insert(key, value.into_owned());
                }
                if stack.is_empty() && root.is_some() {
                    return Err(invalid("multiple XML roots"));
                }
                stack.push(node);
                if matches!(event, Event::Empty(_)) {
                    close_node(&mut stack, &mut root, &mut total_text)?;
                }
            }
            Event::End(_) => close_node(&mut stack, &mut root, &mut total_text)?,
            Event::Text(text) => {
                let text = text
                    .decode()
                    .map_err(|_| invalid("invalid XML text encoding"))?;
                if let Some(node) = stack.last_mut() {
                    node.direct_nonwhitespace |= !text.trim().is_empty();
                    append_text(&mut node.text, &text, &mut total_text)?;
                } else if !text.trim().is_empty() {
                    return Err(invalid("text outside XML root"));
                }
            }
            Event::CData(text) => {
                let text = text
                    .decode()
                    .map_err(|_| invalid("invalid CDATA encoding"))?;
                let node = stack
                    .last_mut()
                    .ok_or_else(|| invalid("CDATA outside XML root"))?;
                node.direct_nonwhitespace |= !text.trim().is_empty();
                append_text(&mut node.text, &text, &mut total_text)?;
            }
            Event::GeneralRef(reference) => {
                let name = reference
                    .decode()
                    .map_err(|_| invalid("invalid XML reference"))?;
                let escaped = format!("&{name};");
                let text = quick_xml::escape::unescape(&escaped)
                    .map_err(|_| invalid("custom or invalid XML entity rejected"))?;
                let node = stack
                    .last_mut()
                    .ok_or_else(|| invalid("entity outside XML root"))?;
                node.direct_nonwhitespace |= !text.trim().is_empty();
                append_text(&mut node.text, &text, &mut total_text)?;
            }
            Event::Decl(decl) => {
                if (event_start != 0 && !(event_start == 3 && xml.starts_with('\u{feff}')))
                    || declaration
                    || !stack.is_empty()
                    || root.is_some()
                    || !declaration_is_valid(decl.as_ref())
                {
                    return Err(invalid("unsupported or misplaced XML declaration"));
                }
                declaration = true;
            }
            Event::DocType(_) | Event::PI(_) => {
                return Err(invalid("DTD and processing instructions are unsupported"));
            }
            Event::Comment(_) => {}
            Event::Eof => break,
        }
    }
    if !stack.is_empty() {
        return Err(invalid("truncated XML document"));
    }
    root.ok_or_else(|| invalid("missing XML root"))
}

fn close_node(
    stack: &mut Vec<Node>,
    root: &mut Option<Node>,
    total: &mut usize,
) -> Result<(), ProviderError> {
    let node = stack
        .pop()
        .ok_or_else(|| invalid("unmatched XML end element"))?;
    if let Some(parent) = stack.last_mut() {
        if parent.name == "PubmedArticleSet" && parent.children.len() >= MAX_RECORDS {
            return Err(invalid("record count limit exceeded"));
        }
        append_text(&mut parent.text, &node.text, total)?;
        parent.children.push(node);
    } else {
        *root = Some(node);
    }
    Ok(())
}

fn parse_record(node: &Node) -> Result<BibliographicRecord, ProviderError> {
    if node.direct_nonwhitespace
        || node
            .children
            .iter()
            .any(|child| !matches!(child.name.as_str(), "MedlineCitation" | "PubmedData"))
    {
        return Err(invalid("unsupported article envelope content"));
    }
    let citation = node.one("MedlineCitation")?;
    let pmid = citation.one("PMID")?.scalar()?;
    requested_set(std::slice::from_ref(&pmid))?;
    let article = citation.one("Article")?;
    let title = article.one("ArticleTitle")?.mixed()?;
    let mut identifiers = RecordIdentifiers {
        pmid: Some(pmid.clone()),
        ..RecordIdentifiers::default()
    };
    let mut elocation_ids = Vec::new();
    for location in article.all("ELocationID") {
        let value = location.scalar()?;
        let kind = location
            .attributes
            .get("EIdType")
            .ok_or_else(|| invalid("electronic location has no identifier type"))?;
        if kind == "doi" {
            if identifiers
                .doi
                .as_ref()
                .is_some_and(|previous| previous != &value)
            {
                return Err(invalid("conflicting repeated article identifier"));
            }
            identifiers.doi = Some(value.clone());
        }
        elocation_ids.push(json!({"type":kind,"value":value,"attributes":location.attributes}));
    }
    if let Some(data) = node.optional("PubmedData")?
        && let Some(list) = data.optional("ArticleIdList")?
    {
        list.container(&["ArticleId"], "ArticleId")?;
        for identifier in list.all("ArticleId") {
            let value = identifier.scalar()?;
            let target = match identifier.attributes.get("IdType").map(String::as_str) {
                Some("doi") => Some(&mut identifiers.doi),
                Some("pmc") => Some(&mut identifiers.pmcid),
                Some("pubmed") => Some(&mut identifiers.pmid),
                _ => None,
            };
            if let Some(target) = target {
                if target.as_ref().is_some_and(|prior| prior != &value) {
                    return Err(invalid("conflicting repeated article identifier"));
                }
                *target = Some(value);
            }
        }
    }
    let mut abstract_sections = Vec::new();
    let mut abstract_lines = Vec::new();
    if let Some(abstract_node) = article.optional("Abstract")? {
        abstract_node.container(&["AbstractText", "CopyrightInformation"], "AbstractText")?;
        for section in abstract_node.all("AbstractText") {
            let text = section.mixed()?;
            let label = section.attributes.get("Label");
            let category = section.attributes.get("NlmCategory");
            abstract_lines.push(
                label
                    .filter(|label| !label.trim().is_empty())
                    .map_or_else(|| text.clone(), |label| format!("{label}: {text}")),
            );
            abstract_sections.push(json!({"label":label,"nlm_category":category,"text":text,"attributes":section.attributes}));
        }
    }
    let mut authors = Vec::new();
    let mut author_components = Vec::new();
    if let Some(list) = article.optional("AuthorList")? {
        list.container(&["Author"], "Author")?;
        for author in list.all("Author") {
            let last = author.field("LastName")?;
            let fore = author.field("ForeName")?;
            let initials = author.field("Initials")?;
            let suffix = author.field("Suffix")?;
            let collective = author.field("CollectiveName")?;
            if collective.is_some()
                && (last.is_some() || fore.is_some() || initials.is_some() || suffix.is_some())
            {
                return Err(invalid("author mixes collective and personal identity"));
            }
            let rendered = if let Some(name) = &collective {
                name.clone()
            } else {
                let family = last
                    .as_deref()
                    .ok_or_else(|| invalid("personal author has no family name"))?;
                [
                    Some(family),
                    fore.as_deref().or(initials.as_deref()),
                    suffix.as_deref(),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" ")
            };
            authors.push(rendered);
            author_components.push(json!({"last_name":last,"fore_name":fore,"initials":initials,"suffix":suffix,"collective_name":collective}));
        }
    }
    let journal = article.optional("Journal")?;
    let container_title = journal
        .map(|node| node.field("Title"))
        .transpose()?
        .flatten();
    let date = journal
        .map(|node| node.optional("JournalIssue"))
        .transpose()?
        .flatten()
        .map(|node| node.optional("PubDate"))
        .transpose()?
        .flatten();
    let mut pub_date = BTreeMap::new();
    if let Some(date) = date {
        for key in ["Year", "Month", "Day", "Season", "MedlineDate"] {
            if let Some(value) = date.field(key)? {
                pub_date.insert(key, value);
            }
        }
        if pub_date.is_empty() {
            return Err(invalid(
                "present publication date has no supported components",
            ));
        }
    }
    if pub_date.contains_key("MedlineDate") && pub_date.len() > 1 {
        return Err(invalid("ambiguous mixed publication date forms"));
    }
    let publication_year = pub_date
        .get("Year")
        .map(|year| {
            year.parse::<i32>()
                .map_err(|_| invalid("publication year is malformed"))
        })
        .transpose()?;
    let parts = ["MedlineDate", "Year", "Month", "Day", "Season"]
        .into_iter()
        .filter_map(|key| pub_date.get(key).map(String::as_str))
        .collect::<Vec<_>>();
    let publication_date = (!parts.is_empty()).then(|| parts.join(" "));
    let languages = article
        .all("Language")
        .map(Node::scalar)
        .collect::<Result<Vec<_>, _>>()?;
    let mut subjects = Vec::new();
    let mut mesh_headings = Vec::new();
    if let Some(list) = citation.optional("MeshHeadingList")? {
        list.container(&["MeshHeading"], "MeshHeading")?;
        for heading in list.all("MeshHeading") {
            heading.container(&["DescriptorName", "QualifierName"], "DescriptorName")?;
            let descriptor = heading.one("DescriptorName")?.scalar()?;
            let qualifiers = heading
                .all("QualifierName")
                .map(Node::scalar)
                .collect::<Result<Vec<_>, _>>()?;
            subjects.push(descriptor.clone());
            mesh_headings.push(json!({"descriptor":descriptor,"qualifiers":qualifiers}));
        }
    }
    let record = BibliographicRecord {
        schema_version: evidence_search_contracts::BIBLIOGRAPHIC_RECORD_SCHEMA_VERSION.to_owned(),
        record_id: format!("pubmed-{pmid}"),
        source_receipt_id: "pending-receipt".to_owned(),
        native_id: pmid.clone(),
        kind: RecordKind::JournalArticle,
        identifiers,
        title,
        abstract_text: (!abstract_lines.is_empty()).then(|| abstract_lines.join("\n")),
        authors,
        container_title,
        publication_year,
        publication_date,
        languages,
        subjects,
        urls: vec![format!("https://pubmed.ncbi.nlm.nih.gov/{pmid}/")],
        provider_metadata: json!({"format":"pubmed-efetch-xml","abstract_sections":abstract_sections,"author_components":author_components,"pub_date":pub_date,"mesh_headings":mesh_headings,"elocation_ids":elocation_ids}),
    };
    record
        .validate()
        .map_err(|_| invalid("normalised citation violates record contract"))?;
    Ok(record)
}

/// Parse only `PubmedArticle` citation/abstract XML and reconcile the exact PMID batch.
///
/// Limits: 8 `MiB` input, depth 64, 1,000 records, 100,000 elements, 64 `KiB` per
/// mapped field and 4 `MiB` cumulative decoded tree text (ancestor copies count).
/// Field text is checked after bounded tree construction; it is not an allocation cap.
/// All DTDs/custom entities are rejected. This does not fetch or certify full text.
pub fn parse_pubmed_fetch_page(
    xml: &str,
    requested_pmids: &[String],
) -> Result<ProviderPage, ProviderError> {
    let requested = requested_set(requested_pmids)?;
    let root = decode_tree(xml)?;
    if root.name != "PubmedArticleSet"
        || root.direct_nonwhitespace
        || root.children.len() > MAX_RECORDS
    {
        return Err(invalid("unsupported EFetch root or record count limit"));
    }
    let mut records = Vec::new();
    let mut seen = BTreeSet::new();
    for node in &root.children {
        if node.name != "PubmedArticle" {
            return Err(invalid("unsupported EFetch record variant"));
        }
        let record = parse_record(node)?;
        if !seen.insert(record.native_id.clone()) {
            return Err(invalid("duplicate returned PMID"));
        }
        records.push(record);
    }
    if seen.iter().map(String::as_str).collect::<BTreeSet<_>>() != requested {
        return Err(invalid("returned PMIDs differ from requested batch"));
    }
    Ok(ProviderPage {
        schema_version: evidence_search_contracts::PROVIDER_PAGE_SCHEMA_VERSION.to_owned(),
        total_available: Some(
            u64::try_from(records.len())
                .map_err(|_| invalid("record count exceeds platform range"))?,
        ),
        records,
        next_cursor: None,
        diagnostics: BTreeMap::from([(
            "raw_response_digest".to_owned(),
            Value::String(blake3::hash(xml.as_bytes()).to_hex().to_string()),
        )]),
    })
}

fn valid_name(name: &str) -> bool {
    name.bytes()
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

#[cfg(test)]
mod tests {
    #[test]
    fn decoded_text_budget_rejects_before_append() {
        let mut text = String::new();
        let mut total = super::MAX_TEXT - 1;
        assert!(super::append_text(&mut text, "x", &mut total).is_ok());
        assert_eq!(total, super::MAX_TEXT);
        assert!(super::append_text(&mut text, "y", &mut total).is_err());
        assert_eq!(text, "x");
    }
}
