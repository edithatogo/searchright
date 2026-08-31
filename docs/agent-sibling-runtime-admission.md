# Local sibling handoff admission

Track 11 owns `searchright_agent::sibling::admit_sibling_handoff`. This is a
local integrity boundary, not a tool launcher or an approval store. Candidate
files are read-only, but the supplied authority adapter may consume approvals;
successful admission does not leave a one-use approval reusable.
The Searchright-owned sibling still requires explicit user routing and keeps
automated invocation disabled.

## Trusted inputs and checked bytes

An accountable integration must supply `SiblingAdmissionPins` through its
trusted configuration, not from retrieved content or the candidate package.
The caller also supplies an explicitly authorized local artifact root and an
implementation of the existing `HandoffApprovalAuthority` contract.

The verifier checks actual package and source-snapshot bytes against those
pins, requires the exact compiled handoff schema, and parses the bounded
handoff JSON. Component bytes are limited to 8 MiB each; the JSON envelope is
limited to 64 KiB. The existing `AgentHandoff` verifier validates role adjacency,
artifact paths, retained-byte digests and purpose-bound approvals. The approval
adapter remains responsible for authenticating, expiring, revoking and atomically
consuming real approvals.

The source revision is a trusted annotation, not proof that source bytes came
from that Git commit. Matching opaque package bytes is not a parser, signature
verification, executable provenance proof, dependency audit or licence decision.
There is no archive extraction, process execution, provider invocation or
upstream modification. Returned package, source, schema and artifact bytes are
retained so a caller need not reopen a mutable path after verification.

Only advisory draft and fixture-replay operation requests may reach handoff
verification. Live handoffs and consequential operations, including final
exclusion, protocol amendment and publication, are rejected. Success is not a
capability to perform a canonical write or enable automated invocation.

## Reproducible checks

Run `cargo test -p searchright-agent --test sibling_admission --locked` for
synthetic admission, tampering, malformed-input, path, approval and replay cases.
Fixtures compute real hashes over original synthetic bytes; the fixture approval
adapter is test-only and must never be used as production authority.

Run `PYTHONPATH=scripts python3 -B -m unittest test_agent_host_eval
test_agent_skill_policy` for host-runner isolation and policy regressions. These
mocked adapter tests do not constitute an external host evaluation.

## Remaining boundaries and rollback

All four mandatory gates remain pending: downstream adoption/maintenance,
host/model evaluation, methodology calibration and registry authorization plus
acceptance. This slice does not resolve the existing conflict between the
two-human calibration protocol and the agent-panel handoff policy; an accountable
owner must resolve that policy before calibration can be closed.

Deployment admission also still needs provenance-bound real package evidence,
production approval integration, supported-host/target evidence and execution
boundary review. Disable the sibling or use the existing manual handoff route
to roll back. There is no persisted state migration and no change to the governed
handoff wire schema, upstream routing or package publication policy.
