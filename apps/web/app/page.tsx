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
  ArrowRight,
  Plus,
  Search,
  Settings,
  User,
  Folder,
  Globe,
  FileText,
  Terminal as TerminalIcon,
  Play,
  RotateCcw,
  Sliders,
  Shield,
  Layers,
  Code,
  Paperclip,
  ArrowUp,
  X,
  ExternalLink,
  Download,
  Clock,
  Activity as ActivityIcon,
  Maximize2,
  Trash2,
  MessageSquare,
  FolderPlus
} from 'lucide-react';
import { NetworkSecurityModal, NetworkSecurityBadge } from '../components/NetworkSecurityModal';
import { ContextBudgetIndicator } from '../components/ContextBudgetIndicator';

import Link from 'next/link';
import { PrivacyBadge } from '../components/PrivacyBadge';
import { PrivacyDecisionInspector } from '../components/PrivacyDecisionInspector';
import { OnlineConsentDialog } from '../components/OnlineConsentDialog';
import { PrivacyRoutingPreview } from '../components/PrivacyRoutingPreview';

interface ProviderStatus {
  provider_id: string;
  status: any;
  latency_ms: number;
}

interface ChatSession {
  id: string;
  title: string;
  createdAt: string;
  messages: Array<{ role: 'user' | 'assistant'; content: string; steps?: Array<{ label: string; detail: string; status: 'done' | 'running' }>; artifact?: any }>;
}

interface FileEntry {
  name: string;
  path: string;
  rel_path: string;
  is_dir: boolean;
  size_bytes: number;
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

export default function App() {
  const [viewMode, setViewMode] = useState<'setup' | 'workspace'>('workspace');
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

  // Workspace Real Inspection State
  const [workspaceData, setWorkspaceData] = useState<{ name: string; total_files: number; path: string } | null>(null);
  const [activeTab, setActiveTab] = useState<'activity' | 'files' | 'terminal' | 'changes'>('activity');
  const [promptInput, setPromptInput] = useState<string>('');
  const [isExecuting, setIsExecuting] = useState<boolean>(false);
  const [activeChatTitle, setActiveChatTitle] = useState<string>('Octrex Workspace Engine');
  const [permissionAllowed, setPermissionAllowed] = useState<boolean | null>(null);
  const [chatMessages, setChatMessages] = useState<Array<{ role: 'user' | 'assistant'; content: string; steps?: Array<{ label: string; detail: string; status: 'done' | 'running' }>; artifact?: any }>>([]);

  // Interactive Left Sidebar State & Sessions
  const [sessions, setSessions] = useState<ChatSession[]>([
    {
      id: 'session-1',
      title: 'Workspace Initialized',
      createdAt: 'Just now',
      messages: []
    }
  ]);
  const [activeSessionId, setActiveSessionId] = useState<string>('session-1');
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [isConnectFolderOpen, setIsConnectFolderOpen] = useState<boolean>(false);
  const [customFolderPath, setCustomFolderPath] = useState<string>('');
  // Network Security State (Phase 7)
  const [isNetworkModalOpen, setIsNetworkModalOpen] = useState<boolean>(false);
  const [networkMode, setNetworkMode] = useState<'local_only' | 'restricted' | 'online_allowed' | 'disabled'>('local_only');

  // Privacy State & Decision State
  const [isSettingsOpen, setIsSettingsOpen] = useState<boolean>(false);
  const [privacyMode, setPrivacyMode] = useState<string>('LOCAL_ONLY');
  const [confidentialMode, setConfidentialMode] = useState<boolean>(false);
  const [workspaceClassification, setWorkspaceClassification] = useState<string>('PUBLIC');
  const [activePrivacyDecision, setActivePrivacyDecision] = useState<any | null>(null);
  const [activeConsentRequest, setActiveConsentRequest] = useState<any | null>(null);

  // Verification State (Phase 13) — independent completion evidence.
  // A model stating "done" never marks a task verified; only the
  // VerificationEngine result does.
  const [lastTaskId, setLastTaskId] = useState<string | null>(null);
  const [taskCompletion, setTaskCompletion] = useState<{
    verified: boolean;
    status: string;
    completion_gate: string;
    failures: string[];
  } | null>(null);

  const fetchTaskCompletion = async (taskId: string) => {
    try {
      const res = await fetch(`/api/tasks/${encodeURIComponent(taskId)}/completion`);
      if (res.ok) {
        const data = await res.json();
        if (data.success) {
          setTaskCompletion({
            verified: data.verified,
            status: data.status,
            completion_gate: data.completion_gate,
            failures: data.failures || [],
          });
        }
      }
    } catch (e) {
      console.log('Error fetching task completion:', e);
    }
  };

  const fetchPrivacySettings = async () => {
    try {
      const res = await fetch('/api/privacy/settings');
      if (res.ok) {
        const data = await res.json();
        setPrivacyMode(data.privacy_mode || 'LOCAL_ONLY');
        setConfidentialMode(data.confidential_mode || false);
      }
    } catch (e) {
      console.log('Error fetching privacy settings:', e);
    }
  };

  // File Inspector & File Tree State
  const [workspaceTree, setWorkspaceTree] = useState<FileEntry[]>([]);
  const [selectedFile, setSelectedFile] = useState<FileEntry | null>(null);
  const [fileContent, setFileContent] = useState<string>('');
  const [isSavingFile, setIsSavingFile] = useState<boolean>(false);
  const [isFileEditorOpen, setIsFileEditorOpen] = useState<boolean>(false);

  const fetchWorkspaceTree = async (targetPath: string) => {
    try {
      const res = await fetch('/api/workspace/tree', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path: targetPath }),
      });
      if (res.ok) {
        const data = await res.json();
        if (data.success && Array.isArray(data.entries)) {
          setWorkspaceTree(data.entries);
        }
      }
    } catch (err) {
      console.log('Error fetching workspace tree:', err);
    }
  };

  const handleOpenFile = async (file: FileEntry) => {
    if (file.is_dir) return;
    const currentPath = workspaceData?.path ? String(workspaceData.path) : '.';
    setSelectedFile(file);
    setIsFileEditorOpen(true);
    try {
      const res = await fetch('/api/workspace/read_file', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          workspace_path: currentPath,
          rel_path: file.rel_path,
        }),
      });
      if (res.ok) {
        const data = await res.json();
        if (data.success) {
          setFileContent(data.content || '');
        } else {
          setFileContent(`// Error reading file: ${data.error}`);
        }
      }
    } catch (err: any) {
      setFileContent(`// Error connecting to backend: ${err.message}`);
    }
  };

  const handleSaveFile = async () => {
    if (!selectedFile) return;
    const currentPath = workspaceData?.path ? String(workspaceData.path) : '.';
    setIsSavingFile(true);
    try {
      const res = await fetch('/api/workspace/write_file', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          workspace_path: currentPath,
          rel_path: selectedFile.rel_path,
          content: fileContent,
        }),
      });
      if (res.ok) {
        const data = await res.json();
        if (data.success) {
          alert(`✓ File '${selectedFile.name}' updated on disk (${data.bytes_written} bytes)!`);
          setIsFileEditorOpen(false);
          fetchWorkspaceTree(currentPath);
        } else {
          alert(`Error writing file: ${data.error}`);
        }
      }
    } catch (err: any) {
      alert(`Failed to save file: ${err.message}`);
    } finally {
      setIsSavingFile(false);
    }
  };

  const handleNewChat = () => {
    const newId = `session-${Date.now()}`;
    const newSession: ChatSession = {
      id: newId,
      title: 'New Session',
      createdAt: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
      messages: []
    };
    setSessions(prev => [newSession, ...prev]);
    setActiveSessionId(newId);
    setChatMessages([]);
    setActiveChatTitle('New Session');
  };

  const handleSelectSession = (session: ChatSession) => {
    setActiveSessionId(session.id);
    setChatMessages(session.messages);
    setActiveChatTitle(session.title);
  };

  const handleDeleteSession = (e: React.MouseEvent, id: string) => {
    e.stopPropagation();
    setSessions(prev => {
      const updated = prev.filter(s => s.id !== id);
      if (updated.length > 0 && activeSessionId === id) {
        setActiveSessionId(updated[0].id);
        setChatMessages(updated[0].messages);
        setActiveChatTitle(updated[0].title);
      } else if (updated.length === 0) {
        const freshId = `session-${Date.now()}`;
        const fresh: ChatSession = {
          id: freshId,
          title: 'New Session',
          createdAt: 'Just now',
          messages: []
        };
        setActiveSessionId(freshId);
        setChatMessages([]);
        setActiveChatTitle('New Session');
        return [fresh];
      }
      return updated;
    });
  };

  const handleConnectFolder = async () => {
    if (!customFolderPath.trim()) return;
    const target = customFolderPath.trim();
    try {
      const res = await fetch('/api/workspace/inspect', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path: target }),
      });
      if (res.ok) {
        const data = await res.json();
        if (data.exists) {
          setWorkspaceData(data);
          setIsConnectFolderOpen(false);
          setCustomFolderPath('');
          fetchWorkspaceTree(target);
        } else {
          alert(`Path '${target}' does not exist on disk.`);
        }
      }
    } catch (err) {
      alert('Failed to reach Rust backend for path inspection');
    }
  };

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

  const inspectWorkspace = async () => {
    try {
      const res = await fetch('/api/workspace/inspect', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path: '.' }),
      });
      if (res.ok) {
        const data = await res.json();
        setWorkspaceData(data);
        fetchWorkspaceTree('.');
      }
    } catch (err) {
      console.log('Workspace inspect error:', err);
    }
  };

  useEffect(() => {
    refreshLiveProviders();
    inspectWorkspace();
    fetchPrivacySettings();
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
    } catch (err) {
      setTestStatus({ type: 'error', message: '✗ Failed to reach server' });
      alert('Network error connecting to Rust backend server');
    } finally {
      setIsVerifying(false);
    }
  };

  const handleSendPrompt = async () => {
    if (!promptInput.trim()) return;

    const userMsg = promptInput.trim();
    setPromptInput('');

    const newTitle = activeChatTitle === 'New Session' || activeChatTitle === 'Octrex Workspace Engine' || activeChatTitle === 'Workspace Initialized'
      ? (userMsg.length > 26 ? `${userMsg.substring(0, 26)}...` : userMsg)
      : activeChatTitle;

    setActiveChatTitle(newTitle);

    const userMsgObj = { role: 'user' as const, content: userMsg };
    setChatMessages(prev => [...prev, userMsgObj]);
    setIsExecuting(true);

    try {
      const res = await fetch('/api/agent/execute', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          prompt: userMsg,
          provider_id: selectedProvider,
        })
      });

      let assistantMsgObj;
      if (res.ok) {
        const data = await res.json();
        if (data.task_id) {
          setLastTaskId(data.task_id);
          fetchTaskCompletion(data.task_id);
        }
        assistantMsgObj = {
          role: 'assistant' as const,
          content: data.output || 'Execution complete.',
          steps: [
            { label: 'Provider', detail: data.provider_used || selectedProvider, status: 'done' as const },
            { label: 'Execution', detail: `${data.execution_time_ms || 0}ms`, status: 'done' as const }
          ]
        };
      } else {
        const errData = await res.text();
        assistantMsgObj = {
          role: 'assistant' as const,
          content: `Error executing task: ${errData}`
        };
      }

      setChatMessages(prev => {
        const updated = [...prev, assistantMsgObj];
        setSessions(sPrev => sPrev.map(s => s.id === activeSessionId ? { ...s, title: newTitle, messages: updated } : s));
        return updated;
      });
    } catch (err: any) {
      const errObj = {
        role: 'assistant' as const,
        content: `Backend error: ${err.message}`
      };
      setChatMessages(prev => {
        const updated = [...prev, errObj];
        setSessions(sPrev => sPrev.map(s => s.id === activeSessionId ? { ...s, title: newTitle, messages: updated } : s));
        return updated;
      });
    } finally {
      setIsExecuting(false);
    }
  };

  // RENDER MODEL SETUP SCREEN
  if (viewMode === 'setup') {
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
              onClick={() => setViewMode('workspace')} 
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

  // RENDER MAIN OCTREX WORKSPACE STUDIO SCREEN
  return (
    <div className="min-h-screen bg-[#eaedf3] text-slate-900 flex overflow-hidden font-sans select-none p-3 gap-3">
      
      {/* 1. LEFT SIDEBAR NAVIGATION */}
      <aside className="w-64 bg-slate-100/70 backdrop-blur-xl border border-white/80 rounded-3xl p-4 flex flex-col justify-between shadow-sm">
        <div className="space-y-4">
          {/* Header & Logo */}
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2.5">
              <div className="w-8 h-8 rounded-xl overflow-hidden bg-slate-950 p-0.5 border border-cyan-500/40">
                <Image src="/logo.jpg" alt="Logo" width={32} height={32} className="w-full h-full object-cover rounded-lg" />
              </div>
              <span className="font-extrabold text-base tracking-tight text-slate-900">Octrex</span>
            </div>
            <button onClick={() => setViewMode('setup')} title="Switch Model Setup" className="text-slate-400 hover:text-slate-700 p-1 rounded-lg hover:bg-white/60">
              <RotateCcw className="w-4 h-4" />
            </button>
          </div>

          {/* New Chat Button */}
          <button 
            onClick={handleNewChat}
            className="w-full py-3 bg-white hover:bg-slate-50 border border-slate-200/90 rounded-2xl font-bold text-xs text-slate-900 shadow-2xs flex items-center justify-center space-x-2 transition-all active:scale-[0.98]"
          >
            <Plus className="w-4 h-4 text-slate-700" />
            <span>New chat</span>
          </button>

          {/* Search Bar */}
          <div className="relative">
            <Search className="w-4 h-4 text-slate-400 absolute left-3.5 top-3" />
            <input 
              type="text" 
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search chats" 
              className="w-full pl-9 pr-9 py-2.5 bg-white/60 border border-slate-200/80 rounded-2xl text-xs font-medium text-slate-800 focus:outline-none focus:bg-white focus:ring-2 focus:ring-slate-900/10 transition-all placeholder:text-slate-400"
            />
            {searchQuery ? (
              <button onClick={() => setSearchQuery('')} className="absolute right-3 top-3 text-slate-400 hover:text-slate-600">
                <X className="w-3.5 h-3.5" />
              </button>
            ) : (
              <span className="text-[10px] font-mono text-slate-400 absolute right-3 top-3">⌘K</span>
            )}
          </div>

          {/* Navigation Tree */}
          <div className="space-y-4 text-xs flex-1 overflow-y-auto">
            {/* Global Workspace Section */}
            <div>
              <div className="text-[10px] font-extrabold uppercase text-slate-400 tracking-wider mb-2 px-1">GLOBAL</div>
              <div 
                onClick={async () => {
                  try {
                    const res = await fetch('/api/workspace/inspect', {
                      method: 'POST',
                      headers: { 'Content-Type': 'application/json' },
                      body: JSON.stringify({ path: '.' }),
                    });
                    if (res.ok) {
                      const data = await res.json();
                      setWorkspaceData(data);
                    }
                  } catch (e) {
                    console.log('Global workspace switch error', e);
                  }
                }}
                className="flex items-center space-x-2.5 p-2.5 rounded-xl text-slate-600 hover:bg-white/70 hover:text-slate-900 cursor-pointer font-semibold transition-all"
              >
                <Globe className="w-4 h-4 text-slate-400 flex-shrink-0" />
                <span>Global workspace</span>
              </div>
            </div>

            {/* Projects Section */}
            <div>
              <div className="flex items-center justify-between text-[10px] font-extrabold uppercase text-slate-400 tracking-wider mb-2 px-1">
                <span>PROJECTS</span>
                <button 
                  onClick={() => setIsConnectFolderOpen(true)}
                  title="Connect folder..."
                  className="p-0.5 text-slate-400 hover:text-slate-700 hover:bg-white/60 rounded transition-all"
                >
                  <Plus className="w-3.5 h-3.5" />
                </button>
              </div>

              {/* Connected Active Project Card */}
              <div 
                onClick={() => setIsConnectFolderOpen(true)}
                className="font-semibold text-slate-900 p-3 bg-white/80 border border-slate-200/90 rounded-2xl flex items-center justify-between shadow-2xs hover:bg-white cursor-pointer transition-all"
              >
                <div className="flex items-center space-x-2.5 truncate">
                  <Folder className="w-4 h-4 text-slate-700 flex-shrink-0" />
                  <span className="truncate">{workspaceData ? workspaceData.name : 'Root'}</span>
                </div>
                {workspaceData && (
                  <span className="text-[10px] font-mono text-slate-400 font-normal whitespace-nowrap">
                    {workspaceData.total_files} files
                  </span>
                )}
              </div>
            </div>

            {/* Chat Sessions History List */}
            {sessions.length > 0 && (
              <div>
                <div className="flex items-center justify-between text-[10px] font-extrabold uppercase text-slate-400 tracking-wider mb-2 px-1">
                  <span>RECENT CHATS ({sessions.length})</span>
                </div>
                <div className="space-y-1">
                  {sessions
                    .filter(s => s.title.toLowerCase().includes(searchQuery.toLowerCase()))
                    .map((s) => {
                      const isActive = s.id === activeSessionId;
                      return (
                        <div
                          key={s.id}
                          onClick={() => handleSelectSession(s)}
                          className={`group p-2.5 rounded-xl flex items-center justify-between cursor-pointer transition-all ${
                            isActive
                              ? 'bg-white font-bold text-slate-900 shadow-2xs border border-slate-200/80'
                              : 'text-slate-600 hover:bg-white/60 font-medium'
                          }`}
                        >
                          <div className="flex items-center space-x-2 truncate">
                            <MessageSquare className={`w-3.5 h-3.5 flex-shrink-0 ${isActive ? 'text-slate-900' : 'text-slate-400'}`} />
                            <span className="truncate text-xs">{s.title}</span>
                          </div>
                          <button
                            onClick={(e) => handleDeleteSession(e, s.id)}
                            title="Delete chat"
                            className="opacity-0 group-hover:opacity-100 p-1 hover:text-rose-600 rounded-md transition-all"
                          >
                            <Trash2 className="w-3.5 h-3.5" />
                          </button>
                        </div>
                      );
                    })}
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Settings & Security Footer */}
        <div className="pt-3 border-t border-slate-200/60 space-y-1">
          <Link
            href="/tasks"
            className="flex items-center space-x-2.5 p-2 rounded-2xl text-slate-700 hover:bg-white/80 hover:text-slate-900 font-semibold text-xs transition-all"
          >
            <Layers className="w-4 h-4 text-blue-500" />
            <span>Task Orchestration</span>
          </Link>
          <div 
            onClick={() => setIsNetworkModalOpen(true)}
            className="flex items-center space-x-2.5 p-2 rounded-2xl text-slate-700 hover:bg-white/80 hover:text-slate-900 cursor-pointer font-semibold text-xs transition-all"
          >
            <Shield className="w-4 h-4 text-cyan-600" />
            <span>Network Security</span>
          </div>
          <div 
            onClick={() => setIsSettingsOpen(true)}
            className="flex items-center space-x-2.5 p-2 rounded-2xl text-slate-600 hover:bg-white/80 hover:text-slate-900 cursor-pointer font-semibold text-xs transition-all"
          >
            <Settings className="w-4 h-4 text-slate-500" />
            <span>Settings</span>
          </div>
        </div>
      </aside>

      {/* 2. CENTER MAIN CHAT CANVAS */}
      <main className="flex-1 flex flex-col bg-slate-50/60 backdrop-blur-xl border border-white/80 rounded-3xl overflow-hidden shadow-sm">
        
        {/* Top Header */}
        <header className="h-14 px-6 border-b border-slate-200/60 flex items-center justify-between bg-white/40">
          <div className="flex items-center space-x-3">
            <h2 className="text-sm font-bold text-slate-900">{activeChatTitle}</h2>
            <span className="px-2 py-0.5 rounded-lg bg-slate-200/60 text-slate-600 text-[11px] font-mono border border-slate-300/50 flex items-center space-x-1">
              <Folder className="w-3 h-3 text-slate-500" />
              <span>octrex-web</span>
            </span>
          </div>
          <div className="flex items-center space-x-3">
            <PrivacyBadge mode={privacyMode} classification={workspaceClassification} />
            <div onClick={() => setIsNetworkModalOpen(true)} className="cursor-pointer">
              <NetworkSecurityBadge mode={networkMode} />
            </div>
            <div className="flex items-center space-x-2">
              <span className="text-xs text-slate-400 font-medium">Model:</span>
              <div className="px-3 py-1 rounded-xl bg-white border border-slate-200/80 text-xs font-semibold text-slate-800 shadow-xs flex items-center space-x-1.5 cursor-pointer">
                <span>{PROVIDER_INFO[selectedProvider]?.name} • {activeModel}</span>
                <ChevronDown className="w-3.5 h-3.5 text-slate-500" />
              </div>
            </div>
          </div>
        </header>

        {/* Chat Feed */}
        <div className="flex-1 p-6 overflow-y-auto space-y-6">
          {chatMessages.length === 0 ? (
            <div className="h-full flex flex-col items-center justify-center text-center p-8 space-y-4 max-w-md mx-auto my-auto">
              <div className="w-14 h-14 rounded-2xl bg-white border border-slate-200/80 shadow-xs flex items-center justify-center font-bold text-xl text-slate-900">
                O
              </div>
              <div className="space-y-1">
                <h3 className="text-base font-bold text-slate-900">What would you like Octrex to build?</h3>
                <p className="text-xs text-slate-500 font-medium leading-relaxed">
                  Type a prompt below to execute code, run commands, or analyze documents with the native Rust backend.
                </p>
              </div>
            </div>
          ) : (
            chatMessages.map((msg, idx) => (
              <div key={idx} className={`flex flex-col ${msg.role === 'user' ? 'items-end' : 'items-start'}`}>
                
                {/* User Bubble */}
                {msg.role === 'user' && (
                  <div className="max-w-xl bg-white border border-slate-200/80 rounded-2xl p-4 text-xs font-medium text-slate-800 shadow-xs">
                    {msg.content}
                  </div>
                )}

                {/* Assistant Bubble & Agent Execution Block */}
                {msg.role === 'assistant' && (
                  <div className="max-w-2xl space-y-3">
                    <div className="flex items-center space-x-2">
                      <div className="w-5 h-5 rounded-md bg-slate-900 text-white font-bold text-[10px] flex items-center justify-center">O</div>
                      <p className="text-xs font-medium text-slate-700 leading-relaxed">{msg.content}</p>
                    </div>

                    {/* Execution Steps List */}
                    {msg.steps && (
                      <div className="bg-slate-100/80 border border-slate-200/70 rounded-2xl p-4 space-y-2.5 font-mono text-xs">
                        {msg.steps.map((st, i) => (
                          <div key={i} className="flex items-center justify-between">
                            <div className="flex items-center space-x-2">
                              {st.status === 'done' ? (
                                <Check className="w-3.5 h-3.5 text-slate-700" />
                              ) : (
                                <span className="w-2 h-2 rounded-full bg-cyan-500 animate-ping" />
                              )}
                              <span className="font-bold text-slate-900">{st.label}</span>
                              <span className="text-slate-500">{st.detail}</span>
                            </div>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                )}
              </div>
            ))
          )}
        </div>

        {/* Floating Command Bar */}
        <div className="p-4 bg-white/40 border-t border-slate-200/60 space-y-2">
          <ContextBudgetIndicator
            usedTokens={1024}
            usableBudget={6568}
            contextWindow={8192}
            tokenCountKind="estimated"
            modelId={activeModel}
          />
          <div className="bg-white border border-slate-200/90 rounded-2xl p-2.5 shadow-sm flex items-center space-x-2">
            <input 
              type="text" 
              value={promptInput}
              onChange={(e) => setPromptInput(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleSendPrompt()}
              placeholder="Ask Octrex to build, write or edit anything..."
              className="flex-1 px-3 py-1 text-xs font-medium text-slate-800 focus:outline-none placeholder:text-slate-400"
            />
            <div className="flex items-center space-x-1.5">
              <button className="p-1.5 rounded-xl hover:bg-slate-100 text-slate-400 hover:text-slate-600">
                <Plus className="w-4 h-4" />
              </button>
              <button className="px-2.5 py-1 rounded-xl bg-slate-100 text-[11px] font-semibold text-slate-600 hover:bg-slate-200">
                @ Skills
              </button>
              <button className="px-2.5 py-1 rounded-xl bg-slate-100 text-[11px] font-semibold text-slate-600 hover:bg-slate-200">
                Plan
              </button>
              <button 
                onClick={handleSendPrompt}
                disabled={isExecuting}
                className="w-8 h-8 rounded-xl bg-slate-900 text-white flex items-center justify-center hover:bg-slate-800 transition-all disabled:opacity-50"
              >
                <ArrowUp className="w-4 h-4" />
              </button>
            </div>
          </div>
        </div>

      </main>

      {/* 3. RIGHT INSPECTOR SIDEBAR */}
      <aside className="w-80 bg-slate-100/70 backdrop-blur-xl border border-white/80 rounded-3xl p-4 flex flex-col justify-between shadow-sm">
        <div>
          {/* Tab Navigation */}
          <div className="flex items-center bg-white/60 p-1 rounded-2xl border border-slate-200/60 mb-5 text-xs font-semibold text-slate-600">
            <button 
              onClick={() => setActiveTab('activity')}
              className={`flex-1 py-1.5 rounded-xl transition-all ${activeTab === 'activity' ? 'bg-white font-bold text-slate-900 shadow-2xs' : 'hover:text-slate-900'}`}
            >
              Activity
            </button>
            <button 
              onClick={() => setActiveTab('files')}
              className={`flex-1 py-1.5 rounded-xl transition-all ${activeTab === 'files' ? 'bg-white font-bold text-slate-900 shadow-2xs' : 'hover:text-slate-900'}`}
            >
              Files
            </button>
            <button 
              onClick={() => setActiveTab('terminal')}
              className={`flex-1 py-1.5 rounded-xl transition-all ${activeTab === 'terminal' ? 'bg-white font-bold text-slate-900 shadow-2xs' : 'hover:text-slate-900'}`}
            >
              Terminal
            </button>
            <button 
              onClick={() => setActiveTab('changes')}
              className={`flex-1 py-1.5 rounded-xl transition-all ${activeTab === 'changes' ? 'bg-white font-bold text-slate-900 shadow-2xs' : 'hover:text-slate-900'}`}
            >
              Changes
            </button>
          </div>

          {/* TAB 1: ACTIVITY PANEL */}
          {activeTab === 'activity' && (
            <div className="space-y-5 text-xs">
              {/* Active Agents */}
              <div>
                <div className="flex items-center justify-between text-[10px] font-extrabold uppercase text-slate-400 tracking-wider mb-2">
                  <span>AGENTS</span>
                  <span className="px-2 py-0.5 rounded-full bg-slate-200 text-slate-700 font-mono text-[10px]">
                    {isExecuting ? '1 active' : '0 active'}
                  </span>
                </div>
                <div className="space-y-2">
                  <div className="p-3 rounded-2xl bg-white border border-slate-200/70 shadow-2xs">
                    <div className="flex items-center space-x-2 font-bold text-slate-900">
                      <span className={`w-2 h-2 rounded-full ${isExecuting ? 'bg-cyan-500 animate-ping' : 'bg-emerald-500'}`} />
                      <span>Octrex Engine Agent</span>
                    </div>
                    <div className="text-[11px] text-slate-500 pl-4 mt-0.5 font-mono">
                      {isExecuting ? 'Executing task...' : 'Idle • Ready for prompt'}
                    </div>
                  </div>
                </div>
              </div>

              {/* Skills */}
              <div>
                <div className="text-[10px] font-extrabold uppercase text-slate-400 tracking-wider mb-2">ENGINE CAPABILITIES</div>
                <div className="flex flex-wrap gap-1.5">
                  <span className="px-3 py-1 rounded-full bg-white border border-slate-200/80 font-medium text-slate-700">code-executor</span>
                  <span className="px-3 py-1 rounded-full bg-white border border-slate-200/80 font-medium text-slate-700">workspace-manager</span>
                  <span className="px-3 py-1 rounded-full bg-white border border-slate-200/80 font-medium text-slate-700">rust-core</span>
                </div>
              </div>

              {/* Created Files */}
              <div>
                <div className="text-[10px] font-extrabold uppercase text-slate-400 tracking-wider mb-2">CREATED FILES</div>
                <div className="p-3 rounded-xl bg-white border border-slate-200/70 text-center text-slate-400 font-medium text-xs">
                  No files generated in current session
                </div>
              </div>

              {/* Verification (Phase 13) */}
              <div>
                <div className="text-[10px] font-extrabold uppercase text-slate-400 tracking-wider mb-2">VERIFICATION</div>
                {!lastTaskId ? (
                  <div className="p-3 rounded-xl bg-white border border-slate-200/70 text-center text-slate-400 font-medium text-xs">
                    No task executed yet — completion is never assumed
                  </div>
                ) : (
                  <div className="p-3 rounded-xl bg-white border border-slate-200/70 space-y-1.5">
                    <div className="flex items-center justify-between">
                      <span className="text-xs font-bold text-slate-900">
                        {taskCompletion?.verified ? '✓ Verified' : '○ Not verified'}
                      </span>
                      <span className="text-[10px] font-mono text-slate-500">
                        {taskCompletion?.status || 'NOT_VERIFIED'}
                      </span>
                    </div>
                    <div className="text-[11px] text-slate-500 font-mono truncate" title={lastTaskId}>
                      {lastTaskId}
                    </div>
                    {taskCompletion && (
                      <div className="text-[11px] text-slate-600">
                        Gate: <span className="font-mono font-semibold">{taskCompletion.completion_gate}</span>
                        {taskCompletion.failures.length > 0 && (
                          <span className="text-rose-600"> · {taskCompletion.failures.length} failure(s)</span>
                        )}
                      </div>
                    )}
                    <Link
                      href={`/tasks/verification?taskId=${encodeURIComponent(lastTaskId)}`}
                      className="inline-block text-[11px] font-bold text-cyan-700 hover:text-cyan-900 hover:underline"
                    >
                      Open verification evidence →
                    </Link>
                  </div>
                )}
              </div>
            </div>
          )}

          {/* TAB 2: FILES PANEL */}
          {activeTab === 'files' && (
            <div className="space-y-3 text-xs flex-1 overflow-y-auto">
              <div className="p-3 bg-white rounded-2xl border border-slate-200/70 shadow-2xs flex items-center justify-between">
                <div className="flex items-center space-x-2">
                  <Folder className="w-4 h-4 text-slate-600" />
                  <span className="font-bold text-slate-900">{workspaceData ? workspaceData.name : 'OCTREX'}</span>
                </div>
                <span className="font-mono text-[10px] text-slate-400">{workspaceData ? `${workspaceData.total_files} total files` : '0 files'}</span>
              </div>

              <div className="space-y-1">
                {workspaceTree.map((entry) => (
                  <div
                    key={entry.rel_path}
                    onClick={() => handleOpenFile(entry)}
                    className={`p-2.5 rounded-xl border flex items-center justify-between transition-all ${
                      entry.is_dir 
                        ? 'bg-slate-100/60 border-slate-200/60 font-semibold text-slate-800' 
                        : 'bg-white border-slate-200/80 hover:bg-slate-50 cursor-pointer font-medium text-slate-700 shadow-2xs hover:border-slate-300'
                    }`}
                  >
                    <div className="flex items-center space-x-2 truncate">
                      {entry.is_dir ? (
                        <Folder className="w-3.5 h-3.5 text-slate-500 flex-shrink-0" />
                      ) : (
                        <FileText className="w-3.5 h-3.5 text-cyan-600 flex-shrink-0" />
                      )}
                      <span className="truncate text-xs font-mono">{entry.rel_path}</span>
                    </div>
                    {!entry.is_dir && (
                      <span className="text-[10px] font-mono text-slate-400 flex-shrink-0 ml-2">
                        {entry.size_bytes > 1024 ? `${(entry.size_bytes / 1024).toFixed(1)} KB` : `${entry.size_bytes} B`}
                      </span>
                    )}
                  </div>
                ))}
                {workspaceTree.length === 0 && (
                  <div className="p-4 rounded-2xl bg-white border border-slate-200/70 text-center text-slate-400 font-medium">
                    No files found in current workspace.
                  </div>
                )}
              </div>
            </div>
          )}

          {/* TAB 3: TERMINAL PANEL */}
          {activeTab === 'terminal' && (
            <div className="space-y-3">
              <div className="p-3 bg-slate-950 text-emerald-400 rounded-2xl font-mono text-[11px] space-y-1.5 min-h-48 overflow-y-auto">
                <div className="text-slate-500">$ octrex-server --port 3000</div>
                <div className="text-cyan-400">✓ Native Rust backend engine connected</div>
                <div className="text-slate-400">Ready for terminal commands</div>
              </div>
              <div className="flex justify-between items-center text-xs text-slate-500 font-mono">
                <span>Status: Connected</span>
                <span className="text-[10px] text-emerald-600 font-bold">● Server Online</span>
              </div>
            </div>
          )}

          {/* TAB 4: CHANGES PANEL */}
          {activeTab === 'changes' && (
            <div className="p-4 rounded-2xl bg-white border border-slate-200/70 text-center text-slate-400 font-medium text-xs">
              No uncommitted workspace changes detected
            </div>
          )}
        </div>

        {/* Context Bar Footer */}
        <div className="pt-3 border-t border-slate-200/60 text-[11px]">
          <div className="text-slate-400 font-extrabold uppercase text-[9px] mb-1">CONTEXT</div>
          <div className="w-full bg-slate-200 rounded-full h-1.5 overflow-hidden mb-1">
            <div className="bg-slate-900 h-full w-[42%]" />
          </div>
          <div className="text-slate-500 font-medium flex justify-between">
            <span>42% of context used</span>
            <span className="font-mono">84k / 200k</span>
          </div>
        </div>
      </aside>

      {/* CONNECT FOLDER MODAL */}
      {isConnectFolderOpen && (
        <div className="fixed inset-0 z-50 bg-slate-950/40 backdrop-blur-md flex items-center justify-center p-4">
          <div className="w-[440px] bg-white/95 backdrop-blur-2xl border border-slate-200 rounded-3xl p-6 shadow-2xl space-y-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-2">
                <FolderPlus className="w-5 h-5 text-slate-900" />
                <h3 className="text-base font-bold text-slate-900">Connect Workspace Folder</h3>
              </div>
              <button onClick={() => setIsConnectFolderOpen(false)} className="p-1 text-slate-400 hover:text-slate-700 rounded-lg">
                <X className="w-4 h-4" />
              </button>
            </div>
            <p className="text-xs text-slate-500 font-medium leading-relaxed">
              Enter absolute directory path on your computer (e.g. <span className="font-mono text-slate-700">d:\OCTREX\crates</span>) to inspect project files.
            </p>
            <div className="space-y-2">
              <input 
                type="text" 
                value={customFolderPath}
                onChange={(e) => setCustomFolderPath(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleConnectFolder()}
                placeholder="Path to folder..."
                className="w-full px-4 py-3 rounded-2xl text-xs font-mono text-slate-900 bg-slate-100/80 border border-slate-200 focus:bg-white focus:outline-none focus:ring-2 focus:ring-slate-900/10 transition-all"
              />
            </div>
            <div className="flex items-center justify-end space-x-2 pt-2">
              <button 
                onClick={() => setIsConnectFolderOpen(false)} 
                className="px-4 py-2 rounded-xl text-xs font-bold text-slate-600 hover:bg-slate-100 transition-all"
              >
                Cancel
              </button>
              <button 
                onClick={handleConnectFolder} 
                className="px-5 py-2 rounded-xl text-xs font-bold bg-slate-900 text-white hover:bg-slate-800 shadow-sm transition-all"
              >
                Inspect & Connect
              </button>
            </div>
          </div>
        </div>
      )}

      {/* SETTINGS MODAL */}
      {isSettingsOpen && (
        <div className="fixed inset-0 z-50 bg-slate-950/40 backdrop-blur-md flex items-center justify-center p-4">
          <div className="w-[500px] bg-white/95 backdrop-blur-2xl border border-slate-200 rounded-3xl p-6 shadow-2xl space-y-5">
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-2">
                <Settings className="w-5 h-5 text-slate-900" />
                <h3 className="text-base font-bold text-slate-900">Octrex Settings</h3>
              </div>
              <button onClick={() => setIsSettingsOpen(false)} className="p-1 text-slate-400 hover:text-slate-700 rounded-lg">
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="space-y-3 text-xs">
              <div className="p-3 bg-slate-100/80 rounded-2xl space-y-1">
                <div className="font-bold text-slate-900">Active Engine</div>
                <div className="text-slate-500 font-mono">octrex-server (Rust) • http://127.0.0.1:3000</div>
              </div>

              <div className="space-y-1.5">
                <label className="block text-[11px] font-extrabold uppercase text-slate-400 tracking-wider">Active Provider</label>
                <div className="p-3 bg-white border border-slate-200 rounded-2xl flex items-center justify-between">
                  <div className="font-bold text-slate-900">{PROVIDER_INFO[selectedProvider]?.name}</div>
                  <button onClick={() => { setIsSettingsOpen(false); setViewMode('setup'); }} className="text-xs text-slate-600 font-bold hover:underline">
                    Change Provider
                  </button>
                </div>
              </div>

              <div className="space-y-1.5">
                <label className="block text-[11px] font-extrabold uppercase text-slate-400 tracking-wider">Default Model</label>
                <div className="p-3 bg-white border border-slate-200 rounded-2xl font-mono font-semibold text-slate-800">
                  {activeModel}
                </div>
              </div>

              <div className="space-y-1.5 pt-2 border-t border-slate-200/60">
                <label className="block text-[11px] font-extrabold uppercase text-slate-400 tracking-wider">Privacy & Policy Governance</label>
                <div className="grid grid-cols-2 gap-2">
                  <Link 
                    href="/settings/privacy" 
                    onClick={() => setIsSettingsOpen(false)}
                    className="p-3 rounded-2xl bg-slate-900 text-white font-bold text-center hover:bg-slate-800 transition-all flex items-center justify-center space-x-1.5"
                  >
                    <Shield className="w-4 h-4 text-cyan-400" />
                    <span>Privacy Gate</span>
                  </Link>
                  <Link 
                    href="/settings/policy" 
                    onClick={() => setIsSettingsOpen(false)}
                    className="p-3 rounded-2xl bg-white border border-slate-200 text-slate-900 font-bold text-center hover:bg-slate-50 transition-all flex items-center justify-center space-x-1.5 shadow-xs"
                  >
                    <Sliders className="w-4 h-4 text-slate-600" />
                    <span>Policy Status</span>
                  </Link>
                </div>
              </div>
            </div>

            <div className="flex items-center justify-end pt-2">
              <button 
                onClick={() => setIsSettingsOpen(false)} 
                className="px-5 py-2.5 rounded-xl text-xs font-bold bg-slate-900 text-white hover:bg-slate-800 shadow-sm transition-all"
              >
                Close Settings
              </button>
            </div>
          </div>
        </div>
      )}

      {/* FILE EDITOR & INSPECTOR MODAL */}
      {isFileEditorOpen && selectedFile && (
        <div className="fixed inset-0 z-50 bg-slate-950/40 backdrop-blur-md flex items-center justify-center p-4">
          <div className="w-[720px] max-h-[85vh] bg-white/95 backdrop-blur-2xl border border-slate-200 rounded-3xl p-6 shadow-2xl flex flex-col space-y-4">
            <div className="flex items-center justify-between border-b border-slate-200/80 pb-3">
              <div className="flex items-center space-x-2.5">
                <FileText className="w-5 h-5 text-slate-900" />
                <div>
                  <h3 className="text-sm font-bold text-slate-900 font-mono">{selectedFile.rel_path}</h3>
                  <span className="text-[10px] text-slate-500 font-mono">
                    {selectedFile.size_bytes} bytes • Path: {selectedFile.path}
                  </span>
                </div>
              </div>
              <button onClick={() => setIsFileEditorOpen(false)} className="p-1 text-slate-400 hover:text-slate-700 rounded-lg">
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="flex-1 min-h-[360px] flex flex-col space-y-2">
              <label className="text-[11px] font-extrabold uppercase text-slate-400 tracking-wider">File Content (Editable)</label>
              <textarea 
                value={fileContent}
                onChange={(e) => setFileContent(e.target.value)}
                className="flex-1 w-full p-4 rounded-2xl bg-slate-950 text-emerald-400 font-mono text-xs focus:outline-none resize-none leading-relaxed overflow-y-auto"
                rows={16}
              />
            </div>

            <div className="flex items-center justify-between pt-2 border-t border-slate-200/80">
              <span className="text-[11px] text-slate-400 font-medium">
                Changes saved directly to your local file system via Rust backend.
              </span>
              <div className="flex items-center space-x-2">
                <button 
                  onClick={() => setIsFileEditorOpen(false)} 
                  className="px-4 py-2 rounded-xl text-xs font-bold text-slate-600 hover:bg-slate-100 transition-all"
                >
                  Cancel
                </button>
                <button 
                  onClick={handleSaveFile} 
                  disabled={isSavingFile}
                  className="px-5 py-2 rounded-xl text-xs font-bold bg-slate-900 text-white hover:bg-slate-800 shadow-sm transition-all disabled:opacity-50"
                >
                  {isSavingFile ? 'Saving to disk...' : 'Save Changes to Disk'}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Network Security Modal (Phase 7) */}
      <NetworkSecurityModal 
        isOpen={isNetworkModalOpen} 
        onClose={() => setIsNetworkModalOpen(false)} 
      />

      {/* Online Consent Dialog (Phase 6) */}
      {activeConsentRequest && (
        <OnlineConsentDialog
          request={activeConsentRequest}
          onApprove={async (id, reason) => {
            await fetch('/api/privacy/consent', {
              method: 'POST',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({ consent_id: id, granted: true, reason }),
            });
            setActiveConsentRequest(null);
          }}
          onDeny={async (id, reason) => {
            await fetch('/api/privacy/consent', {
              method: 'POST',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({ consent_id: id, granted: false, reason }),
            });
            setActiveConsentRequest(null);
          }}
          onClose={() => setActiveConsentRequest(null)}
        />
      )}

    </div>
  );
}
