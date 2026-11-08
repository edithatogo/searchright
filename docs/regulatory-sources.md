# FDA and EMA regulatory records

Use an FDA application number or an EMA human medicine identifier to retrieve one publisher record. Each invocation writes the raw response, normalised record and receipt to a new directory. The FDA connector requests one matching application, retaining its product and submission arrays. The EMA connector checks the complete medicine catalogue and selects one human medicine. Its record includes the EPAR medicine-page link and authorised indication text where provided.

    python scripts/fetch_regulatory_record.py drugsfda BLA761034 fda-run
    python scripts/fetch_regulatory_record.py ema-medicines EMEA/H/C/004143 ema-run
    searchright source regulatory-record drugsfda BLA761034 fda-run/source.json

The fetch command requires Python 3.15, curl, and a built SearchRight CLI. The last command replays a saved response offline. These are exact-ID supplementary searches. A product's current status or a US submission date cannot on its own establish the date a specific indication was withdrawn abroad. Assessment documents and indication history require separate review.

Use `ema-epar` with a publisher document ID to capture the assessment document's metadata and original PDF link. For example, document 3787 is the Tecentriq public assessment report. Join to a medicine using the publisher's ema_product_number in the native metadata. Document metadata and translated links do not establish what the PDF says.
