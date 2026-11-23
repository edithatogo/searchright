# StandardFlow integration boundary

SearchRight and StandardFlow are complementary systems, not interchangeable implementations.

## Authority

SearchRight owns review planning, source selection, query translation, retrieval, deduplication, screening, record–report–study linkage, living-update lineage and the authoritative PRISMA count ledger. StandardFlow owns standards identity and versions, applicability, normalized requirements, evidence expectations and standards-derived artefact rendering.

## Exchange

SearchRight may consume an exact-pinned, rights-cleared Canonical Standard Pack and artefact recipe. It may emit a versioned `review_flow` evidence envelope containing its authoritative counts and lineage reference. StandardFlow may validate the envelope against the selected recipe and render it; it cannot alter screening decisions, silently repair inconsistent counts or infer absent lineage.

## Failure and rollback

The integration is local, read-only, network-off and write-off by default. SearchRight retains its current versioned standards packs and native rendering path as the degraded mode. Pin changes require review of schemas, rights, fixtures and semantic differences. A failed canary restores the prior pin and preserves the receipt.

The verified StandardFlow foundation is pinned at `f1e7b6c6e283865c1b7dbb34dbe33d8bf6f6c799` from `edithatogo/standards_check#59`. The primary foundation contract and Rust gates passed at this revision; merge, consumer parity and external validation remain separate evidence.

## EQUATOR published-directory discovery candidate (not active)

[StandardFlow PR #112](https://github.com/edithatogo/standards_check/pull/112) has merged and provides a published-only directory, pinned at producer revision `f02701c4b6139c6d588b3bfcccbf2e7ef3069bb9` with **719** source paths, Git blob IDs and links. [`integration/candidates/equator-directory-20261123.json`](../integration/candidates/equator-directory-20261123.json) records a **proposed, read-only consumer reference** without changing the older active StandardFlow integration passport or SourceRight search/reporting contracts.

To validate explicitly acquired producer bytes offline:

```sh
python scripts/check_equator_directory_reference.py --catalogue /path/to/standards_check/source/equator/discovery.json
python -m unittest tests.test_equator_directory_reference
```

The validator checks the exact Git blob hash, pinned producer/upstream revisions, identity conservation, and unknown-content/rights rejection. It **does not fetch source data**, recommend standards, transcribe checklists, approve licence rights, change review policy, or claim an executing cross-repo adapter. Runtime selection and real downstream evidence remain with [#843](https://github.com/edithatogo/searchright/issues/843); only a subsequent rights-qualified producer/consumer contract can activate the integration.
