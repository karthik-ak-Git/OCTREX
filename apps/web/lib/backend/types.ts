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
