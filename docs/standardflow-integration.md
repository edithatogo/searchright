# StandardFlow integration boundary

SearchRight and StandardFlow are complementary systems, not interchangeable implementations.

## Authority

SearchRight owns review planning, source selection, query translation, retrieval, deduplication, screening, record–report–study linkage, living-update lineage and the authoritative PRISMA count ledger. StandardFlow owns standards identity and versions, applicability, normalized requirements, evidence expectations and standards-derived artefact rendering.

## Exchange

SearchRight may consume an exact-pinned, rights-cleared Canonical Standard Pack and artefact recipe. It may emit a versioned `review_flow` evidence envelope containing its authoritative counts and lineage reference. StandardFlow may validate the envelope against the selected recipe and render it; it cannot alter screening decisions, silently repair inconsistent counts or infer absent lineage.

## Failure and rollback

The integration is local, read-only, network-off and write-off by default. SearchRight retains its current versioned standards packs and native rendering path as the degraded mode. Pin changes require review of schemas, rights, fixtures and semantic differences. A failed canary restores the prior pin and preserves the receipt.

The verified StandardFlow foundation is pinned at `f1e7b6c6e283865c1b7dbb34dbe33d8bf6f6c799` from `edithatogo/standards_check#59`. The primary foundation contract and Rust gates passed at this revision; merge, consumer parity and external validation remain separate evidence.

## Optional analytical discovery snapshot (Track 13 / Track 08)

A separate experimental, read-only consumer accepts the producer's
`dev.standardflow.equator-analytical-snapshot.v1` manifest with four typed
Parquet tables. It requires the manifest digest, producer commit and upstream
source revision as explicit arguments; historical replay additionally requires
opt-in. No canonical pack, review state, screening decision or PRISMA count
can change. All rights remain unassessed and conflicts remain held. The
existing canonical integration passport and lock are deliberately unchanged.

The initial qualification input is [producer historical replay 14918300555](https://github.com/edithatogo/standards_check/actions/runs/45388593899/artifacts/14918300555):
720 identities, 145 acquisitions, 95 item fingerprints and 2241 review
findings. Manifest SHA-256
`d8bc7e97d1644967e90f7ff6b1b61f3cdba30e43f8e5b1a3e94be1c4b716f893`,
producer tested merge revision `e68ece5dfdbeb1a312cea632686e3c58a9125003`,
source revision `d74413b15d266334067e1e5c104957601be3ba6c`.
This is an immutable historical replay, not a fresh acquisition or a
production integration pin. The artifact is retained for 90 days; promotion
requires an approved durable publication and consumer evidence. A new
manifest requires explicit review and pins, never automatic substitution.

```sh
python -m pip install -r requirements/standardflow-analytics.txt
python scripts/check_standardflow_analytical_snapshot.py /path/to/unpacked-snapshot \
  --manifest-sha256 d8bc7e97d1644967e90f7ff6b1b61f3cdba30e43f8e5b1a3e94be1c4b716f893 \
  --producer-revision e68ece5dfdbeb1a312cea632686e3c58a9125003 \
  --source-revision d74413b15d266334067e1e5c104957601be3ba6c \
  --allow-historical-replay
```

The output is a JSON validation receipt, not imported review state.
The optional hosted canary retrieves the exact historical artifact and
runs the same local checker; it cannot select or approve a reporting
guideline. The consumer and producer qualifications are separate.
Tracked by [#843](https://github.com/edithatogo/searchright/issues/843)
and [StandardFlow #142](https://github.com/edithatogo/standards_check/pull/142).
