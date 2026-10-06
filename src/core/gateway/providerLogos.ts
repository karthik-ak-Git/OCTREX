/**
 * OCTREX CODE V4 — Provider Visual Assets, Brand Logos & Active Route Badges
 */

export interface ProviderMetadata {
  id: string;
  name: string;
  shortName: string;
  badgeLabel: string;
  brandColor: string;
  accentColor: string;
  defaultBaseUrl?: string;
  setupUrl?: string;
  authType: 'api_key' | 'none' | 'local';
  isCloud: boolean;
  isLocal: boolean;
  description: string;
  iconSvg: string;
}

export interface RouteBadgeInfo {
  providerId: string;
  providerName: string;
  modelId: string;
  displayModelName: string;
  textBadge: string;
  htmlBadge: string;
  iconSvg: string;
  brandColor: string;
}

export const PROVIDER_LOGOS: Record<string, string> = {
  'crax-gpt': `<svg viewBox="0 0 24 24" width="16" height="16" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M12 2L2 7V17L12 22L22 17V7L12 2Z" stroke="#A855F7" stroke-width="2" stroke-linejoin="round"/><path d="M12 6L6 9.5V14.5L12 18L18 14.5V9.5L12 6Z" fill="#A855F7" fill-opacity="0.3" stroke="#06B6D4" stroke-width="1.5"/><circle cx="12" cy="12" r="2.5" fill="#38BDF8"/></svg>`,
  'gemini': `<svg viewBox="0 0 24 24" width="16" height="16" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M12 2C12 7.52285 7.52285 12 2 12C7.52285 12 12 16.4772 12 22C12 16.4772 16.4772 12 22 12C16.4772 12 12 7.52285 12 2Z" fill="url(#gemini-grad)"/><defs><linearGradient id="gemini-grad" x1="2" y1="2" x2="22" y2="22" gradientUnits="userSpaceOnUse"><stop stop-color="#4E82EE"/><stop offset="0.5" stop-color="#9C52FD"/><stop offset="1" stop-color="#DE4396"/></linearGradient></defs></svg>`,
  'openrouter': `<svg viewBox="0 0 24 24" width="16" height="16" fill="none" xmlns="http://www.w3.org/2000/svg"><circle cx="12" cy="12" r="9" stroke="#6366F1" stroke-width="2"/><circle cx="12" cy="12" r="4" fill="#818CF8"/><path d="M12 3V7M12 17V21M3 12H7M17 12H21" stroke="#A5B4FC" stroke-width="2" stroke-linecap="round"/></svg>`,
  'nvidia': `<svg viewBox="0 0 24 24" width="16" height="16" fill="none" xmlns="http://www.w3.org/2000/svg"><rect x="3" y="3" width="18" height="18" rx="4" fill="#101010" stroke="#76B900" stroke-width="2"/><path d="M7 14C7.5 11 9.5 9 12 9C14.5 9 16.5 11 17 14C15.5 12.5 13.8 11.5 12 11.5C10.2 11.5 8.5 12.5 7 14Z" fill="#76B900"/><circle cx="12" cy="14" r="1.5" fill="#76B900"/></svg>`,
  'groq': `<svg viewBox="0 0 24 24" width="16" height="16" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M13 2L4 14H11L9 22L20 9H13L15 2H13Z" fill="#F97316" stroke="#EA580C" stroke-width="1.5" stroke-linejoin="round"/></svg>`,
  'ollama': `<svg viewBox="0 0 24 24" width="16" height="16" fill="none" xmlns="http://www.w3.org/2000/svg"><rect x="3" y="4" width="18" height="16" rx="3" fill="#18181B" stroke="#71717A" stroke-width="2"/><circle cx="8" cy="10" r="1.5" fill="#FAFAFA"/><circle cx="16" cy="10" r="1.5" fill="#FAFAFA"/><path d="M8 15H16" stroke="#A1A1AA" stroke-width="2" stroke-linecap="round"/></svg>`,
  'opencode': `<svg viewBox="0 0 24 24" width="16" height="16" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M7 8L3 12L7 16M17 8L21 12L17 16M14 4L10 20" stroke="#10B981" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>`,
  'openai-compatible': `<svg viewBox="0 0 24 24" width="16" height="16" fill="none" xmlns="http://www.w3.org/2000/svg"><circle cx="12" cy="12" r="8.5" stroke="#10A37F" stroke-width="2"/><circle cx="12" cy="12" r="3" fill="#10A37F"/><path d="M12 3.5V6.5M12 17.5V20.5M3.5 12H6.5M17.5 12H20.5" stroke="#10A37F" stroke-width="1.5" stroke-linecap="round"/></svg>`,
  'mock-provider': `<svg viewBox="0 0 24 24" width="16" height="16" fill="none" xmlns="http://www.w3.org/2000/svg"><polygon points="12 2 21 7 21 17 12 22 3 17 3 7" fill="#1E293B" stroke="#64748B" stroke-width="2"/><path d="M9 12L11 14L15 10" stroke="#38BDF8" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>`,
};

export const PROVIDER_METADATA_REGISTRY: Record<string, ProviderMetadata> = {
  'crax-gpt': {
    id: 'crax-gpt',
    name: 'crax-gpt',
    shortName: 'crax-gpt',
    badgeLabel: 'crax-gpt',
    brandColor: '#A855F7',
    accentColor: '#06B6D4',
    defaultBaseUrl: 'https://gpt.crax.lol/v1',
    setupUrl: 'https://gpt.crax.lol',
    authType: 'api_key',
    isCloud: true,
    isLocal: false,
    description: 'Fast OpenAI-compatible cloud gateway with reasoning, tools, and dynamic model catalog discovery.',
    iconSvg: PROVIDER_LOGOS['crax-gpt'],
  },
  'gemini': {
    id: 'gemini',
    name: 'Google Gemini',
    shortName: 'Gemini',
    badgeLabel: 'Google Gemini',
    brandColor: '#4E82EE',
    accentColor: '#DE4396',
    defaultBaseUrl: 'https://generativelanguage.googleapis.com/v1beta',
    setupUrl: 'https://aistudio.google.com/app/apikey',
    authType: 'api_key',
    isCloud: true,
    isLocal: false,
    description: 'Google Multimodal AI platform with 1M+ context window and native function calling.',
    iconSvg: PROVIDER_LOGOS['gemini'],
  },
  'openrouter': {
    id: 'openrouter',
    name: 'OpenRouter',
    shortName: 'OpenRouter',
    badgeLabel: 'OpenRouter',
    brandColor: '#6366F1',
    accentColor: '#818CF8',
    defaultBaseUrl: 'https://openrouter.ai/api/v1',
    setupUrl: 'https://openrouter.ai/keys',
    authType: 'api_key',
    isCloud: true,
    isLocal: false,
    description: 'Universal unified model gateway routing to Claude 3.5 Sonnet, GPT-4o, and hundreds of LLMs.',
    iconSvg: PROVIDER_LOGOS['openrouter'],
  },
  'nvidia': {
    id: 'nvidia',
    name: 'NVIDIA NIM',
    shortName: 'NVIDIA',
    badgeLabel: 'NVIDIA NIM',
    brandColor: '#76B900',
    accentColor: '#84CC16',
    defaultBaseUrl: 'https://integrate.api.nvidia.com/v1',
    setupUrl: 'https://build.nvidia.com',
    authType: 'api_key',
    isCloud: true,
    isLocal: false,
    description: 'High-performance accelerated enterprise microservices powered by NVIDIA GPUs.',
    iconSvg: PROVIDER_LOGOS['nvidia'],
  },
  'groq': {
    id: 'groq',
    name: 'Groq Cloud',
    shortName: 'Groq',
    badgeLabel: 'Groq',
    brandColor: '#F97316',
    accentColor: '#FB923C',
    defaultBaseUrl: 'https://api.groq.com/openai/v1',
    setupUrl: 'https://console.groq.com/keys',
    authType: 'api_key',
    isCloud: true,
    isLocal: false,
    description: 'Ultra low-latency LPU inference engine for rapid code completion and planning.',
    iconSvg: PROVIDER_LOGOS['groq'],
  },
  'ollama': {
    id: 'ollama',
    name: 'Ollama Local',
    shortName: 'Ollama',
    badgeLabel: 'Ollama (Local)',
    brandColor: '#71717A',
    accentColor: '#A1A1AA',
    defaultBaseUrl: 'http://127.0.0.1:11434',
    setupUrl: 'https://ollama.com',
    authType: 'local',
    isCloud: false,
    isLocal: true,
    description: 'Fully local, air-gapped model execution on your own machine without network transmission.',
    iconSvg: PROVIDER_LOGOS['ollama'],
  },
  'opencode': {
    id: 'opencode',
    name: 'OpenCode Gateway',
    shortName: 'OpenCode',
    badgeLabel: 'OpenCode (Free)',
    brandColor: '#10B981',
    accentColor: '#34D399',
    defaultBaseUrl: 'https://openrouter.ai/api/v1',
    authType: 'none',
    isCloud: true,
    isLocal: false,
    description: 'Curated free open-source models (Gemini Flash, DeepSeek R1, Llama 3.3 70B, Qwen 2.5 Coder).',
    iconSvg: PROVIDER_LOGOS['opencode'],
  },
  'openai-compatible': {
    id: 'openai-compatible',
    name: 'Custom OpenAI Endpoint',
    shortName: 'Custom OpenAI',
    badgeLabel: 'Custom OpenAI',
    brandColor: '#10A37F',
    accentColor: '#2DD4BF',
    defaultBaseUrl: 'http://127.0.0.1:8000/v1',
    authType: 'api_key',
    isCloud: false,
    isLocal: true,
    description: 'Connect custom vLLM, LM Studio, TextGen, ngrok-tunneled, or private inference endpoints.',
    iconSvg: PROVIDER_LOGOS['openai-compatible'],
  },
  'mock-provider': {
    id: 'mock-provider',
    name: 'OCTREX Test Harness',
    shortName: 'OCTREX Mock',
    badgeLabel: 'Mock Provider',
    brandColor: '#64748B',
    accentColor: '#38BDF8',
    authType: 'none',
    isCloud: false,
    isLocal: true,
    description: 'Deterministic synthetic model runner for zero-network automated verification and tests.',
    iconSvg: PROVIDER_LOGOS['mock-provider'],
  },
};

export class ProviderLogoService {
  public static getLogoSvg(providerId: string): string {
    return PROVIDER_LOGOS[providerId] || PROVIDER_LOGOS['openai-compatible'];
  }

  public static getMetadata(providerId: string): ProviderMetadata {
    if (PROVIDER_METADATA_REGISTRY[providerId]) {
      return PROVIDER_METADATA_REGISTRY[providerId];
    }
    return {
      id: providerId,
      name: providerId,
      shortName: providerId,
      badgeLabel: providerId,
      brandColor: '#64748B',
      accentColor: '#94A3B8',
      authType: 'api_key',
      isCloud: true,
      isLocal: false,
      description: `Provider ${providerId}`,
      iconSvg: PROVIDER_LOGOS['openai-compatible'],
    };
  }

  public static formatRouteBadge(providerId: string, modelId: string): RouteBadgeInfo {
    const meta = this.getMetadata(providerId);
    const displayModel = modelId.includes('/') ? modelId.split('/').pop() || modelId : modelId;
    const textBadge = `[${meta.shortName}] ${meta.shortName} • ${displayModel}`;
    const htmlBadge = `<span class="octrex-route-badge" style="display:inline-flex;align-items:center;gap:6px;padding:3px 8px;border-radius:6px;background:rgba(255,255,255,0.06);border:1px solid ${meta.brandColor}40;font-size:12px;font-family:ui-monospace,monospace;color:#F4F4F5;">${meta.iconSvg}<strong style="color:${meta.brandColor};">${meta.shortName}</strong><span style="opacity:0.4;">•</span><span>${displayModel}</span></span>`;

    return {
      providerId,
      providerName: meta.name,
      modelId,
      displayModelName: displayModel,
      textBadge,
      htmlBadge,
      iconSvg: meta.iconSvg,
      brandColor: meta.brandColor,
    };
  }

  public static getAllPresets(): ProviderMetadata[] {
    return Object.values(PROVIDER_METADATA_REGISTRY);
  }
}
