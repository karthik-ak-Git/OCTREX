'use client';

import React, { useState, useEffect } from 'react';
import Image from 'next/image';
import { 
  Check, 
  ChevronDown, 
  Cpu, 
  Sparkles, 
  Zap, 
  Server, 
  ArrowRight
} from 'lucide-react';

interface ProviderStatus {
  provider_id: string;
  status: any;
  latency_ms: number;
}

function CodeIcon() {
  return (
    <div className="w-10 h-10 rounded-xl bg-white border border-emerald-200/80 shadow-xs flex items-center justify-center font-bold text-sm text-emerald-600">
      &lt;/&gt;
    </div>
  );
}

const PROVIDER_INFO: Record<string, { name: string; desc: string; icon: any; defaultModels: string[]; tag?: string }> = {
  'opencode': {
    name: 'OpenCode',
    desc: 'Free model routing & OpenCode AI',
    icon: CodeIcon,
    defaultModels: ['opencode-free-router', 'qwen2.5-coder-32b-free', 'deepseek-r1-free', 'llama-3.3-70b-free'],
    tag: 'FREE ROUTER'
  },
  'nvidia-nim': {
    name: 'NVIDIA NIM',
    desc: 'Llama 3.3, Nemotron & DeepSeek NIM',
    icon: Cpu,
    defaultModels: ['meta/llama-3.3-70b-instruct', 'nvidia/llama-3.1-nemotron-70b-instruct', 'deepseek-ai/deepseek-r1']
  },
  'google': {
    name: 'Google Gemini',
    desc: 'Gemini 1.5 Pro & 2.0 Flash',
    icon: Sparkles,
    defaultModels: ['gemini-1.5-pro', 'gemini-1.5-flash', 'gemini-2.0-flash']
  },
  'groq': {
    name: 'Groq',
    desc: 'Ultra-fast Llama 3.3 & DeepSeek',
    icon: Zap,
    defaultModels: ['llama-3.3-70b-versatile', 'deepseek-r1-distill-llama-70b', 'mixtral-8x7b-32768']
  },
  'local': {
    name: 'Local Models',
    desc: 'Ollama / LM Studio',
    icon: Server,
    defaultModels: ['llama3', 'qwen2.5-coder', 'deepseek-r1']
  }
};

export default function OnboardingSetup() {
  const [selectedProvider, setSelectedProvider] = useState<string>('opencode');
  const [apiKeyInput, setApiKeyInput] = useState<string>('');
  const [providersHealth, setProvidersHealth] = useState<Record<string, ProviderStatus>>({});
  const [discoveredModels, setDiscoveredModels] = useState<string[]>(PROVIDER_INFO['opencode'].defaultModels);
  const [activeModel, setActiveModel] = useState<string>(PROVIDER_INFO['opencode'].defaultModels[0]);
  const [isDropdownOpen, setIsDropdownOpen] = useState<boolean>(false);
  const [isVerifying, setIsVerifying] = useState<boolean>(false);
  const [testStatus, setTestStatus] = useState<{ type: 'idle' | 'testing' | 'success' | 'error'; message: string }>({
    type: 'idle',
    message: 'Free Router Active'
  });

  const refreshLiveProviders = async () => {
    try {
      const res = await fetch('/api/providers');
      if (res.ok) {
        const data = await res.json();
        const map: Record<string, ProviderStatus> = {};
        data.providers?.forEach((p: ProviderStatus) => {
          map[p.provider_id] = p;
        });
        setProvidersHealth(map);

        if (map[selectedProvider]?.status?.Connected?.models?.length > 0) {
          const liveList = map[selectedProvider].status.Connected.models;
          setDiscoveredModels(liveList);
          if (!liveList.includes(activeModel)) {
            setActiveModel(liveList[0]);
          }
        }
      }
    } catch (err) {
      console.log('Rust backend API check:', err);
    }
  };

  useEffect(() => {
    refreshLiveProviders();
  }, [selectedProvider]);

  const handleSelectProvider = (id: string) => {
    setSelectedProvider(id);
    const defaults = PROVIDER_INFO[id]?.defaultModels || ['default-model'];
    setDiscoveredModels(defaults);
    setActiveModel(defaults[0]);
    setIsDropdownOpen(false);

    if (id === 'opencode') {
      setTestStatus({ type: 'idle', message: 'Free Router Active' });
    } else {
      setTestStatus({ type: 'idle', message: 'Ready to test' });
    }
  };

  const handleVerifyAndSaveKey = async () => {
    if (selectedProvider !== 'local' && selectedProvider !== 'opencode' && !apiKeyInput.trim()) {
      alert(`Please enter an API key for ${PROVIDER_INFO[selectedProvider]?.name}`);
      return;
    }

    setIsVerifying(true);
    setTestStatus({ type: 'testing', message: 'Verifying with Rust backend...' });

    try {
      const res = await fetch(`/api/providers/${selectedProvider}/connect`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ api_key: apiKeyInput.trim() }),
      });

      const data = await res.json();

      if (data.success && data.health) {
        const health = data.health;
        if (health.status?.Connected) {
          const liveModels = health.status.Connected.models;
          setDiscoveredModels(liveModels);
          if (liveModels.length > 0) setActiveModel(liveModels[0]);
          setTestStatus({ type: 'success', message: `✓ Connected (${health.latency_ms}ms)` });
          alert(`✓ Success! Connected to ${PROVIDER_INFO[selectedProvider]?.name} (${liveModels.length} models discovered).`);
        } else if (health.status?.AuthError) {
          setTestStatus({ type: 'error', message: `✗ Auth Error` });
          alert(`Auth Error: ${health.status.AuthError.message}`);
        } else if (health.status?.Offline) {
          setTestStatus({ type: 'error', message: `✗ Endpoint Offline` });
          alert(`Offline: ${health.status.Offline.reason}`);
        }
      } else {
        setTestStatus({ type: 'error', message: `✗ ${data.error || 'Connection failed'}` });
        alert(`Error: ${data.error || 'Failed to verify key'}`);
      }
      refreshLiveProviders();
    } catch (err: any) {
      setTestStatus({ type: 'error', message: '✗ Failed to reach server' });
      alert('Network error connecting to Rust backend server');
    } finally {
      setIsVerifying(false);
    }
  };

  const handleLaunchWorkspace = () => {
    alert(`Entering Octrex Workspace with Active Provider: [${PROVIDER_INFO[selectedProvider]?.name}] and Model: [${activeModel}]`);
  };

  return (
    <div className="relative min-h-screen flex items-center justify-center bg-[radial-gradient(circle_at_50%_30%,#ffffff_0%,#ebedf3_45%,#e1e4eb_100%)] overflow-hidden select-none p-4">
      
      {/* Glow Orbs */}
      <div className="absolute w-[700px] h-[700px] -top-[200px] -left-[150px] bg-[radial-gradient(circle,rgba(255,255,255,0.95)_0%,rgba(235,238,246,0.3)_65%,transparent_100%)] blur-[70px] pointer-events-none" />
      <div className="absolute w-[800px] h-[800px] -bottom-[250px] -right-[200px] bg-[radial-gradient(circle,rgba(220,226,238,0.85)_0%,rgba(210,216,230,0.15)_70%,transparent_100%)] blur-[90px] pointer-events-none" />

      {/* Main Glass Card Container */}
      <main className="w-[560px] p-9 relative z-10 bg-slate-50/80 backdrop-blur-2xl border border-white/90 rounded-[36px] shadow-[0_35px_70px_-15px_rgba(15,23,42,0.08),0_20px_40px_-20px_rgba(15,23,42,0.04),inset_0_1.5px_1px_rgba(255,255,255,0.95)]">
        
        {/* Branding Header */}
        <div className="flex flex-col items-center text-center mb-6">
          <div className="w-20 h-20 rounded-2xl overflow-hidden bg-slate-950 p-1 shadow-[0_10px_30px_rgba(6,182,212,0.25)] border border-cyan-500/40 mb-3 transition-transform hover:scale-105 duration-300">
            <Image 
              src="/logo.jpg" 
              alt="Octrex Code Logo" 
              width={80} 
              height={80} 
              className="w-full h-full object-cover rounded-xl"
            />
          </div>
          <div className="flex items-center space-x-2">
            <h1 className="text-2xl font-extrabold text-slate-900 tracking-tight">Connect a model</h1>
            <span className="px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-cyan-950 text-cyan-300 border border-cyan-800/60 font-mono">NEXT.JS + RUST</span>
          </div>
          <p className="text-xs font-medium text-slate-500 max-w-sm mt-1 leading-relaxed">
            Select a provider and verify model connection for the Octrex engine.
          </p>
        </div>

        {/* Provider List */}
        <div className="space-y-2.5 mb-6">
          {Object.entries(PROVIDER_INFO).map(([id, info]) => {
            const Icon = info.icon;
            const health = providersHealth[id];
            const isSelected = selectedProvider === id;

            let badgeClass = "px-3 py-1 rounded-xl text-xs font-bold bg-white/75 border border-slate-200/80 text-slate-700";
            let badgeText = "Connect";

            if (health?.status?.Connected) {
              badgeClass = "px-3 py-1 rounded-xl text-xs font-bold bg-slate-900 text-white";
              badgeText = `Connected (${health.latency_ms}ms)`;
            } else if (health?.status?.AuthError) {
              badgeClass = "px-3 py-1 rounded-xl text-xs font-bold bg-rose-500 text-white";
              badgeText = "Auth Error";
            } else if (health?.status?.Offline) {
              badgeClass = "px-3 py-1 rounded-xl text-xs font-bold bg-slate-200 text-slate-600";
              badgeText = "Offline";
            }

            return (
              <div 
                key={id}
                onClick={() => handleSelectProvider(id)}
                className={`p-3.5 rounded-2xl flex items-center justify-between cursor-pointer transition-all duration-200 ${
                  isSelected 
                    ? 'bg-white/95 border border-slate-900/25 shadow-[0_4px_18px_rgba(0,0,0,0.04)]' 
                    : 'bg-white/55 border border-white/80 hover:bg-white/85 hover:border-white/95 hover:-translate-y-[1px]'
                }`}
              >
                <div className="flex items-center space-x-3">
                  {id === 'opencode' ? (
                    <CodeIcon />
                  ) : (
                    <div className="w-10 h-10 rounded-xl bg-white border border-slate-200/70 shadow-xs flex items-center justify-center font-bold text-sm text-slate-900">
                      <Icon className="w-5 h-5 text-slate-800" />
                    </div>
                  )}
                  <div>
                    <div className="flex items-center space-x-1.5">
                      <span className="text-sm font-bold text-slate-900">{info.name}</span>
                      {info.tag && (
                        <span className="px-1.5 py-0.2 rounded text-[9px] font-extrabold bg-emerald-100 text-emerald-800 border border-emerald-300 font-mono">
                          {info.tag}
                        </span>
                      )}
                    </div>
                    <div className="text-xs text-slate-500 font-medium">{info.desc}</div>
                  </div>
                </div>
                <div className={badgeClass}>{badgeText}</div>
              </div>
            );
          })}
        </div>

        {/* API Key Input & Custom Glass Dropdown */}
        <div className="space-y-4 mb-7">
          <div>
            <div className="flex justify-between items-center mb-1.5">
              <label className="block text-[11px] font-extrabold uppercase text-slate-400 tracking-wider">
                {PROVIDER_INFO[selectedProvider]?.name} API key {selectedProvider === 'opencode' ? '(Optional)' : ''}
              </label>
              <span className={`text-[11px] font-bold font-mono ${
                testStatus.type === 'success' ? 'text-emerald-600' :
                testStatus.type === 'error' ? 'text-rose-600' :
                testStatus.type === 'testing' ? 'text-amber-600' : 'text-slate-500'
              }`}>
                {testStatus.message}
              </span>
            </div>

            <div className="flex items-center space-x-2">
              <input 
                type="password" 
                value={apiKeyInput}
                onChange={(e) => setApiKeyInput(e.target.value)}
                placeholder={selectedProvider === 'opencode' ? 'Free models auto-routed or paste API key...' : 'Paste your API key...'} 
                className="flex-1 px-4 py-3 rounded-2xl text-xs font-mono text-slate-800 bg-white/70 border border-slate-900/10 focus:bg-white/95 focus:border-slate-900/25 focus:outline-none focus:ring-4 focus:ring-slate-900/5 transition-all"
              />
              <button 
                onClick={handleVerifyAndSaveKey} 
                disabled={isVerifying}
                className="px-5 py-3 rounded-2xl font-bold text-xs whitespace-nowrap bg-slate-900 text-white hover:bg-slate-800 active:translate-y-0 hover:-translate-y-[1px] shadow-sm transition-all disabled:opacity-50"
              >
                {isVerifying ? 'Verifying...' : 'Verify & Save'}
              </button>
            </div>
          </div>

          {/* Custom Pixel-Perfect Glass Dropdown Picker */}
          <div className="relative">
            <div className="flex justify-between items-center mb-1.5">
              <label className="block text-[11px] font-extrabold uppercase text-slate-400 tracking-wider">
                Discovered models
              </label>
              <span className="text-[11px] font-mono text-slate-400">
                {discoveredModels.length} live models available
              </span>
            </div>

            {/* Selected Dropdown Trigger Button */}
            <button
              onClick={() => setIsDropdownOpen(!isDropdownOpen)}
              className="w-full px-4 py-3 rounded-2xl text-xs font-semibold text-slate-800 bg-white/85 border border-slate-900/10 flex items-center justify-between hover:bg-white/98 transition-all"
            >
              <span className="truncate font-mono">{activeModel}</span>
              <ChevronDown className={`w-4 h-4 text-slate-500 transition-transform duration-200 ${isDropdownOpen ? 'rotate-180' : ''}`} />
            </button>

            {/* Custom Glass Dropdown Popup Menu Container */}
            {isDropdownOpen && (
              <div className="absolute top-full left-0 right-0 mt-2 z-50 p-2 bg-white/95 backdrop-blur-xl border border-slate-900/15 rounded-2xl shadow-xl max-h-56 overflow-y-auto space-y-1">
                {discoveredModels.map((m) => (
                  <div
                    key={m}
                    onClick={() => {
                      setActiveModel(m);
                      setIsDropdownOpen(false);
                    }}
                    className={`px-3.5 py-2.5 rounded-xl text-xs font-mono font-semibold cursor-pointer flex items-center justify-between transition-all ${
                      activeModel === m
                        ? 'bg-slate-900 text-white'
                        : 'text-slate-800 hover:bg-slate-100'
                    }`}
                  >
                    <span className="truncate">{m}</span>
                    {activeModel === m && <Check className="w-3.5 h-3.5 text-white" />}
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>

        {/* Primary Launch Button */}
        <div>
          <button 
            onClick={handleLaunchWorkspace} 
            className="w-full py-4 rounded-2xl font-bold text-sm bg-slate-900 text-white hover:bg-slate-800 hover:-translate-y-[1px] active:translate-y-0 shadow-md transition-all flex items-center justify-center space-x-2"
          >
            <span>Enter Octrex Workspace</span>
            <ArrowRight className="w-4 h-4 text-white" />
          </button>
        </div>

      </main>
    </div>
  );
}
