# Recovery reference rehearsal

`scripts/recovery_rehearsal.py --self-test` exercises a deterministic,
network-free file recovery scenario:

1. write canonical audit, snapshot and contract files through temporary files
   and atomic replacement;
2. leave a stale temporary file and prove it cannot replace canonical state;
3. create a hash-addressed backup manifest;
4. corrupt the primary snapshot;
5. restore into a clean destination;
6. repeat the restore and verify idempotency;
7. tamper with the backup and require hash verification to reject it.

The canonical receipt is `verification/recovery/rehearsal.json`.

This rehearsal proves only the reference mechanics in the current local
filesystem. It does not prove encrypted backup, cross-platform durability,
production recovery time or recovery point objectives, object-store semantics,
or an operational team's incident readiness.
