# 43: Adversarial Advocate-Skeptic-Arbiter screening deliberation

## Objective

Replace single-pass machine screening prompts with a zero-trust, three-role adversarial deliberation protocol (`AdvocateAgent`, `SkepticAgent`, and `CalibratedArbiter`) in `crates/searchright-agent` and `crates/searchright-screening` that requires verbatim source-span citations for both inclusion and exclusion arguments, quantifies epistemic divergence across eligibility criteria, routes ambiguous or contested records to human reviewers, and strictly preserves non-autonomous final exclusion guards.

## Scope

- Isolated three-role screening protocol (`AdvocateAgent`, `SkepticAgent`, `CalibratedArbiter`) with strict context separation between Advocate and Skeptic
- Mandatory verbatim source-span citation validation for every Advocate inclusion claim and Skeptic exclusion claim against the candidate record text
- Criterion-by-criterion epistemic divergence calculator measuring disagreement entropy and abstention triggers between Advocate and Skeptic
- Calibrated Arbiter synthesis rule enforcing conservative recall preservation and routing any contested or low-confidence record to human adjudication
- Immutable deliberation transcript ledger preserving machine role identities distinctly from human dual-reviewer consensus

## Requirements owned

`SR-099`

## Acceptance contract

- All source deliverables exist, are contract-linked and pass the named network-free checks.
- Higher evidence is promoted only by current reproducible receipts at the claimed level.
- Remote, downstream, human and publication work remains approval-gated and reversible.
- Safety, privacy, migration, support and public-claim boundaries are reviewed.

## Granular Agent Execution Playbook

Follow these step-by-step instructions to implement the three-role adversarial deliberation protocol in `crates/searchright-contracts`, `crates/searchright-screening`, and `crates/searchright-agent`.

### Task 1: Define Adversarial Deliberation Transcript Contracts (`crates/searchright-contracts`)

1. **Target Files**:
   - `crates/searchright-contracts/src/screening.rs`
   - `crates/searchright-contracts/src/lib.rs`
2. **Data Structures to Add**:
   - `DeliberationRole`:
     - Enum variants: `Advocate`, `Skeptic`, `CalibratedArbiter`
   - `CriterionArgument`:
     - `criterion_id: String`
     - `role: DeliberationRole`
     - `proposed_status: String` (`"met"`, `"unmet"`, `"unclear"`)
     - `confidence_bps: u16` (`0..=10000`)
     - `verbatim_evidence_quote: String`
     - `byte_range_in_record: (usize, usize)`
     - `rationale: String`
   - `ArbiterDisposition`:
     - Enum variants:
       - `RecommendInclude { confidence_bps: u16 }`
       - `EscalateToHumanConflict { divergence_bps: u16, contested_criteria: Vec<String> }`
       - `CandidateExcludeRequiresHumanConfirmation { confidence_bps: u16, disqualifying_criteria: Vec<String> }`
   - `DeliberationTranscript`:
     - `transcript_id: String`
     - `record_id: String`
     - `record_text_sha256: String`
     - `policy_version: String`
     - `advocate_arguments: Vec<CriterionArgument>`
     - `skeptic_arguments: Vec<CriterionArgument>`
     - `epistemic_divergence_bps: u16`
     - `arbiter_disposition: ArbiterDisposition`
     - `counts_as_human_consensus: bool` (MUST always be `false`; validated fail-closed)
3. **Verification Command**:
   - `cargo test --locked -p searchright-contracts`

### Task 2: Implement Verbatim Span Guard & Epistemic Divergence Calculator (`crates/searchright-screening`)

1. **Target Files**:
   - `crates/searchright-screening/src/deliberation.rs` (new module, exported in `crates/searchright-screening/src/lib.rs`)
2. **Algorithms to Implement**:
   - **Verbatim Quote Guard**:
     - `pub fn validate_role_arguments(record_text: &str, expected_role: DeliberationRole, args: &[CriterionArgument]) -> Result<(), ScreeningError>`:
       - Reject if `args.is_empty()`.
       - For each `CriterionArgument`:
         - Assert `arg.role == expected_role`.
         - Assert `arg.confidence_bps <= 10000`.
         - Check `arg.byte_range_in_record.0 < arg.byte_range_in_record.1 && arg.byte_range_in_record.1 <= record_text.len()`.
         - Assert `&record_text.as_bytes()[arg.byte_range_in_record.0..arg.byte_range_in_record.1] == arg.verbatim_evidence_quote.as_bytes()`. (Prevents hallucinated quotes from either Advocate or Skeptic).
   - **Epistemic Divergence Formula**:
     - `pub fn compute_epistemic_divergence_bps(advocate: &[CriterionArgument], skeptic: &[CriterionArgument]) -> (u16, Vec<String>)`:
       - Match arguments by `criterion_id`.
       - For each criterion $c$, map `(status, confidence_bps)` to a signed inclusion belief $b \in [-10000, +10000]$:
         - `"met"` $\rightarrow +\text{confidence\_bps}$
         - `"unmet"` $\rightarrow -\text{confidence\_bps}$
         - `"unclear"` $\rightarrow 0$
       - Criterion divergence is $d_c = \frac{|b_{\text{adv}, c} - b_{\text{skep}, c}|}{2} \in [0, 10000]$.
       - If $d_c \ge 3000$ or either role reports `"unclear"`, add `criterion_id` to `contested_criteria`.
       - Return `(max_c d_c, contested_criteria)`.
   - **Calibrated Arbiter Decision Rule**:
     - `pub fn adjudicate_deliberation(record_id: &str, record_text: &str, policy_version: &str, advocate: Vec<CriterionArgument>, skeptic: Vec<CriterionArgument>, max_exclude_divergence_bps: u16, min_exclude_confidence_bps: u16) -> Result<DeliberationTranscript, ScreeningError>`:
       - Validate both `advocate` and `skeptic` arguments against `record_text`.
       - Compute `(divergence_bps, contested_criteria)`.
       - If `!contested_criteria.is_empty()` or `divergence_bps > max_exclude_divergence_bps`, emit `ArbiterDisposition::EscalateToHumanConflict`.
       - Else if any criterion is agreed `"unmet"` by both roles with confidence $\ge \text{min\_exclude\_confidence\_bps}$, emit `ArbiterDisposition::CandidateExcludeRequiresHumanConfirmation` (never a final autonomous exclusion).
       - Else emit `ArbiterDisposition::RecommendInclude`.
3. **Unit Tests to Add**:
   - **Hallucinated Span Rejection Test**: Pass a Skeptic argument whose `verbatim_evidence_quote` does not match `record_text` at `byte_range_in_record`; assert `Err(ScreeningError::InvalidEvidenceSpan)`.
   - **Divergence Escalation Test**: Advocate claims `"met"` (9000 bps) on Population criterion while Skeptic claims `"unmet"` (8500 bps); assert `divergence_bps == 8750` and `ArbiterDisposition::EscalateToHumanConflict`.
   - **Non-Autonomous Exclusion Guard Test**: Even when both Advocate and Skeptic agree `"unmet"` at 9900 bps, assert disposition is `CandidateExcludeRequiresHumanConfirmation` and `counts_as_human_consensus == false`.
4. **Verification Command**:
   - `cargo test --locked -p searchright-screening`

### Task 3: Integrate Deliberation Workflow into Agent Runtime (`crates/searchright-agent`)

1. **Target Files**:
   - `crates/searchright-agent/src/lib.rs`
2. **Behavior to Implement**:
   - Add context-isolation check `ensure_adversarial_isolation(advocate_prompt_hash: &str, skeptic_prompt_hash: &str) -> Result<(), AgentError>` verifying that the Skeptic prompt does not contain the Advocate's output or rationale (preventing sycophantic cascading).
   - Reject any attempt to convert a `DeliberationTranscript` directly into a final `ScreeningDecision::Excluded` without a verified human confirmation signature.
3. **Verification Command**:
   - `cargo test --locked -p searchright-agent`

## Out of scope

Claims or capabilities owned by later evidence gates remain explicit blockers rather than inferred completion.
