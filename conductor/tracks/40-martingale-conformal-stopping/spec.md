# 40: Anytime-valid martingale stopping and conformal risk control

## Objective

Upgrade active-learning screening prioritization (`crates/searchright-ranking` and `crates/searchright-bench`) with anytime-valid sequential hypothesis testing (hypergeometric betting martingales / e-values) and conformal risk control so reviewers receive a mathematically rigorous, finite-sample upper bound on missed relevant studies ($P(\text{Recall} < \tau) \le \alpha$) that remains valid under continuous sequential peeking without leaking unscreened holdout labels.

## Scope

- Pluggable active-learning ranking worker interface with strict separation between observed training labels and sealed holdout pools
- Anytime-valid hypergeometric betting martingale and e-value wealth tracker for sequential without-replacement screening
- Conformal risk-controlling threshold calibrator bounding false-negative screening error rates at user-specified significance levels
- Tamper-evident stopping certificate recording martingale wealth trajectory, remaining pool size, target recall and significance level
- Sealed benchmark evaluation harness over SYNERGY and CLEF TAR datasets measuring workload reduction (WSS) and empirical stopping coverage

## Requirements owned

`SR-096`

## Acceptance contract

- All source deliverables exist, are contract-linked and pass the named network-free checks.
- Higher evidence is promoted only by current reproducible receipts at the claimed level.
- Remote, downstream, human and publication work remains approval-gated and reversible.
- Safety, privacy, migration, support and public-claim boundaries are reviewed.

## Granular Agent Execution Playbook

Follow these exact mathematical formulas and file targets to implement anytime-valid sequential stopping and conformal calibration in pure Rust.

### Task 1: Define Stopping Certificate & Martingale Contracts (`crates/searchright-contracts`)

1. **Target Files**:
   - `crates/searchright-contracts/src/screening.rs`
   - `crates/searchright-contracts/src/lib.rs`
2. **Data Structures to Add**:
   - `StoppingConfig`:
     - `target_recall: f64` (e.g. `0.95`, must satisfy $0.5 \le \tau < 1.0$)
     - `alpha: f64` (significance level, e.g. `0.05`, must satisfy $0.0001 \le \alpha \le 0.25$)
     - `min_screened_records: usize`
   - `MartingaleStep`:
     - `step_index: usize`
     - `record_id: String`
     - `is_relevant: bool`
     - `screened_relevant_so_far: usize`
     - `remaining_unscreened: usize`
     - `hypothesized_missed_rel: usize`
     - `bet_fraction_lambda: f64`
     - `e_value_wealth: f64`
     - `anytime_p_value: f64`
   - `StoppingCertificate`:
     - `certificate_version: String` (`"org.searchright.martingale-stop.v1"`)
     - `config: StoppingConfig`
     - `total_candidates: usize`
     - `screened_count: usize`
     - `observed_relevant_count: usize`
     - `max_plausible_unseen_relevant: usize`
     - `current_e_value_wealth: f64`
     - `anytime_p_value: f64`
     - `can_stop_safely: bool`
     - `trajectory_sha256: String`
3. **Verification Command**:
   - `cargo test --locked -p searchright-contracts`

### Task 2: Implement Hypergeometric Betting Martingale Engine (`crates/searchright-ranking`)

1. **Target Files**:
   - `crates/searchright-ranking/src/martingale.rs` (new module, exported in `crates/searchright-ranking/src/lib.rs`)
2. **Mathematical Formulation to Implement**:
   - Let $N$ be total retrieved candidate records. At step $t \in \{1, \dots, N\}$ (1-indexed), $t-1$ records have been screened, finding $R_{t-1}$ relevant items. The remaining pool size is $U_t = N - t + 1$.
   - To test whether achieved recall is at least $\tau \in (0, 1)$, the null hypothesis $H_0(r_{\text{rem}})$ states that the remaining unscreened pool of size $U_t$ contains at least $r_{\text{rem}}$ relevant documents, where:
     $$r_{\text{rem}} = \max\left(1, \left\lceil \frac{1 - \tau}{\tau} R_{t-1} \right\rceil \right)$$
   - Under $H_0(r_{\text{rem}})$ (sampling without replacement, worst-case random order bound), the conditional probability of observing a relevant document at draw $t$ is at least:
     $$\mu_t = \frac{r_{\text{rem}}}{U_t}$$
   - Let $Y_t \in \{0, 1\}$ be the observed human relevance label at step $t$. Using predictable Kelly-bounded betting fraction $\lambda_t \in [0, 0.5]$ against observing $Y_t = 0$:
     - Define factor $M_t(Y_t) = 1 + \lambda_t \left(\frac{\mu_t - Y_t}{\mu_t}\right)$ when $\mu_t > 0$.
     - Specifically:
       - If $Y_t = 0$ (irrelevant found), $\frac{\mu_t - 0}{\mu_t} = 1$, so $M_t(0) = 1 + \lambda_t$.
       - If $Y_t = 1$ (relevant found), $M_t(1) = 1 - \lambda_t \frac{1 - \mu_t}{\mu_t}$, clamped below at $10^{-6}$ by constraining $\lambda_t = \min\left(0.5, \frac{0.5 \mu_t}{1 - \mu_t + 10^{-9}}\right)$.
     - Since $\mathbb{E}_{H_0}[M_t(Y_t) \mid \mathcal{F}_{t-1}] \le 1$, the cumulative product $W_t = \prod_{i=1}^t M_i(Y_i)$ is a non-negative supermartingale with $W_0 = 1$.
     - By Ville's Maximal Inequality, $P_{H_0}\left(\exists t \ge 1 : W_t \ge \frac{1}{\alpha}\right) \le \alpha$.
     - The anytime-valid p-value at step $t$ is $p_t = \min\left(1.0, \min_{1 \le i \le t} \frac{1}{W_i}\right)$.
     - `can_stop_safely` is `true` if and only if `t >= config.min_screened_records && p_t <= config.alpha`.
3. **Functions to Implement**:
   - `pub fn evaluate_martingale_trajectory(total_candidates: usize, observations: &[(String, bool)], config: &StoppingConfig) -> Result<(StoppingCertificate, Vec<MartingaleStep>), RankingError>`
4. **Unit Tests to Add**:
   - **Ville's Inequality Calibration Test**: Simulate 500 synthetic streams under $H_0$ where true recall is below $\tau = 0.95$; assert the false-stop rate across the entire trajectory is $\le \alpha + 0.02$.
   - **Long Tail Zero-Yield Stopping Test**: Feed 20 relevant records followed by 400 consecutive irrelevant records in a pool of $N = 600$; assert `W_t` monotonically grows during the zero-yield tail and `can_stop_safely` flips to `true` once $W_t \ge 1 / \alpha$.
   - **Reset on Late Relevant Discovery Test**: Inject `is_relevant = true` after 150 irrelevant records; assert `W_t` drops sharply and `can_stop_safely` remains `false`.
5. **Verification Command**:
   - `cargo test --locked -p searchright-ranking`

### Task 3: Implement Conformal Risk-Controlling Threshold Calibrator (`crates/searchright-ranking`)

1. **Target Files**:
   - `crates/searchright-ranking/src/conformal.rs` (new module, exported in `crates/searchright-ranking/src/lib.rs`)
2. **Algorithm to Implement**:
   - Given a calibration set of $(s_i, y_i)_{i=1}^n$ where $s_i \in [0, 1]$ is the model score and $y_i \in \{0, 1\}$ is human ground truth:
   - For candidate threshold $\hat{\lambda} \in [0, 1]$, define the empirical false-negative loss on relevant items ($n_{\text{pos}} = \sum y_i$):
     $$\hat{R}_n(\hat{\lambda}) = \frac{1}{n_{\text{pos}} + 1} \left( 1 + \sum_{i : y_i = 1} \mathbb{I}(s_i < \hat{\lambda}) \right)$$
   - Select the largest threshold $\hat{\lambda}^* \in \{s_1, \dots, s_n, 0.0\}$ such that $\hat{R}_n(\hat{\lambda}^*) \le 1 - \tau$.
   - Implement `pub fn calibrate_conformal_threshold(calibration_scores: &[(f64, bool)], target_recall: f64) -> Result<f64, RankingError>`.
3. **Verification Command**:
   - `cargo test --locked -p searchright-ranking`

### Task 4: Wire Stopping Certificates into Sealed Benchmark Evaluator (`crates/searchright-bench`)

1. **Target Files**:
   - `crates/searchright-bench/src/lib.rs`
2. **Behavior to Implement**:
   - Ensure the ranking simulator never passes unscreened holdout labels into the ranker or martingale evaluator (`assert_no_holdout_leakage`).
   - Compute `realised_recall_at_stop` and `workload_saved_at_martingale_stop` alongside oracle WSS@95.
3. **Verification Command**:
   - `cargo test --locked -p searchright-bench`

## Out of scope

Claims or capabilities owned by later evidence gates remain explicit blockers rather than inferred completion.
