import { useState } from 'react'
import {
  Code2,
  FileText,
  Shield,
  Folder,
  Globe,
} from 'lucide-react'
import { OctrexLogo } from '../OctrexBrand'

export type OnboardingModalProps = {
  isOpen: boolean
  onClose: () => void
  onFinish?: () => void
  onOpenFolder?: () => void
  onSelectWorkspace?: (path: string | null) => void
  onConnectProvider?: (providerId: string, apiKey: string, defaultModel?: string) => Promise<boolean>
  recentWorkspaces?: Array<{ name: string; path: string }>
}

export function OnboardingModal({
  isOpen,
  onClose,
  onFinish,
  onOpenFolder,
  onSelectWorkspace,
  onConnectProvider,
  recentWorkspaces = [
    { name: 'octrex-web', path: '~/dev/octrex-web' },
    { name: 'api-gateway', path: '~/dev/api-gateway' },
  ],
}: OnboardingModalProps) {
  const [step, setStep] = useState<1 | 2 | 3>(1)
  const [selectedProvider, setSelectedProvider] = useState<string>('anthropic')
  const [apiKey, setApiKey] = useState<string>('')
  const [defaultModel, setDefaultModel] = useState<string>('Claude Sonnet')
  const [connectedIds, setConnectedIds] = useState<Set<string>>(new Set(['anthropic']))

  if (!isOpen) return null

  const popularProviders = [
    { id: 'anthropic', letter: 'A', name: 'Anthropic', desc: 'Claude models', models: ['Claude Sonnet', 'Claude 3.7 Sonnet', 'Claude Haiku'] },
    { id: 'openai', letter: 'O', name: 'OpenAI', desc: 'GPT models', models: ['GPT-4o', 'GPT-4o-mini', 'o3-mini'] },
    { id: 'google', letter: 'G', name: 'Google', desc: 'Gemini models', models: ['Gemini 2.5 Flash', 'Gemini 2.5 Pro'] },
    { id: 'ollama', letter: 'L', name: 'Local', desc: 'Ollama / LM Studio', models: ['Qwen 2.5 Coder', 'Llama 3.3', 'DeepSeek R1'] },
    { id: 'crax-gpt', letter: 'C', name: 'crax-gpt', desc: 'Free OpenCode Gateway', models: ['crax-auto', 'crax-coder', 'crax-fast'] },
    { id: 'nvidia', letter: 'N', name: 'NVIDIA NIM', desc: 'High-throughput models', models: ['Llama-3.3-70B-Instruct', 'Qwen2.5-Coder-32B'] },
    { id: 'groq', letter: 'GR', name: 'Groq', desc: 'Ultra-fast inference', models: ['Llama-3.3-70B-Versatile', 'Qwen-2.5-Coder-32B'] },
    { id: 'openrouter', letter: 'R', name: 'OpenRouter', desc: 'Universal routing & free tier', models: ['OpenRouter Auto', 'Llama 3.3 70B (Free)'] },
  ]

  const handleConnect = async (provId: string) => {
    if (onConnectProvider) {
      await onConnectProvider(provId, apiKey, defaultModel)
    }
    setConnectedIds((prev) => new Set([...prev, provId]))
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/20 backdrop-blur-md p-4 animate-in fade-in duration-200">
      <div className="relative w-full max-w-[500px] rounded-3xl bg-white/95 p-8 shadow-[0_20px_60px_-15px_rgba(0,0,0,0.12)] border border-slate-200/80 backdrop-blur-2xl transition-all">
        {/* STEP 1: WELCOME (FIGMA IMAGE 1) */}
        {step === 1 && (
          <div className="flex flex-col items-center text-center">
            <div className="mb-6 flex h-14 w-14 items-center justify-center rounded-2xl bg-slate-50 border border-slate-200/80 shadow-sm">
              <OctrexLogo size={28} className="text-slate-900" />
            </div>

            <h2 className="text-2xl font-bold tracking-tight text-slate-900">Welcome to Octrex</h2>
            <p className="mt-2 text-sm text-slate-500 max-w-[340px] leading-relaxed">
              Your AI workspace for code, documents and decks — all through chat.
            </p>

            <div className="mt-7 w-full space-y-3 text-left">
              <div className="flex items-center gap-4 rounded-2xl bg-slate-50/80 p-3.5 border border-slate-100 hover:border-slate-200 transition-colors">
                <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-white border border-slate-200/60 shadow-2xs text-slate-700">
                  <Code2 size={18} />
                </div>
                <div>
                  <div className="text-sm font-semibold text-slate-900">Build software</div>
                  <div className="text-xs text-slate-500">Write, run and debug code with agents</div>
                </div>
              </div>

              <div className="flex items-center gap-4 rounded-2xl bg-slate-50/80 p-3.5 border border-slate-100 hover:border-slate-200 transition-colors">
                <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-white border border-slate-200/60 shadow-2xs text-slate-700">
                  <FileText size={18} />
                </div>
                <div>
                  <div className="text-sm font-semibold text-slate-900">Create documents</div>
                  <div className="text-xs text-slate-500">Word files, slides and reports from a prompt</div>
                </div>
              </div>

              <div className="flex items-center gap-4 rounded-2xl bg-slate-50/80 p-3.5 border border-slate-100 hover:border-slate-200 transition-colors">
                <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-white border border-slate-200/60 shadow-2xs text-slate-700">
                  <Shield size={18} />
                </div>
                <div>
                  <div className="text-sm font-semibold text-slate-900">Stay in control</div>
                  <div className="text-xs text-slate-500">Approve every command and file action</div>
                </div>
              </div>
            </div>

            <button
              onClick={() => setStep(2)}
              className="mt-8 w-full rounded-2xl bg-slate-950 py-3.5 text-sm font-medium text-white shadow-sm hover:bg-slate-800 active:scale-[0.99] transition-all cursor-pointer"
            >
              Get started
            </button>

            <button
              onClick={onClose}
              className="mt-3 text-xs text-slate-400 hover:text-slate-600 transition-colors cursor-pointer"
            >
              I already have an account
            </button>
          </div>
        )}

        {/* STEP 2: CONNECT A MODEL (FIGMA IMAGE 2) */}
        {step === 2 && (
          <div>
            <div className="text-left">
              <h2 className="text-2xl font-bold tracking-tight text-slate-900">Connect a model</h2>
              <p className="mt-1 text-xs text-slate-500">
                Pick a provider and add your key. You can change this later in Settings.
              </p>
            </div>

            <div className="mt-5 max-h-[220px] overflow-y-auto space-y-2 pr-1 custom-scrollbar">
              {popularProviders.map((p) => {
                const connected = connectedIds.has(p.id)
                const isSelected = selectedProvider === p.id
                return (
                  <div
                    key={p.id}
                    onClick={() => {
                      setSelectedProvider(p.id)
                      if (p.models[0]) setDefaultModel(p.models[0])
                    }}
                    className={`flex items-center justify-between rounded-2xl p-3 border transition-all cursor-pointer ${
                      isSelected
                        ? 'bg-slate-50/90 border-slate-300 ring-1 ring-slate-900/5'
                        : 'bg-white border-slate-150 hover:bg-slate-50/50'
                    }`}
                  >
                    <div className="flex items-center gap-3">
                      <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-slate-100 font-bold text-xs text-slate-800 border border-slate-200">
                        {p.letter}
                      </div>
                      <div>
                        <div className="text-sm font-semibold text-slate-900">{p.name}</div>
                        <div className="text-[11px] text-slate-500">{p.desc}</div>
                      </div>
                    </div>

                    <button
                      type="button"
                      onClick={(e) => {
                        e.stopPropagation()
                        handleConnect(p.id)
                      }}
                      className={`rounded-full px-3.5 py-1 text-xs font-medium transition-all cursor-pointer ${
                        connected
                          ? 'bg-slate-900 text-white'
                          : 'bg-slate-100 text-slate-700 hover:bg-slate-200'
                      }`}
                    >
                      {connected ? 'Connected' : 'Connect'}
                    </button>
                  </div>
                )
              })}
            </div>

            <div className="mt-4 space-y-3">
              <div>
                <label className="text-[11px] font-medium text-slate-500">
                  API key {selectedProvider === 'ollama' && '(Optional for local Ollama)'}
                </label>
                <input
                  type="password"
                  value={apiKey}
                  onChange={(e) => setApiKey(e.target.value)}
                  placeholder={
                    selectedProvider === 'crax-gpt'
                      ? 'Free OpenCode token (optional)'
                      : selectedProvider === 'ollama'
                      ? 'http://127.0.0.1:11434'
                      : 'sk-••••••••••••••••••••••••'
                  }
                  className="mt-1 w-full rounded-xl border border-slate-200 bg-slate-50/60 px-3 py-2 text-xs text-slate-800 placeholder-slate-400 focus:bg-white focus:border-slate-400 focus:outline-none focus:ring-2 focus:ring-slate-900/5 transition-all"
                />
              </div>

              <div>
                <label className="text-[11px] font-medium text-slate-500">Default model</label>
                <select
                  value={defaultModel}
                  onChange={(e) => setDefaultModel(e.target.value)}
                  className="mt-1 w-full rounded-xl border border-slate-200 bg-slate-50/60 px-3 py-2 text-xs text-slate-800 focus:bg-white focus:border-slate-400 focus:outline-none transition-all cursor-pointer"
                >
                  {(popularProviders.find((p) => p.id === selectedProvider)?.models || [
                    'Claude Sonnet',
                    'Gemini 2.5 Flash',
                    'GPT-4o',
                  ]).map((m) => (
                    <option key={m} value={m}>
                      {m}
                    </option>
                  ))}
                </select>
              </div>
            </div>

            <div className="mt-6 flex items-center justify-between gap-3">
              <button
                onClick={() => setStep(1)}
                className="rounded-2xl border border-slate-200 bg-slate-50 px-5 py-2.5 text-xs font-medium text-slate-700 hover:bg-slate-100 transition-colors cursor-pointer"
              >
                Back
              </button>
              <button
                onClick={() => {
                  if (apiKey) handleConnect(selectedProvider)
                  setStep(3)
                }}
                className="rounded-2xl bg-slate-950 px-6 py-2.5 text-xs font-medium text-white shadow-sm hover:bg-slate-800 transition-colors cursor-pointer"
              >
                Continue
              </button>
            </div>
          </div>
        )}

        {/* STEP 3: CHOOSE WORKSPACE (FIGMA IMAGE 3) */}
        {step === 3 && (
          <div>
            <div className="text-left">
              <h2 className="text-2xl font-bold tracking-tight text-slate-900">Choose your workspace</h2>
              <p className="mt-1 text-xs text-slate-500">
                Connect a project folder, or start in the global workspace.
              </p>
            </div>

            <div className="mt-6 grid grid-cols-2 gap-3.5">
              <div
                onClick={() => {
                  if (onOpenFolder) onOpenFolder()
                  onClose()
                }}
                className="group flex flex-col justify-between rounded-2xl border border-slate-200 bg-white p-4.5 hover:border-slate-300 hover:shadow-sm cursor-pointer transition-all"
              >
                <div>
                  <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-slate-100 border border-slate-200 text-slate-800 group-hover:bg-slate-200 transition-colors">
                    <Folder size={18} />
                  </div>
                  <div className="mt-3 text-sm font-semibold text-slate-900">Open a folder</div>
                  <div className="mt-1 text-xs text-slate-500 leading-snug">
                    Give Octrex access to a project on your machine
                  </div>
                </div>
                <div className="mt-4 text-xs font-medium text-slate-900 underline underline-offset-2">
                  Browse...
                </div>
              </div>

              <div
                onClick={() => {
                  if (onSelectWorkspace) onSelectWorkspace(null)
                  onClose()
                }}
                className="group flex flex-col justify-between rounded-2xl border border-slate-200 bg-white p-4.5 hover:border-slate-300 hover:shadow-sm cursor-pointer transition-all"
              >
                <div>
                  <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-slate-100 border border-slate-200 text-slate-800 group-hover:bg-slate-200 transition-colors">
                    <Globe size={18} />
                  </div>
                  <div className="mt-3 text-sm font-semibold text-slate-900">Global workspace</div>
                  <div className="mt-1 text-xs text-slate-500 leading-snug">
                    No folder — chat, docs and decks anywhere
                  </div>
                </div>
                <div className="mt-4 text-[11px] font-medium text-slate-400">
                  Recommended to explore
                </div>
              </div>
            </div>

            {recentWorkspaces.length > 0 && (
              <div className="mt-6 text-left">
                <div className="text-[11px] font-semibold tracking-wider text-slate-400 uppercase">
                  Recent
                </div>
                <div className="mt-2 space-y-1.5">
                  {recentWorkspaces.map((ws) => (
                    <div
                      key={ws.path}
                      onClick={() => {
                        if (onSelectWorkspace) onSelectWorkspace(ws.path)
                        onClose()
                      }}
                      className="flex items-center justify-between rounded-xl bg-slate-50/80 px-3.5 py-2.5 border border-slate-100 hover:bg-slate-100/80 hover:border-slate-200 cursor-pointer transition-colors"
                    >
                      <span className="text-xs font-semibold text-slate-900">{ws.name}</span>
                      <span className="text-[11px] font-mono text-slate-400">{ws.path}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}

            <button
              onClick={() => {
                if (onFinish) onFinish()
                else onClose()
              }}
              className="mt-6 w-full rounded-2xl bg-slate-950 py-3 text-xs font-medium text-white shadow-sm hover:bg-slate-800 transition-colors cursor-pointer"
            >
              Enter Octrex
            </button>
          </div>
        )}

        {/* PAGINATION DOTS */}
        <div className="mt-6 flex items-center justify-center gap-1.5">
          <div className={`h-1.5 rounded-full transition-all ${step === 1 ? 'w-4 bg-slate-900' : 'w-1.5 bg-slate-300'}`} />
          <div className={`h-1.5 rounded-full transition-all ${step === 2 ? 'w-4 bg-slate-900' : 'w-1.5 bg-slate-300'}`} />
          <div className={`h-1.5 rounded-full transition-all ${step === 3 ? 'w-4 bg-slate-900' : 'w-1.5 bg-slate-300'}`} />
        </div>
      </div>
    </div>
  )
}
