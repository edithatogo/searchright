# Backup, restore and disaster recovery

Backups are content-addressed, encrypted, scoped, retention-bounded and described
by `BackupManifest`. Key references are recorded; key material is not. Full and
incremental chains preserve immutable audit and protocol-amendment history.

A backup is not considered usable until a clean-room restore rehearsal confirms
integrity, schema compatibility, tenant isolation and audit-chain continuity.
Restore requires an authenticated, explicitly authorised operator and produces a
separate receipt. Production recovery objectives remain deployment-specific and
must be approved before a hosted service claim.
