# 42: Sub-document byte-span entailment and transitive retraction propagation

## Objective

Extend full-text extraction (`crates/searchright-citeweft`), study aggregation (`crates/searchright-study`), and living updates (`crates/searchright-living`) with proof-carrying sub-document coordinates (`doc_sha256`, page number, exact UTF-8 byte offsets, and PDF bounding boxes), deterministic natural-language entailment verification, and automated transitive retraction/errata poisoning so that every extracted finding is anchored to verifiable source bytes and any post-publication retraction immediately quarantines dependent synthesis claims.

## Scope

- Proof-carrying sub-document anchor contract binding extracted claims to `doc_sha256`, page index, UTF-8 byte range, bounding box and verbatim quote digest
- Deterministic byte-slice verifier rejecting fabricated quotes, offset mismatches or out-of-bounds span references before evidence admission
- Entailment verification gate scoring whether the anchored verbatim span supports, contradicts or is neutral toward the structured extraction claim
- Retraction Watch and Crossref update-notice ingestion contract classifying retractions, expressions of concern and major errata
- Transitive retraction poisoning across the Report-to-Study-to-Synthesis graph flagging affected forest-plot rows and PRISMA counts for mandatory human review

## Requirements owned

`SR-098`

## Acceptance contract

- All source deliverables exist, are contract-linked and pass the named network-free checks.
- Higher evidence is promoted only by current reproducible receipts at the claimed level.
- Remote, downstream, human and publication work remains approval-gated and reversible.
- Safety, privacy, migration, support and public-claim boundaries are reviewed.

## Granular Agent Execution Playbook

Follow these step-by-step instructions to implement proof-carrying sub-document spans, entailment checking, and transitive retraction propagation.

### Task 1: Define Sub-Document Coordinate & Retraction Notice Contracts (`crates/searchright-contracts`)

1. **Target Files**:
   - `crates/searchright-contracts/src/document.rs`
   - `crates/searchright-contracts/src/study.rs`
2. **Data Structures to Add**:
   - In `crates/searchright-contracts/src/document.rs`:
     - `PdfBoundingBox`:
       - `page_number: u32` (1-indexed)
       - `x0_pt: f32`, `y0_pt: f32`, `x1_pt: f32`, `y1_pt: f32`
     - `ProofCarryingSpan`:
       - `doc_sha256: String`
       - `byte_start: usize`
       - `byte_end: usize`
       - `verbatim_quote: String`
       - `quote_sha256: String`
       - `bbox: Option<PdfBoundingBox>`
     - `EntailmentVerdict`:
       - Enum variants: `Entailed`, `Neutral`, `Contradicted`, `Unverified`
     - `GroundedClaim`:
       - `claim_id: String`
       - `study_id: String`
       - `report_id: String`
       - `field_name: String` (e.g. `"sample_size"`, `"primary_endpoint_hr"`, `"blinding_method"`)
       - `extracted_value: String`
       - `supporting_span: ProofCarryingSpan`
       - `entailment_verdict: EntailmentVerdict`
       - `entailment_score_bps: u16` (`0..=10000` basis points)
   - In `crates/searchright-contracts/src/study.rs`:
     - `IntegrityNoticeKind`:
       - Enum variants: `Retraction`, `ExpressionOfConcern`, `Erratum`
     - `IntegrityNotice`:
       - `notice_id: String`
       - `target_doi: Option<String>`
       - `target_pmid: Option<String>`
       - `kind: IntegrityNoticeKind`
       - `published_date: String`
       - `source_authority: String` (e.g. `"retraction_watch"`, `"crossref_update"`, `"pubmed_erratum"`)
     - `SynthesisQuarantineFlag`:
       - `study_id: String`
       - `triggering_report_id: String`
       - `notice: IntegrityNotice`
       - `affected_claim_ids: Vec<String>`
       - `requires_human_adjudication: bool`
3. **Verification Command**:
   - `cargo test --locked -p searchright-contracts`

### Task 2: Implement Byte-Slice & Entailment Verifier (`crates/searchright-citeweft`)

1. **Target Files**:
   - `crates/searchright-citeweft/src/lib.rs`
2. **Functions to Implement**:
   - `pub fn verify_proof_carrying_span(document_bytes: &[u8], span: &ProofCarryingSpan) -> Result<(), CiteweftError>`:
     - Compute SHA-256 of `document_bytes` and assert exact match with `span.doc_sha256`.
     - Check `span.byte_start < span.byte_end && span.byte_end <= document_bytes.len()`.
     - Extract `&document_bytes[span.byte_start..span.byte_end]`, validate UTF-8, and assert exact byte equality with `span.verbatim_quote.as_bytes()`.
     - Compute SHA-256 of `span.verbatim_quote.as_bytes()` and assert exact match with `span.quote_sha256`.
     - If `span.bbox` is `Some(b)`, assert `b.page_number >= 1 && b.x0_pt < b.x1_pt && b.y0_pt < b.y1_pt`.
   - `pub fn verify_claim_entailment(claim: &GroundedClaim, min_score_bps: u16) -> Result<EntailmentVerdict, CiteweftError>`:
     - Verify that key numeric tokens or controlled clinical terms in `claim.extracted_value` appear in `claim.supporting_span.verbatim_quote` without negation markers (`"not"`, `"no"`, `"excluded"`, `"failed to"`) immediately preceding the value when `claim.extracted_value` asserts a positive presence.
     - Reject any claim where `claim.entailment_verdict == EntailmentVerdict::Contradicted` or `claim.entailment_score_bps < min_score_bps`.
3. **Unit Tests to Add**:
   - Test off-by-one byte offset or single-character hallucination in `verbatim_quote` fails closed with `CiteweftError::SpanByteMismatch`.
   - Test numeric mismatch (e.g. `extracted_value = "n=450"` when span text says `"enrolled 405 patients"`) fails entailment check.
4. **Verification Command**:
   - `cargo test --locked -p searchright-citeweft`

### Task 3: Implement Transitive Retraction Poisoning Graph (`crates/searchright-study` & `crates/searchright-living`)

1. **Target Files**:
   - `crates/searchright-study/src/lib.rs`
   - `crates/searchright-living/src/lib.rs`
2. **Functions to Implement**:
   - `pub fn propagate_integrity_notices(studies: &[StudyAggregate], claims: &[GroundedClaim], notices: &[IntegrityNotice]) -> Vec<SynthesisQuarantineFlag>`:
     - Match each `IntegrityNotice` by normalized DOI or PMID against every `Report` linked inside `studies`.
     - Whenever a report matches a `Retraction` or `ExpressionOfConcern`, quarantine the parent `StudyAggregate`, collect all `GroundedClaim` IDs derived from that report or study, and emit a `SynthesisQuarantineFlag` with `requires_human_adjudication: true`.
     - Never silently delete the retracted report; preserve the historical inclusion record alongside the quarantine flag so PRISMA living flow charts can report `"Studies retracted post-inclusion (n = k)"`.
3. **Unit Tests to Add**:
   - Test a multi-report study (1 primary trial report + 1 follow-up subgroup report): when the primary trial report receives a `Retraction` notice, verify the parent study and all claims anchored to it are flagged in `SynthesisQuarantineFlag`, while unrelated studies remain unflagged.
4. **Verification Command**:
   - `cargo test --locked -p searchright-study -p searchright-living`

## Out of scope

Claims or capabilities owned by later evidence gates remain explicit blockers rather than inferred completion.
