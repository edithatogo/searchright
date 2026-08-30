import type { AgentHandoff, IntegrationPassport, ReviewPlanProtocol } from '../../../sdk/typescript/src/index.ts';

// Required base fields cannot disappear when a root also contains alternatives.
// @ts-expect-error Missing required handoff fields.
const missingHandoff: AgentHandoff = {};
// @ts-expect-error Missing required integration fields.
const missingPassport: IntegrationPassport = {};
// @ts-expect-error Missing required protocol fields.
const missingProtocol: ReviewPlanProtocol = {};

const handoff: AgentHandoff = {
  schema_version: 'org.searchright.agent-handoff.v1',
  handoff_id: 'handoff', review_id: 'review',
  from_role: 'question-framer', to_role: 'information-specialist',
  context_policy: 'minimum_necessary', execution_mode: null,
  artifacts: [{ path: 'plan.json', sha256: 'a'.repeat(64), media_type: 'application/json' }],
  approval_references: [{ receipt_id: 'receipt', review_id: 'review', purpose: 'review_plan', scope_sha256: 'a'.repeat(64) }],
};
const identifier: string = handoff.handoff_id;
// @ts-expect-error Approval references retain their object shape.
const invalidApproval: AgentHandoff['approval_references'] = ['receipt'];
declare const passport: IntegrationPassport;
const repository: string = passport.repository;
declare const protocol: ReviewPlanProtocol;
const version: string = protocol.version;
void [missingHandoff, missingPassport, missingProtocol, identifier, invalidApproval, repository, version];
