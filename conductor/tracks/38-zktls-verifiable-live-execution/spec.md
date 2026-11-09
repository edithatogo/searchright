# 38: Verifiable live two-source execution and zkTLS provenance receipts

## Objective

Wire real, resumable two-source live retrieval (PubMed E-utilities and Europe PMC REST) through the shared `evidence-search-core` runtime, CLI and MCP surfaces, and upgrade `.srpack` execution receipts with cryptographic zkTLS/TLSNotary transcript commitments so a third party can verify wire-level search authenticity without trusting the executing host.

## Scope

- Stateful cursor pagination, HTTP 429 Retry-After backoff and token-bucket rate governance in the shared live provider runtime
- Durable checkpoint and resume state machine preventing duplicate record emission or lost decisions across interrupted runs
- Live two-source execution wiring for PubMed and Europe PMC through CLI and MCP under explicit local authority and budget guards
- Cryptographic zkTLS transcript commitment and redacted server-certificate chain attestation structures in provenance receipts
- Independent offline `.srpack` verifier checking TLS transcript Merkle commitments, query hashes and retrieved record digests

## Requirements owned

`SR-094`

## Acceptance contract

- All source deliverables exist, are contract-linked and pass the named network-free checks.
- Higher evidence is promoted only by current reproducible receipts at the claimed level.
- Remote, downstream, human and publication work remains approval-gated and reversible.
- Safety, privacy, migration, support and public-claim boundaries are reviewed.

## Granular Agent Execution Playbook

This playbook is written for deterministic, step-by-step execution by an autonomous coding agent. Follow the tasks in exact sequence. Do not skip tests or weaken existing authority checks.

### Task 1: Define zkTLS Transcript Attestation Contracts (`crates/evidence-search-contracts` & `crates/searchright-contracts`)

1. **Target Files**:
   - `crates/evidence-search-contracts/src/lib.rs`
   - `crates/searchright-contracts/src/lib.rs`
2. **Data Structures to Add**:
   - In `crates/evidence-search-contracts/src/lib.rs`, define `TlsTranscriptCommitment`:
     - `protocol_version: String` (e.g. `"org.searchright.zktls-commitment.v1"`)
     - `server_name: String` (e.g. `"eutils.ncbi.nlm.nih.gov"` or `"www.ebi.ac.uk"`)
     - `cipher_suite: String`
     - `cert_chain_sha256: Vec<String>` (SHA-256 hex digests of the DER-encoded server X.509 chain)
     - `request_commitment_sha256: String` (SHA-256 of canonical HTTP request bytes after credential-header redaction)
     - `response_merkle_root_sha256: String` (SHA-256 Merkle root over 4 KiB chunks of the HTTP response body)
     - `notary_signature_hex: Option<String>` (Optional hex-encoded secp256k1/Ed25519 notary signature over `(server_name, request_commitment_sha256, response_merkle_root_sha256, timestamp_utc)`)
     - `redacted_byte_ranges: Vec<(usize, usize)>` (Byte offsets of redacted API keys or bearer tokens in the request stream)
   - Derive `Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema`.
3. **Validation Logic**:
   - Implement `pub fn validate_commitment(&self) -> Result<(), String>` on `TlsTranscriptCommitment`:
     - Reject empty `server_name` or `cert_chain_sha256`.
     - Verify `request_commitment_sha256` and `response_merkle_root_sha256` are valid 64-character lowercase hex strings.
     - Verify every `(start, end)` in `redacted_byte_ranges` satisfies `start < end`.
4. **Verification Command**:
   - `cargo test --locked -p evidence-search-contracts -p searchright-contracts`

### Task 2: Implement Resumable Cursor Pagination and 429 Backoff State Machine (`crates/evidence-search-core`)

1. **Target Files**:
   - `crates/evidence-search-core/src/lib.rs`
2. **Data Structures to Add**:
   - `PaginationCheckpoint`:
     - `run_id: String`
     - `provider_id: String`
     - `query_sha256: String`
     - `page_index: u32`
     - `next_cursor: Option<String>`
     - `numeric_offset: u64`
     - `emitted_record_hashes: Vec<String>`
     - `accumulated_commitments: Vec<TlsTranscriptCommitment>`
   - `RetryDirective`:
     - `attempt: u32`
     - `backoff_ms: u64`
     - `reason: String`
3. **Functions to Implement**:
   - `pub fn compute_429_backoff_ms(attempt: u32, retry_after_header: Option<&str>, base_delay_ms: u64, max_delay_ms: u64) -> u64`:
     - If `retry_after_header` parses as a positive integer $s$, use $\min(\text{max\_delay\_ms}, s \times 1000)$.
     - Otherwise compute full-jitter deterministic exponential backoff: $\min(\text{max\_delay\_ms}, \text{base\_delay\_ms} \times 2^{\min(\text{attempt}, 10)})$.
   - `pub fn resume_from_checkpoint(checkpoint: &PaginationCheckpoint, incoming_record_id: &str) -> bool`:
     - Compute SHA-256 of `incoming_record_id` and return `false` if already present in `checkpoint.emitted_record_hashes` (preventing duplicate emission on mid-page crash recovery).
4. **Unit Tests to Add**:
   - Test `Retry-After: 3` yields `3000` ms.
   - Test exponential growth caps at `max_delay_ms`.
   - Test `resume_from_checkpoint` skips previously emitted records and accepts unseen records.
5. **Verification Command**:
   - `cargo test --locked -p evidence-search-core`

### Task 3: Wire Two-Source Live Adapter for PubMed and Europe PMC (`crates/searchright-connectors` & `crates/searchright-store`)

1. **Target Files**:
   - `crates/searchright-connectors/src/lib.rs`
   - `crates/searchright-store/src/lib.rs`
2. **Functions to Implement**:
   - In `crates/searchright-connectors/src/lib.rs`:
     - Implement deterministic chunk Merkle root builder `pub fn compute_response_merkle_root(body: &[u8]) -> String` that splits `body` into 4096-byte slices, hashes each leaf with SHA-256, and folds pairwise up to a single 64-char hex root.
     - Implement `pub fn redact_request_headers(raw_request: &[u8], secrets: &[&[u8]]) -> (Vec<u8>, Vec<(usize, usize)>)` that replaces any occurrence of `secrets` in `raw_request` with `0x2A` (`'*'`) of equal byte length and records exact `(start, end)` byte offsets.
   - In `crates/searchright-store/src/lib.rs`:
     - Add checkpoint persistence helper `save_pagination_checkpoint` and `load_pagination_checkpoint` writing atomically to `<store_dir>/checkpoints/<run_id>-<provider_id>.json`.
3. **Unit Tests to Add**:
   - Verify `redact_request_headers` masks NCBI `api_key` values without shifting byte offsets of subsequent headers.
   - Verify `compute_response_merkle_root` is deterministic and changes if 1 byte in a 16 KiB XML payload is flipped.
4. **Verification Command**:
   - `cargo test --locked -p searchright-connectors -p searchright-store`

### Task 4: Expose Live Two-Source Execution in Facade, CLI, and MCP (`crates/searchright`, `crates/searchright-cli`, `crates/searchright-mcp`)

1. **Target Files**:
   - `crates/searchright/src/engine.rs`
   - `crates/searchright-cli/src/main.rs`
   - `crates/searchright-mcp/src/lib.rs`
2. **Behavior to Implement**:
   - Require an explicit `LiveExecutionAuthorization` containing `principal_id`, `allowed_providers: Vec<String>`, `max_requests: u32`, and `allow_network: bool`.
   - Fail closed with `SearchrightError::AuthorityDenied` if `allow_network` is `false` or if a requested provider is not in `allowed_providers`.
   - When authorized, execute PubMed and Europe PMC in sequence, persisting `PaginationCheckpoint` after each page, deduplicating retrieved records conservatively, and attaching `TlsTranscriptCommitment` entries to the run receipt.
3. **Verification Command**:
   - `cargo test --locked -p searchright -p searchright-cli -p searchright-mcp`

### Task 5: Offline `.srpack` zkTLS Proof Verifier (`crates/searchright-provenance`)

1. **Target Files**:
   - `crates/searchright-provenance/src/lib.rs`
2. **Functions to Implement**:
   - Implement `pub fn verify_zktls_receipt_chain(commitments: &[TlsTranscriptCommitment], expected_Bodies: &[(&str, &[u8])]) -> Result<(), ProvenanceError>`:
     - Check each `TlsTranscriptCommitment::validate_commitment()`.
     - Recompute `compute_response_merkle_root(body)` for each stored payload and verify exact equality with `response_merkle_root_sha256`.
     - Reject any receipt where redacted byte ranges overlap with the URL path or query parameters rather than header/query secret values.
3. **Verification Command**:
   - `cargo test --locked -p searchright-provenance`

## Out of scope

Claims or capabilities owned by later evidence gates remain explicit blockers rather than inferred completion.
