export const desktopChannels = {
  openProject: 'desktop:project:open',
  recentProject: 'desktop:project:recent',
  runtimeInfo: 'desktop:runtime:info',
  providerStatus: 'desktop:provider:status',
  providerModels: 'desktop:provider:models',
  providerRefreshModels: 'desktop:provider:refresh-models',
  providerInstallLocalModel: 'desktop:provider:install-local-model',
  providerDiagnostics: 'desktop:provider:diagnostics',
  attachmentPick: 'desktop:attachment:pick',
  providerTest: 'desktop:provider:test',
  providerConnect: 'desktop:provider:connect',
  providerDisconnect: 'desktop:provider:disconnect',
  providerOpenExternal: 'desktop:provider:open-external',
  chatStart: 'desktop:chat:start',
  chatCancel: 'desktop:chat:cancel',
  runRevise: 'desktop:run:revise',
  projectRuns: 'desktop:project:runs',
  chatEvent: 'desktop:chat:event',
} as const

export type ChatStreamEvent = {
  type: string
  payload?: any
}

export type DesktopApi = {
  openProject: () => Promise<any>
  getRecentProject: () => Promise<any>
  getRuntimeInfo: () => Promise<any>
  getProviderStatus: () => Promise<any>
  getProviderModels: (providerId: string) => Promise<any>
  refreshProviderModels: () => Promise<any>
  installLocalModel: (modelId: string) => Promise<any>
  getProviderDiagnostics: () => Promise<any>
  pickAttachments: () => Promise<any>
  testProvider: (input: any) => Promise<any>
  connectProvider: (input: any) => Promise<any>
  disconnectProvider: (providerId: string) => Promise<any>
  openExternalProviderLink: (providerId: string, kind: string) => Promise<any>
  startChat: (request: any) => Promise<any>
  cancelChat: (requestId: string) => Promise<any>
  reviseRun: (requestId: string, text: string) => Promise<any>
  getProjectRuns: (projectPath: string) => Promise<any>
  onChatEvent: (listener: (payload: ChatStreamEvent) => void) => () => void
}
