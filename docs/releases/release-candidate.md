# Release-candidate rehearsal

A release candidate is built only from a clean clone and committed lockfile.
Source archives and platform binaries are rebuilt independently and compared,
then signed, checksummed, SBOM-described and attested. The exact CLI and MCP
binaries are install-smoked before downstream canaries run.

Failure in any critical gate blocks promotion. A release candidate does not
change registry claims until explicit human approval and public acceptance
receipts exist.
