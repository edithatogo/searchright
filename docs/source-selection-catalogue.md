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

## External directories and registers of candidate search sources (23 November 2026)

Searchright now records a **link-only, internally authored** [directory-of-directories index](../data/source-selection/directories.v1.json) with 10 candidate references, a source class, first-party URL, date, rights/API uncertainty, planned use and GitHub intake issue [#845](https://github.com/edithatogo/searchright/issues/845). It is not an import of directory records. The 12-item source catalogue above remains a *different dataset*: it indexes actual candidate databases/registers/platforms, not the reference services that suggest them. Python unittest discovery runs [directory policy tests](../tests/test_source_directories.py) with network disabled.

| Reference | What it enumerates | How to use it |
| --- | --- | --- |
| [DBIS](https://dbis.ur.de/) · [published API](https://dbis-api.ur.de/) | Multidisciplinary academic databases, with institutional access overlays and a documented JSON API | **Highest-priority machine-discovery pilot** after terms and data-reuse review, maintaining database/platform identity |
| [RefHunter](https://refhunter.org/en/) | Health and adjacent-field database features and search-interface capabilities | Human comparison and source-selection planning; link only pending clearance of its CC BY-SA and public-copy restrictions |
| [FAIRsharing](https://fairsharing.org/databases) | Research databases, repositories, standards and policies | Discover domain-specific data/knowledge bases; licence/share-alike and API rules require separate review |
| [OpenDOAR](https://www.jisc.ac.uk/opendoar) | Curated open-access scholarly repositories | Supplementary/grey-literature repository candidates; not a bibliographic database compiler |
| [re3data](https://www.re3data.org/about) | Research data repositories, with CC0 registry metadata and documented API | Strong potential rights-clear dataset repository discovery pilot; still check current API and provenance |
| [OpenAIRE PROVIDE](https://www.openaire.eu/validator-registration-guide) | Scholarly data providers, repositories, journals, aggregators and CRIS | Assess source coverage and harvesting relationships; not presumed a general database directory |
| [WHO ICTRP network](https://www.who.int/tools/clinical-trials-registry-platform/network/primary-registries) | Primary trial registries, distinct from [ICTRP data providers](https://www.who.int/tools/clinical-trials-registry-platform/network/data-providers) | Crosswalk with Track 20's ClinicalTrials.gov, ANZCTR and ICTRP manual procedures; avoid trial double counting |
| [CDA-AMC Grey Matters](https://www.cda-amc.ca/grey-matters-practical-tool-searching-health-related-grey-literature) | Grey-literature source websites for HTA, guidelines, economics, policy and regulators | Topic-specific, human-guided supplementary search plans; do not copy the checklist |
| [ISSN ROAD](https://www.issn.org/services/online-services/road-the-directory-of-open-access-scholarly-resources/) | Open-access scholarly journals, repositories, proceedings and other serials | Optional source-name/serial identity reconciliation, not an adapter |
| [UQ sources-to-search guide](https://guides.library.uq.edu.au/tools-and-techniques/systematic-reviews/sources-to-search) | Topic-based library recommendations for databases | Method guidance only; subscriptions depend on the actual institution |

**Supplementary methodological authority** remains independent of directories: [Cochrane Handbook Chapter 4](https://www.cochrane.org/authors/handbooks-and-manuals/handbook/current/chapter-04) gives intervention-search minimum sources and method-specific considerations; [NHMRC guidance](https://www.nhmrc.gov.au/guidelinesforguidelines/develop/identifying-evidence) identifies Australian and broader health-evidence resources. These do not automatically certify a particular source set or substitute for human method review.

### Admission and verification contract

1. Crosswalk each external directory candidate against the already-declared `ReviewPlan`, source-methods and licensed/open-provider manifests before a new source issue or adapter is proposed. Preserve database, platform, API/provider and aggregator identities separately.
2. A directory's published API is a *candidate metadata interface*, not permission to crawl, mirror, redistribute or execute it. Before ingesting, check licence and share-alike, robots/terms, attribution, request budgets, source revision, stable ID mapping and derivative-export obligations. A public first-party page is an evidence link, **not** observed live interoperability.
3. A human information specialist must decide which sources are suitable for each topic, review type and region; evaluate key seed-record recall, overlap, omitted-source risk, licensing and translation losses before reporting search adequacy. PRISMA-S supplies reporting evidence, not methodological certification.
4. Keep the `directories.v1.json` and `catalogue.v1.json` registers **advisory**. No entry permits active network connections, final screening decisions, data import or unsupported source claims. The unresolved Track 16 dependency-security gate [#831](https://github.com/edithatogo/searchright/issues/831) also remains independent of this catalogue work.
