import {
  AgentExecuteResponse,
  ApplicationInfo,
  BackendHealth,
  CompatibilityResult,
  ConfigurationSummary,
  ExecutionMode,
  FileEntry,
  HardwareProfile,
  HardwareSnapshot,
  ModelCapability,
  ModelDescriptor,
  ProviderDescriptor,
  ProviderHealthCheck,
  AllowlistEntry,
  NetworkCapability,
  NetworkConsentRecord,
  NetworkDecision,
  NetworkMode,
  NetworkRule,
  NetworkSecurityStatus,
  RuntimeStatus,
  WorkspaceInfo,
  ContextStatusResponse,
  TokenBudget,
  ContextBuildResult,
  CompactionStatus,
  ContextCheckpoint,
  TaskContextState,
  ToolDescriptor,
  ToolRequest,
  ToolDecision,
  ToolResponse,
  ToolExecutionActivity,
  McpServerInfo,
  RoutingStatusResponse,
  RoutingMode,
  RoutingDecision,
  SkillDefinition,
  SkillSummary,
  WorkflowDefinition,
  WorkflowRun,
  MemoryItem,
  MemoryCandidate,
  MemoryResultRow,
  OrchestrationStatusResponse,
  TaskSummary,
  TaskPlan,
  TaskStep,
  ExecutionDecision,
  VerificationRunResponse,
  TaskCompletionResponse,
  RepairAttemptResponse,
  VerificationResult,
  DocumentSummary,
  DocumentImportResult,
  DocumentDetail,
  DocumentChunkView,
  DocumentSearchResult,
  ArtifactSummary,
  ArtifactLineageEntry,
  LocalRuntimeDescriptor,
  LocalModelRecord,
  LocalCompatibilityReport,
  LocalRuntimeHealthReport,
  LocalInferenceMetrics,
  DownloadSummary,
  LocalRoutingCandidateExplanation,
} from './types';

export class OctrexBackendClient {
  private baseUrl: string;

  constructor(baseUrl = '') {
    this.baseUrl = baseUrl;
  }

  async getApplicationInfo(): Promise<ApplicationInfo> {
    const res = await fetch(`${this.baseUrl}/api/info`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getRuntimeStatus(): Promise<RuntimeStatus> {
    const res = await fetch(`${this.baseUrl}/api/status`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getBackendHealth(): Promise<BackendHealth> {
    const res = await fetch(`${this.baseUrl}/api/health`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getConfigurationSummary(): Promise<ConfigurationSummary> {
    const res = await fetch(`${this.baseUrl}/api/config/summary`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getProviders(): Promise<{ providers: ProviderHealthCheck[]; descriptors?: ProviderDescriptor[] }> {
    const res = await fetch(`${this.baseUrl}/api/providers`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getProviderById(id: string): Promise<{ success: boolean; provider?: ProviderDescriptor; capabilities?: ModelCapability[]; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/providers/${id}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getModels(params?: { execution_mode?: ExecutionMode; capability?: ModelCapability }): Promise<{ success: boolean; total: number; models: ModelDescriptor[] }> {
    const query = new URLSearchParams();
    if (params?.execution_mode) query.append('execution_mode', params.execution_mode);
    if (params?.capability) query.append('capability', params.capability);
    const url = `${this.baseUrl}/api/models${query.toString() ? `?${query.toString()}` : ''}`;
    const res = await fetch(url);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getModelById(id: string): Promise<{ success: boolean; model?: ModelDescriptor; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/models/${encodeURIComponent(id)}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async connectProvider(providerId: string, apiKey: string): Promise<{ success: boolean; health: ProviderHealthCheck }> {
    const res = await fetch(`${this.baseUrl}/api/providers/${providerId}/connect`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ api_key: apiKey }),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async inspectWorkspace(path: string): Promise<{ workspace_id: string; info: WorkspaceInfo }> {
    const res = await fetch(`${this.baseUrl}/api/workspace/inspect`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ path }),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getWorkspaceTree(path: string): Promise<{ success: boolean; entries: FileEntry[] }> {
    const res = await fetch(`${this.baseUrl}/api/workspace/tree`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ path }),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async readFile(workspacePath: string, relPath: string): Promise<{ success: boolean; content?: string; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/workspace/read_file`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ workspace_path: workspacePath, rel_path: relPath }),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async writeFile(
    workspacePath: string,
    relPath: string,
    content: string
  ): Promise<{ success: boolean; bytes_written?: number; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/workspace/write_file`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ workspace_path: workspacePath, rel_path: relPath, content }),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async executeAgent(
    prompt: string,
    providerId?: string,
    workspacePath?: string
  ): Promise<AgentExecuteResponse> {
    const res = await fetch(`${this.baseUrl}/api/agent/execute`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ prompt, provider_id: providerId, workspace_path: workspacePath }),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getHardware(): Promise<{ success: boolean; profile?: HardwareProfile; snapshot?: HardwareSnapshot; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/hardware`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async refreshHardware(): Promise<{ success: boolean; profile?: HardwareProfile; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/hardware/refresh`, { method: 'POST' });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getHardwareCompatibility(modelId?: string): Promise<{ success: boolean; model_id?: string; compatibility?: CompatibilityResult; compatibilities?: CompatibilityResult[]; total?: number; error?: string }> {
    const url = modelId
      ? `${this.baseUrl}/api/hardware/compatibility?model_id=${encodeURIComponent(modelId)}`
      : `${this.baseUrl}/api/hardware/compatibility`;
    const res = await fetch(url);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getPrivacyStatus(): Promise<any> {
    const res = await fetch(`${this.baseUrl}/api/privacy/status`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getPrivacySettings(): Promise<{ privacy_mode: string; confidential_mode: boolean; consent_behavior: string; enforcement_level: string }> {
    const res = await fetch(`${this.baseUrl}/api/privacy/settings`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async updatePrivacySettings(settings: { privacy_mode?: string; confidential_mode?: boolean }): Promise<{ success: boolean; privacy_mode: string; confidential_mode: boolean }> {
    const res = await fetch(`${this.baseUrl}/api/privacy/settings`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(settings),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getPrivacyPolicies(): Promise<{ version: number; rules_count: number; rules: any[] }> {
    const res = await fetch(`${this.baseUrl}/api/privacy/policies`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async evaluatePrivacy(payload: { prompt: string; requested_mode?: string; provider_id?: string; model_id?: string; workspace_classification?: string }): Promise<{ success: boolean; decision: any }> {
    const res = await fetch(`${this.baseUrl}/api/privacy/evaluate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async submitPrivacyConsent(payload: { consent_id: string; granted: boolean; reason?: string }): Promise<{ success: boolean; consent_decision?: any; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/privacy/consent`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getNetworkStatus(): Promise<NetworkSecurityStatus> {
    const res = await fetch(`${this.baseUrl}/api/network/status`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getNetworkPolicy(): Promise<{ success: boolean; mode: NetworkMode; policy_hierarchy: string[]; fail_closed: boolean; active_rules: NetworkRule[]; allowlist: { entries: AllowlistEntry[] } }> {
    const res = await fetch(`${this.baseUrl}/api/network/policy`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getNetworkRules(): Promise<{ success: boolean; total: number; rules: NetworkRule[] }> {
    const res = await fetch(`${this.baseUrl}/api/network/rules`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async evaluateNetworkRequest(payload: { destination: string; capability?: NetworkCapability; source?: string; method?: string }): Promise<{ success: boolean; decision: NetworkDecision }> {
    const res = await fetch(`${this.baseUrl}/api/network/evaluate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getNetworkDecisions(): Promise<{ success: boolean; total: number; decisions: NetworkDecision[] }> {
    const res = await fetch(`${this.baseUrl}/api/network/decisions`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getNetworkDecisionById(id: string): Promise<{ success: boolean; decision?: NetworkDecision; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/network/decisions/${encodeURIComponent(id)}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async requestNetworkConsent(requestId: string, destination: string, granted: boolean): Promise<{ success: boolean; consent: NetworkConsentRecord }> {
    const res = await fetch(`${this.baseUrl}/api/network/consent`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ request_id: requestId, destination, granted }),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async addNetworkAllowlist(domainPattern: string, description?: string): Promise<{ success: boolean; message: string }> {
    const res = await fetch(`${this.baseUrl}/api/network/allowlist`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ domain_pattern: domainPattern, description }),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async deleteNetworkAllowlist(id: string): Promise<{ success: boolean; message: string }> {
    const res = await fetch(`${this.baseUrl}/api/network/allowlist/${encodeURIComponent(id)}`, {
      method: 'DELETE',
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async setNetworkMode(mode: NetworkMode): Promise<{ success: boolean; mode: NetworkMode; status: NetworkSecurityStatus }> {
    const res = await fetch(`${this.baseUrl}/api/network/mode`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ mode }),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  // Context Engine (Phase 10)
  async getContextStatus(): Promise<ContextStatusResponse> {
    const res = await fetch(`${this.baseUrl}/api/context/status`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getContextBudget(sessionId: string): Promise<{ success: boolean; budget?: TokenBudget; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/context/budget/${encodeURIComponent(sessionId)}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getContextForSession(sessionId: string): Promise<{ success: boolean; result?: ContextBuildResult; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/context/${encodeURIComponent(sessionId)}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async previewContext(payload: {
    session_id: string;
    task_id?: string;
    model_id: string;
    user_request?: string;
  }): Promise<{ success: boolean; preview?: ContextBuildResult; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/context/preview`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async compactContext(payload: {
    session_id: string;
    task_id?: string;
    model_id: string;
  }): Promise<{ success: boolean; compaction?: CompactionStatus; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/context/compact`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getContextCheckpoints(sessionId: string): Promise<{ success: boolean; checkpoints?: ContextCheckpoint[]; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/context/checkpoints/${encodeURIComponent(sessionId)}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getTaskContext(taskId: string): Promise<{ success: boolean; task_context?: TaskContextState; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/tasks/${encodeURIComponent(taskId)}/context`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  // Phase 9 Tool Runtime & MCP Security API Client Methods
  async getTools(): Promise<{ success: boolean; tools: ToolDescriptor[]; count: number }> {
    const res = await fetch(`${this.baseUrl}/api/tools`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getTool(id: string): Promise<{ success: boolean; tool?: ToolDescriptor; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/tools/${encodeURIComponent(id)}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async enableTool(id: string): Promise<{ success: boolean; message?: string; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/tools/${encodeURIComponent(id)}/enable`, { method: 'POST' });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async disableTool(id: string): Promise<{ success: boolean; message?: string; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/tools/${encodeURIComponent(id)}/disable`, { method: 'POST' });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async evaluateTool(request: ToolRequest): Promise<{ success: boolean; decision: ToolDecision }> {
    const res = await fetch(`${this.baseUrl}/api/tools/evaluate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(request),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async executeTool(request: ToolRequest): Promise<{ success: boolean; response: ToolResponse }> {
    const res = await fetch(`${this.baseUrl}/api/tools/execute`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(request),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getToolActivity(): Promise<{ success: boolean; activity: ToolExecutionActivity[]; count: number }> {
    const res = await fetch(`${this.baseUrl}/api/tools/activity`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getMcpConnections(): Promise<{ success: boolean; servers: McpServerInfo[]; count: number }> {
    const res = await fetch(`${this.baseUrl}/api/mcp`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getMcpConnection(id: string): Promise<{ success: boolean; server?: McpServerInfo; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/mcp/${encodeURIComponent(id)}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async connectMcp(server: Partial<McpServerInfo>): Promise<{ success: boolean; server: McpServerInfo }> {
    const res = await fetch(`${this.baseUrl}/api/mcp/connect`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(server),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async enableMcp(id: string): Promise<{ success: boolean }> {
    const res = await fetch(`${this.baseUrl}/api/mcp/${encodeURIComponent(id)}/enable`, { method: 'POST' });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async disableMcp(id: string): Promise<{ success: boolean }> {
    const res = await fetch(`${this.baseUrl}/api/mcp/${encodeURIComponent(id)}/disable`, { method: 'POST' });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async evaluateMcpTool(id: string, request: ToolRequest): Promise<{ success: boolean; decision?: ToolDecision; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/mcp/${encodeURIComponent(id)}/evaluate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(request),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  // Filesystem Security (Phase 8)
  async getFilesystemSecurity(): Promise<{ success: boolean; active_workspaces: number; service_status: string }> {
    const res = await fetch(`${this.baseUrl}/api/filesystem/security`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async evaluateFilesystemOperation(payload: {
    workspace_id: string;
    operation: string;
    path: string;
  }): Promise<{ success: boolean; decision: any }> {
    const res = await fetch(`${this.baseUrl}/api/filesystem/evaluate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getWorkspaceSecurity(workspaceId: string): Promise<{ success: boolean; status?: any; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/workspaces/${encodeURIComponent(workspaceId)}/security`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async updateWorkspacePermissions(workspaceId: string, policy: Record<string, unknown>): Promise<{ success: boolean; policy?: any; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/workspaces/${encodeURIComponent(workspaceId)}/permissions`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(policy),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getProtectedPaths(workspaceId: string): Promise<{ success: boolean; protected_paths?: any[]; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/workspaces/${encodeURIComponent(workspaceId)}/protected-paths`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async confirmFilesystemOperation(workspaceId: string, decisionId: string, confirmed: boolean): Promise<{ success: boolean; confirmed: boolean; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/workspaces/${encodeURIComponent(workspaceId)}/confirm`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ decision_id: decisionId, confirmed }),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async exportFile(workspaceId: string, relativePath: string, destinationPath: string): Promise<{ success: boolean; decision?: any; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/filesystem/export`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ workspace_id: workspaceId, relative_path: relativePath, destination_path: destinationPath }),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  // Model Router APIs (Phase 12)
  async getRouterStatus(): Promise<RoutingStatusResponse> {
    const res = await fetch(`${this.baseUrl}/api/router/status`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async evaluateRouting(payload: {
    purpose?: string;
    routing_mode?: RoutingMode;
    user_selected_model?: string;
    workspace_id?: string;
    required_context_tokens?: number;
    required_output_tokens?: number;
  }): Promise<{ success: boolean; decision: RoutingDecision; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/router/evaluate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async previewRouting(payload: {
    purpose?: string;
    routing_mode?: RoutingMode;
    user_selected_model?: string;
    workspace_id?: string;
    required_context_tokens?: number;
    required_output_tokens?: number;
  }): Promise<{ success: boolean; decision: RoutingDecision; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/router/preview`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getCompatibleModels(): Promise<{ success: boolean; total: number; models: ModelDescriptor[]; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/models/compatible`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getRecommendedModels(purpose?: string): Promise<{ success: boolean; total: number; models: ModelDescriptor[]; error?: string }> {
    const url = purpose ? `${this.baseUrl}/api/models/recommended?purpose=${encodeURIComponent(purpose)}` : `${this.baseUrl}/api/models/recommended`;
    const res = await fetch(url);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  // Phase 14 Skills APIs
  async listSkills(): Promise<{ success: boolean; total: number; skills: SkillSummary[] }> {
    const res = await fetch(`${this.baseUrl}/api/skills`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getSkill(id: string): Promise<{ success: boolean; skill?: SkillDefinition; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/skills/${encodeURIComponent(id)}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async createSkill(def: Partial<SkillDefinition>): Promise<{ success: boolean; skill?: SkillDefinition; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/skills`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(def),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async enableSkill(id: string): Promise<{ success: boolean; skill?: SkillDefinition; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/skills/${encodeURIComponent(id)}/enable`, { method: 'POST' });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async disableSkill(id: string): Promise<{ success: boolean; skill?: SkillDefinition; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/skills/${encodeURIComponent(id)}/disable`, { method: 'POST' });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async validateSkill(id: string): Promise<{ success: boolean; valid: boolean; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/skills/${encodeURIComponent(id)}/validate`, { method: 'POST' });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async approveSkill(id: string, approver?: string): Promise<{ success: boolean; skill?: SkillDefinition; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/skills/${encodeURIComponent(id)}/approve`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ approver }),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async matchSkills(payload: { user_request: string; task_type?: string; workspace_id?: string; limit?: number }): Promise<{ success: boolean; matches: Array<{ skill_id: string; version: string; score: number; reasons: string[] }> }> {
    const res = await fetch(`${this.baseUrl}/api/skills/match`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  // Phase 14 Workflows APIs
  async listWorkflows(): Promise<{ success: boolean; total: number; workflows: WorkflowDefinition[] }> {
    const res = await fetch(`${this.baseUrl}/api/workflows`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getWorkflow(id: string): Promise<{ success: boolean; workflow?: WorkflowDefinition; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/workflows/${encodeURIComponent(id)}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async createWorkflow(def: Partial<WorkflowDefinition>): Promise<{ success: boolean; workflow?: WorkflowDefinition; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/workflows`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(def),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async validateWorkflow(id: string): Promise<{ success: boolean; valid: boolean; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/workflows/${encodeURIComponent(id)}/validate`, { method: 'POST' });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async runWorkflow(id: string, payload?: { workflow_version?: string; task_id?: string; session_id?: string; workspace_id?: string; inputs?: Record<string, unknown> }): Promise<{ success: boolean; run?: WorkflowRun; plan?: unknown; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/workflows/${encodeURIComponent(id)}/run`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload ?? {}),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  // Phase 14 Memory APIs
  async queryMemory(payload: Record<string, unknown>): Promise<{ success: boolean; total: number; results: MemoryResultRow[]; error?: string }> {
    const params = new URLSearchParams();
    for (const [k, v] of Object.entries(payload || {})) {
      if (v === undefined || v === null) continue;
      params.append(k, String(v));
    }
    const url = params.toString() ? `${this.baseUrl}/api/memory?${params.toString()}` : `${this.baseUrl}/api/memory`;
    const res = await fetch(url);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async queryMemoryPost(payload: Record<string, unknown>): Promise<{ success: boolean; total: number; results: MemoryResultRow[]; error?: string }> {
    return this.queryMemory(payload);
  }

  async createMemory(item: Partial<MemoryItem>): Promise<{ success: boolean; memory?: MemoryItem; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/memory`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(item),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async deleteMemory(id: string): Promise<{ success: boolean; deleted?: boolean; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/memory/${encodeURIComponent(id)}`, { method: 'DELETE' });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async listMemoryCandidates(status?: string): Promise<{ success: boolean; total: number; candidates: MemoryCandidate[] }> {
    const url = status ? `${this.baseUrl}/api/memory/candidates?status=${encodeURIComponent(status)}` : `${this.baseUrl}/api/memory/candidates`;
    const res = await fetch(url);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async approveMemoryCandidate(id: string): Promise<{ success: boolean; memory?: MemoryItem; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/memory/candidates/${encodeURIComponent(id)}/approve`, { method: 'POST' });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async rejectMemoryCandidate(id: string): Promise<{ success: boolean; candidate?: MemoryCandidate; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/memory/candidates/${encodeURIComponent(id)}/reject`, { method: 'POST' });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  // --- Phase 11 Task Orchestration APIs ---

  async getOrchestrationStatus(): Promise<OrchestrationStatusResponse> {
    const res = await fetch(`${this.baseUrl}/api/orchestration/status`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getTasks(): Promise<{ success: boolean; total: number; tasks: TaskSummary[] }> {
    const res = await fetch(`${this.baseUrl}/api/tasks`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async createTask(objective: string, session_id?: string, workspace_id?: string): Promise<{ success: boolean; task: TaskSummary; plan?: TaskPlan; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/tasks`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ objective, session_id, workspace_id }),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getTaskById(id: string): Promise<{ success: boolean; task?: TaskSummary; plan?: TaskPlan; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/tasks/${encodeURIComponent(id)}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async startTask(id: string, workspace_path?: string): Promise<{ success: boolean; task_id: string; status: string; message: string }> {
    const query = workspace_path ? `?workspace_path=${encodeURIComponent(workspace_path)}` : '';
    const res = await fetch(`${this.baseUrl}/api/tasks/${encodeURIComponent(id)}/start${query}`, { method: 'POST' });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async pauseTask(id: string): Promise<{ success: boolean; task_id?: string; status?: string; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/tasks/${encodeURIComponent(id)}/pause`, { method: 'POST' });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async resumeTask(id: string, workspace_path?: string): Promise<{ success: boolean; task_id: string; status: string; message: string }> {
    const query = workspace_path ? `?workspace_path=${encodeURIComponent(workspace_path)}` : '';
    const res = await fetch(`${this.baseUrl}/api/tasks/${encodeURIComponent(id)}/resume${query}`, { method: 'POST' });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async cancelTask(id: string): Promise<{ success: boolean; task_id?: string; status?: string; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/tasks/${encodeURIComponent(id)}/cancel`, { method: 'POST' });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getTaskPlan(id: string): Promise<{ success: boolean; plan?: TaskPlan; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/tasks/${encodeURIComponent(id)}/plan`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async replanTask(id: string, revised_objective: string): Promise<{ success: boolean; plan?: TaskPlan; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/tasks/${encodeURIComponent(id)}/replan`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ revised_objective }),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getTaskSteps(id: string): Promise<{ success: boolean; steps: TaskStep[]; current_step?: number; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/tasks/${encodeURIComponent(id)}/steps`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async submitUserInput(id: string, input: string, workspace_path?: string): Promise<{ success: boolean; task_id?: string; decision?: ExecutionDecision; message?: string; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/tasks/${encodeURIComponent(id)}/input`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ input, workspace_path }),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  // Phase 13: Verification & Reliability Engine
  async listTaskVerifications(id: string): Promise<VerificationRunResponse> {
    const res = await fetch(`${this.baseUrl}/api/tasks/${encodeURIComponent(id)}/verification`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async verifyTask(id: string, request: Record<string, unknown>): Promise<VerificationRunResponse> {
    const res = await fetch(`${this.baseUrl}/api/tasks/${encodeURIComponent(id)}/verify`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ task_id: id, ...request }),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getTaskVerification(id: string, verificationId: string): Promise<VerificationRunResponse> {
    const res = await fetch(`${this.baseUrl}/api/tasks/${encodeURIComponent(id)}/verification/${encodeURIComponent(verificationId)}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async retryTaskVerification(id: string, verificationId: string): Promise<RepairAttemptResponse> {
    const res = await fetch(`${this.baseUrl}/api/tasks/${encodeURIComponent(id)}/verification/${encodeURIComponent(verificationId)}/retry`, {
      method: 'POST',
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getTaskCompletion(id: string): Promise<TaskCompletionResponse> {
    const res = await fetch(`${this.baseUrl}/api/tasks/${encodeURIComponent(id)}/completion`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  // Phase 16: Local Runtime & Model Management

  async listLocalRuntimes(): Promise<{ success: boolean; total: number; runtimes: LocalRuntimeDescriptor[] }> {
    const res = await fetch(`${this.baseUrl}/api/local-runtimes`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async discoverLocalRuntimes(payload?: { include_defaults?: boolean; endpoints?: Array<{ endpoint: string; runtime_type?: string; name?: string }> }): Promise<{ success: boolean; total: number; runtimes: LocalRuntimeDescriptor[] }> {
    const res = await fetch(`${this.baseUrl}/api/local-runtimes/discover`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload ?? { include_defaults: true }),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async registerLocalRuntime(payload: { runtime_type: string; endpoint: string; name?: string }): Promise<{ success: boolean; runtime?: LocalRuntimeDescriptor; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/local-runtimes/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getLocalRuntime(id: string): Promise<{ success: boolean; runtime?: LocalRuntimeDescriptor; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/local-runtimes/${encodeURIComponent(id)}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async refreshLocalRuntime(id: string): Promise<{ success: boolean; runtime?: LocalRuntimeDescriptor; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/local-runtimes/${encodeURIComponent(id)}/refresh`, { method: 'POST' });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async testLocalRuntime(id: string): Promise<{ success: boolean; health?: LocalRuntimeHealthReport; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/local-runtimes/${encodeURIComponent(id)}/test`, { method: 'POST' });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async startLocalRuntime(id: string): Promise<{ success: boolean; runtime?: LocalRuntimeDescriptor; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/local-runtimes/${encodeURIComponent(id)}/start`, { method: 'POST' });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async stopLocalRuntime(id: string): Promise<{ success: boolean; runtime?: LocalRuntimeDescriptor; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/local-runtimes/${encodeURIComponent(id)}/stop`, { method: 'POST' });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async listLocalModels(): Promise<{ success: boolean; total: number; models: LocalModelRecord[] }> {
    const res = await fetch(`${this.baseUrl}/api/local-models`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async discoverLocalModels(runtimeId: string): Promise<{ success: boolean; total: number; models: LocalModelRecord[]; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/local-models/discover`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ runtime_id: runtimeId }),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async registerLocalModel(payload: Record<string, unknown>): Promise<{ success: boolean; model?: LocalModelRecord; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/local-models/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getLocalModel(id: string): Promise<{ success: boolean; model?: LocalModelRecord; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/local-models/${encodeURIComponent(id)}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async enableLocalModel(id: string): Promise<{ success: boolean; model?: LocalModelRecord; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/local-models/${encodeURIComponent(id)}/enable`, { method: 'POST' });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async disableLocalModel(id: string): Promise<{ success: boolean; model?: LocalModelRecord; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/local-models/${encodeURIComponent(id)}/disable`, { method: 'POST' });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getLocalModelCompatibility(id: string, inputTokens = 1024, outputTokens?: number): Promise<{ success: boolean; compatibility?: LocalCompatibilityReport; error?: string }> {
    const params = new URLSearchParams({ input_tokens: String(inputTokens) });
    if (outputTokens !== undefined) params.append('output_tokens', String(outputTokens));
    const res = await fetch(`${this.baseUrl}/api/local-models/${encodeURIComponent(id)}/compatibility?${params.toString()}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async explainLocalRouting(inputTokens = 1024, outputTokens?: number): Promise<{ success: boolean; total: number; candidates: LocalRoutingCandidateExplanation[] }> {
    const params = new URLSearchParams({ input_tokens: String(inputTokens) });
    if (outputTokens !== undefined) params.append('output_tokens', String(outputTokens));
    const res = await fetch(`${this.baseUrl}/api/local-models/routing/explain?${params.toString()}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async previewLocalInference(payload: { registry_model_id: string; input_tokens?: number; output_tokens?: number; classification?: string }): Promise<{ success: boolean; preview?: Record<string, unknown>; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/local-inference/preview`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async executeLocalInference(payload: { registry_model_id: string; messages: Array<{ role: string; content: string }>; system_instructions?: string; max_output_tokens?: number; temperature?: number; classification?: string }): Promise<{ success: boolean; response?: { content: string; model_id: string }; error?: string; no_cloud_fallback?: boolean }> {
    const res = await fetch(`${this.baseUrl}/api/local-inference/execute`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async cancelLocalInference(callId: string): Promise<{ success: boolean; cancelled: boolean }> {
    const res = await fetch(`${this.baseUrl}/api/local-inference/cancel`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ call_id: callId }),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async listLocalInferenceMetrics(): Promise<{ success: boolean; total: number; metrics: LocalInferenceMetrics[]; active_calls: string[]; slot_occupants: string[] }> {
    const res = await fetch(`${this.baseUrl}/api/local-inference/metrics`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getLocalInferenceMetric(callId: string): Promise<{ success: boolean; metric?: LocalInferenceMetrics; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/local-inference/metrics/${encodeURIComponent(callId)}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async downloadLocalModel(payload: { source: string; url: string; file_name: string; runtime_id?: string; model_id?: string; expected_bytes?: number; checksum_sha256?: string; consent: boolean }): Promise<{ success: boolean; installation?: DownloadSummary; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/local-models/download`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getLocalStorage(): Promise<{ success: boolean; model_dir: string; exists: boolean }> {
    const res = await fetch(`${this.baseUrl}/api/local-storage`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  // --- Phase 15: Document Intelligence & Artifact Pipeline ---

  async listDocuments(workspaceId: string): Promise<{ success: boolean; total: number; documents: DocumentSummary[]; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/documents?workspace_id=${encodeURIComponent(workspaceId)}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async importDocument(payload: { workspace_id: string; rel_path: string; session_id?: string; task_id?: string }): Promise<DocumentImportResult> {
    const res = await fetch(`${this.baseUrl}/api/documents/import`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getDocument(id: string, workspaceId: string): Promise<{ success: boolean; document?: DocumentDetail; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/documents/${encodeURIComponent(id)}?workspace_id=${encodeURIComponent(workspaceId)}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getDocumentSections(id: string, workspaceId: string): Promise<{ success: boolean; sections?: Array<{ section_id: string }>; pages?: number[]; chunks?: number; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/documents/${encodeURIComponent(id)}/sections?workspace_id=${encodeURIComponent(workspaceId)}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getDocumentChunks(id: string, workspaceId: string): Promise<{ success: boolean; total: number; chunks: DocumentChunkView[]; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/documents/${encodeURIComponent(id)}/chunks?workspace_id=${encodeURIComponent(workspaceId)}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async searchDocument(id: string, payload: { workspace_id: string; query: string; ceiling?: string; limit?: number }): Promise<{ success: boolean; total: number; results: DocumentSearchResult[]; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/documents/${encodeURIComponent(id)}/search`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async ingestDocument(id: string, payload: { workspace_id: string; session_id: string; task_id?: string; model_id?: string; context_window?: number }): Promise<{ success: boolean; items_ingested?: number; safe_summaries?: string[]; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/documents/${encodeURIComponent(id)}/ingest`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async assistDocument(payload: { workspace_id: string; document_id: string; operation?: string; session_id?: string; task_id?: string }): Promise<{ success: boolean; assist?: Record<string, unknown>; error?: string; no_cloud_fallback?: boolean }> {
    const res = await fetch(`${this.baseUrl}/api/documents/assist`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async listArtifacts(params: { workspace_id?: string; task_id?: string }): Promise<{ success: boolean; total: number; artifacts: ArtifactSummary[]; error?: string }> {
    const query = new URLSearchParams();
    if (params.workspace_id) query.append('workspace_id', params.workspace_id);
    if (params.task_id) query.append('task_id', params.task_id);
    const res = await fetch(`${this.baseUrl}/api/artifacts?${query.toString()}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getArtifact(id: string): Promise<{ success: boolean; artifact?: ArtifactSummary; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/artifacts/${encodeURIComponent(id)}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getArtifactLineage(id: string): Promise<{ success: boolean; lineage: ArtifactLineageEntry[]; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/artifacts/${encodeURIComponent(id)}/lineage`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async getArtifactVerification(id: string): Promise<{ success: boolean; verification_status?: string; checksum?: string | null; lineage?: ArtifactLineageEntry[]; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/artifacts/${encodeURIComponent(id)}/verification`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async createArtifact(payload: { workspace_id: string; task_id?: string; session_id?: string; rel_path: string; name: string; content: string; artifact_type?: string; source_document_id?: string; parent_artifact_id?: string; producing_workflow?: string; producing_skill?: string }): Promise<{ success: boolean; artifact?: ArtifactSummary; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/artifacts`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async verifyArtifact(id: string, payload: { workspace_id: string; task_id: string; session_id?: string }): Promise<{ success: boolean; verification?: VerificationResult; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/artifacts/${encodeURIComponent(id)}/verify`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  async exportArtifact(id: string, payload: { workspace_id: string; dest_external_path: string }): Promise<{ success: boolean; error?: string }> {
    const res = await fetch(`${this.baseUrl}/api/artifacts/${encodeURIComponent(id)}/export`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }
}

export const backendClient = new OctrexBackendClient();
