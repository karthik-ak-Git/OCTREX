import React, { useState } from 'react';
import {
  Code2,
  FileText,
  Shield,
  Folder,
  Globe,
  ArrowRight
} from 'lucide-react';
import { OctrexLogo } from '../OctrexBrand';
import { providerDefinitions, ProviderDefinition } from '../../../shared/provider-registry';

export type OnboardingModalProps = {
  isOpen: boolean;
  onClose: () => void;
  onFinish?: () => void;
  onOpenFolder?: () => void;
  onSelectWorkspace?: (path: string | null) => void;
  onConnectProvider?: (providerId: string, apiKey: string) => Promise<boolean>;
  recentWorkspaces?: Array<{ name: string; path: string }>;
};

export const OnboardingModal: React.FC<OnboardingModalProps> = ({
  isOpen,
  onClose,
  onFinish,
  onOpenFolder,
  onSelectWorkspace,
  onConnectProvider,
  recentWorkspaces = [
    { name: 'octrex-web', path: '~/dev/octrex-web' },
    { name: 'api-gateway', path: '~/dev/api-gateway' },
    { name: 'docs-site', path: '~/dev/docs-site' }
  ]
}) => {
  const [step, setStep] = useState(1);
  const [selectedProvider, setSelectedProvider] = useState('google');
  const [apiKey, setApiKey] = useState('');
  const [connectedIds, setConnectedIds] = useState<Set<string>>(new Set(['google']));

  if (!isOpen) return null;

  const handleConnect = async (providerId: string) => {
    if (onConnectProvider) {
      await onConnectProvider(providerId, apiKey || 'auto-key');
    }
    setConnectedIds((prev) => new Set(prev).add(providerId));
    setStep(3);
  };

  const handleComplete = () => {
    if (onFinish) onFinish();
    else onClose();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/40 backdrop-blur-md p-6 select-none animate-in fade-in duration-300">
      <div className="w-full max-w-md overflow-hidden rounded-3xl glass-modal p-8 text-center shadow-2xl border border-white/80">
        {/* LOGO ICON & TITLE */}
        <div className="mx-auto flex h-14 w-14 items-center justify-center rounded-2xl bg-slate-950 text-white shadow-md mb-4">
          <OctrexLogo size={28} />
        </div>

        {/* STEP 1: WELCOME TO OCTREX (FIGMA IMAGE 1) */}
        {step === 1 && (
          <div className="space-y-6">
            <div>
              <h2 className="text-2xl font-bold tracking-tight text-slate-900">
                Welcome to Octrex
              </h2>
              <p className="mt-1 text-xs text-slate-500 font-normal">
                Your autonomous software engineering and multi-agent workspace.
              </p>
            </div>

            <div className="space-y-3 text-left">
              <div className="flex items-start gap-3.5 rounded-2xl glass-card p-3.5 border border-white/80 shadow-2xs">
                <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-slate-100 text-slate-800 shrink-0">
                  <Code2 size={18} />
                </div>
                <div>
                  <div className="text-xs font-bold text-slate-900">Work from anywhere</div>
                  <div className="text-[11px] text-slate-500">
                    Use cloud inference or private local AI on your own hardware
                  </div>
                </div>
              </div>

              <div className="flex items-start gap-3.5 rounded-2xl glass-card p-3.5 border border-white/80 shadow-2xs">
                <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-slate-100 text-slate-800 shrink-0">
                  <FileText size={18} />
                </div>
                <div>
                  <div className="text-xs font-bold text-slate-900">Read & generate documents</div>
                  <div className="text-[11px] text-slate-500">
                    Create presentations, docs, and code with interactive slide decks
                  </div>
                </div>
              </div>

              <div className="flex items-start gap-3.5 rounded-2xl glass-card p-3.5 border border-white/80 shadow-2xs">
                <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-slate-100 text-slate-800 shrink-0">
                  <Shield size={18} />
                </div>
                <div>
                  <div className="text-xs font-bold text-slate-900">Stay in control</div>
                  <div className="text-[11px] text-slate-500">
                    Fine-grained permission prompts and sandboxed verification
                  </div>
                </div>
              </div>
            </div>

            <button
              onClick={() => setStep(2)}
              className="w-full flex items-center justify-center gap-2 rounded-2xl bg-slate-950 hover:bg-slate-800 py-3.5 text-xs font-semibold text-white shadow-sm hover:scale-[1.01] active:scale-[0.99] transition-all cursor-pointer"
            >
              <span>Get started</span>
              <ArrowRight size={14} />
            </button>

            <button
              onClick={handleComplete}
              className="text-xs text-slate-400 hover:text-slate-700 transition-colors cursor-pointer"
            >
              Skip setup
            </button>
          </div>
        )}

        {/* STEP 2: CONNECT A MODEL (FIGMA IMAGE 2) */}
        {step === 2 && (
          <div className="space-y-4">
            <div className="text-left">
              <h2 className="text-2xl font-bold tracking-tight text-slate-900">Connect a model</h2>
              <p className="mt-1 text-xs text-slate-500">
                Pick a provider and enter your key. You can add more later in Settings.
              </p>
            </div>

            {/* PROVIDERS SCROLL */}
            <div className="max-h-[220px] overflow-y-auto space-y-2 text-left custom-scrollbar pr-1">
              {providerDefinitions.slice(0, 6).map((p: ProviderDefinition) => {
                const connected = connectedIds.has(p.id);
                const isSelected = selectedProvider === p.id;
                return (
                  <div
                    key={p.id}
                    onClick={() => setSelectedProvider(p.id)}
                    className={`flex items-center justify-between rounded-2xl p-3 border transition-all cursor-pointer ${
                      isSelected
                        ? 'glass-card border-slate-900 shadow-2xs ring-1 ring-cyan-500/20'
                        : 'bg-white/60 border-slate-200/80 hover:bg-white'
                    }`}
                  >
                    <div className="flex items-center gap-3">
                      <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-slate-900 font-bold text-xs text-white">
                        {p.logo}
                      </div>
                      <div>
                        <div className="text-xs font-bold text-slate-900">{p.name}</div>
                        <div className="text-[10px] text-slate-500">{p.description}</div>
                      </div>
                    </div>

                    <button
                      type="button"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleConnect(p.id);
                      }}
                      className={`rounded-full px-3 py-1 text-[11px] font-semibold transition-all cursor-pointer ${
                        connected
                          ? 'bg-slate-900 text-white'
                          : 'bg-slate-100 hover:bg-slate-200 text-slate-700'
                      }`}
                    >
                      {connected ? 'Connected' : 'Connect'}
                    </button>
                  </div>
                );
              })}
            </div>

            <div className="space-y-2 text-left">
              <label className="text-[11px] font-semibold text-slate-700">API Key / Token</label>
              <input
                type="password"
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder="Paste API Key (sk-...) or leave blank for local"
                className="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-xs text-slate-800 placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-cyan-500/20"
              />
            </div>

            <div className="flex items-center justify-between gap-2 pt-2">
              <button
                onClick={() => setStep(1)}
                className="rounded-2xl border border-slate-200 px-4 py-3 text-xs font-semibold text-slate-700 hover:bg-slate-100"
              >
                Back
              </button>
              <button
                onClick={() => setStep(3)}
                className="flex-1 rounded-2xl bg-slate-950 hover:bg-slate-800 py-3 text-xs font-semibold text-white shadow-sm transition-all cursor-pointer"
              >
                Continue
              </button>
            </div>
          </div>
        )}

        {/* STEP 3: CHOOSE WORKSPACE (FIGMA IMAGE 3) */}
        {step === 3 && (
          <div className="space-y-4 text-left">
            <div>
              <h2 className="text-2xl font-bold tracking-tight text-slate-900">
                Choose a workspace
              </h2>
              <p className="mt-1 text-xs text-slate-500">
                Open a folder to give Octrex access to your files, code, and project context.
              </p>
            </div>

            <div className="space-y-2.5">
              <button
                onClick={() => {
                  if (onOpenFolder) onOpenFolder();
                  handleComplete();
                }}
                className="w-full flex items-center gap-3.5 rounded-2xl glass-card border border-white/80 p-4 hover:border-slate-400 hover:bg-white cursor-pointer transition-all shadow-2xs"
              >
                <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-slate-100 text-slate-800 shrink-0">
                  <Folder size={18} />
                </div>
                <div>
                  <div className="text-xs font-bold text-slate-900">Open a folder...</div>
                  <div className="text-[11px] text-slate-500">Choose a local repository or directory</div>
                </div>
              </button>

              <button
                onClick={() => {
                  if (onSelectWorkspace) onSelectWorkspace(null);
                  handleComplete();
                }}
                className="w-full flex items-center gap-3.5 rounded-2xl glass-card border border-white/80 p-4 hover:border-slate-400 hover:bg-white cursor-pointer transition-all shadow-2xs"
              >
                <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-slate-100 text-slate-800 shrink-0">
                  <Globe size={18} />
                </div>
                <div>
                  <div className="text-xs font-bold text-slate-900">Global workspace</div>
                  <div className="text-[11px] text-slate-500">Chat without a specific project root</div>
                </div>
              </button>
            </div>

            {/* RECENT WORKSPACES */}
            <div className="pt-2">
              <div className="text-[10px] font-bold tracking-wider text-slate-400 uppercase mb-1.5">
                Recent workspaces
              </div>
              <div className="space-y-1">
                {recentWorkspaces.map((ws) => (
                  <div
                    key={ws.path}
                    onClick={() => {
                      if (onSelectWorkspace) onSelectWorkspace(ws.name);
                      handleComplete();
                    }}
                    className="flex items-center justify-between rounded-xl p-2.5 hover:bg-white/80 border border-slate-150 cursor-pointer transition-colors"
                  >
                    <span className="text-xs font-semibold text-slate-900">{ws.name}</span>
                    <span className="text-[10px] font-mono text-slate-400">{ws.path}</span>
                  </div>
                ))}
              </div>
            </div>

            <button
              onClick={handleComplete}
              className="mt-4 w-full rounded-2xl bg-slate-950 hover:bg-slate-800 py-3.5 text-xs font-semibold text-white shadow-sm transition-all cursor-pointer"
            >
              Enter Octrex
            </button>
          </div>
        )}

        {/* PAGINATION DOTS */}
        <div className="mt-6 flex items-center justify-center gap-1.5">
          <div
            onClick={() => setStep(1)}
            className={`h-1.5 rounded-full transition-all cursor-pointer ${
              step === 1 ? 'w-4 bg-slate-900' : 'w-1.5 bg-slate-300'
            }`}
          />
          <div
            onClick={() => setStep(2)}
            className={`h-1.5 rounded-full transition-all cursor-pointer ${
              step === 2 ? 'w-4 bg-slate-900' : 'w-1.5 bg-slate-300'
            }`}
          />
          <div
            onClick={() => setStep(3)}
            className={`h-1.5 rounded-full transition-all cursor-pointer ${
              step === 3 ? 'w-4 bg-slate-900' : 'w-1.5 bg-slate-300'
            }`}
          />
        </div>
      </div>
    </div>
  );
};
