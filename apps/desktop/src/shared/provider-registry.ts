export type ProviderId =
  | 'openai'
  | 'google'
  | 'cerebras'
  | 'cloudflare'
  | 'ollama'
  | 'sambanova'
  | 'groq'
  | 'openrouter'
  | 'nvidia'
  | 'nim-local'
  | 'crax-gpt'
  | 'custom'

export type ProviderSection = 'recommended' | 'local' | 'additional' | 'advanced'
export type ProviderLinkKind = 'apiKey' | 'accountId' | 'install' | 'docs'
export type ProviderTestStrategy = 'openai-compatible' | 'cloudflare-workers-ai' | 'ollama-local'

export type ProviderFieldDefinition = {
  id: 'apiKey' | 'accountId'
  label: string
  placeholder: string
  secret: boolean
  helper?: string
}

export type ProviderDefinition = {
  id: ProviderId
  name: string
  description: string
  logo: string
  section: ProviderSection
  apiKeyUrl: string | null
  accountIdUrl?: string
  installUrl?: string
  docsUrl: string
  requiresApiKey: boolean
  requiredFields: readonly ProviderFieldDefinition[]
  recommended: boolean
  supportsLocal: boolean
  testStrategy: ProviderTestStrategy
  baseUrl: string
  defaultModel: string
  approvedHosts: readonly string[]
  userBaseUrl?: boolean
}

const apiKey = (label = 'API key'): ProviderFieldDefinition => ({
  id: 'apiKey',
  label,
  placeholder: `Paste ${label.toLowerCase()}`,
  secret: true,
})

export const PROVIDER_REGISTRY: Readonly<Record<ProviderId, ProviderDefinition>> = {
  google: {
    id: 'google',
    name: 'Google Gemini',
    logo: 'G',
    section: 'recommended',
    recommended: true,
    supportsLocal: false,
    description: 'Powerful general, coding and reasoning models (Gemini 2.5 Flash & Pro).',
    apiKeyUrl: 'https://aistudio.google.com/apikey',
    docsUrl: 'https://ai.google.dev/gemini-api/docs/api-key',
    requiresApiKey: true,
    requiredFields: [apiKey()],
    testStrategy: 'openai-compatible',
    baseUrl: 'https://generativelanguage.googleapis.com/v1beta/openai',
    defaultModel: 'gemini-2.5-flash',
    approvedHosts: ['aistudio.google.com', 'ai.google.dev'],
  },
  nvidia: {
    id: 'nvidia',
    name: 'NVIDIA NIM',
    logo: 'N',
    section: 'recommended',
    recommended: true,
    supportsLocal: false,
    description: 'High-throughput enterprise cloud inference with Llama 3.3, Qwen 2.5, DeepSeek.',
    apiKeyUrl: 'https://build.nvidia.com/',
    docsUrl: 'https://docs.api.nvidia.com/nim/reference',
    requiresApiKey: true,
    requiredFields: [apiKey('NVIDIA API Key')],
    testStrategy: 'openai-compatible',
    baseUrl: 'https://integrate.api.nvidia.com/v1',
    defaultModel: 'meta/llama-3.3-70b-instruct',
    approvedHosts: ['build.nvidia.com', 'integrate.api.nvidia.com'],
  },
  'crax-gpt': {
    id: 'crax-gpt',
    name: 'OpenCode Gateway (crax-gpt)',
    logo: 'C',
    section: 'recommended',
    recommended: true,
    supportsLocal: false,
    description: 'Free curated OpenAI-compatible coding AI gateway with 1-click token connection.',
    apiKeyUrl: 'https://opencode.ai/',
    docsUrl: 'https://opencode.ai/docs',
    requiresApiKey: false,
    requiredFields: [apiKey('Optional OpenCode Key')],
    testStrategy: 'openai-compatible',
    baseUrl: 'https://gateway.opencode.ai/v1',
    defaultModel: 'crax-auto',
    approvedHosts: ['opencode.ai', 'gateway.opencode.ai'],
  },
  openrouter: {
    id: 'openrouter',
    name: 'OpenRouter',
    logo: 'R',
    section: 'recommended',
    recommended: true,
    supportsLocal: false,
    description: 'Unified multi-provider routing catalog and generous free model tiers.',
    apiKeyUrl: 'https://openrouter.ai/keys',
    docsUrl: 'https://openrouter.ai/docs',
    requiresApiKey: true,
    requiredFields: [apiKey()],
    testStrategy: 'openai-compatible',
    baseUrl: 'https://openrouter.ai/api/v1',
    defaultModel: 'openrouter/auto',
    approvedHosts: ['openrouter.ai'],
  },
  groq: {
    id: 'groq',
    name: 'Groq',
    logo: 'GR',
    section: 'recommended',
    recommended: true,
    supportsLocal: false,
    description: 'Ultra low-latency LPU inference for Llama 3.3 70B & Qwen 2.5 Coder.',
    apiKeyUrl: 'https://console.groq.com/keys',
    docsUrl: 'https://console.groq.com/docs/quickstart',
    requiresApiKey: true,
    requiredFields: [apiKey()],
    testStrategy: 'openai-compatible',
    baseUrl: 'https://api.groq.com/openai/v1',
    defaultModel: 'llama-3.3-70b-versatile',
    approvedHosts: ['console.groq.com'],
  },
  ollama: {
    id: 'ollama',
    name: 'Local AI (Ollama)',
    logo: 'L',
    section: 'local',
    recommended: true,
    supportsLocal: true,
    description: 'Private offline local models running on your hardware with no API key needed.',
    apiKeyUrl: null,
    installUrl: 'https://ollama.com/download',
    docsUrl: 'https://github.com/ollama/ollama/blob/main/docs/openai.md',
    requiresApiKey: false,
    requiredFields: [],
    testStrategy: 'ollama-local',
    baseUrl: 'http://127.0.0.1:11434/v1',
    defaultModel: 'qwen2.5-coder:7b',
    approvedHosts: ['ollama.com', 'github.com'],
  },
  openai: {
    id: 'openai',
    name: 'OpenAI',
    logo: 'O',
    section: 'additional',
    recommended: false,
    supportsLocal: false,
    description: 'GPT-4o, GPT-4o-mini and OpenAI reasoning models.',
    apiKeyUrl: 'https://platform.openai.com/api-keys',
    docsUrl: 'https://platform.openai.com/docs/overview',
    requiresApiKey: true,
    requiredFields: [apiKey()],
    testStrategy: 'openai-compatible',
    baseUrl: 'https://api.openai.com/v1',
    defaultModel: 'gpt-4o',
    approvedHosts: ['platform.openai.com'],
  },
  cerebras: {
    id: 'cerebras',
    name: 'Cerebras',
    logo: 'CB',
    section: 'additional',
    recommended: false,
    supportsLocal: false,
    description: 'Ultra-fast open weights inference at thousand tokens/sec.',
    apiKeyUrl: 'https://cloud.cerebras.ai/',
    docsUrl: 'https://inference-docs.cerebras.ai/api-reference/authentication',
    requiresApiKey: true,
    requiredFields: [apiKey()],
    testStrategy: 'openai-compatible',
    baseUrl: 'https://api.cerebras.ai/v1',
    defaultModel: 'llama3.3-70b',
    approvedHosts: ['cloud.cerebras.ai'],
  },
  sambanova: {
    id: 'sambanova',
    name: 'SambaNova',
    logo: 'SN',
    section: 'additional',
    recommended: false,
    supportsLocal: false,
    description: 'High token throughput with free development cloud tier.',
    apiKeyUrl: 'https://cloud.sambanova.ai/apis',
    docsUrl: 'https://community.sambanova.ai/docs',
    requiresApiKey: true,
    requiredFields: [apiKey()],
    testStrategy: 'openai-compatible',
    baseUrl: 'https://api.sambanova.ai/v1',
    defaultModel: 'Meta-Llama-3.3-70B-Instruct',
    approvedHosts: ['cloud.sambanova.ai'],
  },
  cloudflare: {
    id: 'cloudflare',
    name: 'Cloudflare Workers AI',
    logo: 'CF',
    section: 'additional',
    recommended: false,
    supportsLocal: false,
    description: 'Global serverless server and edge model inference.',
    apiKeyUrl: 'https://dash.cloudflare.com/',
    docsUrl: 'https://developers.cloudflare.com/workers-ai/',
    requiresApiKey: true,
    requiredFields: [apiKey('API Token'), { id: 'accountId', label: 'Account ID', placeholder: 'Cloudflare Account ID', secret: false }],
    testStrategy: 'cloudflare-workers-ai',
    baseUrl: 'https://api.cloudflare.com/client/v4/accounts/{accountId}/ai/v1',
    defaultModel: '@cf/meta/llama-3.3-70b-instruct',
    approvedHosts: ['dash.cloudflare.com', 'developers.cloudflare.com'],
  },
  'nim-local': {
    id: 'nim-local',
    name: 'Local NIM Container',
    logo: 'N',
    section: 'local',
    recommended: false,
    supportsLocal: true,
    description: 'Self-hosted local NVIDIA NIM microservice container.',
    apiKeyUrl: null,
    docsUrl: 'https://docs.nvidia.com/nim/',
    requiresApiKey: false,
    requiredFields: [],
    testStrategy: 'openai-compatible',
    baseUrl: 'http://127.0.0.1:8000/v1',
    defaultModel: 'meta/llama-3.3-70b-instruct',
    approvedHosts: ['docs.nvidia.com'],
    userBaseUrl: true,
  },
  custom: {
    id: 'custom',
    name: 'Custom OpenAI Endpoint',
    logo: 'API',
    section: 'advanced',
    recommended: false,
    supportsLocal: true,
    description: 'Connect any OpenAI-compatible API proxy, LM Studio, or self-hosted endpoint.',
    apiKeyUrl: null,
    docsUrl: 'https://platform.openai.com/docs/api-reference',
    requiresApiKey: false,
    requiredFields: [apiKey('Optional API Key')],
    testStrategy: 'openai-compatible',
    baseUrl: 'https://',
    defaultModel: '',
    approvedHosts: ['localhost', '127.0.0.1'],
    userBaseUrl: true,
  },
}

export const providerDefinitions = Object.values(PROVIDER_REGISTRY)
export const providerRegistry = providerDefinitions

export function providerDefinition(providerId: ProviderId): ProviderDefinition {
  return PROVIDER_REGISTRY[providerId]
}
