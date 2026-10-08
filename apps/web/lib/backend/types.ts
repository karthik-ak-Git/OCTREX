export type ErrorCode =
  | 'VALIDATION_ERROR'
  | 'NOT_FOUND'
  | 'CONFLICT'
  | 'PERMISSION_DENIED'
  | 'POLICY_DENIED'
  | 'PRIVACY_BLOCKED'
  | 'NETWORK_BLOCKED'
  | 'MODEL_UNAVAILABLE'
  | 'HARDWARE_INCOMPATIBLE'
  | 'CONTEXT_OVERFLOW'
  | 'TOOL_FAILED'
  | 'VERIFICATION_FAILED'
  | 'FILESYSTEM_ERROR'
  | 'PROCESS_ERROR'
  | 'MCP_ERROR'
  | 'PROVIDER_ERROR'
  | 'TIMEOUT'
  | 'CANCELLED'
  | 'INTERNAL_ERROR';

export interface AppErrorResponse {
  code: ErrorCode;
  message: string;
  details?: Record<string, unknown> | null;
  correlation_id?: string | null;
}

export interface ApplicationInfo {
  name: string;
  version: string;
  backend_version: string;
  protocol_version: string;
  environment: string;
}

export interface RuntimeStatus {
  state: string;
  uptime_ms: number;
  active_tasks: number;
  active_sessions: number;
  active_workspaces: number;
}

export interface BackendHealth {
  status: 'HEALTHY' | 'DEGRADED' | 'UNHEALTHY';
  version: string;
  runtime: string;
  initialized: boolean;
  subsystems: Record<string, string>;
}

export interface ConfigurationSummary {
  active_provider: string;
  active_workspace?: string | null;
  provider_count: number;
  privacy_mode: string;
  enforcement_level: string;
}

export type EventType =
  | 'APPLICATION_READY'
  | 'TASK_CREATED'
  | 'TASK_STARTED'
  | 'TASK_PAUSED'
  | 'TASK_RESUMED'
  | 'TASK_CANCELLED'
  | 'TASK_COMPLETED'
  | 'TASK_FAILED'
  | 'STEP_STARTED'
  | 'STEP_COMPLETED'
  | 'PRIVACY_CHECK_STARTED'
  | 'PRIVACY_DECISION'
  | 'MODEL_SELECTED'
  | 'MODEL_STARTED'
  | 'MODEL_COMPLETED'
  | 'MODEL_FAILED'
  | 'PERMISSION_REQUIRED'
  | 'PERMISSION_GRANTED'
  | 'PERMISSION_DENIED'
  | 'TOOL_STARTED'
  | 'TOOL_OUTPUT'
  | 'TOOL_COMPLETED'
  | 'TOOL_FAILED'
  | 'FILE_READ'
  | 'FILE_CREATED'
  | 'FILE_CHANGED'
  | 'FILE_DELETED'
  | 'TERMINAL_STARTED'
  | 'TERMINAL_OUTPUT'
  | 'TERMINAL_EXITED'
  | 'VERIFICATION_STARTED'
  | 'VERIFICATION_PASSED'
  | 'VERIFICATION_FAILED'
  | 'NETWORK_ALLOWED'
  | 'NETWORK_BLOCKED'
  | 'HARDWARE_DETECTION_STARTED'
  | 'HARDWARE_DETECTION_COMPLETED'
  | 'HARDWARE_DETECTION_FAILED'
  | 'HARDWARE_PROFILE_CHANGED'
  | 'MODEL_COMPATIBILITY_CHECKED';

export interface EventEnvelope {
  event_id: string;
  event_type: EventType;
  timestamp: number;
  request_id?: string | null;
  task_id?: string | null;
  session_id?: string | null;
  workspace_id?: string | null;
  payload: Record<string, unknown>;
}

export interface ProviderHealthCheck {
  provider_id: string;
  status: {
    Connected?: { models: string[] };
    MissingApiKey?: null;
    AuthError?: { message: string };
    Offline?: { reason: string };
  };
  latency_ms: number;
}

export type ExecutionMode = 'local' | 'on_premise' | 'cloud';

export type ModelCapability =
  | 'text_generation'
  | 'vision'
  | 'image_input'
  | 'image_output'
  | 'tool_calling'
  | 'function_calling'
  | 'structured_output'
  | 'json_output'
  | 'streaming'
  | 'embedding'
  | 'code_generation';

export type ModelAvailability = 'available' | 'unavailable' | 'unknown' | 'disabled';

export interface ProviderDescriptor {
  id: string;
  name: string;
  provider_type: string;
  execution_mode: ExecutionMode;
  enabled: boolean;
  status: string;
  base_url?: string | null;
}

export interface HardwareRequirement {
  minimum_ram?: number | null;
  recommended_ram?: number | null;
  minimum_vram?: number | null;
  recommended_vram?: number | null;
}

export interface ModelDescriptor {
  id: string;
  provider_id: string;
  model_identifier: string;
  display_name: string;
  execution_mode: ExecutionMode;
  capabilities: ModelCapability[];
  context_window?: number | null;
  max_output_tokens?: number | null;
  tokenizer?: Record<string, unknown> | string | null;
  hardware_requirements?: HardwareRequirement | null;
  availability: ModelAvailability;
  metadata?: Record<string, string>;
}

export interface ModelRequirement {
  execution_mode?: ExecutionMode | null;
  capabilities?: ModelCapability[];
  minimum_context_window?: number | null;
  vision_required?: boolean;
  tool_calling_required?: boolean;
  structured_output_required?: boolean;
}

export interface WorkspaceInfo {
  path: string;
  exists: boolean;
  is_dir: boolean;
  total_files: number;
  name: string;
}

export interface FileEntry {
  name: string;
  path: string;
  rel_path: string;
  is_dir: boolean;
  size_bytes: number;
}

export interface AgentExecuteResponse {
  success: boolean;
  task_id?: string;
  provider_used?: string;
  output?: string;
  execution_time_ms?: number;
  error?: AppErrorResponse | string;
}

export type OperatingSystem = 'windows' | 'linux' | 'mac_os' | string;
export type Architecture = 'x86_64' | 'aarch64' | 'arm' | 'x86' | string;
export type DetectionConfidence = 'exact' | 'high' | 'estimated' | 'unknown';
export type HardwareSource = 'os' | 'cpu_info' | 'system_api' | 'nvidia_smi' | 'metal' | 'wmi' | 'dxgi' | 'proc_fs' | 'sys_fs' | 'mock' | 'unknown';
export type GpuVendor = 'nvidia' | 'amd' | 'intel' | 'apple' | string;
export type AcceleratorCategory = 'discrete_gpu' | 'integrated_gpu' | 'unified_memory' | 'npu' | 'none' | 'unknown';

export interface CpuInfo {
  architecture: Architecture;
  logical_cores: number;
  physical_cores?: number | null;
  vendor?: string | null;
  model_name?: string | null;
  features: string[];
}

export interface MemoryInfo {
  total_bytes: number;
  available_bytes?: number | null;
  used_bytes?: number | null;
  is_unified_memory: boolean;
}

export interface GpuInfo {
  vendor: GpuVendor;
  name: string;
  vram_bytes?: number | null;
  available_vram_bytes?: number | null;
  driver_version?: string | null;
  device_index: number;
  accelerator_category: AcceleratorCategory;
}

export interface HardwareProfile {
  os: OperatingSystem;
  os_detail?: string | null;
  arch: Architecture;
  cpu: CpuInfo;
  memory: MemoryInfo;
  gpus: GpuInfo[];
  detected_at_timestamp: number;
  confidence: DetectionConfidence;
  source: HardwareSource;
}

export interface GpuSnapshot {
  device_index: number;
  name: string;
  vram_used_bytes?: number | null;
  vram_available_bytes?: number | null;
  utilization_percent?: number | null;
  temperature_celsius?: number | null;
}

export interface HardwareSnapshot {
  timestamp: number;
  total_ram_bytes: number;
  available_ram_bytes: number;
  ram_usage_percent: number;
  cpu_usage_percent: number;
  gpu_snapshots: GpuSnapshot[];
}

export type CompatibilityStatus = 'compatible' | 'compatible_with_warnings' | 'incompatible' | 'unknown';

export interface CompatibilityResult {
  model_id: string;
  status: CompatibilityStatus;
  reasons: Record<string, unknown>[];
  warnings: string[];
  estimated_constraints: string[];
  suitable_devices: string[];
  confidence: DetectionConfidence;
  evaluated_at_timestamp: number;
}


export type NetworkMode = 'local_only' | 'restricted' | 'online_allowed' | 'disabled';
export type NetworkProtocol = 'http' | 'https' | 'ws' | 'tcp';
export type NetworkCapability =
  | 'external_https'
  | 'external_http'
  | 'cloud_model_inference'
  | 'web_fetch'
  | 'web_search'
  | 'remote_mcp'
  | 'provider_api'
  | 'local_network'
  | 'loopback';

export interface WorkspaceInfo {
  id: string;
  name: string;
  root_path: string;
  classification: string;
  files_count?: number;
}

export type NetworkPolicySource =
  | 'system_policy'
  | 'company_policy'
  | 'security_policy'
  | 'privacy_policy'
  | 'permission_policy'
  | 'user_policy';

export type Disposition = 'allow' | 'block' | 'require_consent' | 'unknown';

export interface NetworkEndpoint {
  raw_url: string;
  protocol: NetworkProtocol;
  host: string;
  port: number;
  path: string;
  has_query: boolean;
  resolved_ips?: string[];
}

export interface NetworkDecision {
  decision_id: string;
  timestamp: number;
  disposition: Disposition;
  reason: string;
  matched_rule?: string | null;
  policy_source: NetworkPolicySource;
  endpoint: NetworkEndpoint;
  requires_consent: boolean;
  warnings: string[];
}

export interface NetworkRule {
  id: string;
  name: string;
  source: NetworkPolicySource;
  action: Disposition;
  domain_pattern: string;
  protocol?: NetworkProtocol | null;
  port?: number | null;
  capability?: NetworkCapability | null;
  provider_id?: string | null;
  priority: number;
  enabled: boolean;
  description: string;
}

export interface AllowlistEntry {
  id: string;
  domain_pattern: string;
  description: string;
  added_at: number;
  enabled: boolean;
}

export interface NetworkSecurityStatus {
  enabled: boolean;
  mode: NetworkMode;
  policy_version: string;
  active_rules_count: number;
  allowlist_count: number;
  total_decisions_count: number;
  blocked_count: number;
  allowed_count: number;
  consent_required_count: number;
  last_decision?: NetworkDecision | null;
  service_health: string;
}

export interface NetworkConsentRecord {
  id: string;
  request_id: string;
  destination: string;
  granted: boolean;
  timestamp: number;
}

export type PrivacyMode = 'AUTO' | 'LOCAL_ONLY' | 'ONLINE_ONLY' | 'CONFIDENTIAL';
export type PrivacyClassification = 'PUBLIC' | 'INTERNAL' | 'CONFIDENTIAL' | 'RESTRICTED' | 'SECRET';
export type PolicySource = 'SYSTEM' | 'COMPANY' | 'SECURITY' | 'PRIVACY' | 'USER' | 'system_policy' | 'company_policy' | 'security_policy' | 'privacy_policy' | 'permission_policy' | 'user_policy';
export type PolicyAction = 'ALLOW' | 'DENY' | 'REQUIRE_CONSENT' | 'REQUIRE_REVIEW';
export type DecisionState = 'ALLOW_LOCAL' | 'ALLOW_ON_PREMISE' | 'ALLOW_ONLINE' | 'DENY_ONLINE' | 'DENY' | 'REQUIRE_CONSENT' | 'REQUIRE_REVIEW';
export type ClassificationConfidence = 'LOW' | 'MEDIUM' | 'HIGH';

export interface ClassificationSignal {
  category: string;
  signal_type: string;
  summary: string;
  severity: string;
}

export interface PolicyRule {
  id: string;
  name: string;
  source: PolicySource;
  priority: number;
  enabled: boolean;
  action: PolicyAction;
  reason: string;
  version: number;
}

export interface PrivacyDecision {
  id: string;
  request_id: string;
  session_id?: string | null;
  task_id?: string | null;
  workspace_id?: string | null;
  classification: PrivacyClassification;
  requested_execution_mode: ExecutionMode;
  allowed_execution_modes: ExecutionMode[];
  selected_policy_source: PolicySource;
  policy_version: number;
  decision: DecisionState;
  reason: string;
  confidence: ClassificationConfidence;
  timestamp: number;
  evidence_summary: String;
}

export interface OutboundPayloadPreview {
  request_id: string;
  target_provider: string;
  target_model: string;
  execution_mode: ExecutionMode;
  classification: PrivacyClassification;
  data_categories: string[];
  files_included: string[];
  approximate_payload_bytes: number;
  sensitive_fields_detected: string[];
  policy_decision: DecisionState;
}

export interface ConsentRequest {
  id: string;
  request_id: string;
  task_id?: string | null;
  session_id?: string | null;
  workspace_id?: string | null;
  classification: PrivacyClassification;
  destination_provider: string;
  destination_model: string;
  requested_mode: ExecutionMode;
  payload_preview: OutboundPayloadPreview;
  reasoning: string;
  created_at: number;
}

export interface ConsentDecision {
  id: string;
  request_id: string;
  consent_id: string;
  granted: boolean;
  reason?: string | null;
  timestamp: number;
}

export interface EffectivePrivacyStatus {
  privacy_mode: PrivacyMode;
  confidential_mode: boolean;
  active_workspace_id?: string | null;
  active_workspace_classification: PrivacyClassification;
  active_policies_count: number;
  system_policy_status: string;
  company_policy_status: string;
  default_mode: PrivacyMode;
}

// Context Engine Types (Phase 10)
export type ContextSource =
  | 'system_policy'
  | 'company_policy'
  | 'security_policy'
  | 'privacy_policy'
  | 'user_request'
  | 'conversation_message'
  | 'task_state'
  | 'workspace_context'
  | 'file_content'
  | 'tool_result'
  | 'model_output'
  | 'verification_result'
  | 'compaction_summary'
  | 'artifact_reference'
  | 'runtime_instruction';

export type ContextRole =
  | 'system'
  | 'user'
  | 'assistant'
  | 'tool'
  | 'file_content'
  | 'tool_result'
  | 'model_output'
  | 'compaction_summary';

export type ContextTrustLevel =
  | 'trusted_system'
  | 'trusted_company'
  | 'trusted_security'
  | 'trusted_privacy'
  | 'user_controlled'
  | 'model_generated'
  | 'untrusted_file'
  | 'untrusted_tool'
  | 'untrusted_web'
  | 'external_data';

export type TokenCountKind = 'exact' | 'estimated' | 'unknown';

export type ContextInclusionReason =
  | 'mandatory'
  | 'current_task'
  | 'recent'
  | 'relevant'
  | 'dependency'
  | 'verification_required'
  | 'user_selected'
  | 'policy_required';

export interface ContextItemMeta {
  id: string;
  source: ContextSource;
  source_id?: string | null;
  role: ContextRole;
  trust_level: ContextTrustLevel;
  classification: PrivacyClassification;
  priority: number;
  token_count?: number | null;
  token_count_kind: TokenCountKind;
  created_at: number;
  expires_at?: number | null;
  inclusion_reason?: ContextInclusionReason | null;
  safe_summary?: string | null;
}

export interface TokenBudget {
  model_id: string;
  context_window: number;
  reserved_output: number;
  safety_margin: number;
  fixed_overhead: number;
  usable_input_budget: number;
  current_usage: number;
  utilization_percent: number;
  token_count_kind: TokenCountKind;
}

export interface CompactionStatus {
  performed: boolean;
  rounds: number;
  tokens_before: number;
  tokens_after: number;
  tokens_saved: number;
  items_compacted: number;
  classification: PrivacyClassification;
  warnings: string[];
}

export interface ContextDiagnostics {
  warnings: string[];
  token_count_kind: TokenCountKind;
  compaction_status: CompactionStatus;
  selected_items_count: number;
  excluded_items_count: number;
  aggregate_classification: PrivacyClassification;
  is_local_mode: boolean;
}

export interface AssembledContextMeta {
  model_id: string;
  context_window: number;
  input_budget: number;
  reserved_output: number;
  safety_margin: number;
  total_tokens: number;
  token_count_kind: TokenCountKind;
  compaction_performed: boolean;
  items_count: number;
  aggregate_classification: PrivacyClassification;
}

export interface ContextBuildResult {
  session_id: string;
  task_id?: string | null;
  assembled: AssembledContextMeta;
  budget: TokenBudget;
  diagnostics: ContextDiagnostics;
  items: ContextItemMeta[];
}

export interface ContextCheckpoint {
  id: string;
  session_id: string;
  task_id?: string | null;
  model_id: string;
  context_version: number;
  selected_item_ids: string[];
  summary_ids: string[];
  task_state_version: number;
  created_at: number;
}

export interface TaskContextState {
  task_id: string;
  objective: string;
  requirements: string[];
  constraints: string[];
  completed_steps: string[];
  pending_steps: string[];
  blocked_steps: string[];
  decisions: Record<string, string>;
  verification_status: string;
  unresolved_questions: string[];
  context_checkpoint_id?: string | null;
  last_model_call?: number | null;
}

export interface ContextStatusResponse {
  service: string;
  status: string;
  active_sessions: number;
  total_items_tracked: number;
  total_checkpoints: number;
  fail_closed: boolean;
}

// ============================================================================
// PHASE 9 TOOL RUNTIME & MCP SECURITY TYPES
// ============================================================================

export type ToolSource = 'BUILT_IN' | 'SKILL' | 'MCP' | 'PROVIDER' | 'SYSTEM' | 'USER_INSTALLED';
export type RiskLevel = 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL';
export type ToolExecutionStatus = 'VALID' | 'REDACTED' | 'TRUNCATED' | 'INVALID' | 'FAILED' | 'BLOCKED' | 'CANCELLED' | 'TIMED_OUT';
export type ToolDecisionState = 'ALLOW' | 'BLOCK' | 'REQUIRE_CONSENT' | 'UNKNOWN';
export type McpTrustLevel = 'UNTRUSTED' | 'RESTRICTED' | 'TRUSTED';

export type ToolCapability =
  | 'FILESYSTEM_READ'
  | 'FILESYSTEM_WRITE'
  | 'FILESYSTEM_CREATE'
  | 'FILESYSTEM_DELETE'
  | 'FILESYSTEM_LIST'
  | 'FILESYSTEM_EXPORT'
  | 'EXTERNAL_HTTPS'
  | 'EXTERNAL_HTTP'
  | 'PROVIDER_API'
  | 'WEB_FETCH'
  | 'WEB_SEARCH'
  | 'REMOTE_MCP'
  | 'PROCESS_EXECUTE'
  | 'PROCESS_SPAWN'
  | 'WORKSPACE_READ'
  | 'WORKSPACE_WRITE'
  | 'WORKSPACE_EXPORT'
  | 'READ_CONFIDENTIAL_DATA'
  | 'READ_RESTRICTED_DATA'
  | 'READ_SECRET_DATA'
  | 'SYSTEM_INFO'
  | 'ENVIRONMENT_ACCESS';

export interface CapabilityScope {
  workspace_id?: string | null;
  allowed_paths: string[];
  allowed_domains: string[];
  allowed_commands: string[];
}

export interface ToolDescriptor {
  id: string;
  name: string;
  version: string;
  description: string;
  source: ToolSource;
  capabilities: ToolCapability[];
  input_schema: Record<string, unknown>;
  output_schema?: Record<string, unknown> | null;
  risk_level: RiskLevel;
  enabled: boolean;
  requires_confirmation: boolean;
  timeout_ms: number;
  metadata: Record<string, string>;
}

export interface CapabilityGrant {
  grant_id: string;
  tool_id: string;
  task_id?: string | null;
  session_id?: string | null;
  workspace_id?: string | null;
  capabilities: ToolCapability[];
  scope: CapabilityScope;
  expiration?: number | null;
  policy_source: string;
  consent: boolean;
  created_at: number;
}

export interface ToolRequest {
  request_id?: string;
  tool_id: string;
  task_id?: string | null;
  session_id?: string | null;
  workspace_id?: string | null;
  arguments: Record<string, unknown>;
  requested_capabilities: ToolCapability[];
  context?: Record<string, string>;
  parent_tool_call_id?: string | null;
  recursion_depth?: number;
}

export interface ToolDecision {
  decision: ToolDecisionState;
  reason: string;
  matched_policy?: string | null;
  policy_source: string;
  granted_capabilities: ToolCapability[];
  denied_capabilities: ToolCapability[];
  warnings: string[];
  risk_level: RiskLevel;
}

export interface ToolResponse {
  tool_call_id: string;
  tool_id: string;
  status: ToolExecutionStatus;
  result: Record<string, unknown>;
  error?: string | null;
  warnings: string[];
  security_notices: string[];
  duration_ms: number;
}

export interface ToolExecutionActivity {
  id: string;
  timestamp: number;
  event_type: string;
  task_id?: string | null;
  session_id?: string | null;
  workspace_id?: string | null;
  actor: string;
  policy_source?: string | null;
  tool?: string | null;
  success: boolean;
  reason?: string | null;
}

export interface McpServerInfo {
  server_id: string;
  name: string;
  transport: string;
  endpoint: string;
  enabled: boolean;
  trust_level: McpTrustLevel;
  declared_capabilities: ToolCapability[];
  allowed_tools: string[];
  status: string;
  created_at: number;
  updated_at: number;
}

// ============================================================================
// PHASE 8 FILESYSTEM & WORKSPACE SECURITY TYPES
// ============================================================================

export type FilesystemOperation =
  | 'read'
  | 'write'
  | 'create'
  | 'delete'
  | 'rename'
  | 'move'
  | 'list'
  | 'stat'
  | 'create_directory'
  | 'delete_directory'
  | 'copy'
  | 'export'
  | 'import'
  | 'recursive_delete';

export type OperationRisk = 'low' | 'medium' | 'high' | 'critical';

export type FilesystemDisposition = 'allow' | 'block' | 'require_confirmation' | 'unknown';

export type ProtectedPathAction = 'deny' | 'require_confirmation' | 'allow_with_audit';

export interface ProtectedPath {
  id: string;
  pattern: string;
  description: string;
  action: ProtectedPathAction;
  policy_source: string;
  enabled: boolean;
}

export interface FilesystemLimits {
  max_file_size_bytes: number;
  max_directory_depth: number;
  max_read_bytes_per_op: number;
  max_write_bytes_per_op: number;
  max_listed_files: number;
  block_binary_files: boolean;
}

export interface WorkspaceSecurityPolicy {
  workspace_id: string;
  read_only: boolean;
  allow_external_export: boolean;
  block_hidden_files: boolean;
  allowed_file_extensions: string[];
  blocked_file_extensions: string[];
  require_confirmation_for_write: boolean;
  require_confirmation_for_delete: boolean;
  custom_protected_paths: ProtectedPath[];
  limits: FilesystemLimits;
}

export interface FilesystemDecision {
  decision_id: string;
  operation: FilesystemOperation;
  disposition: FilesystemDisposition;
  workspace_id: string;
  requested_path: string;
  resolved_path?: string | null;
  reason: string;
  matched_policy?: string | null;
  policy_source: string;
  classification: PrivacyClassification;
  risk_level: OperationRisk;
  warnings: string[];
  requires_user_confirmation: boolean;
  timestamp: number;
}

export interface WorkspaceSecurityStatus {
  workspace_id: string;
  workspace_name: string;
  root_path: string;
  classification: PrivacyClassification;
  policy: WorkspaceSecurityPolicy;
  active_protected_paths_count: number;
  total_evaluations: number;
  blocked_evaluations: number;
  allowed_evaluations: number;
  service_status: string;
}

// ============================================================================
// PHASE 12 MODEL ROUTER TYPES
// ============================================================================

export type RoutingMode = 'auto' | 'local_only' | 'on_premise_only' | 'online_only' | 'explicit_model';

export type RoutingDecisionState =
  | 'selected'
  | 'require_user_selection'
  | 'blocked'
  | 'no_compatible_model'
  | 'no_authorized_model'
  | 'context_too_large'
  | 'hardware_incompatible'
  | 'provider_unavailable'
  | 'policy_denied'
  | 'unknown';

export interface CandidateEvaluation {
  model_id: string;
  provider_id: string;
  display_name: string;
  execution_mode: ExecutionMode;
  eligible: boolean;
  elimination_reason?: Record<string, unknown> | null;
  score: number;
  details: string;
}

export interface RoutingEvidence {
  candidate_count: number;
  eligible_count: number;
  excluded_candidates: CandidateEvaluation[];
  selected_candidate?: CandidateEvaluation | null;
  privacy_status: string;
  hardware_status: string;
  context_status: string;
  provider_status: string;
  policy_status: string;
}

export interface RoutingDecision {
  id: string;
  request_id: string;
  task_id?: string | null;
  session_id?: string | null;
  workspace_id?: string | null;
  selected_model_id?: string | null;
  selected_provider_id?: string | null;
  selected_descriptor?: ModelDescriptor | null;
  execution_mode?: ExecutionMode | null;
  state: RoutingDecisionState;
  reason: string;
  confidence: number;
  timestamp: number;
  evidence: RoutingEvidence;
  policy_evidence: string[];
  hardware_compatibility?: CompatibilityResult | null;
  context_compatibility?: string | null;
}

export interface RoutingStatusResponse {
  service: string;
  status: string;
  registered_models: number;
  registered_providers: number;
  hardware_confidence: string;
  fail_closed: boolean;
  no_automatic_fallback: boolean;
  privacy_first: boolean;
}

// ============================================================================
// PHASE 14 SKILLS / WORKFLOWS / MEMORY TYPES
// ============================================================================

export type SkillSource =
  | 'SYSTEM'
  | 'COMPANY'
  | 'BUILTIN'
  | 'USER'
  | 'IMPORTED'
  | 'MODEL_GENERATED'
  | 'EXTERNAL'
  | 'UNKNOWN';

export type SkillStatus = 'DRAFT' | 'PENDING_VALIDATION' | 'ACTIVE' | 'DISABLED' | 'BLOCKED';

export type SkillStepKind =
  | 'SKILL'
  | 'TOOL'
  | 'MODEL_OPERATION'
  | 'FILE_OPERATION'
  | 'VERIFICATION'
  | 'USER_INPUT';

export interface SkillStep {
  id: string;
  kind: SkillStepKind;
  reference: string;
  description: string;
  required_capabilities: ToolCapability[];
  inputs?: Record<string, unknown> | null;
  expected_output?: string | null;
}

export interface SkillProvenance {
  created_by: string;
  source: SkillSource;
  imported_from?: string | null;
  approval_status: string;
  validation_status: string;
  tasks_used_in: string[];
  last_updated: number;
}

export interface SkillDefinition {
  id: string;
  name: string;
  description: string;
  version: string;
  owner: string;
  source: SkillSource;
  status: SkillStatus;
  classification: PrivacyClassification;
  capabilities_required: ToolCapability[];
  allowed_tools: string[];
  workflow: SkillStep[];
  input_schema: Record<string, unknown>;
  output_schema: Record<string, unknown>;
  verification_requirements: string[];
  provenance: SkillProvenance;
  created_at: number;
  updated_at: number;
}

export interface SkillSummary {
  id: string;
  name: string;
  description: string;
  version: string;
  versioned_id: string;
  owner: string;
  source: SkillSource;
  status: SkillStatus;
  classification: PrivacyClassification;
  capabilities_required: ToolCapability[];
  allowed_tools: string[];
  verification_requirements: string[];
  provenance: Record<string, unknown>;
  created_at: number;
  updated_at: number;
}

export type WorkflowStepKind =
  | 'SKILL'
  | 'TOOL'
  | 'MODEL_OPERATION'
  | 'FILE_OPERATION'
  | 'VERIFICATION'
  | 'USER_INPUT';

export type WorkflowStatus = 'DRAFT' | 'ACTIVE' | 'DISABLED' | 'BLOCKED';

export interface WorkflowStep {
  id: string;
  kind: WorkflowStepKind;
  reference: string;
  description: string;
  dependencies: string[];
  required_capabilities: ToolCapability[];
  inputs?: Record<string, unknown> | null;
  expected_output?: string | null;
  verification_required: boolean;
}

export interface WorkflowDefinition {
  id: string;
  name: string;
  version: string;
  description: string;
  source: SkillSource;
  status: WorkflowStatus;
  classification: PrivacyClassification;
  inputs: Record<string, unknown>;
  steps: WorkflowStep[];
  outputs: Record<string, unknown>;
  verification: string[];
  policy_requirements: string[];
  created_at: number;
  updated_at: number;
}

export type WorkflowRunStatus = 'PENDING' | 'RUNNING' | 'COMPLETED' | 'FAILED' | 'CANCELLED';

export interface WorkflowRun {
  id: string;
  workflow_id: string;
  workflow_version: string;
  task_id?: string | null;
  session_id?: string | null;
  workspace_id?: string | null;
  status: WorkflowRunStatus;
  inputs: Record<string, unknown>;
  outputs?: Record<string, unknown> | null;
  created_at: number;
  updated_at: number;
}

export type MemoryType =
  | 'USER_PREFERENCE'
  | 'PROJECT_FACT'
  | 'WORKSPACE_FACT'
  | 'TASK_FACT'
  | 'WORKFLOW_FACT'
  | 'SKILL_FACT'
  | 'ARTIFACT_FACT'
  | 'DECISION'
  | 'CONSTRAINT'
  | 'PROCEDURE'
  | 'TEMPORARY_FACT';

export type MemoryScope = 'GLOBAL' | 'WORKSPACE' | 'PROJECT' | 'SESSION' | 'TASK';

export type MemorySource =
  | 'SYSTEM'
  | 'COMPANY'
  | 'USER_EXPLICIT'
  | 'OBSERVED'
  | 'TOOL_DERIVED'
  | 'MODEL_DERIVED'
  | 'IMPORTED'
  | 'UNKNOWN';

export interface MemoryItem {
  id: string;
  mem_type: MemoryType;
  scope: MemoryScope;
  workspace_id?: string | null;
  project_id?: string | null;
  session_id?: string | null;
  task_id?: string | null;
  content: string;
  classification: PrivacyClassification;
  source: MemorySource;
  confidence: number;
  provenance: {
    source: MemorySource;
    actor: string;
    tool_id?: string | null;
    model_id?: string | null;
    imported_from?: string | null;
    evidence: string;
  };
  created_at: number;
  updated_at: number;
  expires_at?: number | null;
  version: number;
}

export interface MemoryCandidate {
  id: string;
  mem_type: MemoryType;
  scope: MemoryScope;
  workspace_id?: string | null;
  session_id?: string | null;
  task_id?: string | null;
  content: string;
  classification: PrivacyClassification;
  source: MemorySource;
  confidence: number;
  provenance: MemoryItem['provenance'];
  status: string;
  created_at: number;
  updated_at: number;
}

export interface MemoryResultRow {
  id: string;
  type: MemoryType;
  scope: MemoryScope;
  classification: PrivacyClassification;
  source: MemorySource;
  confidence: number;
  content_preview: string;
  relevance: number;
  workspace_id?: string | null;
  session_id?: string | null;
  task_id?: string | null;
  expires_at?: number | null;
  version: number;
}

// --- Orchestration Phase 11 Types ---

export type StepActionType =
  | 'ThinkAnalyze'
  | 'RetrieveContext'
  | 'ReadFile'
  | 'WriteFile'
  | 'ExecuteTool'
  | 'ModelCall'
  | 'Verify'
  | 'AskUser'
  | 'Complete';

export type StepStatus =
  | 'Pending'
  | 'InProgress'
  | 'WaitingForUser'
  | 'Verifying'
  | 'Completed'
  | 'Failed'
  | 'Skipped';

export type PlanStatus =
  | 'Draft'
  | 'Approved'
  | 'Executing'
  | 'Completed'
  | 'Failed'
  | 'Cancelled'
  | 'RequiresRevision';

export type OrchestratorState =
  | 'Created'
  | 'Planning'
  | 'PlanReady'
  | 'Executing'
  | 'WaitingForTool'
  | 'WaitingForUser'
  | 'Verifying'
  | 'Retrying'
  | 'Blocked'
  | 'Completed'
  | 'Failed'
  | 'Cancelled';

export interface TaskStep {
  id: string;
  order: number;
  objective: string;
  action_type: StepActionType;
  dependencies: string[];
  status: StepStatus;
  attempts: number;
  max_attempts: number;
  requires_user_input: boolean;
  verification_required: boolean;
  inputs?: Record<string, any> | null;
  outputs?: Record<string, any> | null;
  failure_reason?: string | null;
}

export interface TaskPlan {
  id: string;
  task_id: string;
  objective: string;
  constraints: string[];
  steps: TaskStep[];
  current_step: number;
  status: PlanStatus;
  created_at: number;
  updated_at: number;
  version: number;
}

export interface OrchestrationStatusResponse {
  success: boolean;
  service: string;
  status: string;
  fail_closed: boolean;
  untrusted_model_actions: boolean;
  authoritative_security: boolean;
}

export interface TaskSummary {
  id: string;
  title: string;
  status: string;
  created_at: number;
  updated_at: number;
  workspace_id?: string | null;
  session_id?: string | null;
  error?: string | null;
}

export interface TaskDetailResponse {
  success: boolean;
  task: TaskSummary;
  plan?: TaskPlan | null;
  error?: string;
}

export interface ExecutionDecision {
  type: 'Continue' | 'Retry' | 'AskUser' | 'Block' | 'Complete' | 'Fail';
  reason?: string;
  output?: string;
  next_step?: number;
}

// ============================================================================
// Phase 13: Verification & Reliability Engine
// Structured verification results, evidence references, and completion gate.
// Raw secrets and full file contents are never exposed in these payloads.
// ============================================================================

export type VerificationStatus =
  | 'PASS'
  | 'PASS_WITH_WARNINGS'
  | 'FAIL'
  | 'BLOCKED'
  | 'UNKNOWN'
  | 'NOT_VERIFIED';

export type CheckStatus =
  | 'PASSED'
  | 'WARNING'
  | 'FAILED'
  | 'BLOCKED'
  | 'UNKNOWN'
  | 'SKIPPED';

export type CompletionDecision =
  | 'ALLOW_COMPLETION'
  | 'REQUIRE_REPAIR'
  | 'REQUIRE_USER'
  | 'BLOCKED';

export type VerificationAction =
  | 'Complete'
  | 'Retry'
  | 'Repair'
  | 'Replan'
  | 'AskUser'
  | 'Fail';

export interface VerificationCheck {
  id: string;
  name: string;
  check_type: string;
  status: CheckStatus;
  severity: string;
  expected: string;
  actual: string;
  evidence_ref?: string | null;
  message: string;
}

export interface VerificationResult {
  verification_id: string;
  task_id: string;
  step_id?: string | null;
  status: VerificationStatus;
  checks: VerificationCheck[];
  evidence_refs: string[];
  warnings: string[];
  failures: string[];
  confidence: number;
  repair_attempt: number;
  created_at: number;
}

export interface VerificationRunResponse {
  success: boolean;
  task_id?: string;
  total?: number;
  runs?: VerificationResult[];
  verification?: VerificationResult;
  completion_gate?: string;
  orchestrator_action?: VerificationAction;
  error?: string;
}

export interface TaskCompletionResponse {
  success: boolean;
  task_id: string;
  verified: boolean;
  status: string;
  confidence?: number;
  completion_gate: string;
  orchestrator_action: string;
  verification_id?: string;
  failures: string[];
  warnings?: string[];
  error?: string;
}

export interface RepairAttemptResponse {
  success: boolean;
  task_id?: string;
  verification_id?: string;
  repair_attempt?: number;
  max_repair_attempts?: number;
  latest?: VerificationResult | null;
  message?: string;
  error?: string;
}

// ============================================================================
// Phase 15: Document Intelligence & Artifact Pipeline
// SECRET document content is never exposed raw in these payloads; the backend
// returns redacted previews. Untrusted document text never becomes policy.
// ============================================================================

export type DocumentFormat =
  | 'TXT'
  | 'MARKDOWN'
  | 'JSON'
  | 'CSV'
  | 'XML'
  | 'PDF'
  | 'DOCX'
  | 'XLSX'
  | string;

export type DocumentExtractionStatus =
  | 'PENDING'
  | 'PARSING'
  | 'COMPLETED'
  | 'UNSUPPORTED_FORMAT'
  | 'FAILED';

export interface DocumentParseWarning {
  code: string;
  message: string;
  location?: string | null;
}

export interface DocumentSecurityFinding {
  category: string;
  severity: string;
  summary: string;
  location?: string | null;
}

export interface DocumentSummary {
  id: string;
  rel_path: string;
  file_name: string;
  format: string;
  mime: string;
  size_bytes: number;
  content_hash: string;
  classification: PrivacyClassification;
  extraction_status: DocumentExtractionStatus;
  title?: string | null;
  created_at: number;
}

export interface DocumentImportResult {
  success: boolean;
  document_id?: string;
  version_id?: string;
  title?: string | null;
  format?: string;
  mime?: string;
  size_bytes?: number;
  content_hash?: string;
  classification?: PrivacyClassification;
  extraction_status?: DocumentExtractionStatus;
  blocks?: number;
  sections?: number;
  pages?: number;
  chunks?: number;
  warnings?: DocumentParseWarning[];
  security_findings?: DocumentSecurityFinding[];
  text_preview?: string;
  error?: string;
}

export interface DocumentDetail {
  id: string;
  workspace_id: string;
  rel_path: string;
  file_name: string;
  format: string;
  mime: string;
  size_bytes: number;
  content_hash: string;
  classification: PrivacyClassification;
  extraction_status: DocumentExtractionStatus;
  title?: string | null;
  provenance: Record<string, unknown>;
  warnings: DocumentParseWarning[];
  findings: DocumentSecurityFinding[];
  created_at: number;
}

export interface DocumentChunkView {
  chunk_id: string;
  chunk_index: number;
  classification: PrivacyClassification;
  section?: string | null;
  page?: number | null;
  source: string;
  char_count: number;
  token_estimate: number;
  text: string;
}

export interface DocumentSearchResult {
  chunk_id: string;
  relevance: number;
  classification: PrivacyClassification;
  section?: string | null;
  page?: number | null;
  source: string;
  text: string;
}

export type ArtifactKind =
  | 'GENERATED_DOCUMENT'
  | 'GENERATED_TEXT'
  | 'GENERATED_JSON'
  | 'GENERATED_CSV'
  | 'GENERATED_MARKDOWN'
  | 'GENERATED_CODE'
  | 'GENERATED_REPORT'
  | 'TRANSFORMED_DOCUMENT'
  | 'EXTRACTED_DATASET';

export interface ArtifactSummary {
  id: string;
  task_id?: string | null;
  workspace_id?: string | null;
  name: string;
  path: string;
  artifact_type: string;
  size: number;
  checksum?: string | null;
  created_at: number;
  verification_status: string;
}

export interface ArtifactLineageEntry {
  id: string;
  artifact_id: string;
  parent_artifact_id?: string | null;
  source_document_id?: string | null;
  source_document_version?: string | null;
  producing_workflow?: string | null;
  producing_skill?: string | null;
  producing_model?: string | null;
  producing_provider?: string | null;
  classification: string;
  created_at: number;
}


// Phase 16: Local Runtime & Model Management

export type LocalRuntimeType = 'ollama' | 'llama_cpp_server' | 'local_openai_compatible' | 'other' | string;
export type LocalRuntimeHealth = 'healthy' | 'degraded' | 'unavailable' | 'unknown';
export type LocalExecutionMode = 'process' | 'endpoint' | 'managed';
export type LocalModelState =
  | 'discovered'
  | 'registered'
  | 'available'
  | 'loading'
  | 'loaded'
  | 'running'
  | 'unloading'
  | 'unavailable'
  | 'failed'
  | 'disabled';

export interface LocalRuntimeDescriptor {
  id: string;
  name: string;
  runtime_type: LocalRuntimeType;
  endpoint: string;
  version?: string | null;
  capabilities: ModelCapability[];
  execution_mode: LocalExecutionMode;
  health: LocalRuntimeHealth;
  last_checked_timestamp?: number | null;
  last_error?: string | null;
  metadata?: Record<string, string>;
}

export interface LocalModelRecord {
  id: string;
  runtime_id: string;
  model_identifier: string;
  display_name: string;
  registry_model_id: string;
  state: LocalModelState;
  context_window?: number | null;
  max_output_tokens?: number | null;
  capabilities: ModelCapability[];
  tokenizer: string;
  quantization?: string | null;
  parameter_count_billions?: number | null;
  required_ram_mb?: number | null;
  required_vram_mb?: number | null;
  requirements_estimated: boolean;
  architecture?: string | null;
  model_format?: string | null;
  local_path?: string | null;
  checksum_sha256?: string | null;
  license?: string | null;
  source?: string | null;
  health: LocalRuntimeHealth;
  last_error?: string | null;
  created_at_timestamp: number;
  updated_at_timestamp: number;
  metadata?: Record<string, string>;
}

export interface ResourceEstimate {
  model_id: string;
  estimated_ram_mb?: number | null;
  estimated_vram_mb?: number | null;
  estimated_kv_cache_mb?: number | null;
  runtime_overhead_mb: number;
  is_estimate: boolean;
  confidence: string;
  assumptions: string[];
  warnings: string[];
}

export interface LocalCompatibilityReport {
  model_id: string;
  runtime_id: string;
  hardware: CompatibilityResult;
  context_ok: boolean;
  context_detail: string;
  resource_estimate: ResourceEstimate;
  routable: boolean;
  reasons: string[];
}

export interface LocalRuntimeHealthReport {
  runtime_id: string;
  health: LocalRuntimeHealth;
  reachable: boolean;
  models_available: number;
  models_loaded: number;
  latency_ms?: number | null;
  last_success_timestamp?: number | null;
  last_error?: string | null;
  checked_at_timestamp: number;
}

export interface LocalInferenceMetrics {
  call_id: string;
  model_id: string;
  runtime_id: string;
  time_to_first_token_ms?: number | null;
  total_latency_ms?: number | null;
  tokens_per_second?: number | null;
  prompt_tokens?: number | null;
  output_tokens?: number | null;
  model_load_time_ms?: number | null;
  success: boolean;
  error?: string | null;
}

export interface DownloadSummary {
  installation_id: string;
  destination: string;
  size_bytes: number;
  status: string;
  warnings: string[];
}

export interface LocalRoutingCandidateExplanation {
  model_id: string;
  registry_model_id: string;
  runtime_id: string;
  state: string;
  health: string;
  hardware_status: string;
  routable: boolean;
  reasons: string[];
}


