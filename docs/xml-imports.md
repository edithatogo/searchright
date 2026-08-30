# Offline XML imports

PubMed XML and EndNote XML import use structural XML events. Record and field
elements may carry attributes. Inline markup contributes its text, entity and
numeric character references are decoded once, and CDATA remains literal.
Whitespace is normalised in extracted bibliographic fields; quarantine retains
the original input bytes and one-based line span.

PubMed primary identifiers are selected by their owning paths: the citation's
PMID and the article's PubmedData/ArticleIdList DOI with IdType="doi". Identifiers
inside cited references never become the imported report's identity. Bibliographic
fields are scoped to the primary citation/article; compact direct-root fields
remain readable. XML names and raw attribute values are checked; escaped
`&lt;` is valid in an attribute, but a literal `<` is not.

Each record is validated independently for balanced structure, attributes,
references and forbidden XML characters before bibliographic validation. An
unclosed record is quarantined at the next actual record-start event so that a
following valid record remains importable. Record-like text in comments or
CDATA is not a boundary. If tokenisation fails inside ambiguous markup, such as
an unfinished quoted attribute, the remaining affected input is quarantined
rather than guessed into records.

The importer does not fetch, validate against or expand DTDs and does not
resolve external entities. Unknown entity references are quarantined. A DTD
or XML declaration inside a record is rejected. A document-level declaration
or DTD is not evidence of schema conformance. This is a deterministic local
interchange contract, not proof of complete PubMed/EndNote schema support,
provider support, or methodological validity.
