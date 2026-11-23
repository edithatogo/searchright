# Source-selection catalogue: assessed leads, not executable providers

**Work items:** [#835 source coverage umbrella](https://github.com/edithatogo/searchright/issues/835), [#836 inventory identity](https://github.com/edithatogo/searchright/issues/836) and [#834 RefHunter assessment](https://github.com/edithatogo/searchright/issues/834). **Primary Conductor owner:** Track 11 (review planning and source selection). Existing Track 02 (dialects), 04 (open providers), 08 (reporting), 20 (supplementary methods), 21 (licensed access) retain their own scope. Frontier priority, if required, belongs to Track 30. No historical track is reopened.

## Why a separate advisory catalogue?

The internal data file [`data/source-selection/catalogue.v1.json`](../data/source-selection/catalogue.v1.json) is a small, versioned **assessment index** for source choice. It is not a new provider registry, supported-database list, API adapter, rights grant, full reproduction of RefHunter, or a substitute for the canonical `ReviewPlan`. Implementations remain in `crates/`, provider manifests and the fixed [`discovery-source-methods.json`](../contracts/fixtures/discovery-source-methods.json) method catalogue. A source can have many platforms, and a platform can host many sources.

- **Source**: database/register/index identified by a stable `source_id`; aliases are secondary, never used as stable keys.
- **Platform**: user interface or API identified by a stable `platform_id` within a source link. Two distinct sources can share a platform (CENTRAL and CDSR on Cochrane Library); one source may have distinct platforms (MEDLINE on PubMed and Ovid). The platform ID remains consistent across links.
- **Provider**: an executable Searchright adapter governed by its *existing* provider manifest, policy, fixtures and opt-in live evidence. This assessment index intentionally has **no executable provider identifier** or `live` state.
- **Evidence**: `existing_contract_unverified` means an on-repository specification exists, **not** that the current service works. `manual_method_documented` means a declared manual method exists. `not_claimed` implies no support claim. `verification_state` records **source identity** checks, not completeness of capability, licence or native syntax verification.

The initial 12 entries cover different source types and access models. They are not 12 new integrations or an exhaustive sample of the RefHunter directory. The 107 entries visible in RefHunter on 23 November 2026 are **leads**, not entries automatically copied into this index.

## Evidence, rights and updates

A reviewer starts with a specific need, checks the relevant existing Searchright manifests, confirms the candidate source and platform against **first-party documentation**, records date, source scope, access and unknowns, and documents why complementary sources may be required. [RefHunter](https://refhunter.org/en/) is a useful **secondary directory**. Do not copy its descriptions or bulk-scrape its pages without an independently documented licence/permission decision; ordinary attribution is not permission to redistribute an entire directory. Source URLs and a short independently written status are the default.

For a catalogue update, keep `source_id` stable, record a first-party link/date and source limitations; reuse a platform identifier only where it denotes the same interface/access identity. Mark unresolved conflicts as `conflict_unresolved` and leave disposition `under_review` or `deferred`. A ceased source may be `retired` with context; no GitHub issue is required solely because a row exists. Never silently promote `execution_evidence`, amend protocol choices or assert translation equivalence. Native syntax, controlled-vocabulary mapping and export permissions must be verified separately.

Run `python3 scripts/check_source_catalogue.py` and `python3 -m unittest discover -s tests -p test_source_catalogue.py`; the normal [`scripts/verify.sh`](../scripts/verify.sh) already discovers these Python tests. Offline checks only ensure referential/identity invariants and conservative claim boundaries. They cannot establish current live provider behaviour, rights, completeness, expert PRESS acceptance, or comparative retrieval performance.

## Source-to-issue admission

Issue #835 is the coordination index; #836 owns the catalogue increment. The source-specific evaluations #837–#842 cover MEDLINE, Embase, CINAHL, APA PsycInfo, Cochrane CENTRAL/CDSR and ClinicalTrials.gov respectively. **A catalogue row is not a GitHub ticket.** Open an additional issue only for an evidenced, non-duplicate assessment or implementation gap; otherwise retain a catalogue entry with no issue. Link any prospective implementation to the existing primary Conductor owner and complete the relevant native tests. GitHub links are coordination pointers, **not native subissue relationships** or a change in evidence level.

## Three synthetic planning scenarios (future methodological validation)

These scenarios describe **expected human review questions**, not an implemented recommendation/ranking algorithm:

1. **Intervention effect, trial evidence:** consider MEDLINE, Embase, CENTRAL and ClinicalTrials.gov, with registry/publication deduplication and explicit access constraints. Flag PubMed's non-MEDLINE scope and Ovid translation differences. No claim that these are sufficient for every topic.
2. **Allied health and behavioural intervention:** consider CINAHL, PsycInfo and MEDLINE; test native controlled vocabulary, separate subject headings and licence-restricted/manual export. Do not treat a query compiler as cross-platform semantic equivalence.
3. **Umbrella review / current evidence surveillance:** keep CDSR separate from CENTRAL, consider additional open discovery indexes only as supplementary, and document why a bibliographic discovery API is not a substitute for each source required by the protocol.

Any source-selection quality claims need a separately specified baseline comparison, independent information-specialist review and reproducible retrieval evidence. These cases are currently **planning fixtures only**, not that evaluation.
