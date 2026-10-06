/// <reference types="vite/client" />

interface Window {
  octrex?: {
    openProject: () => Promise<{ path: string; name: string } | null>;
    pickAttachments: () => Promise<string[]>;
    runtimeInfo: () => Promise<{ version: string; os: string }>;
    listRecentProjects: () => Promise<any>;
    getProviderStatus: () => Promise<any>;
    listProviderModels: (providerId: string) => Promise<any>;
    refreshProviderModels: (providerId: string) => Promise<any>;
    installLocalModel: (providerId: string, model: string) => Promise<any>;
    getProviderDiagnostics: (providerId: string) => Promise<any>;
    testProvider: (providerId: string, apiKey: string) => Promise<{ ok: boolean; latencyMs: number }>;
    connectProvider: (providerId: string, config: any) => Promise<{ ok: boolean }>;
    disconnectProvider: (providerId: string) => Promise<{ ok: boolean }>;
    openExternalLink: (providerId: string, kind: string) => Promise<boolean>;
    startChat: (payload: any) => Promise<any>;
    cancelChat: (payload: any) => Promise<any>;
    requestRevision: (payload: any) => Promise<any>;
    listProjectRuns: (projectPath: string) => Promise<any>;
  };
  octrexCore?: {
    invokeCommand: (name: string, req: any) => Promise<any>;
    onEvent: (cb: (event: any) => void) => () => void;
  };
}
