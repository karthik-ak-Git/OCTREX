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
}

export const backendClient = new OctrexBackendClient();
