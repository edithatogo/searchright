# Schema evolution and migrations

Searchright rejects silent write upgrades and destructive contract migrations.

`contracts/migrations/registry.json` identifies every contract family with more
than one checked-in schema version. Each family records its minimum readable
version, current write version and explicit migration plans. A plan must state:

- source and target versions;
- preconditions;
- transformations;
- whether data is destructive;
- backup requirements;
- rollback support and known projection losses;
- verification steps;
- the claim boundary.

The current v1-to-v2 GitHub issue-hierarchy plan preserves existing stable issue
keys and derives task-level nodes and native relationships. Remote identifiers
remain runtime evidence and are never canonical migration input.

`scripts/check_schema_migrations.py` ensures that every multi-version catalogue
family has sufficient explicit plans and that default policy remains:

- reject unknown versions;
- deny destructive migration;
- deny implicit write upgrades;
- require backup and a migration receipt.

Compiled migration tests against representative persisted data remain a higher
evidence gate.
