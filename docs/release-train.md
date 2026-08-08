# Cross-repository contract release train

Searchright, CiteWeft and Sourceright evolve independently but share contracts.
The release train prevents a source change in one repository from being treated
as compatible merely because its own tests pass.

Promotion order is document-evidence producer, Searchright/shared search core,
then downstream bibliographic verification. Each boundary requires exact
revision pins, consumer fixtures, compiler evidence, a downstream canary and an
explicit human promotion decision. Failed canaries restore the previous pin and
feature boundary; no schema or migration evidence is deleted.

The canonical machine-readable plan is `integration/release-train.json`.
Scheduled jobs may detect drift and prepare receipts, but may not change pins,
merge changes or publish releases automatically.
