# Tool map

| Task | MCP tool | CLI command | Status |
| --- | --- | --- | --- |
| Validate plan | `validate_plan` | `validate-plan` | Scaffolded source |
| Compile query | `compile_strategy` | `compile` | Scaffolded source |
| Deduplicate | `deduplicate_records` | `deduplicate` | Scaffolded source |
| PRISMA output | `generate_prisma` | `prisma` | Scaffolded source |
| Verify audit | `verify_audit` | `verify-audit` | Scaffolded source |
| View authority workflow | `workflow` | `workflow` | Scaffolded source |
| Plan/draft persistence | `plan_review` | `searchright plan review` | Local current-protocol preview/apply with exact human-confirmed immutable bytes |
| PRESS persistence | `press_review_strategy` | `searchright strategy press-review` | Local current-protocol preview/apply; records evidence without certifying completeness |
| Search execution | `execute_search` | `searchright run execute` | Deterministic fixture execution and immutable commit; live network denied pending H-002 |
| Screening write | `record_screening_decision` | `searchright screen record-decision` | Local role-policy immutable decision; agent exclusions denied without atomic human confirmation |

`Scaffolded source` is not runtime proof until Rust gates and MCP transcripts pass.
