"""Generated contract-only types. Do not edit by hand."""
from __future__ import annotations

from typing import Literal, NotRequired, Required, TypeAlias, TypedDict

JsonValue: TypeAlias = None | bool | int | float | str | list['JsonValue'] | dict[str, 'JsonValue']
CONTRACT_IDS = ('access-decision', 'access-request', 'agent-workflow', 'architecture-policy', 'audit-event', 'audit-event-registry', 'backup-manifest', 'benchmark-report', 'bibliographic-record', 'compiled-strategy', 'component-health', 'consumer-contract-suite', 'data-handling-decision', 'data-handling-request', 'data-lifecycle-decision', 'data-lifecycle-request', 'diagnostic', 'discovery-run', 'document-evidence', 'evidence-debt', 'execution-envelope', 'gate-catalog', 'github-control-plane-apply-summary', 'github-issue-hierarchy', 'github-issue-hierarchy-v2', 'github-project', 'github-repository-settings', 'incident-record', 'institutional-policy', 'integration-passport', 'integration-release-train', 'interchange-receipt', 'licensed-adapter', 'living-update', 'named-filter-pack', 'native-search-strategy', 'prisma-flow', 'protocol-amendment', 'provider-component', 'provider-component-release-signature', 'provider-component-trust-policy', 'provider-manifest', 'provider-page', 'provider-policy-set', 'query-ast', 'ranking-calibration', 'recovery-rehearsal', 'redaction-profile', 'release-rehearsal', 'research-object-handoff-plan', 'review-bundle-manifest', 'review-plan', 'review-state-snapshot', 'schema-migration-plan', 'schema-migration-registry', 'screening-decision', 'screening-policy', 'search-run', 'search-strategy', 'search-validation', 'source-receipt', 'sourceright-parity-report', 'standard-assessment', 'standard-pack', 'study-graph', 'telemetry-policy', 'tenant-policy', 'workflow-trace')

AccessDecision = TypedDict(
    'AccessDecision',
    {
    'blockers': Required['list[str]'],
    'human_approval_required': Required['bool'],
    'permitted': Required['bool'],
    'request_id': Required['str'],
    'schema_version': Required["Literal['org.searchright.access-decision.v1']"],
    'tenant_id': Required['str'],
    },
)

AccessRequest = TypedDict(
    'AccessRequest',
    {
    'authenticated': Required['bool'],
    'external_write': Required['bool'],
    'final_eligibility_decision': Required['bool'],
    'human_approval': Required['bool'],
    'principal_id': Required['str'],
    'principal_kind': Required["Literal['human', 'service', 'agent']"],
    'region': Required['str'],
    'request_id': Required['str'],
    'schema_version': Required["Literal['org.searchright.access-request.v1']"],
    'scopes': Required["list[Literal['review_read', 'review_write', 'search_execute', 'screening_recommend', 'screening_decide', 'tenant_admin', 'external_write']]"],
    'tenant_id': Required['str'],
    },
)

AgentWorkflow = TypedDict(
    'AgentWorkflow',
    {
    'schema_version': Required["Literal['org.searchright.agent-workflow.v1']"],
    'screening_authority': Required["Literal['advisory_only', 'include_only', 'exclusion_with_human_confirmation']"],
    'steps': Required['list[AgentWorkflowStepsItem]'],
    },
)

AgentWorkflowStepsItem = TypedDict(
    'AgentWorkflowStepsItem',
    {
    'authority': Required["Literal['read_only_automatic', 'human_confirmation', 'explicit_approval', 'role_policy', 'human_only']"],
    'blocking_conditions': Required['list[str]'],
    'outputs': Required['list[str]'],
    'required_inputs': Required['list[str]'],
    'stage': Required["Literal['scope', 'eligibility', 'source_selection', 'strategy_design', 'press_review', 'execute', 'deduplicate', 'title_abstract_screening', 'full_text_screening', 'report', 'update']"],
    },
)

ArchitecturePolicy = TypedDict(
    'ArchitecturePolicy',
    {
    'claim_boundary': Required['str'],
    'default_policy': Required["Literal['deny']"],
    'external_write_scripts': Required['list[ArchitecturePolicyExternalWriteScriptsItem]'],
    'final_eligibility_authority_source_roots': Required['list[str]'],
    'forbidden_dependency_prefixes_for_neutral_crates': Required['list[str]'],
    'forbidden_internal_edges': Required['list[ArchitecturePolicyForbiddenInternalEdgesItem]'],
    'network_dependencies': Required['ArchitecturePolicyNetworkDependencies'],
    'neutral_crates': Required['list[str]'],
    'provider_endpoint_source_roots': Required['list[str]'],
    'public_package_default': Required["Literal['deny']"],
    'schema_version': Required["Literal['org.searchright.architecture-policy.v1']"],
    },
)

ArchitecturePolicyExternalWriteScriptsItem = TypedDict(
    'ArchitecturePolicyExternalWriteScriptsItem',
    {
    'apply_flag': Required['str'],
    'environment_gate': Required['str'],
    'path': Required['str'],
    },
)

ArchitecturePolicyForbiddenInternalEdgesItem = TypedDict(
    'ArchitecturePolicyForbiddenInternalEdgesItem',
    {
    'from': Required['str'],
    'reason': Required['str'],
    'to': Required['str'],
    },
)

ArchitecturePolicyNetworkDependencies = TypedDict(
    'ArchitecturePolicyNetworkDependencies',
    {

    },
)

AuditEvent = TypedDict(
    'AuditEvent',
    {
    'actor': Required['AuditEventActor'],
    'event_hash': Required['str'],
    'event_id': Required['str'],
    'event_type': Required['str'],
    'occurred_at': Required['str'],
    'payload': Required['JsonValue'],
    'previous_hash': Required["Literal['GENESIS'] | str"],
    'review_id': Required['str'],
    'schema_version': Required["Literal['org.searchright.audit-event.v1']"],
    },
)

AuditEventActor = TypedDict(
    'AuditEventActor',
    {
    'actor_id': Required['str'],
    'actor_type': Required['str'],
    'provenance': Required['None | str'],
    },
)

AuditEventRegistry = TypedDict(
    'AuditEventRegistry',
    {
    'claim_boundary': Required['str'],
    'envelope_schema_version': Required["Literal['org.searchright.audit-event.v1']"],
    'event_types': Required['list[AuditEventRegistryEventTypesItem]'],
    'maximum_payload_bytes': Required['int'],
    'prohibited_payload_keys': Required['list[str]'],
    'schema_version': Required["Literal['org.searchright.audit-event-registry.v1']"],
    'unknown_event_type_policy': Required["Literal['reject']"],
    'unknown_payload_version_policy': Required["Literal['reject']"],
    },
)

AuditEventRegistryEventTypesItem = TypedDict(
    'AuditEventRegistryEventTypesItem',
    {
    'allowed_payload_keys': Required['list[str]'],
    'current_payload_version': Required['int'],
    'event_type': Required['str'],
    'legacy_unversioned_payload_version': Required['int'],
    'migrations': Required['list[str]'],
    'payload_field_types': Required['AuditEventRegistryEventTypesItemPayloadFieldTypes'],
    'versions': Required['list[AuditEventRegistryEventTypesItemVersionsItem]'],
    },
)

AuditEventRegistryEventTypesItemPayloadFieldTypes = TypedDict(
    'AuditEventRegistryEventTypesItemPayloadFieldTypes',
    {

    },
)

AuditEventRegistryEventTypesItemVersionsItem = TypedDict(
    'AuditEventRegistryEventTypesItemVersionsItem',
    {
    'status': Required["Literal['migration_only', 'current']"],
    'version': Required['int'],
    },
)

BackupManifest = TypedDict(
    'BackupManifest',
    {
    'backup_id': Required['str'],
    'content_classes': Required['list[str]'],
    'created_at': Required['str'],
    'digest': Required['str'],
    'digest_algorithm': Required['str'],
    'encrypted': Required['bool'],
    'key_reference': Required['str | None'],
    'kind': Required["Literal['full', 'incremental', 'research_object']"],
    'parent_backup_id': Required['str | None'],
    'restore_test_required': Required['bool'],
    'retention_days': Required['int'],
    'schema_version': Required["Literal['org.searchright.backup-manifest.v1']"],
    'scope_id': Required['str'],
    },
)

BenchmarkReport = TypedDict(
    'BenchmarkReport',
    {
    'benchmark_id': Required['str'],
    'claim_boundary': Required['str'],
    'configuration_digest': Required['str'],
    'corpus_id': Required['str'],
    'corpus_version': Required['str'],
    'environment': Required['list[str]'],
    'generated_at': Required['str'],
    'implementation_version': Required['str'],
    'leakage_controls': Required['list[str]'],
    'metrics': Required['list[BenchmarkReportMetric]'],
    'rights_basis': Required['str'],
    'schema_version': Required["Literal['org.searchright.benchmark-report.v1']"],
    },
)

BenchmarkReportMetric = TypedDict(
    'BenchmarkReportMetric',
    {
    'lower_bound': Required['int | float | None'],
    'name': Required['str'],
    'sample_size': Required['int'],
    'unit': Required['str'],
    'upper_bound': Required['int | float | None'],
    'value': Required['int | float'],
    },
)

BibliographicRecord = TypedDict(
    'BibliographicRecord',
    {
    'abstract_text': Required['str | None'],
    'authors': Required['list[str]'],
    'container_title': Required['str | None'],
    'identifiers': Required['BibliographicRecordIdentifiers'],
    'kind': Required["Literal['journal_article', 'preprint', 'conference', 'trial_registry', 'thesis', 'report', 'dataset'] | BibliographicRecordKindVariant2"],
    'languages': Required['list[str]'],
    'native_id': Required['str'],
    'provider_metadata': Required['JsonValue'],
    'publication_date': Required['str | None'],
    'publication_year': Required['int | None'],
    'record_id': Required['str'],
    'schema_version': Required["Literal['org.searchright.bibliographic-record.v1']"],
    'source_receipt_id': Required['str'],
    'subjects': Required['list[str]'],
    'title': Required['str'],
    'urls': Required['list[str]'],
    },
)

BibliographicRecordIdentifiers = TypedDict(
    'BibliographicRecordIdentifiers',
    {
    'doi': NotRequired['str | None'],
    'isbn': NotRequired['str | None'],
    'openalex': NotRequired['str | None'],
    'other': NotRequired['BibliographicRecordIdentifiersOther'],
    'pmcid': NotRequired['str | None'],
    'pmid': NotRequired['str | None'],
    'trial_registration': NotRequired['str | None'],
    },
)

BibliographicRecordIdentifiersOther = TypedDict(
    'BibliographicRecordIdentifiersOther',
    {

    },
)

BibliographicRecordKindVariant2 = TypedDict(
    'BibliographicRecordKindVariant2',
    {
    'other': Required['str'],
    },
)

CompiledStrategy = TypedDict(
    'CompiledStrategy',
    {
    'compilation_hash': Required['str'],
    'compiler_version': Required['str'],
    'dialect': Required["Literal['pub_med', 'ovid_medline', 'embase', 'europe_pmc', 'cinahl_ebsco', 'psyc_info_ovid', 'scopus', 'web_of_science', 'crossref', 'open_alex', 'clinical_trials_gov', 'generic_boolean'] | CompiledStrategyDialectVariant2"],
    'fidelity': Required["Literal['exact', 'source_equivalent', 'approximate', 'degraded']"],
    'loss_codes': Required['list[str]'],
    'query': Required['str'],
    'review_required': Required['bool'],
    'schema_version': Required["Literal['org.searchright.compiled-strategy.v1']"],
    'strategy_id': Required['str'],
    'warnings': Required['list[CompiledStrategyWarningsItem]'],
    },
)

CompiledStrategyDialectVariant2 = TypedDict(
    'CompiledStrategyDialectVariant2',
    {
    'custom': Required['str'],
    },
)

CompiledStrategyWarningsItem = TypedDict(
    'CompiledStrategyWarningsItem',
    {
    'code': Required['str'],
    'message': Required['str'],
    'review_required': Required['bool'],
    },
)

ComponentHealth = TypedDict(
    'ComponentHealth',
    {
    'component': Required['str'],
    'diagnostics': Required['list[str]'],
    'observed_at': Required['str'],
    'ready': Required['bool'],
    'schema_version': Required["Literal['org.searchright.component-health.v1']"],
    'state': Required["Literal['healthy', 'degraded', 'unhealthy', 'disabled']"],
    },
)

ConsumerContractSuite = TypedDict(
    'ConsumerContractSuite',
    {
    'claim_boundary': Required['str'],
    'generated_at': Required['str'],
    'interactions': Required['list[ConsumerContractSuiteInteraction]'],
    'schema_version': Required["Literal['org.searchright.consumer-contract-suite.v1']"],
    },
)

ConsumerContractSuiteInteraction = TypedDict(
    'ConsumerContractSuiteInteraction',
    {
    'automatic_promotion': Required['Literal[False]'],
    'consumer': Required['str'],
    'consumer_contracts': Required['ConsumerContractSuiteNonemptyStrings'],
    'consumer_gates': Required['ConsumerContractSuiteNonemptyStrings'],
    'contract_version': Required['str'],
    'failure_semantics': Required['str'],
    'fixture_paths': Required['ConsumerContractSuiteNonemptyStrings'],
    'id': Required['str'],
    'integration_id': Required['str'],
    'producer': Required['str'],
    'producer_contracts': Required['ConsumerContractSuiteNonemptyStrings'],
    'producer_gates': Required['ConsumerContractSuiteNonemptyStrings'],
    'status': Required["Literal['prepared_not_executed', 'fixture_verified', 'downstream_verified', 'suspended']"],
    },
)

ConsumerContractSuiteNonemptyStrings: TypeAlias = list[str]

DataHandlingDecision = TypedDict(
    'DataHandlingDecision',
    {
    'blockers': Required['list[str]'],
    'human_approval_required': Required['bool'],
    'permitted': Required['bool'],
    'policy_id': Required['str'],
    'request_id': Required['str'],
    'schema_version': Required["Literal['org.searchright.data-handling-decision.v1']"],
    'warnings': Required['list[str]'],
    },
)

DataHandlingRequest = TypedDict(
    'DataHandlingRequest',
    {
    'classification': Required["Literal['public_metadata', 'internal_review_data', 'confidential', 'restricted_full_text', 'sensitive_personal_data']"],
    'cross_border_transfer': Required['bool'],
    'deployment_mode': Required["Literal['local_only', 'institution_self_hosted', 'hosted_single_tenant', 'hosted_multi_tenant']"],
    'dry_run': Required['bool'],
    'operation': Required["Literal['metadata', 'full_text_analysis', 'full_text_persistence', 'export', 'telemetry', 'external_model_processing']"],
    'region': Required['str | None'],
    'request_id': Required['str'],
    'retention_days': Required['int'],
    'review_id': Required['str'],
    'schema_version': Required["Literal['org.searchright.data-handling-request.v1']"],
    },
)

DataLifecycleDecision = TypedDict(
    'DataLifecycleDecision',
    {
    'blockers': Required['list[str]'],
    'effects_authorized': Required['bool'],
    'immutable_audit_preserved': Required['Literal[True]'],
    'permitted': Required['bool'],
    'policy_id': Required['str'],
    'receipt_required': Required['bool'],
    'request_id': Required['str'],
    'schema_version': Required["Literal['org.searchright.data-lifecycle-decision.v1']"],
    'tombstone_target_ids': Required['list[str]'],
    'warnings': Required['list[str]'],
    },
)

DataLifecycleRequest = TypedDict(
    'DataLifecycleRequest',
    {
    'action': Required["Literal['retain', 'export', 'delete']"],
    'approval': Required['None | DataLifecycleRequestApprovalVariant2'],
    'classification': Required["Literal['public_metadata', 'internal_review_data', 'confidential', 'restricted_full_text', 'sensitive_personal_data']"],
    'execution_mode': Required["Literal['preview', 'apply']"],
    'export_destination': Required['str | None'],
    'includes_audit_log': Required['bool'],
    'legal_hold': Required['bool'],
    'request_id': Required['str'],
    'retention_days': Required['int | None'],
    'review_id': Required['str'],
    'schema_version': Required["Literal['org.searchright.data-lifecycle-request.v1']"],
    'target_ids': Required['list[str]'],
    },
)

DataLifecycleRequestApprovalVariant2 = TypedDict(
    'DataLifecycleRequestApprovalVariant2',
    {
    'action': Required["Literal['retain', 'export', 'delete']"],
    'approval_id': Required['str'],
    'approved_at': Required['str'],
    'approved_by': Required['str'],
    'expires_at': Required['str'],
    'nonce': Required['str'],
    'policy_id': Required['str'],
    'request_digest': Required['str'],
    'review_id': Required['str'],
    },
)

Diagnostic = TypedDict(
    'Diagnostic',
    {
    'blocking': Required['bool'],
    'code': Required['str'],
    'column': Required['int | None'],
    'evidence_ids': Required['list[str]'],
    'line': Required['int | None'],
    'locale': Required["Literal['en_au', 'en_nz', 'en_us', 'mi_nz'] | DiagnosticLocaleVariant2"],
    'message': Required['str'],
    'path': Required['str | None'],
    'remediation': Required['str | None'],
    'schema_version': Required["Literal['org.searchright.diagnostic.v1']"],
    'severity': Required["Literal['information', 'warning', 'error', 'blocking']"],
    },
)

DiagnosticLocaleVariant2 = TypedDict(
    'DiagnosticLocaleVariant2',
    {
    'custom': Required['str'],
    },
)

DiscoveryRun = TypedDict(
    'DiscoveryRun',
    {
    'edges': Required['list[DiscoveryRunEdgesItem]'],
    'maximum_depth': Required['int'],
    'maximum_records': Required['int'],
    'method': Required["Literal['backward_citation', 'forward_citation', 'similar_articles', 'trial_registry', 'repository', 'grey_literature', 'handsearch', 'contact'] | DiscoveryRunMethodVariant2"],
    'requires_human_release': Required['bool'],
    'review_id': Required['str'],
    'run_id': Required['str'],
    'schema_version': Required["Literal['org.searchright.discovery-run.v1']"],
    'seed_ids': Required['list[str]'],
    },
)

DiscoveryRunEdgesItem = TypedDict(
    'DiscoveryRunEdgesItem',
    {
    'discovered_id': Required['str'],
    'edge_id': Required['str'],
    'method': Required["Literal['backward_citation', 'forward_citation', 'similar_articles', 'trial_registry', 'repository', 'grey_literature', 'handsearch', 'contact'] | DiscoveryRunEdgesItemMethodVariant2"],
    'provider_id': Required['str'],
    'receipt_id': Required['str'],
    'seed_id': Required['str'],
    },
)

DiscoveryRunEdgesItemMethodVariant2 = TypedDict(
    'DiscoveryRunEdgesItemMethodVariant2',
    {
    'other': Required['str'],
    },
)

DiscoveryRunMethodVariant2 = TypedDict(
    'DiscoveryRunMethodVariant2',
    {
    'other': Required['str'],
    },
)

DocumentEvidence = TypedDict(
    'DocumentEvidence',
    {
    'canonical_write_permitted': Required['Literal[False]'],
    'citation_callouts': Required['list[DocumentEvidenceCallout]'],
    'diagnostics': Required['list[DocumentEvidenceDiagnostic]'],
    'document_id': Required['str'],
    'provenance': Required['DocumentEvidenceProvenance'],
    'references': Required['list[DocumentEvidenceReference]'],
    'retained_full_text': Required['bool'],
    'schema_version': Required["Literal['org.searchright.document-evidence.v1']"],
    'upstream_schema_version': Required['str'],
    },
)

DocumentEvidenceCallout = TypedDict(
    'DocumentEvidenceCallout',
    {
    'callout_id': Required['str'],
    'confidence': Required['int | float | None'],
    'reference_id': Required['str | None'],
    'review_required': Required['bool'],
    'span': Required['DocumentEvidenceSpan'],
    'surface': Required['str'],
    },
)

DocumentEvidenceDiagnostic = TypedDict(
    'DocumentEvidenceDiagnostic',
    {
    'code': Required['str'],
    'message': Required['str'],
    'severity': Required["Literal['info', 'warning', 'error']"],
    'subject_id': Required['str | None'],
    },
)

DocumentEvidenceField = TypedDict(
    'DocumentEvidenceField',
    {
    'field': Required['str'],
    'span': Required['DocumentEvidenceSpan | None'],
    'value': Required['str'],
    },
)

DocumentEvidenceProvenance = TypedDict(
    'DocumentEvidenceProvenance',
    {
    'backend': Required['str'],
    'backend_version': Required['str | None'],
    'configuration': Required['str'],
    'endpoint_class': Required['str | None'],
    'input_sha256': Required['str | None'],
    'route_trace_digest': Required['str | None'],
    },
)

DocumentEvidenceReference = TypedDict(
    'DocumentEvidenceReference',
    {
    'confidence': Required['int | float | None'],
    'fields': Required['list[DocumentEvidenceField]'],
    'raw_citation': Required['str'],
    'reference_id': Required['str'],
    'review_required': Required['bool'],
    'span': Required['DocumentEvidenceSpan | None'],
    },
)

DocumentEvidenceSpan = TypedDict(
    'DocumentEvidenceSpan',
    {
    'bounding_box': Required['list[int | float]'],
    'end_byte': Required['int | None'],
    'page': Required['int | None'],
    'source_id': Required['str | None'],
    'start_byte': Required['int | None'],
    'surface': Required['str'],
    },
)

EvidenceDebtRegister = TypedDict(
    'EvidenceDebtRegister',
    {
    'assertions': Required['EvidenceDebtRegisterAssertions'],
    'claim_boundary': Required['str'],
    'evidence_ceiling': Required["Literal['source_verified']"],
    'maturity': Required['EvidenceDebtRegisterMaturity'],
    'priority_queue': Required['list[EvidenceDebtRegisterPriorityQueueItem]'],
    'provider_policy': Required['EvidenceDebtRegisterProviderPolicy'],
    'publication': Required['EvidenceDebtRegisterPublication'],
    'schema_version': Required["Literal['org.searchright.evidence-debt.v1']"],
    'static_gates': Required['EvidenceDebtRegisterStaticGates'],
    'tracks': Required['EvidenceDebtRegisterTracks'],
    },
)

EvidenceDebtRegisterAssertions = TypedDict(
    'EvidenceDebtRegisterAssertions',
    {
    'by_mapping_confidence': Required['EvidenceDebtRegisterAssertionsByMappingConfidence'],
    'by_state': Required['EvidenceDebtRegisterAssertionsByState'],
    'open_gate_entries': Required['int'],
    'total': Required['int'],
    'track_level_only': Required['int'],
    'without_symbol_mapping': Required['int'],
    },
)

EvidenceDebtRegisterAssertionsByMappingConfidence = TypedDict(
    'EvidenceDebtRegisterAssertionsByMappingConfidence',
    {

    },
)

EvidenceDebtRegisterAssertionsByState = TypedDict(
    'EvidenceDebtRegisterAssertionsByState',
    {

    },
)

EvidenceDebtRegisterMaturity = TypedDict(
    'EvidenceDebtRegisterMaturity',
    {
    'critical_blockers': Required['list[str]'],
    'decision': Required['str'],
    },
)

EvidenceDebtRegisterPriorityQueueItem = TypedDict(
    'EvidenceDebtRegisterPriorityQueueItem',
    {
    'closure_evidence': Required['list[str]'],
    'debt': Required['str'],
    'priority': Required['int'],
    'reason': Required['str'],
    },
)

EvidenceDebtRegisterProviderPolicy = TypedDict(
    'EvidenceDebtRegisterProviderPolicy',
    {
    'providers': Required['int'],
    'reviewed_with_evidence': Required['int'],
    },
)

EvidenceDebtRegisterPublication = TypedDict(
    'EvidenceDebtRegisterPublication',
    {
    'candidate_packages': Required['int'],
    'publish_ready': Required['int'],
    },
)

EvidenceDebtRegisterStaticGates = TypedDict(
    'EvidenceDebtRegisterStaticGates',
    {
    'catalogued': Required['int'],
    'harness': Required['int'],
    'unregistered_assertion_commands': Required['list[str]'],
    },
)

EvidenceDebtRegisterTracks = TypedDict(
    'EvidenceDebtRegisterTracks',
    {
    'by_state': Required['EvidenceDebtRegisterTracksByState'],
    'total': Required['int'],
    },
)

EvidenceDebtRegisterTracksByState = TypedDict(
    'EvidenceDebtRegisterTracksByState',
    {

    },
)

ExecutionEnvelope = TypedDict(
    'ExecutionEnvelope',
    {
    'allowed_hosts': Required['list[str]'],
    'approved_by': Required['str | None'],
    'dry_run': Required['bool'],
    'full_text_handling': Required["Literal['metadata_only', 'local_rights_compliant', 'minimal_evidence_excerpt']"],
    'maximum_records': Required['int'],
    'maximum_seconds': Required['int'],
    'network': Required["Literal['disabled', 'allowlisted_https', 'licensed_allowlist']"],
    'operation_id': Required['str'],
    'review_id': Required['str'],
    'schema_version': Required["Literal['org.searchright.execution-envelope.v1']"],
    'secret_handling': Required["Literal['none', 'environment_redacted', 'host_managed_reference']"],
    'untrusted_content': Required["Literal['data_only', 'sanitise_then_data_only', 'human_inspection_required']"],
    },
)

GateCatalog = TypedDict(
    'GateCatalog',
    {
    'default_capabilities': Required['GateCatalogDefaultCapabilities'],
    'gates': Required['list[GateCatalogGatesItem]'],
    'generated_from': Required['list[str]'],
    'schema_version': Required["Literal['org.searchright.gate-catalog.v1']"],
    },
)

GateCatalogDefaultCapabilities = TypedDict(
    'GateCatalogDefaultCapabilities',
    {
    'compiler_required': Required['Literal[False]'],
    'external_writes': Required['Literal[False]'],
    'network': Required['Literal[False]'],
    },
)

GateCatalogGatesItem = TypedDict(
    'GateCatalogGatesItem',
    {
    'category': Required['str'],
    'claim_boundary': Required['str'],
    'command': Required['str'],
    'compiler_required': Required['Literal[False]'],
    'covered_assertions': Required['list[str]'],
    'evidence_ceiling': Required["Literal['source_verified', 'source_reproducible']"],
    'external_writes': Required['Literal[False]'],
    'gate_id': Required['str'],
    'harness_gate': Required['bool'],
    'network': Required['Literal[False]'],
    'script': Required['str'],
    },
)

GithubControlPlaneApplySummary = TypedDict(
    'GithubControlPlaneApplySummary',
    {
    'artifact': Required['GithubControlPlaneApplySummaryArtifact'],
    'audit': Required['GithubControlPlaneApplySummaryAudit'],
    'claim_boundary': Required['str'],
    'issue_sync': Required['GithubControlPlaneApplySummaryIssueSync'],
    'project_sync': Required['GithubControlPlaneApplySummaryProjectSync'],
    'repository': Required['str'],
    'schema_version': Required["Literal['org.searchright.github-control-plane-apply-summary.v1']"],
    'source_revision': Required['str'],
    'status': Required["Literal['passed']"],
    'workflow_run': Required['GithubControlPlaneApplySummaryWorkflowRun'],
    },
)

GithubControlPlaneApplySummaryArtifact = TypedDict(
    'GithubControlPlaneApplySummaryArtifact',
    {
    'digest': Required['str'],
    'expires_at': Required['str'],
    'files': Required['list[GithubControlPlaneApplySummaryArtifactFilesItem]'],
    'id': Required['int'],
    'name': Required['str'],
    },
)

GithubControlPlaneApplySummaryArtifactFilesItem = TypedDict(
    'GithubControlPlaneApplySummaryArtifactFilesItem',
    {
    'name': Required['str'],
    'sha256': Required['str'],
    },
)

GithubControlPlaneApplySummaryAudit = TypedDict(
    'GithubControlPlaneApplySummaryAudit',
    {
    'canonical_issues_observed': Required['GithubControlPlaneApplySummaryNonnegativeCount'],
    'canonical_project_items_observed': Required['GithubControlPlaneApplySummaryNonnegativeCount'],
    'content_drift': Required['Literal[0]'],
    'label_drift': Required['Literal[0]'],
    'mutation_operations': Required['Literal[0]'],
    'noncanonical_project_items_observed': Required['GithubControlPlaneApplySummaryNonnegativeCount'],
    'recognised_value_mismatches': Required['Literal[0]'],
    'relationships_observed': Required['GithubControlPlaneApplySummaryNonnegativeCount'],
    'status': Required["Literal['passed']"],
    'task_state_drift': Required['Literal[0]'],
    'unrecognised_value_shapes': Required['Literal[0]'],
    },
)

GithubControlPlaneApplySummaryIssueSync = TypedDict(
    'GithubControlPlaneApplySummaryIssueSync',
    {
    'canonical_issues': Required['GithubControlPlaneApplySummaryNonnegativeCount'],
    'delete_operations': Required['Literal[0]'],
    'issues_unchanged': Required['GithubControlPlaneApplySummaryNonnegativeCount'],
    'mode': Required["Literal['apply']"],
    'relationships_existing': Required['GithubControlPlaneApplySummaryNonnegativeCount'],
    'remaining_after_run': Required['Literal[0]'],
    'task_states_unchanged': Required['GithubControlPlaneApplySummaryNonnegativeCount'],
    },
)

GithubControlPlaneApplySummaryProjectSync = TypedDict(
    'GithubControlPlaneApplySummaryProjectSync',
    {
    'canonical_items': Required['GithubControlPlaneApplySummaryNonnegativeCount'],
    'delete_operations': Required['Literal[0]'],
    'existing_items': Required['GithubControlPlaneApplySummaryNonnegativeCount'],
    'field_updates': Required['GithubControlPlaneApplySummaryNonnegativeCount'],
    'known_equal_field_values': Required['GithubControlPlaneApplySummaryNonnegativeCount'],
    'mode': Required["Literal['apply']"],
    'project_id': Required['str'],
    'project_number': Required['int'],
    'project_url': Required['str'],
    'remaining_after_run': Required['Literal[0]'],
    },
)

GithubControlPlaneApplySummaryWorkflowRun = TypedDict(
    'GithubControlPlaneApplySummaryWorkflowRun',
    {
    'credential_check': Required["Literal['passed']"],
    'environment': Required["Literal['github-project-write']"],
    'event': Required["Literal['workflow_dispatch']"],
    'id': Required['int'],
    'url': Required['str'],
    },
)

GithubControlPlaneApplySummaryNonnegativeCount: TypeAlias = int

GitHubIssueHierarchy = TypedDict(
    'GitHubIssueHierarchy',
    {
    'apply_permitted': Required['Literal[False]'],
    'epic_key': Required['str'],
    'generated_at': Required['str'],
    'nodes': Required['list[GitHubIssueHierarchyNodesItem]'],
    'repository': Required['str'],
    'schema_version': Required["Literal['org.searchright.github-issue-hierarchy.v1']"],
    },
)

GitHubIssueHierarchyNodesItem = TypedDict(
    'GitHubIssueHierarchyNodesItem',
    {
    'body_path': Required['str'],
    'key': Required['str'],
    'kind': Required["Literal['epic', 'track', 'phase']"],
    'labels': Required['list[str]'],
    'parent_key': Required['str | None'],
    'status': Required['str'],
    'title': Required['str'],
    },
)

GitHubIssueHierarchy = TypedDict(
    'GitHubIssueHierarchy',
    {
    'apply_permitted': Required['Literal[False]'],
    'epic_key': Required['str'],
    'generated_at': Required['str'],
    'nodes': Required['list[GitHubIssueHierarchyNodesItem]'],
    'project_manifest': Required["Literal['conductor/github/project.json']"],
    'repository': Required['str'],
    'schema_version': Required["Literal['org.searchright.github-issue-hierarchy.v2']"],
    'state_sync_policy': Required["Literal['task_issues_only']"],
    },
)

GitHubIssueHierarchyNodesItem = TypedDict(
    'GitHubIssueHierarchyNodesItem',
    {
    'body_path': Required['str'],
    'desired_state': Required["Literal['open', 'closed']"],
    'key': Required['str'],
    'kind': Required["Literal['epic', 'track', 'phase', 'task']"],
    'labels': Required['list[str]'],
    'parent_key': Required['str | None'],
    'phase_number': Required['int | None'],
    'project_fields': Required['GitHubIssueHierarchyNodesItemProjectFields'],
    'status': Required["Literal['prepared_not_synced']"],
    'task_number': Required['int | None'],
    'title': Required['str'],
    'track_id': Required['str | None'],
    },
)

GitHubIssueHierarchyNodesItemProjectFields = TypedDict(
    'GitHubIssueHierarchyNodesItemProjectFields',
    {

    },
)

GitHubProjectManifest = TypedDict(
    'GitHubProjectManifest',
    {
    'apply_permitted': Required['Literal[False]'],
    'fields': Required['list[GitHubProjectManifestFieldsItem]'],
    'link_repository': Required['bool'],
    'owner': Required['str'],
    'owner_type': Required["Literal['user', 'organization']"],
    'project_number': Required['None'],
    'readme_path': Required['str'],
    'repository': Required['str'],
    'schema_version': Required["Literal['org.searchright.github-project.v1']"],
    'short_description': Required['str'],
    'sync': Required['GitHubProjectManifestSync'],
    'title': Required['str'],
    'views': Required['list[GitHubProjectManifestViewsItem]'],
    'visibility': Required["Literal['public', 'private']"],
    },
)

GitHubProjectManifestFieldsItem = TypedDict(
    'GitHubProjectManifestFieldsItem',
    {
    'data_type': Required["Literal['SINGLE_SELECT', 'TEXT', 'NUMBER', 'DATE']"],
    'name': Required['str'],
    'options': NotRequired['list[str]'],
    },
)

GitHubProjectManifestSync = TypedDict(
    'GitHubProjectManifestSync',
    {
    'archive_policy': Required["Literal['never_automatic']"],
    'checkpoint_policy': Required["Literal['ignored_atomic_resumable']"],
    'delete_policy': Required["Literal['never']"],
    'field_policy': Required["Literal['manifest_owned_custom_fields_only']"],
    'hierarchy_path': Required['str'],
    'identity_field': Required['str'],
    'labels_path': Required['str'],
    'minimum_interval_ms': Required['int'],
    'partial_run_policy': Required["Literal['canonical_order_resume_after']"],
    'promotion_policy': Required["Literal['remote_state_cannot_promote_evidence']"],
    'receipt_directory': Required["Literal['.searchright/receipts']"],
    'remote_audit_path': Required["Literal['scripts/audit_github_control_plane.py']"],
    'state_policy': Required["Literal['task_issues_only']"],
    },
)

GitHubProjectManifestViewsItem = TypedDict(
    'GitHubProjectManifestViewsItem',
    {
    'filter': Required['str'],
    'layout': Required["Literal['BOARD_LAYOUT', 'ROADMAP_LAYOUT', 'TABLE_LAYOUT']"],
    'name': Required['str'],
    },
)

GitHubRepositorySettings = TypedDict(
    'GitHubRepositorySettings',
    {
    'apply_permitted': Required['Literal[False]'],
    'claim_boundary': Required['str'],
    'description': Required['str'],
    'environments': Required['list[str]'],
    'features': Required['GitHubRepositorySettingsFeatures'],
    'homepage': Required['str'],
    'merge_policy': Required['GitHubRepositorySettingsMergePolicy'],
    'repository': Required['str'],
    'ruleset': Required['GitHubRepositorySettingsRuleset'],
    'schema_version': Required["Literal['org.searchright.github-repository-settings.v1']"],
    'security': Required['GitHubRepositorySettingsSecurity'],
    'topics': Required['list[str]'],
    'visibility': Required["Literal['public', 'private']"],
    },
)

GitHubRepositorySettingsFeatures = TypedDict(
    'GitHubRepositorySettingsFeatures',
    {
    'discussions': Required['bool'],
    'issues': Required['bool'],
    'projects': Required['bool'],
    'wiki': Required['bool'],
    },
)

GitHubRepositorySettingsMergePolicy = TypedDict(
    'GitHubRepositorySettingsMergePolicy',
    {
    'allow_auto_merge': Required['bool'],
    'allow_update_branch': Required['bool'],
    'delete_head_branch': Required['bool'],
    'merge_commit': Required['bool'],
    'rebase': Required['bool'],
    'squash': Required['bool'],
    },
)

GitHubRepositorySettingsRuleset = TypedDict(
    'GitHubRepositorySettingsRuleset',
    {
    'deletion': Required['Literal[False]'],
    'enforcement': Required["Literal['active', 'evaluate', 'disabled']"],
    'include': Required['list[str]'],
    'name': Required['str'],
    'non_fast_forward': Required['Literal[False]'],
    'required_linear_history': Required['bool'],
    'required_signed_commits': Required['bool'],
    'required_status_checks': Required['list[str]'],
    'target': Required["Literal['branch']"],
    },
)

GitHubRepositorySettingsSecurity = TypedDict(
    'GitHubRepositorySettingsSecurity',
    {
    'automated_security_fixes': Required['bool'],
    'private_vulnerability_reporting': Required['bool'],
    'push_protection': Required['bool'],
    'secret_scanning': Required['bool'],
    'vulnerability_alerts': Required['bool'],
    },
)

IncidentRecord = TypedDict(
    'IncidentRecord',
    {
    'components': Required['list[str]'],
    'containment': Required['list[str]'],
    'data_exposure_suspected': Required['bool'],
    'detected_at': Required['str'],
    'impact': Required['str'],
    'incident_id': Required['str'],
    'postmortem_required': Required['bool'],
    'schema_version': Required["Literal['org.searchright.incident-record.v1']"],
    'severity': Required["Literal['informational', 'low', 'medium', 'high', 'critical']"],
    'status': Required['str'],
    },
)

InstitutionalPolicy = TypedDict(
    'InstitutionalPolicy',
    {
    'allowed_classifications': Required["list[Literal['public_metadata', 'internal_review_data', 'confidential', 'restricted_full_text', 'sensitive_personal_data']]"],
    'approved_by': Required['str'],
    'cross_border_transfer_allowed': Required['bool'],
    'deployment_modes': Required["list[Literal['local_only', 'institution_self_hosted', 'hosted_single_tenant', 'hosted_multi_tenant']]"],
    'effective_from': Required['str'],
    'external_model_processing_allowed': Required['bool'],
    'full_text_persistence_allowed': Required['bool'],
    'institution': Required['str'],
    'maximum_retention_days': Required['int'],
    'permitted_regions': Required['list[str]'],
    'policy_id': Required['str'],
    'review_by': Required['str | None'],
    'schema_version': Required["Literal['org.searchright.institutional-policy.v1']"],
    'telemetry_allowed': Required['bool'],
    },
)

IntegrationPassport = TypedDict(
    'IntegrationPassport',
    {
    'allowed_capabilities': Required['list[str]'],
    'automatic_revision_updates': Required['Literal[False]'],
    'canonical_upstream': Required['None | IntegrationPassportCanonicalUpstreamVariant2'],
    'claim_boundary': Required['str'],
    'code_license': Required['str'],
    'content_license': Required['str'],
    'default_branch': Required['str'],
    'default_external_writes': Required['Literal[False]'],
    'default_network': Required['Literal[False]'],
    'default_telemetry': Required['Literal[False]'],
    'dependency_direction': Required["Literal['searchright_consumes_upstream', 'downstream_consumes_searchright', 'bidirectional_via_neutral_contract', 'no_runtime_dependency']"],
    'drift_policy': Required['str'],
    'feature_flag': Required['str | None'],
    'inputs': Required['IntegrationPassportContracts'],
    'integration_id': Required['str'],
    'licence_review_status': Required["Literal['cleared', 'reference_only', 'review_required', 'upstream_licence_inherited']"],
    'local_fork_role': Required["Literal['original', 'mirror', 'patch_carrier', 'derived_product']"],
    'mode': Required["Literal['rust_dependency', 'cli_json', 'mcp', 'wasi_component', 'dataset_fixture', 'documentation_pack', 'policy_inheritance', 'generated_adapter']"],
    'model_license': Required['str | None'],
    'outputs': Required['IntegrationPassportContracts'],
    'redistribution': Required['str'],
    'repository': Required['str'],
    'revision': Required['str'],
    'rollback': Required['list[str]'],
    'schema_version': Required["Literal['org.searchright.integration-passport.v1']"],
    'verification': Required['list[IntegrationPassportVerificationItem]'],
    },
)

IntegrationPassportCanonicalUpstreamVariant2 = TypedDict(
    'IntegrationPassportCanonicalUpstreamVariant2',
    {
    'default_branch': Required['str | None'],
    'repository': Required['str'],
    'revision': Required['str | None'],
    'verification_status': Required['str'],
    },
)

IntegrationPassportContractsItem = TypedDict(
    'IntegrationPassportContractsItem',
    {
    'name': Required['str'],
    'path': Required['str'],
    'required': Required['bool'],
    'version': Required['str'],
    },
)

IntegrationPassportVariant1 = TypedDict(
    'IntegrationPassportVariant1',
    {
    'inputs': NotRequired['JsonValue'],
    },
)

IntegrationPassportVariant2 = TypedDict(
    'IntegrationPassportVariant2',
    {
    'outputs': NotRequired['JsonValue'],
    },
)

IntegrationPassportVerificationItem = TypedDict(
    'IntegrationPassportVerificationItem',
    {
    'command': Required['str'],
    'evidence_level': Required['str'],
    'id': Required['str'],
    'required': Required['bool'],
    },
)

IntegrationPassportContracts: TypeAlias = list[IntegrationPassportContractsItem]

IntegrationReleaseTrain = TypedDict(
    'IntegrationReleaseTrain',
    {
    'automatic_promotion': Required['Literal[False]'],
    'claim_boundary': Required['str'],
    'components': Required['list[IntegrationReleaseTrainComponentsItem]'],
    'contract_surface': Required['str'],
    'ecosystem_lock': Required['str'],
    'generated_at': Required['str'],
    'public_package_policy': Required['str'],
    'release_train_id': Required['str'],
    'rollback': Required['list[str]'],
    'schema_version': Required["Literal['org.searchright.integration-release-train.v1']"],
    'stages': Required['list[IntegrationReleaseTrainStagesItem]'],
    },
)

IntegrationReleaseTrainComponentsItem = TypedDict(
    'IntegrationReleaseTrainComponentsItem',
    {
    'consumer_contract': Required['str | None'],
    'passport': Required['str | None'],
    'promotion_order': Required['int'],
    'repository': Required['str'],
    'role': Required['str'],
    },
)

IntegrationReleaseTrainStagesItem = TypedDict(
    'IntegrationReleaseTrainStagesItem',
    {
    'automatic': Required['Literal[False]'],
    'id': Required['str'],
    'required_evidence': Required['str'],
    },
)

InterchangeReceipt = TypedDict(
    'InterchangeReceipt',
    {
    'input_digest': Required['str'],
    'input_format': Required["Literal['searchright_json', 'json_lines', 'csl_json', 'ris', 'nbib', 'bibtex', 'endnote_xml', 'csv', 'parquet'] | InterchangeReceiptInputFormatVariant2"],
    'lossless': Required['bool'],
    'operation_id': Required['str'],
    'output_digest': Required['str'],
    'output_format': Required["Literal['searchright_json', 'json_lines', 'csl_json', 'ris', 'nbib', 'bibtex', 'endnote_xml', 'csv', 'parquet'] | InterchangeReceiptOutputFormatVariant2"],
    'records_read': Required['int'],
    'records_written': Required['int'],
    'review_id': Required['str'],
    'schema_version': Required["Literal['org.searchright.interchange-receipt.v1']"],
    'warnings': Required['list[str]'],
    },
)

InterchangeReceiptInputFormatVariant2 = TypedDict(
    'InterchangeReceiptInputFormatVariant2',
    {
    'custom': Required['str'],
    },
)

InterchangeReceiptOutputFormatVariant2 = TypedDict(
    'InterchangeReceiptOutputFormatVariant2',
    {
    'custom': Required['str'],
    },
)

LicensedAdapterProfile = TypedDict(
    'LicensedAdapterProfile',
    {
    'allowed_hosts': Required['list[str]'],
    'credential_environment_variable': Required['str'],
    'database': Required['str'],
    'dialect': Required["Literal['embase', 'scopus', 'web_of_science', 'ovid_medline', 'psyc_info_ovid']"],
    'export_formats': Required['list[str]'],
    'live_opt_in_environment_variable': Required['str'],
    'persist_raw_responses': Required['Literal[False]'],
    'platform': Required['str'],
    'provider_id': Required['str'],
    'schema_version': Required["Literal['org.searchright.licensed-adapter.v1']"],
    'terms_review': Required['str'],
    },
)

LivingUpdateRun = TypedDict(
    'LivingUpdateRun',
    {
    'changes': Required['list[LivingUpdateRunChangesItem]'],
    'completed_at': Required['str | None'],
    'cursors_after': Required['list[LivingUpdateRunCursorsAfterItem]'],
    'cursors_before': Required['list[LivingUpdateRunCursorsBeforeItem]'],
    'parent_run_id': Required['str | None'],
    'protocol_version': Required['str'],
    'requires_human_release': Required['bool'],
    'review_id': Required['str'],
    'run_id': Required['str'],
    'schema_version': Required["Literal['org.searchright.living-update.v1']"],
    'started_at': Required['str'],
    'status': Required["Literal['planned', 'running', 'completed', 'failed', 'cancelled']"],
    'supersedes_run_id': Required['str | None'],
    },
)

LivingUpdateRunChangesItem = TypedDict(
    'LivingUpdateRunChangesItem',
    {
    'after_digest': Required['str | None'],
    'before_digest': Required['str | None'],
    'kind': Required["Literal['added', 'updated', 'missing_from_source', 'merged', 'restored']"],
    'note': Required['str'],
    'record_id': Required['str'],
    },
)

LivingUpdateRunCursorsAfterItem = TypedDict(
    'LivingUpdateRunCursorsAfterItem',
    {
    'cursor_kind': Required['str'],
    'provider_id': Required['str'],
    'retrieved_through': Required['str | None'],
    'value': Required['str'],
    },
)

LivingUpdateRunCursorsBeforeItem = TypedDict(
    'LivingUpdateRunCursorsBeforeItem',
    {
    'cursor_kind': Required['str'],
    'provider_id': Required['str'],
    'retrieved_through': Required['str | None'],
    'value': Required['str'],
    },
)

NamedFilterPack = TypedDict(
    'NamedFilterPack',
    {
    'expires_on': Required['NamedFilterPackDate'],
    'filters': Required['list[NamedFilterPackNamedFilter]'],
    'pack_id': Required['str'],
    'schema_version': Required["Literal['org.searchright.named-filter-pack.v1']"],
    'title': Required['str'],
    'validated_on': Required['NamedFilterPackDate'],
    'version': Required['str'],
    },
)

NamedFilterPackApplicability = TypedDict(
    'NamedFilterPackApplicability',
    {
    'intended_use': Required['str'],
    'limitations': Required['list[str]'],
    'platform_versions': Required['list[str]'],
    'source_ids': Required['list[str]'],
    },
)

NamedFilterPackChecksum = TypedDict(
    'NamedFilterPackChecksum',
    {
    'algorithm': Required["Literal['sha256']"],
    'digest': Required['str'],
    },
)

NamedFilterPackDialectVariant2 = TypedDict(
    'NamedFilterPackDialectVariant2',
    {
    'custom': Required['str'],
    },
)

NamedFilterPackNamedFilter = TypedDict(
    'NamedFilterPackNamedFilter',
    {
    'applicability': Required['NamedFilterPackApplicability'],
    'checksum': Required['NamedFilterPackChecksum'],
    'dialect': Required['NamedFilterPackDialect'],
    'effective_from': Required['NamedFilterPackDate'],
    'expires_on': Required['NamedFilterPackDate'],
    'expression': Required['str'],
    'filter_id': Required['str'],
    'name': Required['str'],
    'rights': Required['NamedFilterPackRights'],
    'source': Required['NamedFilterPackSource'],
    'validation': Required['NamedFilterPackValidation'],
    'version': Required['str'],
    },
)

NamedFilterPackRights = TypedDict(
    'NamedFilterPackRights',
    {
    'basis': Required['str'],
    'decided_by': Required['str'],
    'evidence_reference': Required['str'],
    'redistribution': Required["Literal['permitted', 'prohibited', 'review_required']"],
    },
)

NamedFilterPackSource = TypedDict(
    'NamedFilterPackSource',
    {
    'citation': Required['str'],
    'source_uri': Required['None | str'],
    'source_version': Required['str'],
    'title': Required['str'],
    },
)

NamedFilterPackValidation = TypedDict(
    'NamedFilterPackValidation',
    {
    'evidence_reference': Required['str'],
    'evidence_sha256': Required['str'],
    'method': Required['str'],
    'reviewer_id': Required['str'],
    'reviewer_role': Required['str'],
    'state': Required["Literal['structural_only', 'methodologically_reviewed', 'methodologically_reviewed_and_provider_current']"],
    },
)

NamedFilterPackDate: TypeAlias = str

NamedFilterPackDialect: TypeAlias = Literal['pub_med', 'ovid_medline', 'embase', 'europe_pmc', 'cinahl_ebsco', 'psyc_info_ovid', 'scopus', 'web_of_science', 'crossref', 'open_alex', 'clinical_trials_gov', 'generic_boolean'] | NamedFilterPackDialectVariant2

NativeSearchStrategy = TypedDict(
    'NativeSearchStrategy',
    {
    'diagnostics': Required['list[NativeSearchStrategyDiagnosticsItem]'],
    'dialect': Required["Literal['pub_med', 'ovid_medline', 'embase', 'europe_pmc', 'cinahl_ebsco', 'psyc_info_ovid', 'scopus', 'web_of_science', 'crossref', 'open_alex', 'clinical_trials_gov', 'generic_boolean'] | NativeSearchStrategyDialectVariant2"],
    'lines': Required['list[NativeSearchStrategyLinesItem]'],
    'normalisation_state': Required["Literal['raw_only', 'partial', 'complete']"],
    'parser_version': Required['str'],
    'raw_text': Required['str'],
    'schema_version': Required["Literal['org.searchright.native-search-strategy.v1']"],
    'semantic_strategy': Required['None | JsonValue'],
    'strategy_id': Required['str'],
    },
)

NativeSearchStrategyDiagnosticsItem = TypedDict(
    'NativeSearchStrategyDiagnosticsItem',
    {
    'code': Required['str'],
    'message': Required['str'],
    'review_required': Required['bool'],
    'severity': Required["Literal['info', 'warning', 'error']"],
    'span': Required['None | NativeSearchStrategySpan'],
    },
)

NativeSearchStrategyDialectVariant2 = TypedDict(
    'NativeSearchStrategyDialectVariant2',
    {
    'custom': Required['str'],
    },
)

NativeSearchStrategyLinesItem = TypedDict(
    'NativeSearchStrategyLinesItem',
    {
    'kind': Required["Literal['expression', 'set_combination', 'limit', 'comment', 'blank', 'unknown']"],
    'line_id': Required['str'],
    'native_set_id': Required['str | None'],
    'span': Required['NativeSearchStrategySpan'],
    'text': Required['str'],
    },
)

NativeSearchStrategySpan = TypedDict(
    'NativeSearchStrategySpan',
    {
    'end_byte': Required['int'],
    'start_byte': Required['int'],
    },
)

PrismaFlow = TypedDict(
    'PrismaFlow',
    {
    'automation_removed': Required['int'],
    'duplicates_removed': Required['int'],
    'full_text_exclusions': Required['list[PrismaFlowFullTextExclusionsItem]'],
    'other_removed': Required['int'],
    'records_databases': Required['int'],
    'records_excluded': Required['int'],
    'records_other': Required['int'],
    'records_registers': Required['int'],
    'records_screened': Required['int'],
    'reports_assessed': Required['int'],
    'reports_included': Required['int'],
    'reports_not_retrieved': Required['int'],
    'reports_sought': Required['int'],
    'review_id': Required['str'],
    'schema_version': Required["Literal['org.searchright.prisma-flow.v1']"],
    'studies_included': Required['int'],
    },
)

PrismaFlowFullTextExclusionsItem = TypedDict(
    'PrismaFlowFullTextExclusionsItem',
    {
    'count': Required['int'],
    'label': Required['str'],
    'reason_id': Required['str'],
    },
)

ProtocolAmendment = TypedDict(
    'ProtocolAmendment',
    {
    'amendment_id': Required['str'],
    'changes': Required['list[ProtocolAmendmentChangesItem]'],
    'decided_at': Required['str | None'],
    'decided_by': Required['str | None'],
    'decision': Required["Literal['proposed', 'approved', 'rejected', 'withdrawn']"],
    'kind': Required["Literal['scope', 'eligibility', 'information_sources', 'search_strategy', 'screening', 'analysis', 'governance'] | ProtocolAmendmentKindVariant2"],
    'proposed_at': Required['str'],
    'proposed_by': Required['str'],
    'requires_reprocessing': Required['bool'],
    'retrospective_impact': Required['str'],
    'review_id': Required['str'],
    'schema_version': Required["Literal['org.searchright.protocol-amendment.v1']"],
    'version_after': Required['str'],
    'version_before': Required['str'],
    },
)

ProtocolAmendmentChangesItem = TypedDict(
    'ProtocolAmendmentChangesItem',
    {
    'after': Required['str'],
    'before': Required['str | None'],
    'path': Required['str'],
    'rationale': Required['str'],
    },
)

ProtocolAmendmentKindVariant2 = TypedDict(
    'ProtocolAmendmentKindVariant2',
    {
    'other': Required['str'],
    },
)

ProviderComponentManifest = TypedDict(
    'ProviderComponentManifest',
    {
    'abi_version': Required['str'],
    'allowed_hosts': Required['list[str]'],
    'authority_rationale': Required['list[str]'],
    'capabilities': Required["list[Literal['search', 'metadata_read', 'input_file_read', 'workspace_write', 'network_read', 'telemetry']]"],
    'component_digest': Required['str'],
    'component_id': Required['str'],
    'component_version': Required['str'],
    'fixture_mode': Required['bool'],
    'max_fuel': Required['int'],
    'max_memory_mib': Required['int'],
    'schema_version': Required["Literal['org.searchright.provider-component.v1']"],
    },
)

ProviderComponentReleaseSignature = TypedDict(
    'ProviderComponentReleaseSignature',
    {
    'algorithm': Required["Literal['ed25519']"],
    'component_digest': Required['str'],
    'component_id': Required['str'],
    'component_version': Required['str'],
    'expires_at': Required['str'],
    'key_id': Required['str'],
    'manifest_digest': Required['str'],
    'schema_version': Required["Literal['org.searchright.provider-component-release-signature.v1']"],
    'signature': Required['str'],
    'signed_at': Required['str'],
    'trust_policy_id': Required['str'],
    },
)

ProviderComponentTrustPolicy = TypedDict(
    'ProviderComponentTrustPolicy',
    {
    'policy_id': Required['str'],
    'revocations': Required['list[ProviderComponentTrustPolicyRevocationsItem]'],
    'schema_version': Required["Literal['org.searchright.provider-component-trust-policy.v1']"],
    'trusted_keys': Required['list[ProviderComponentTrustPolicyTrustedKeysItem]'],
    },
)

ProviderComponentTrustPolicyRevocationsItem = TypedDict(
    'ProviderComponentTrustPolicyRevocationsItem',
    {
    'evidence_reference': Required['str'],
    'key_id': Required['str'],
    'revoked_at': Required['str'],
    },
)

ProviderComponentTrustPolicyTrustedKeysItem = TypedDict(
    'ProviderComponentTrustPolicyTrustedKeysItem',
    {
    'algorithm': Required["Literal['ed25519']"],
    'component_ids': Required['list[str]'],
    'key_id': Required['str'],
    'public_key': Required['str'],
    'valid_from': Required['str'],
    'valid_until': Required['str'],
    },
)

ProviderManifest = TypedDict(
    'ProviderManifest',
    {
    'allowed_hosts': Required['list[str]'],
    'authentication_required': Required['bool'],
    'capabilities': Required["list[Literal['search', 'pagination', 'import', 'lookup', 'backward_citation', 'forward_citation', 'updates']]"],
    'default_min_interval_ms': Required['int'],
    'display_name': Required['str'],
    'licensed': Required['bool'],
    'policy_notes': Required['list[str]'],
    'provider_id': Required['str'],
    'support_level': Required["Literal['planned', 'fixture_backed', 'opt_in_live', 'maintained']"],
    'version': Required['str'],
    },
)

ProviderPage = TypedDict(
    'ProviderPage',
    {
    'diagnostics': Required['ProviderPageDiagnostics'],
    'next_cursor': Required['str | None'],
    'records': Required['list[JsonValue]'],
    'schema_version': Required["Literal['org.searchright.provider-page.v1']"],
    'total_available': Required['int | None'],
    },
)

ProviderPageDiagnostics = TypedDict(
    'ProviderPageDiagnostics',
    {

    },
)

ProviderPolicySet = TypedDict(
    'ProviderPolicySet',
    {
    'automatic_approval': Required['Literal[False]'],
    'claim_boundary': Required['str'],
    'providers': Required['list[ProviderPolicySetProvidersItem]'],
    'schema_version': Required["Literal['org.searchright.provider-policy-set.v1']"],
    'source_epoch': Required['str'],
    },
)

ProviderPolicySetProvidersItem = TypedDict(
    'ProviderPolicySetProvidersItem',
    {
    'access_class': Required['str'],
    'contact_identity_required': Required['bool'],
    'credential_environment_variables': Required['list[str]'],
    'credential_receipt_policy': Required["Literal['never_persist', 'contact_value_redacted', 'not_applicable']"],
    'documentation_url': Required['str'],
    'endpoint': Required['str'],
    'live_canary_requires_opt_in': Required['Literal[True]'],
    'manual_review_required_before_live_release': Required['Literal[True]'],
    'minimum_interval_ms_with_key': Required['int | None'],
    'minimum_interval_ms_without_key': Required['int'],
    'notes': Required['list[str]'],
    'policy_review_status': Required["Literal['source_identified_not_legally_approved', 'reviewed_with_evidence', 'blocked']"],
    'privacy_url': Required['str'],
    'provider_id': Required['str'],
    'query_classification': Required["Literal['public_metadata', 'internal_review_data', 'confidential']"],
    'raw_response_retention': Required["Literal['disabled_by_default']"],
    'redistribution_policy': Required['str'],
    'response_classification': Required["Literal['public_metadata']"],
    'review_due': Required['str'],
    'review_evidence': Required['list[str]'],
    'source_checked_at': Required['str'],
    'terms_or_usage_url': Required['str'],
    },
)

QueryExprFieldVariant2 = TypedDict(
    'QueryExprFieldVariant2',
    {
    'custom': Required['str'],
    },
)

QueryExprQueryVariant1 = TypedDict(
    'QueryExprQueryVariant1',
    {
    'op': Required["Literal['term']"],
    'term': Required['QueryExprTerm'],
    },
)

QueryExprQueryVariant2 = TypedDict(
    'QueryExprQueryVariant2',
    {
    'children': Required['list[QueryExprQuery]'],
    'op': Required["Literal['and', 'or']"],
    },
)

QueryExprQueryVariant3 = TypedDict(
    'QueryExprQueryVariant3',
    {
    'exclude': Required['QueryExprQuery'],
    'include': Required['QueryExprQuery'],
    'op': Required["Literal['not']"],
    },
)

QueryExprQueryVariant4 = TypedDict(
    'QueryExprQueryVariant4',
    {
    'distance': Required['int'],
    'left': Required['QueryExprQuery'],
    'op': Required["Literal['proximity']"],
    'ordered': Required['bool'],
    'right': Required['QueryExprQuery'],
    },
)

QueryExprTerm = TypedDict(
    'QueryExprTerm',
    {
    'explode': Required['bool'],
    'fields': Required['list[QueryExprField]'],
    'phrase': Required['bool'],
    'text': Required['str'],
    'truncation': Required['bool'],
    'vocabulary': Required['None | str'],
    },
)

QueryExprField: TypeAlias = Literal['all', 'title', 'abstract', 'title_abstract', 'author', 'journal', 'identifier', 'subject_heading', 'keyword'] | QueryExprFieldVariant2

QueryExprQuery: TypeAlias = QueryExprQueryVariant1 | QueryExprQueryVariant2 | QueryExprQueryVariant3 | QueryExprQueryVariant4

QueryExpr: TypeAlias = QueryExprQuery

RankingCalibration = TypedDict(
    'RankingCalibration',
    {
    'approved_by': Required['str | None'],
    'approved_for_prioritisation': Required['bool'],
    'auto_exclusion_prohibited': Required['Literal[True]'],
    'counts': Required['RankingCalibrationCounts'],
    'minimum_sensitivity': Required['int | float'],
    'ranker_version': Required['str'],
    'review_id': Required['str'],
    'schema_version': Required["Literal['org.searchright.ranking-calibration.v1']"],
    'threshold': Required['int | float'],
    },
)

RankingCalibrationCounts = TypedDict(
    'RankingCalibrationCounts',
    {
    'false_negative': Required['int'],
    'false_positive': Required['int'],
    'true_negative': Required['int'],
    'true_positive': Required['int'],
    },
)

RecoveryRehearsal = TypedDict(
    'RecoveryRehearsal',
    {
    'atomic_replace_checked': Required['bool'],
    'claim_boundary': Required['str'],
    'errors': Required['list[str]'],
    'files': Required['int'],
    'restore_idempotency_checked': Required['bool'],
    'scenario_id': Required['str'],
    'schema_version': Required["Literal['org.searchright.recovery-rehearsal.v1']"],
    'stale_temporary_checked': Required['bool'],
    'status': Required["Literal['passed', 'failed']"],
    'tamper_detection_checked': Required['bool'],
    },
)

RedactionProfile = TypedDict(
    'RedactionProfile',
    {
    'claim_boundary': Required['str'],
    'minimum_high_entropy_length': Required['int'],
    'preserve_key_names': Required['Literal[True]'],
    'profile_id': Required['str'],
    'redact_bearer_tokens': Required['Literal[True]'],
    'redact_email_addresses': Required['Literal[True]'],
    'redact_probable_high_entropy_values': Required['Literal[True]'],
    'replacement': Required["Literal['[REDACTED]']"],
    'safe_query_keys': Required['list[str]'],
    'schema_version': Required["Literal['org.searchright.redaction-profile.v1']"],
    'sensitive_object_keys': Required['list[str]'],
    'sensitive_query_keys': Required['list[str]'],
    },
)

ReleaseRehearsal = TypedDict(
    'ReleaseRehearsal',
    {
    'automatic_registry_submission': Required['Literal[False]'],
    'automatic_release': Required['Literal[False]'],
    'claim_boundary': Required['str'],
    'pilot_profiles': Required['list[str]'],
    'required_gates': Required['list[str]'],
    'rollback_required': Required['Literal[True]'],
    'schema_version': Required["Literal['org.searchright.release-rehearsal.v1']"],
    'source_epoch': Required['str'],
    'status': Required["Literal['prepared_not_executed', 'in_progress', 'failed', 'passed']"],
    'target': Required['str'],
    },
)

ResearchObjectHandoffPlan = TypedDict(
    'ResearchObjectHandoffPlan',
    {
    'claim_boundary': Required['str'],
    'delegated_export_track': Required["Literal['25']"],
    'deposit_authorized': Required['Literal[False]'],
    'execution_mode': Required["Literal['dry_run']"],
    'input_artifacts': Required['list[ResearchObjectHandoffPlanInputArtifactsItem]'],
    'osf_acceptance_claimed': Required['Literal[False]'],
    'plan_id': Required['str'],
    'prerequisites': Required['list[str]'],
    'proposed_destinations': Required['list[ResearchObjectHandoffPlanProposedDestinationsItem]'],
    'review_id': Required['str'],
    'ro_crate_conformance_claimed': Required['Literal[False]'],
    'schema_version': Required["Literal['org.searchright.research-object-handoff-plan.v1']"],
    },
)

ResearchObjectHandoffPlanInputArtifactsItem = TypedDict(
    'ResearchObjectHandoffPlanInputArtifactsItem',
    {
    'artifact_id': Required['str'],
    'kind': Required["Literal['srpack', 'audit_export', 'source_receipts']"],
    'sha256': Required['str'],
    },
)

ResearchObjectHandoffPlanProposedDestinationsItem = TypedDict(
    'ResearchObjectHandoffPlanProposedDestinationsItem',
    {
    'external_write': Required['Literal[True]'],
    'kind': Required["Literal['ro_crate', 'osf']"],
    'target_reference': Required['str'],
    },
)

ReviewBundleManifest = TypedDict(
    'ReviewBundleManifest',
    {
    'bundle_id': Required['str'],
    'claim_boundary': Required['str'],
    'descriptor_merkle_root': Required['str'],
    'entries': Required['list[ReviewBundleManifestEntriesItem]'],
    'entry_count': Required['int'],
    'format_version': Required["Literal['1']"],
    'payload_bytes': Required['int'],
    'policy': Required['ReviewBundleManifestPolicy'],
    'review_id': Required['str'],
    'schema_version': Required["Literal['org.searchright.review-bundle-manifest.v1']"],
    'source_epoch': Required['str'],
    },
)

ReviewBundleManifestEntriesItem = TypedDict(
    'ReviewBundleManifestEntriesItem',
    {
    'archive_path': Required['str'],
    'media_type': Required['str'],
    'path': Required['str'],
    'role': Required['str'],
    'sha256': Required['str'],
    'size': Required['int'],
    },
)

ReviewBundleManifestPolicy = TypedDict(
    'ReviewBundleManifestPolicy',
    {
    'deterministic': Required['Literal[True]'],
    'external_writes_allowed': Required['Literal[False]'],
    'max_file_bytes': Required['int'],
    'max_files': Required['int'],
    'max_total_bytes': Required['int'],
    'network_required': Required['Literal[False]'],
    'secret_scan_required': Required['Literal[True]'],
    'symlinks_allowed': Required['Literal[False]'],
    },
)

ReviewPlan = TypedDict(
    'ReviewPlan',
    {
    'eligibility': Required['ReviewPlanEligibility'],
    'governance': Required['ReviewPlanGovernance'],
    'information_sources': Required['list[ReviewPlanSource]'],
    'objectives': Required['list[str]'],
    'protocol': Required['ReviewPlanProtocol'],
    'question': Required['ReviewPlanQuestion'],
    'review_id': Required['str'],
    'review_kind': Required['ReviewPlanReviewKind'],
    'schema_version': Required["Literal['org.searchright.review-plan.v1']"],
    'strategy_ids': Required['list[str]'],
    'title': Required['str'],
    },
)

ReviewPlanCriterion = TypedDict(
    'ReviewPlanCriterion',
    {
    'domain': Required['str'],
    'id': Required['str'],
    'priority': Required['int'],
    'rationale': Required['str'],
    'rule': Required['str'],
    'stage': Required["Literal['title_abstract', 'full_text', 'any']"],
    },
)

ReviewPlanEligibility = TypedDict(
    'ReviewPlanEligibility',
    {
    'exclude': Required['list[ReviewPlanCriterion]'],
    'include': Required['list[ReviewPlanCriterion]'],
    'version': Required['str'],
    },
)

ReviewPlanFrameworkKindVariant2 = TypedDict(
    'ReviewPlanFrameworkKindVariant2',
    {
    'custom': Required['str'],
    },
)

ReviewPlanGovernance = TypedDict(
    'ReviewPlanGovernance',
    {
    'conflict_resolution': Required['str'],
    'full_text_reviewers': Required['int'],
    'press_review_required': Required['bool'],
    'protocol_amendment_roles': Required['list[str]'],
    'title_abstract_reviewers': Required['int'],
    },
)

ReviewPlanProtocol = TypedDict(
    'ReviewPlanProtocol',
    {
    'amendments': Required['list[str]'],
    'identifier': Required['None | str'],
    'registry': Required['None | str'],
    'version': Required['str'],
    },
)

ReviewPlanProtocolVariant1 = TypedDict(
    'ReviewPlanProtocolVariant1',
    {
    'identifier': NotRequired['None'],
    'registry': NotRequired['None'],
    },
)

ReviewPlanProtocolVariant2 = TypedDict(
    'ReviewPlanProtocolVariant2',
    {
    'identifier': NotRequired['str'],
    'registry': NotRequired['str'],
    },
)

ReviewPlanQuestion = TypedDict(
    'ReviewPlanQuestion',
    {
    'framework': Required['ReviewPlanQuestionFramework'],
    'notes': NotRequired['list[str]'],
    'text': Required['str'],
    },
)

ReviewPlanQuestionFramework = TypedDict(
    'ReviewPlanQuestionFramework',
    {
    'elements': Required['ReviewPlanQuestionFrameworkElements'],
    'kind': Required['ReviewPlanFrameworkKind'],
    },
)

ReviewPlanQuestionFrameworkElements = TypedDict(
    'ReviewPlanQuestionFrameworkElements',
    {

    },
)

ReviewPlanReviewKindVariant2 = TypedDict(
    'ReviewPlanReviewKindVariant2',
    {
    'other': Required['str'],
    },
)

ReviewPlanSource = TypedDict(
    'ReviewPlanSource',
    {
    'access_notes': Required['list[str]'],
    'id': Required['str'],
    'kind': Required['ReviewPlanSourceKind'],
    'name': Required['str'],
    'platform': Required['None | str'],
    'provider': Required['str'],
    'required': Required['bool'],
    },
)

ReviewPlanSourceKindVariant2 = TypedDict(
    'ReviewPlanSourceKindVariant2',
    {
    'other': Required['str'],
    },
)

ReviewPlanFrameworkKind: TypeAlias = Literal['pico', 'pecos', 'pcc', 'spider', 'peo'] | ReviewPlanFrameworkKindVariant2

ReviewPlanReviewKind: TypeAlias = Literal['systematic', 'scoping', 'rapid', 'living', 'evidence_map', 'umbrella'] | ReviewPlanReviewKindVariant2

ReviewPlanSourceKind: TypeAlias = Literal['database', 'register', 'repository', 'website', 'citation_search', 'contact', 'handsearch', 'grey_literature', 'import'] | ReviewPlanSourceKindVariant2

DerivedReviewStateSnapshot = TypedDict(
    'DerivedReviewStateSnapshot',
    {
    'claim_boundary': Required['str'],
    'event_type_counts': Required['DerivedReviewStateSnapshotEventTypeCounts'],
    'last_event_id': Required['str'],
    'plan_validated': Required['bool'],
    'protocol_amendments': Required['list[str]'],
    'review_id': Required['str'],
    'schema_version': Required["Literal['org.searchright.review-state-snapshot.v1']"],
    'screening': Required['DerivedReviewStateSnapshotScreening'],
    'search_runs': Required['list[DerivedReviewStateSnapshotSearchRunsItem]'],
    'source_event_count': Required['int'],
    'source_head_hash': Required['str'],
    'state_sha256': Required['str'],
    'state_version': Required['Literal[1]'],
    'status': Required['str'],
    'unknown_event_types': Required['list[str]'],
    },
)

DerivedReviewStateSnapshotEventTypeCounts = TypedDict(
    'DerivedReviewStateSnapshotEventTypeCounts',
    {

    },
)

DerivedReviewStateSnapshotScreening = TypedDict(
    'DerivedReviewStateSnapshotScreening',
    {
    'advisory_recommendation_count': Required['int'],
    'final_decision_counts': Required['DerivedReviewStateSnapshotScreeningFinalDecisionCounts'],
    'final_decisions': Required['list[DerivedReviewStateSnapshotScreeningFinalDecisionsItem]'],
    'rejected_final_authority_event_ids': Required['list[str]'],
    },
)

DerivedReviewStateSnapshotScreeningFinalDecisionCounts = TypedDict(
    'DerivedReviewStateSnapshotScreeningFinalDecisionCounts',
    {

    },
)

DerivedReviewStateSnapshotScreeningFinalDecisionsItem = TypedDict(
    'DerivedReviewStateSnapshotScreeningFinalDecisionsItem',
    {
    'decision': Required['str | None'],
    'event_id': Required['str'],
    'record_id': Required['str'],
    'reviewer_id': Required['str | None'],
    'stage': Required['str | None'],
    },
)

DerivedReviewStateSnapshotSearchRunsItem = TypedDict(
    'DerivedReviewStateSnapshotSearchRunsItem',
    {
    'event_id': Required['str'],
    'record_count': Required['int | None'],
    'run_id': Required['str'],
    'source_id': Required['str | None'],
    },
)

SchemaMigrationPlan = TypedDict(
    'SchemaMigrationPlan',
    {
    'automatic_apply': Required['Literal[False]'],
    'backup_required': Required['Literal[True]'],
    'claim_boundary': Required['str'],
    'classification': Required['str'],
    'destructive': Required['Literal[False]'],
    'family': Required['str'],
    'from_version': Required['int'],
    'migration_id': Required['str'],
    'preconditions': Required['list[str]'],
    'rollback': Required['SchemaMigrationPlanRollback'],
    'schema_version': Required["Literal['org.searchright.schema-migration-plan.v1']"],
    'to_version': Required['int'],
    'transformations': Required['list[SchemaMigrationPlanTransformationsItem]'],
    'verification': Required['list[str]'],
    },
)

SchemaMigrationPlanRollback = TypedDict(
    'SchemaMigrationPlanRollback',
    {
    'losses': Required['list[str]'],
    'strategy': Required['str'],
    'supported': Required['Literal[True]'],
    },
)

SchemaMigrationPlanTransformationsItem = TypedDict(
    'SchemaMigrationPlanTransformationsItem',
    {
    'description': Required['str'],
    'operation': Required["Literal['preserve', 'derive', 'rename', 'transform']"],
    'path': Required['str'],
    },
)

SchemaMigrationRegistry = TypedDict(
    'SchemaMigrationRegistry',
    {
    'automatic_migration': Required['Literal[False]'],
    'claim_boundary': Required['str'],
    'default_policy': Required['SchemaMigrationRegistryDefaultPolicy'],
    'families': Required['list[SchemaMigrationRegistryFamiliesItem]'],
    'schema_version': Required["Literal['org.searchright.schema-migration-registry.v1']"],
    },
)

SchemaMigrationRegistryDefaultPolicy = TypedDict(
    'SchemaMigrationRegistryDefaultPolicy',
    {
    'backup_required': Required['Literal[True]'],
    'destructive_migration': Required["Literal['deny']"],
    'implicit_write_upgrade': Required["Literal['deny']"],
    'receipt_required': Required['Literal[True]'],
    'unknown_version': Required["Literal['reject']"],
    },
)

SchemaMigrationRegistryFamiliesItem = TypedDict(
    'SchemaMigrationRegistryFamiliesItem',
    {
    'current_write_version': Required['int'],
    'family': Required['str'],
    'migrations': Required['list[str]'],
    'minimum_read_version': Required['int'],
    'versions': Required['list[SchemaMigrationRegistryFamiliesItemVersionsItem]'],
    },
)

SchemaMigrationRegistryFamiliesItemVersionsItem = TypedDict(
    'SchemaMigrationRegistryFamiliesItemVersionsItem',
    {
    'schema_id': Required['str'],
    'status': Required["Literal['supported_read_only', 'current', 'deprecated']"],
    'version': Required['int'],
    },
)

ScreeningDecision = TypedDict(
    'ScreeningDecision',
    {
    'agent_provenance': Required['None | str'],
    'confidence': Required['None | int | float'],
    'decided_at': Required['str'],
    'decision': Required["Literal['include', 'exclude', 'unclear']"],
    'decision_id': Required['str'],
    'eligibility_version': Required['str'],
    'exclusion_reason': Required['None | ScreeningDecisionExclusionReasonVariant2'],
    'rationale': Required['str'],
    'review_id': Required['str'],
    'reviewer_id': Required['str'],
    'reviewer_kind': Required["Literal['human', 'agent', 'adjudicator']"],
    'round': Required["Literal['title_abstract', 'full_text']"],
    'subject_id': Required['str'],
    },
)

ScreeningDecisionExclusionReasonVariant2 = TypedDict(
    'ScreeningDecisionExclusionReasonVariant2',
    {
    'criterion_id': Required['str'],
    'evidence': Required['None | str'],
    'label': Required['str'],
    'reason_id': Required['str'],
    },
)

ScreeningPolicy = TypedDict(
    'ScreeningPolicy',
    {
    'adjudication_rule': Required['str'],
    'agent_authority': Required["Literal['advisory_only', 'include_only', 'exclusion_with_human_confirmation']"],
    'full_text_reviewers': Required['int'],
    'independent_blinding': Required['bool'],
    'minimum_agent_sensitivity': Required['int | float | None'],
    'schema_version': Required["Literal['org.searchright.screening-policy.v1']"],
    'title_abstract_reviewers': Required['int'],
    },
)

SearchRun = TypedDict(
    'SearchRun',
    {
    'completed_at': Required['str | None'],
    'purpose': Required['str'],
    'receipts': Required['list[JsonValue]'],
    'review_id': Required['str'],
    'run_id': Required['str'],
    'schema_version': Required["Literal['org.searchright.search-run.v1']"],
    'started_at': Required['str'],
    'supersedes_run_id': Required['str | None'],
    },
)

SearchStrategy = TypedDict(
    'SearchStrategy',
    {
    'dialect': Required["Literal['pub_med', 'ovid_medline', 'embase', 'europe_pmc', 'cinahl_ebsco', 'psyc_info_ovid', 'scopus', 'web_of_science', 'crossref', 'open_alex', 'clinical_trials_gov', 'generic_boolean'] | SearchStrategyDialectVariant2"],
    'limits': Required['SearchStrategyLimits'],
    'notes': Required['list[str]'],
    'query': Required['JsonValue'],
    'review_id': Required['str'],
    'schema_version': Required["Literal['org.searchright.search-strategy.v1']"],
    'source_id': Required['str'],
    'strategy_id': Required['str'],
    'translated_from': Required['None | str'],
    },
)

SearchStrategyDialectVariant2 = TypedDict(
    'SearchStrategyDialectVariant2',
    {
    'custom': Required['str'],
    },
)

SearchStrategyLimits = TypedDict(
    'SearchStrategyLimits',
    {
    'filters': Required['list[str]'],
    'languages': Required['list[str]'],
    'publication_date': Required['None | SearchStrategyLimitsPublicationDateVariant2'],
    'publication_types': Required['list[str]'],
    'rationale': Required['list[str]'],
    },
)

SearchStrategyLimitsPublicationDateVariant2 = TypedDict(
    'SearchStrategyLimitsPublicationDateVariant2',
    {
    'from_year': Required['None | int'],
    'to_year': Required['None | int'],
    },
)

SearchValidationReport = TypedDict(
    'SearchValidationReport',
    {
    'approved_by': Required['str | None'],
    'approved_for_execution': Required['bool'],
    'minimum_seed_recall': Required['int | float | None'],
    'press_reviews': Required['list[SearchValidationReportPressReviewsItem]'],
    'review_id': Required['str'],
    'schema_version': Required["Literal['org.searchright.search-validation.v1']"],
    'seed_records': Required['list[SearchValidationReportSeedRecordsItem]'],
    'strategy_id': Required['str'],
    'strategy_version': Required['str'],
    'translation_assessments': Required['list[SearchValidationReportTranslationAssessmentsItem]'],
    },
)

SearchValidationReportPressReviewsItem = TypedDict(
    'SearchValidationReportPressReviewsItem',
    {
    'decision': Required['str'],
    'findings': Required['list[SearchValidationReportPressReviewsItemFindingsItem]'],
    'press_review_id': Required['str'],
    'reviewed_at': Required['str'],
    'reviewer_id': Required['str'],
    'strategy_id': Required['str'],
    'strategy_version': Required['str'],
    },
)

SearchValidationReportPressReviewsItemFindingsItem = TypedDict(
    'SearchValidationReportPressReviewsItemFindingsItem',
    {
    'element': Required["Literal['translation_of_question', 'boolean_and_proximity', 'subject_headings', 'text_words', 'spelling_syntax_and_lines', 'limits_and_filters']"],
    'finding_id': Required['str'],
    'message': Required['str'],
    'recommendation': Required['str'],
    'resolved': Required['bool'],
    'severity': Required["Literal['note', 'advisory', 'major', 'critical']"],
    },
)

SearchValidationReportSeedRecordsItem = TypedDict(
    'SearchValidationReportSeedRecordsItem',
    {
    'identifier': Required['str'],
    'relevance_basis': Required['str'],
    'retrieved': Required['bool'],
    'seed_id': Required['str'],
    'source_id': Required['str'],
    },
)

SearchValidationReportTranslationAssessmentsItem = TypedDict(
    'SearchValidationReportTranslationAssessmentsItem',
    {
    'human_approved': Required['bool'],
    'maximum_material_warnings': Required['int'],
    'notes': Required['list[str]'],
    'observed_material_warnings': Required['int'],
    'strategy_id': Required['str'],
    'target_dialect': Required['str'],
    },
)

SourceReceipt = TypedDict(
    'SourceReceipt',
    {
    'cache_hits': Required['int'],
    'cache_writes': Required['int'],
    'compiler_version': Required['str'],
    'endpoint': Required['str | None'],
    'executed_at': Required['str'],
    'execution_mode': Required["Literal['fixture', 'replay', 'live', 'mixed-fixture-replay', 'mixed-live-replay']"],
    'pages_retrieved': Required['int'],
    'policy': Required['SourceReceiptPolicy'],
    'provider_id': Required['str'],
    'provider_version': Required['str'],
    'query_hash': Required['str'],
    'receipt_id': Required['str'],
    'records_retrieved': Required['int'],
    'result_digest': Required['str'],
    'review_id': Required['str'],
    'run_id': Required['str'],
    'schema_version': Required["Literal['org.searchright.source-receipt.v1']"],
    'source_label': Required['str'],
    'strategy_id': Required['str'],
    'warnings': Required['list[str]'],
    },
)

SourceReceiptPolicy = TypedDict(
    'SourceReceiptPolicy',
    {
    'cache_write_enabled': Required['bool'],
    'live_enabled': Required['bool'],
    'max_pages': Required['int'],
    'max_records': Required['int'],
    'max_response_bytes': NotRequired['int | None'],
    'max_retries': Required['int'],
    'min_interval_ms': Required['int'],
    'replay_enabled': Required['bool'],
    'retry_base_delay_ms': NotRequired['int | None'],
    'retry_max_delay_ms': NotRequired['int | None'],
    'timeout_seconds': Required['int'],
    'total_timeout_seconds': NotRequired['int | None'],
    },
)

SourcerightParityReport = TypedDict(
    'SourcerightParityReport',
    {
    'blockers': Required['list[str]'],
    'case_ids': Required['list[str]'],
    'cutover_ready': Required['bool'],
    'dimensions': Required['list[SourcerightParityReportDimensionsItem]'],
    'generated_at': Required['str'],
    'legacy_revision': Required['str'],
    'schema_version': Required["Literal['org.searchright.sourceright-parity-report.v1']"],
    'shared_revision': Required['str'],
    },
)

SourcerightParityReportDimensionsItem = TypedDict(
    'SourcerightParityReportDimensionsItem',
    {
    'approved_difference_id': Required['str | None'],
    'dimension': Required['str'],
    'equivalent': Required['bool'],
    'legacy_digest': Required['str'],
    'note': Required['str'],
    'shared_digest': Required['str'],
    },
)

StandardAssessment = TypedDict(
    'StandardAssessment',
    {
    'assessed_at': Required['str'],
    'assessed_by': Required['str'],
    'items': Required['list[StandardAssessmentItemsItem]'],
    'pack_id': Required['str'],
    'pack_version': Required['str'],
    'review_id': Required['str'],
    'schema_version': Required["Literal['org.searchright.standard-assessment.v1']"],
    },
)

StandardAssessmentItemsItem = TypedDict(
    'StandardAssessmentItemsItem',
    {
    'evidence': Required['list[str]'],
    'item_id': Required['str'],
    'note': Required['str'],
    'state': Required["Literal['met', 'partial', 'unmet', 'not_applicable', 'not_assessed']"],
    },
)

StandardPack = TypedDict(
    'StandardPack',
    {
    'family': Required["Literal['prisma2020', 'prisma_s', 'prisma_sc_r', 'prisma_lsr', 'prisma_p', 'press2015', 'cochrane_handbook', 'mecir', 'jbi', 'campbell'] | StandardPackFamilyVariant2"],
    'items': Required['list[StandardPackItemsItem]'],
    'pack_id': Required['str'],
    'provenance_note': Required['str'],
    'schema_version': Required["Literal['org.searchright.standard-pack.v1']"],
    'source': Required['str'],
    'version': Required['str'],
    },
)

StandardPackFamilyVariant2 = TypedDict(
    'StandardPackFamilyVariant2',
    {
    'custom': Required['str'],
    },
)

StandardPackItemsItem = TypedDict(
    'StandardPackItemsItem',
    {
    'crosswalks': Required['list[str]'],
    'item_id': Required['str'],
    'label': Required['str'],
    'requirement_summary': Required['str'],
    'scope': Required['str'],
    },
)

StudyGraph = TypedDict(
    'StudyGraph',
    {
    'links': Required['list[StudyGraphLinksItem]'],
    'reports': Required['list[StudyGraphReportsItem]'],
    'review_id': Required['str'],
    'schema_version': Required["Literal['org.searchright.study-graph.v1']"],
    'studies': Required['list[StudyGraphStudiesItem]'],
    },
)

StudyGraphLinksItem = TypedDict(
    'StudyGraphLinksItem',
    {
    'asserted_at': Required['str'],
    'asserted_by': Required['str'],
    'confidence': Required['int | float'],
    'evidence': Required['list[str]'],
    'from_id': Required['str'],
    'link_id': Required['str'],
    'relationship': Required["Literal['record_describes_report', 'report_of_study', 'protocol_for_study', 'secondary_analysis_of_study', 'updates_report', 'duplicate_of'] | StudyGraphLinksItemRelationshipVariant2"],
    'to_id': Required['str'],
    },
)

StudyGraphLinksItemRelationshipVariant2 = TypedDict(
    'StudyGraphLinksItemRelationshipVariant2',
    {
    'custom': Required['str'],
    },
)

StudyGraphReportsItem = TypedDict(
    'StudyGraphReportsItem',
    {
    'doi': Required['str | None'],
    'pmid': Required['str | None'],
    'publication_year': Required['int | None'],
    'record_ids': Required['list[str]'],
    'registry_ids': Required['list[str]'],
    'report_id': Required['str'],
    'retrieval_attempts': Required['list[StudyGraphReportsItemRetrievalAttemptsItem]'],
    'title': Required['str'],
    },
)

StudyGraphReportsItemRetrievalAttemptsItem = TypedDict(
    'StudyGraphReportsItemRetrievalAttemptsItem',
    {
    'attempt_id': Required['str'],
    'attempted_at': Required['str'],
    'content_digest': Required['str | None'],
    'method': Required['str'],
    'note': Required['str'],
    'report_id': Required['str'],
    'rights_basis': Required['str | None'],
    'status': Required["Literal['not_attempted', 'retrieved', 'not_retrieved', 'restricted', 'unavailable', 'awaiting_contact'] | StudyGraphReportsItemRetrievalAttemptsItemStatusVariant2"],
    },
)

StudyGraphReportsItemRetrievalAttemptsItemStatusVariant2 = TypedDict(
    'StudyGraphReportsItemRetrievalAttemptsItemStatusVariant2',
    {
    'other': Required['str'],
    },
)

StudyGraphStudiesItem = TypedDict(
    'StudyGraphStudiesItem',
    {
    'label': Required['str'],
    'notes': Required['list[str]'],
    'registration_ids': Required['list[str]'],
    'report_ids': Required['list[str]'],
    'study_design': Required['str | None'],
    'study_id': Required['str'],
    },
)

TelemetryPolicy = TypedDict(
    'TelemetryPolicy',
    {
    'approved_by': Required['str | None'],
    'attribute_allowlist': Required['list[str]'],
    'enabled': Required['bool'],
    'endpoint': Required['str | None'],
    'policy_id': Required['str'],
    'prohibited_attributes': Required['list[str]'],
    'retention_days': Required['int'],
    'sampling_per_million': Required['int'],
    'schema_version': Required["Literal['org.searchright.telemetry-policy.v1']"],
    },
)

TenantPolicy = TypedDict(
    'TenantPolicy',
    {
    'allowed_regions': Required['list[str]'],
    'allowed_scopes': Required["list[Literal['review_read', 'review_write', 'search_execute', 'screening_recommend', 'screening_decide', 'tenant_admin', 'external_write']]"],
    'approved_by': Required['str'],
    'cross_tenant_aggregation_allowed': Required['Literal[False]'],
    'external_model_processing_allowed': Required['bool'],
    'maximum_concurrent_tasks': Required['int'],
    'policy_version': Required['str'],
    'restricted_full_text_persistence_allowed': Required['bool'],
    'schema_version': Required["Literal['org.searchright.tenant-policy.v1']"],
    'tenant_id': Required['str'],
    },
)

WorkflowTrace = TypedDict(
    'WorkflowTrace',
    {
    'initial_stage': Required['WorkflowTraceStage'],
    'review_id': Required['str'],
    'schema_version': Required["Literal['org.searchright.workflow-trace.v1']"],
    'transitions': Required['list[WorkflowTraceTransition]'],
    },
)

WorkflowTraceTransition = TypedDict(
    'WorkflowTraceTransition',
    {
    'actor_id': Required['str'],
    'actor_kind': Required['WorkflowTraceActorKind'],
    'approved': Required['bool'],
    'evidence_ids': Required['list[str]'],
    'from': Required['WorkflowTraceStage'],
    'occurred_at': Required['str'],
    'to': Required['WorkflowTraceStage'],
    'transition_id': Required['str'],
    },
)

WorkflowTraceActorKind: TypeAlias = Literal['human', 'tool', 'agent']

WorkflowTraceStage: TypeAlias = Literal['draft', 'plan_approved', 'strategy_validated', 'execution_approved', 'search_executed', 'deduplicated', 'title_abstract_complete', 'full_text_complete', 'reported', 'update_planned']

__all__ = [
    'CONTRACT_IDS',
    'JsonValue',
    'AccessDecision',
    'AccessRequest',
    'AgentWorkflow',
    'AgentWorkflowStepsItem',
    'ArchitecturePolicy',
    'ArchitecturePolicyExternalWriteScriptsItem',
    'ArchitecturePolicyForbiddenInternalEdgesItem',
    'ArchitecturePolicyNetworkDependencies',
    'AuditEvent',
    'AuditEventActor',
    'AuditEventRegistry',
    'AuditEventRegistryEventTypesItem',
    'AuditEventRegistryEventTypesItemPayloadFieldTypes',
    'AuditEventRegistryEventTypesItemVersionsItem',
    'BackupManifest',
    'BenchmarkReport',
    'BenchmarkReportMetric',
    'BibliographicRecord',
    'BibliographicRecordIdentifiers',
    'BibliographicRecordIdentifiersOther',
    'BibliographicRecordKindVariant2',
    'CompiledStrategy',
    'CompiledStrategyDialectVariant2',
    'CompiledStrategyWarningsItem',
    'ComponentHealth',
    'ConsumerContractSuite',
    'ConsumerContractSuiteInteraction',
    'ConsumerContractSuiteNonemptyStrings',
    'DataHandlingDecision',
    'DataHandlingRequest',
    'DataLifecycleDecision',
    'DataLifecycleRequest',
    'DataLifecycleRequestApprovalVariant2',
    'DerivedReviewStateSnapshot',
    'DerivedReviewStateSnapshotEventTypeCounts',
    'DerivedReviewStateSnapshotScreening',
    'DerivedReviewStateSnapshotScreeningFinalDecisionCounts',
    'DerivedReviewStateSnapshotScreeningFinalDecisionsItem',
    'DerivedReviewStateSnapshotSearchRunsItem',
    'Diagnostic',
    'DiagnosticLocaleVariant2',
    'DiscoveryRun',
    'DiscoveryRunEdgesItem',
    'DiscoveryRunEdgesItemMethodVariant2',
    'DiscoveryRunMethodVariant2',
    'DocumentEvidence',
    'DocumentEvidenceCallout',
    'DocumentEvidenceDiagnostic',
    'DocumentEvidenceField',
    'DocumentEvidenceProvenance',
    'DocumentEvidenceReference',
    'DocumentEvidenceSpan',
    'EvidenceDebtRegister',
    'EvidenceDebtRegisterAssertions',
    'EvidenceDebtRegisterAssertionsByMappingConfidence',
    'EvidenceDebtRegisterAssertionsByState',
    'EvidenceDebtRegisterMaturity',
    'EvidenceDebtRegisterPriorityQueueItem',
    'EvidenceDebtRegisterProviderPolicy',
    'EvidenceDebtRegisterPublication',
    'EvidenceDebtRegisterStaticGates',
    'EvidenceDebtRegisterTracks',
    'EvidenceDebtRegisterTracksByState',
    'ExecutionEnvelope',
    'GateCatalog',
    'GateCatalogDefaultCapabilities',
    'GateCatalogGatesItem',
    'GitHubIssueHierarchy',
    'GitHubIssueHierarchyNodesItem',
    'GitHubIssueHierarchyNodesItemProjectFields',
    'GitHubProjectManifest',
    'GitHubProjectManifestFieldsItem',
    'GitHubProjectManifestSync',
    'GitHubProjectManifestViewsItem',
    'GitHubRepositorySettings',
    'GitHubRepositorySettingsFeatures',
    'GitHubRepositorySettingsMergePolicy',
    'GitHubRepositorySettingsRuleset',
    'GitHubRepositorySettingsSecurity',
    'GithubControlPlaneApplySummary',
    'GithubControlPlaneApplySummaryArtifact',
    'GithubControlPlaneApplySummaryArtifactFilesItem',
    'GithubControlPlaneApplySummaryAudit',
    'GithubControlPlaneApplySummaryIssueSync',
    'GithubControlPlaneApplySummaryNonnegativeCount',
    'GithubControlPlaneApplySummaryProjectSync',
    'GithubControlPlaneApplySummaryWorkflowRun',
    'IncidentRecord',
    'InstitutionalPolicy',
    'IntegrationPassport',
    'IntegrationPassportCanonicalUpstreamVariant2',
    'IntegrationPassportContracts',
    'IntegrationPassportContractsItem',
    'IntegrationPassportVariant1',
    'IntegrationPassportVariant2',
    'IntegrationPassportVerificationItem',
    'IntegrationReleaseTrain',
    'IntegrationReleaseTrainComponentsItem',
    'IntegrationReleaseTrainStagesItem',
    'InterchangeReceipt',
    'InterchangeReceiptInputFormatVariant2',
    'InterchangeReceiptOutputFormatVariant2',
    'LicensedAdapterProfile',
    'LivingUpdateRun',
    'LivingUpdateRunChangesItem',
    'LivingUpdateRunCursorsAfterItem',
    'LivingUpdateRunCursorsBeforeItem',
    'NamedFilterPack',
    'NamedFilterPackApplicability',
    'NamedFilterPackChecksum',
    'NamedFilterPackDate',
    'NamedFilterPackDialect',
    'NamedFilterPackDialectVariant2',
    'NamedFilterPackNamedFilter',
    'NamedFilterPackRights',
    'NamedFilterPackSource',
    'NamedFilterPackValidation',
    'NativeSearchStrategy',
    'NativeSearchStrategyDiagnosticsItem',
    'NativeSearchStrategyDialectVariant2',
    'NativeSearchStrategyLinesItem',
    'NativeSearchStrategySpan',
    'PrismaFlow',
    'PrismaFlowFullTextExclusionsItem',
    'ProtocolAmendment',
    'ProtocolAmendmentChangesItem',
    'ProtocolAmendmentKindVariant2',
    'ProviderComponentManifest',
    'ProviderComponentReleaseSignature',
    'ProviderComponentTrustPolicy',
    'ProviderComponentTrustPolicyRevocationsItem',
    'ProviderComponentTrustPolicyTrustedKeysItem',
    'ProviderManifest',
    'ProviderPage',
    'ProviderPageDiagnostics',
    'ProviderPolicySet',
    'ProviderPolicySetProvidersItem',
    'QueryExpr',
    'QueryExprField',
    'QueryExprFieldVariant2',
    'QueryExprQuery',
    'QueryExprQueryVariant1',
    'QueryExprQueryVariant2',
    'QueryExprQueryVariant3',
    'QueryExprQueryVariant4',
    'QueryExprTerm',
    'RankingCalibration',
    'RankingCalibrationCounts',
    'RecoveryRehearsal',
    'RedactionProfile',
    'ReleaseRehearsal',
    'ResearchObjectHandoffPlan',
    'ResearchObjectHandoffPlanInputArtifactsItem',
    'ResearchObjectHandoffPlanProposedDestinationsItem',
    'ReviewBundleManifest',
    'ReviewBundleManifestEntriesItem',
    'ReviewBundleManifestPolicy',
    'ReviewPlan',
    'ReviewPlanCriterion',
    'ReviewPlanEligibility',
    'ReviewPlanFrameworkKind',
    'ReviewPlanFrameworkKindVariant2',
    'ReviewPlanGovernance',
    'ReviewPlanProtocol',
    'ReviewPlanProtocolVariant1',
    'ReviewPlanProtocolVariant2',
    'ReviewPlanQuestion',
    'ReviewPlanQuestionFramework',
    'ReviewPlanQuestionFrameworkElements',
    'ReviewPlanReviewKind',
    'ReviewPlanReviewKindVariant2',
    'ReviewPlanSource',
    'ReviewPlanSourceKind',
    'ReviewPlanSourceKindVariant2',
    'SchemaMigrationPlan',
    'SchemaMigrationPlanRollback',
    'SchemaMigrationPlanTransformationsItem',
    'SchemaMigrationRegistry',
    'SchemaMigrationRegistryDefaultPolicy',
    'SchemaMigrationRegistryFamiliesItem',
    'SchemaMigrationRegistryFamiliesItemVersionsItem',
    'ScreeningDecision',
    'ScreeningDecisionExclusionReasonVariant2',
    'ScreeningPolicy',
    'SearchRun',
    'SearchStrategy',
    'SearchStrategyDialectVariant2',
    'SearchStrategyLimits',
    'SearchStrategyLimitsPublicationDateVariant2',
    'SearchValidationReport',
    'SearchValidationReportPressReviewsItem',
    'SearchValidationReportPressReviewsItemFindingsItem',
    'SearchValidationReportSeedRecordsItem',
    'SearchValidationReportTranslationAssessmentsItem',
    'SourceReceipt',
    'SourceReceiptPolicy',
    'SourcerightParityReport',
    'SourcerightParityReportDimensionsItem',
    'StandardAssessment',
    'StandardAssessmentItemsItem',
    'StandardPack',
    'StandardPackFamilyVariant2',
    'StandardPackItemsItem',
    'StudyGraph',
    'StudyGraphLinksItem',
    'StudyGraphLinksItemRelationshipVariant2',
    'StudyGraphReportsItem',
    'StudyGraphReportsItemRetrievalAttemptsItem',
    'StudyGraphReportsItemRetrievalAttemptsItemStatusVariant2',
    'StudyGraphStudiesItem',
    'TelemetryPolicy',
    'TenantPolicy',
    'WorkflowTrace',
    'WorkflowTraceActorKind',
    'WorkflowTraceStage',
    'WorkflowTraceTransition'
]
