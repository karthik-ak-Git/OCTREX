import React, { useState } from 'react';
import {
  ArrowLeft,
  Settings as SettingsIcon,
  Moon,
  Sun,
  Laptop,
  Cpu,
  Layers,
  Network,
  ShieldCheck,
  Search,
  Plus,
  Trash2,
  Check,
  X
} from 'lucide-react';
import { providerDefinitions, ProviderDefinition } from '../../../shared/provider-registry';
import { ChatSession, SkillItem, McpServerItem } from '../state';

export type SettingsHubTab =
  | 'general'
  | 'appearance'
  | 'updates'
  | 'models'
  | 'sessions'
  | 'skills'
  | 'mcp'
  | 'permissions'
  | 'chats';

export type SettingsHubProps = {
  isOpen: boolean;
  onClose: () => void;
  initialTab?: SettingsHubTab;
  sessions: ChatSession[];
  onSelectSession: (id: string) => void;
  onDeleteSession: (id: string) => void;
  skills: SkillItem[];
  onToggleSkill: (id: string) => void;
  onAddSkill: (name: string, desc: string, systemPrompt?: string) => void;
  mcpServers: McpServerItem[];
  onToggleMcpServer: (id: string) => void;
  onAddMcpServer: (name: string, urlOrCmd: string, transport: 'HTTP' | 'stdio') => void;
  connectedProviders: Record<string, { apiKey?: string; connected: boolean }>;
  onConnectProvider: (providerId: string, apiKey: string) => Promise<boolean>;
};

export const SettingsHub: React.FC<SettingsHubProps> = ({
  isOpen,
  onClose,
  initialTab = 'sessions',
  sessions,
  onSelectSession,
  onDeleteSession,
  skills,
  onToggleSkill,
  onAddSkill,
  mcpServers,
  onToggleMcpServer,
  onAddMcpServer,
  connectedProviders,
  onConnectProvider
}) => {
  const [activeTab, setActiveTab] = useState<SettingsHubTab>(initialTab);
  const [skillSearch, setSkillSearch] = useState('');
  const [activeProviderId, setActiveProviderId] = useState<string>('google');
  const [providerApiKey, setProviderApiKey] = useState('');
  const [connecting, setConnecting] = useState(false);
  const [theme, setTheme] = useState<'light' | 'dark' | 'system'>('light');

  // Import Skill modal
  const [isImportSkillOpen, setIsImportSkillOpen] = useState(false);
  const [newSkillName, setNewSkillName] = useState('');
  const [newSkillDesc, setNewSkillDesc] = useState('');

  // Add MCP server modal
  const [isAddMcpOpen, setIsAddMcpOpen] = useState(false);
  const [newMcpName, setNewMcpName] = useState('');
  const [newMcpUrl, setNewMcpUrl] = useState('');
  const [newMcpTransport, setNewMcpTransport] = useState<'HTTP' | 'stdio'>('HTTP');

  if (!isOpen) return null;

  const filteredSkills = skills.filter(
    (s) =>
      s.name.toLowerCase().includes(skillSearch.toLowerCase()) ||
      s.desc.toLowerCase().includes(skillSearch.toLowerCase())
  );

  const handleConnectProvider = async (pId: string) => {
    setConnecting(true);
    await onConnectProvider(pId, providerApiKey || 'demo-connected-key');
    setConnecting(false);
  };

  const navItems = [
    { id: 'sessions' as const, label: 'Sessions', icon: SettingsIcon },
    { id: 'models' as const, label: 'Models & Providers', icon: Cpu },
    { id: 'skills' as const, label: 'Skills', icon: Layers },
    { id: 'mcp' as const, label: 'MCP Connections', icon: Network },
    { id: 'permissions' as const, label: 'Chats & Permissions', icon: ShieldCheck },
    { id: 'appearance' as const, label: 'Appearance', icon: Sun },
    { id: 'general' as const, label: 'General', icon: SettingsIcon }
  ];

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/40 backdrop-blur-md p-6 select-none animate-in fade-in duration-200">
      <div className="flex h-[660px] w-[1040px] max-w-[95vw] overflow-hidden rounded-3xl glass-modal shadow-2xl border border-white/80">
        {/* LEFT SETTINGS SIDEBAR */}
        <div className="w-60 border-r border-slate-200/60 bg-white/40 p-4 flex flex-col justify-between">
          <div className="space-y-4">
            <button
              onClick={onClose}
              className="flex items-center gap-2 text-xs font-semibold text-slate-700 hover:text-slate-950 p-1.5 rounded-xl hover:bg-white/80 transition-all cursor-pointer"
            >
              <ArrowLeft size={16} />
              <span>Back to canvas</span>
            </button>

            <div className="space-y-1">
              <div className="text-[10px] font-semibold tracking-wider text-slate-400 uppercase px-3 py-1">
                Preferences
              </div>
              {navItems.map((item) => {
                const Icon = item.icon;
                const isActive = activeTab === item.id;
                return (
                  <button
                    key={item.id}
                    onClick={() => setActiveTab(item.id)}
                    className={`flex w-full items-center gap-2.5 rounded-2xl px-3 py-2 text-xs font-medium transition-all cursor-pointer ${
                      isActive
                        ? 'bg-white text-slate-950 font-semibold shadow-2xs border border-slate-200/80'
                        : 'text-slate-600 hover:bg-white/60 hover:text-slate-900'
                    }`}
                  >
                    <Icon size={14} className={isActive ? 'text-cyan-600' : 'text-slate-400'} />
                    <span>{item.label}</span>
                  </button>
                );
              })}
            </div>
          </div>

          <div className="text-[10px] font-mono text-slate-400 px-3">
            OCTREX CODE V4.0.0
          </div>
        </div>

        {/* RIGHT CONTENT WORKSPACE */}
        <div className="flex-1 overflow-y-auto p-8 custom-scrollbar bg-slate-50/40">
          {/* SESSIONS TAB (FIGMA IMAGE 8) */}
          {activeTab === 'sessions' && (
            <div className="space-y-6">
              <div>
                <h3 className="text-xl font-bold tracking-tight text-slate-900">Sessions</h3>
                <p className="mt-1 text-xs text-slate-500">
                  Manage active and historical multi-agent conversations across your workspaces.
                </p>
              </div>

              <div className="rounded-2xl glass-card border border-white/80 p-2 shadow-2xs">
                <div className="grid grid-cols-12 px-4 py-2 text-[10px] font-bold tracking-wider text-slate-400 uppercase">
                  <div className="col-span-5">Session</div>
                  <div className="col-span-3">Workspace</div>
                  <div className="col-span-2">Started</div>
                  <div className="col-span-2 text-right">Actions</div>
                </div>

                <div className="divide-y divide-slate-100">
                  {sessions.map((s) => (
                    <div
                      key={s.id}
                      className="grid grid-cols-12 items-center px-4 py-3 text-xs hover:bg-white/80 rounded-xl transition-colors"
                    >
                      <div className="col-span-5 font-semibold text-slate-900 truncate pr-2">
                        {s.title}
                      </div>
                      <div className="col-span-3 font-mono text-[11px] text-slate-500 truncate">
                        {s.workspaceName}
                      </div>
                      <div className="col-span-2 text-slate-400 text-[11px]">{s.startedAt}</div>
                      <div className="col-span-2 flex items-center justify-end gap-1.5">
                        <button
                          onClick={() => {
                            onSelectSession(s.id);
                            onClose();
                          }}
                          className="rounded-xl bg-slate-900 hover:bg-slate-800 text-white px-2.5 py-1 text-[11px] font-medium shadow-2xs transition-all cursor-pointer"
                        >
                          Resume
                        </button>
                        <button
                          onClick={() => onDeleteSession(s.id)}
                          className="p-1 rounded-lg text-slate-400 hover:text-rose-500 hover:bg-rose-50 transition-colors cursor-pointer"
                          title="Delete session"
                        >
                          <Trash2 size={13} />
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}

          {/* MODELS TAB (FIGMA IMAGE 2 & 10) */}
          {activeTab === 'models' && (
            <div className="space-y-6">
              <div>
                <h3 className="text-xl font-bold tracking-tight text-slate-900">AI Models & Providers</h3>
                <p className="mt-1 text-xs text-slate-500">
                  Configure Google Gemini, NVIDIA NIM, OpenAI, Anthropic, OpenRouter, Groq, Ollama, and Free OpenCode Gateway.
                </p>
              </div>

              {/* API KEY INPUT */}
              <div className="rounded-2xl glass-card border border-white/80 p-4 shadow-2xs space-y-2">
                <label className="text-xs font-semibold text-slate-800">
                  API Key / Endpoint Token
                </label>
                <div className="flex gap-2">
                  <input
                    type="password"
                    value={providerApiKey}
                    onChange={(e) => setProviderApiKey(e.target.value)}
                    placeholder="Paste provider key (sk-...) or custom endpoint URI"
                    className="flex-1 rounded-xl border border-slate-200 bg-white px-3 py-2 text-xs text-slate-800 placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-cyan-500/20"
                  />
                  <button
                    onClick={() => handleConnectProvider(activeProviderId)}
                    disabled={connecting}
                    className="rounded-xl bg-slate-950 hover:bg-slate-800 text-white px-4 py-2 text-xs font-semibold shadow-sm transition-all cursor-pointer disabled:opacity-50"
                  >
                    {connecting ? 'Connecting...' : 'Save & Connect'}
                  </button>
                </div>
              </div>

              {/* PROVIDERS GRID */}
              <div className="space-y-2.5">
                {providerDefinitions.map((prov: ProviderDefinition) => {
                  const isConnected = !!connectedProviders[prov.id]?.connected;
                  const isSelected = activeProviderId === prov.id;

                  return (
                    <div
                      key={prov.id}
                      onClick={() => setActiveProviderId(prov.id)}
                      className={`flex items-center justify-between rounded-2xl p-3.5 border transition-all cursor-pointer ${
                        isSelected
                          ? 'glass-card border-slate-900 shadow-sm'
                          : 'bg-white/60 border-slate-200/80 hover:bg-white'
                      }`}
                    >
                      <div className="flex items-center gap-3">
                        <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-slate-900 font-bold text-xs text-white shadow-2xs">
                          {prov.logo}
                        </div>
                        <div>
                          <div className="text-xs font-bold text-slate-900">{prov.name}</div>
                          <div className="text-[11px] text-slate-500">{prov.description}</div>
                        </div>
                      </div>

                      <div className="flex items-center gap-2">
                        {isConnected ? (
                          <span className="rounded-full bg-emerald-100 text-emerald-800 font-semibold px-2.5 py-0.5 text-[10px] flex items-center gap-1">
                            <Check size={11} /> Connected
                          </span>
                        ) : (
                          <button
                            onClick={(e) => {
                              e.stopPropagation();
                              setActiveProviderId(prov.id);
                              handleConnectProvider(prov.id);
                            }}
                            className="rounded-full bg-slate-100 hover:bg-slate-200 text-slate-800 font-medium px-3 py-1 text-xs transition-colors cursor-pointer"
                          >
                            Connect
                          </button>
                        )}
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          )}

          {/* SKILLS TAB (FIGMA IMAGE 9) */}
          {activeTab === 'skills' && (
            <div className="space-y-6">
              <div className="flex items-center justify-between">
                <div>
                  <h3 className="text-xl font-bold tracking-tight text-slate-900">Skills & Automations</h3>
                  <p className="mt-1 text-xs text-slate-500">
                    Extend Octrex with specialized agents, document generators, and coding conventions.
                  </p>
                </div>
                <button
                  onClick={() => setIsImportSkillOpen(true)}
                  className="flex items-center gap-1.5 rounded-xl bg-slate-950 hover:bg-slate-800 text-white px-3 py-1.5 text-xs font-semibold shadow-2xs transition-all cursor-pointer"
                >
                  <Plus size={13} />
                  <span>Import Skill</span>
                </button>
              </div>

              {/* SEARCH FILTER */}
              <div className="relative">
                <Search size={14} className="absolute left-3.5 top-1/2 -translate-y-1/2 text-slate-400" />
                <input
                  type="text"
                  value={skillSearch}
                  onChange={(e) => setSkillSearch(e.target.value)}
                  placeholder="Search installed skills (pptx, frontend, pdf)..."
                  className="w-full rounded-2xl glass-card pl-9 pr-4 py-2 text-xs text-slate-800 placeholder-slate-400 focus:outline-none"
                />
              </div>

              {/* SKILLS GRID */}
              <div className="grid grid-cols-2 gap-3">
                {filteredSkills.map((sk) => (
                  <div
                    key={sk.id}
                    className="rounded-2xl glass-card border border-white/80 p-4 shadow-2xs space-y-3 flex flex-col justify-between"
                  >
                    <div>
                      <div className="flex items-center justify-between">
                        <span className="font-mono text-xs font-bold text-slate-900">{sk.name}</span>
                        <span className="text-[10px] rounded-md bg-slate-100 text-slate-500 px-1.5 py-0.5 font-medium">
                          {sk.type}
                        </span>
                      </div>
                      <p className="mt-1 text-[11px] text-slate-500 leading-relaxed">{sk.desc}</p>
                    </div>

                    <div className="flex items-center justify-between pt-2 border-t border-slate-100">
                      <span className="text-[11px] font-medium text-slate-400">
                        {sk.enabled ? 'Enabled' : 'Disabled'}
                      </span>
                      <button
                        onClick={() => onToggleSkill(sk.id)}
                        className={`h-5 w-9 rounded-full transition-colors cursor-pointer relative p-0.5 ${
                          sk.enabled ? 'bg-slate-900' : 'bg-slate-200'
                        }`}
                      >
                        <div
                          className={`h-4 w-4 rounded-full bg-white transition-transform ${
                            sk.enabled ? 'translate-x-4' : 'translate-x-0'
                          }`}
                        />
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* MCP CONNECTIONS TAB (FIGMA IMAGE 10) */}
          {activeTab === 'mcp' && (
            <div className="space-y-6">
              <div className="flex items-center justify-between">
                <div>
                  <h3 className="text-xl font-bold tracking-tight text-slate-900">MCP Connections</h3>
                  <p className="mt-1 text-xs text-slate-500">
                    Connect Model Context Protocol servers to provide real-time tools and data sources.
                  </p>
                </div>
                <button
                  onClick={() => setIsAddMcpOpen(true)}
                  className="flex items-center gap-1.5 rounded-xl bg-slate-950 hover:bg-slate-800 text-white px-3 py-1.5 text-xs font-semibold shadow-2xs transition-all cursor-pointer"
                >
                  <Plus size={13} />
                  <span>Add Server</span>
                </button>
              </div>

              <div className="space-y-2.5">
                {mcpServers.map((srv) => (
                  <div
                    key={srv.id}
                    className="flex items-center justify-between rounded-2xl glass-card border border-white/80 p-4 shadow-2xs"
                  >
                    <div className="flex items-center gap-3">
                      <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-slate-100 font-bold text-slate-800 text-xs border border-slate-200">
                        {srv.initial}
                      </div>
                      <div>
                        <div className="text-xs font-bold text-slate-900">{srv.name}</div>
                        <div className="text-[11px] text-slate-400 font-mono">
                          {srv.toolsCount} tools • {srv.transport}
                        </div>
                      </div>
                    </div>

                    <div className="flex items-center gap-3">
                      <span
                        className={`text-[10px] font-semibold px-2 py-0.5 rounded-full ${
                          srv.status === 'Connected'
                            ? 'bg-emerald-100 text-emerald-800'
                            : srv.status === 'Needs sign-in'
                            ? 'bg-amber-100 text-amber-800'
                            : 'bg-slate-100 text-slate-500'
                        }`}
                      >
                        {srv.status}
                      </span>
                      <button
                        onClick={() => onToggleMcpServer(srv.id)}
                        className={`h-5 w-9 rounded-full transition-colors cursor-pointer relative p-0.5 ${
                          srv.enabled ? 'bg-slate-900' : 'bg-slate-200'
                        }`}
                      >
                        <div
                          className={`h-4 w-4 rounded-full bg-white transition-transform ${
                            srv.enabled ? 'translate-x-4' : 'translate-x-0'
                          }`}
                        />
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* CHATS & PERMISSIONS TAB (FIGMA IMAGE 11) */}
          {activeTab === 'permissions' && (
            <div className="space-y-6">
              <div>
                <h3 className="text-xl font-bold tracking-tight text-slate-900">Chats & Permissions</h3>
                <p className="mt-1 text-xs text-slate-500">
                  Fine-grained sandbox policies and tool execution limits per conversation.
                </p>
              </div>

              <div className="rounded-2xl glass-card border border-white/80 p-5 shadow-2xs space-y-4 text-xs">
                {[
                  { key: 'runCommands', label: 'Run shell commands', sub: 'Terminal $ execution', val: 'Ask every time' },
                  { key: 'editFiles', label: 'Create and edit files', sub: 'Workspace write access', val: 'This conversation' },
                  { key: 'outsideWorkspace', label: 'Outside workspace access', sub: 'File system boundaries', val: 'Deny always' },
                  { key: 'networkAccess', label: 'Outbound network requests', sub: 'HTTP & Web fetch', val: 'Ask every time' },
                  { key: 'mcpTools', label: 'MCP server tool calls', sub: 'Connected servers', val: 'Allow always' }
                ].map((perm) => (
                  <div key={perm.key} className="flex items-center justify-between py-1.5 border-b border-slate-100">
                    <div>
                      <div className="font-semibold text-slate-900">{perm.label}</div>
                      <div className="text-[10px] text-slate-400">{perm.sub}</div>
                    </div>
                    <select
                      defaultValue={perm.val}
                      className="rounded-xl border border-slate-200 bg-white px-3 py-1.5 text-xs text-slate-800 font-medium focus:outline-none cursor-pointer"
                    >
                      <option value="Allow always">Allow always</option>
                      <option value="This conversation">This conversation</option>
                      <option value="Ask every time">Ask every time</option>
                      <option value="Deny always">Deny always</option>
                    </select>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* APPEARANCE TAB */}
          {activeTab === 'appearance' && (
            <div className="space-y-6">
              <div>
                <h3 className="text-xl font-bold tracking-tight text-slate-900">Appearance & Theme</h3>
                <p className="mt-1 text-xs text-slate-500">Customize the visual theme and glassmorphic effects.</p>
              </div>

              <div className="grid grid-cols-3 gap-3">
                {[
                  { id: 'light' as const, label: 'Light (Default)', icon: Sun },
                  { id: 'dark' as const, label: 'Dark Obsidian', icon: Moon },
                  { id: 'system' as const, label: 'Match System', icon: Laptop }
                ].map((item) => {
                  const Icon = item.icon;
                  const isSelected = theme === item.id;
                  return (
                    <div
                      key={item.id}
                      onClick={() => setTheme(item.id)}
                      className={`rounded-2xl p-4 text-center cursor-pointer transition-all border ${
                        isSelected
                          ? 'glass-card border-slate-900 shadow-md ring-2 ring-cyan-500/20'
                          : 'bg-white/60 border-slate-200 hover:bg-white'
                      }`}
                    >
                      <Icon size={24} className="mx-auto mb-2 text-slate-800" />
                      <div className="text-xs font-bold text-slate-900">{item.label}</div>
                    </div>
                  );
                })}
              </div>
            </div>
          )}

          {/* GENERAL PREFERENCES TAB */}
          {activeTab === 'general' && (
            <div className="space-y-6">
              <div>
                <h3 className="text-xl font-bold tracking-tight text-slate-900">General Settings</h3>
                <p className="mt-1 text-xs text-slate-500">Configure global app behaviors and defaults.</p>
              </div>

              <div className="rounded-2xl glass-card border border-white/80 p-4 space-y-3 text-xs shadow-2xs">
                <div className="flex justify-between items-center py-2 border-b border-slate-100">
                  <span className="font-semibold text-slate-800">Auto-restore sessions on launch</span>
                  <input type="checkbox" defaultChecked className="accent-slate-900 cursor-pointer" />
                </div>
                <div className="flex justify-between items-center py-2 border-b border-slate-100">
                  <span className="font-semibold text-slate-800">Keep long-running tasks active in background</span>
                  <input type="checkbox" defaultChecked className="accent-slate-900 cursor-pointer" />
                </div>
                <div className="flex justify-between items-center py-2">
                  <span className="font-semibold text-slate-800">Telemetry & Crash Analytics</span>
                  <span className="text-slate-400 font-mono text-[11px]">Disabled (Private)</span>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* MODAL: IMPORT SKILL */}
      {isImportSkillOpen && (
        <div className="fixed inset-0 z-60 bg-slate-950/40 backdrop-blur-sm flex items-center justify-center p-4">
          <div className="w-full max-w-md glass-modal rounded-3xl p-6 shadow-2xl space-y-4">
            <div className="flex items-center justify-between">
              <h4 className="text-sm font-bold text-slate-900">Import Custom Skill</h4>
              <button
                onClick={() => setIsImportSkillOpen(false)}
                className="p-1 rounded-lg hover:bg-slate-100 text-slate-400 hover:text-slate-700"
              >
                <X size={14} />
              </button>
            </div>
            <div className="space-y-3">
              <div>
                <label className="text-xs font-semibold text-slate-800">Skill Name</label>
                <input
                  type="text"
                  value={newSkillName}
                  onChange={(e) => setNewSkillName(e.target.value)}
                  placeholder="e.g. data-analyst"
                  className="mt-1 w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-xs text-slate-800 focus:outline-none focus:ring-2 focus:ring-cyan-500/20"
                />
              </div>
              <div>
                <label className="text-xs font-semibold text-slate-800">Description</label>
                <input
                  type="text"
                  value={newSkillDesc}
                  onChange={(e) => setNewSkillDesc(e.target.value)}
                  placeholder="e.g. Cleans datasets and builds charts"
                  className="mt-1 w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-xs text-slate-800 focus:outline-none focus:ring-2 focus:ring-cyan-500/20"
                />
              </div>
            </div>
            <div className="flex justify-end gap-2 pt-2">
              <button
                onClick={() => setIsImportSkillOpen(false)}
                className="rounded-xl border border-slate-200 px-4 py-2 text-xs font-semibold text-slate-700 hover:bg-slate-100"
              >
                Cancel
              </button>
              <button
                onClick={() => {
                  if (newSkillName.trim()) {
                    onAddSkill(newSkillName.trim(), newSkillDesc.trim() || 'Custom imported skill');
                    setNewSkillName('');
                    setNewSkillDesc('');
                    setIsImportSkillOpen(false);
                  }
                }}
                className="rounded-xl bg-slate-950 text-white px-4 py-2 text-xs font-semibold hover:bg-slate-800"
              >
                Add Skill
              </button>
            </div>
          </div>
        </div>
      )}

      {/* MODAL: ADD MCP SERVER */}
      {isAddMcpOpen && (
        <div className="fixed inset-0 z-60 bg-slate-950/40 backdrop-blur-sm flex items-center justify-center p-4">
          <div className="w-full max-w-md glass-modal rounded-3xl p-6 shadow-2xl space-y-4">
            <div className="flex items-center justify-between">
              <h4 className="text-sm font-bold text-slate-900">Add MCP Server</h4>
              <button
                onClick={() => setIsAddMcpOpen(false)}
                className="p-1 rounded-lg hover:bg-slate-100 text-slate-400 hover:text-slate-700"
              >
                <X size={14} />
              </button>
            </div>
            <div className="space-y-3">
              <div>
                <label className="text-xs font-semibold text-slate-800">Server Name</label>
                <input
                  type="text"
                  value={newMcpName}
                  onChange={(e) => setNewMcpName(e.target.value)}
                  placeholder="e.g. Sentry / Linear"
                  className="mt-1 w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-xs text-slate-800 focus:outline-none"
                />
              </div>
              <div>
                <label className="text-xs font-semibold text-slate-800">Transport</label>
                <select
                  value={newMcpTransport}
                  onChange={(e) => setNewMcpTransport(e.target.value as any)}
                  className="mt-1 w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-xs text-slate-800 focus:outline-none"
                >
                  <option value="HTTP">HTTP Server (SSE)</option>
                  <option value="stdio">stdio (Command / npx)</option>
                </select>
              </div>
              <div>
                <label className="text-xs font-semibold text-slate-800">URL / Command</label>
                <input
                  type="text"
                  value={newMcpUrl}
                  onChange={(e) => setNewMcpUrl(e.target.value)}
                  placeholder="http://localhost:8000 or npx @mcp/server"
                  className="mt-1 w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-xs text-slate-800 focus:outline-none"
                />
              </div>
            </div>
            <div className="flex justify-end gap-2 pt-2">
              <button
                onClick={() => setIsAddMcpOpen(false)}
                className="rounded-xl border border-slate-200 px-4 py-2 text-xs font-semibold text-slate-700 hover:bg-slate-100"
              >
                Cancel
              </button>
              <button
                onClick={() => {
                  if (newMcpName.trim()) {
                    onAddMcpServer(newMcpName.trim(), newMcpUrl.trim() || 'http://localhost:3000', newMcpTransport);
                    setNewMcpName('');
                    setNewMcpUrl('');
                    setIsAddMcpOpen(false);
                  }
                }}
                className="rounded-xl bg-slate-950 text-white px-4 py-2 text-xs font-semibold hover:bg-slate-800"
              >
                Save Server
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
