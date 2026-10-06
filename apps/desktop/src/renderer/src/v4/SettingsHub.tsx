import { useState } from 'react'
import {
  ArrowLeft,
  Settings as SettingsIcon,
  Moon,
  Sun,
  RefreshCw,
  Cpu,
  Layers,
  Sparkles,
  Network,
  ShieldCheck,
  Search,
  Plus,
  Edit2,
  Trash2,
  Archive,
  Check,
  Folder,
  Globe,
  Sliders,
  ExternalLink,
  Key,
} from 'lucide-react'
import type { AltrexCoreBridge } from '@altrex/contracts'
import { providerRegistry } from '../../../shared/provider-registry'

export type SettingsHubProps = {
  isOpen: boolean
  onClose: () => void
  core?: AltrexCoreBridge | undefined
  providers?: any[]
  skills?: any[]
  sessions?: any[]
  onConnectProvider?: (providerId: string, apiKey: string, model?: string) => Promise<boolean>
  onDisconnectProvider?: (providerId: string) => Promise<boolean>
}

export function SettingsHub({
  isOpen,
  onClose,
  core,
  providers = [],
  skills = [],
  sessions = [],
  onConnectProvider,
  onDisconnectProvider,
}: SettingsHubProps) {
  const [activeTab, setActiveTab] = useState<
    'general' | 'appearance' | 'updates' | 'models' | 'sessions' | 'skills' | 'mcp' | 'permissions'
  >('sessions')

  // Sessions state
  const [sessionList, setSessionList] = useState([
    { id: '1', title: 'Q4 roadmap deck', workspace: 'octrex-web', started: 'Today, 10:42', status: 'Active' },
    { id: '2', title: 'Fix login redirect', workspace: 'octrex-web', started: 'Today, 09:15', status: 'Idle' },
    { id: '3', title: 'Refactor API client', workspace: 'api-gateway', started: 'Yesterday', status: 'Idle' },
    { id: '4', title: 'Weekly report', workspace: 'Global', started: 'Oct 3', status: 'Ended' },
  ])
  const [restoreOnLaunch, setRestoreOnLaunch] = useState(true)
  const [keepTasksRunning, setKeepTasksRunning] = useState(true)

  // Skills state
  const [skillSearch, setSkillSearch] = useState('')
  const [skillItems, setSkillItems] = useState([
    { id: 'pptx', name: 'pptx', desc: 'Create and edit PowerPoint decks', type: 'Built-in', enabled: true },
    { id: 'docx', name: 'docx', desc: 'Write and edit Word documents', type: 'Built-in', enabled: true },
    { id: 'xlsx', name: 'xlsx', desc: 'Build spreadsheets and models', type: 'Built-in', enabled: true },
    { id: 'pdf', name: 'pdf', desc: 'Read, merge and create PDFs', type: 'Built-in', enabled: false },
    { id: 'frontend-design', name: 'frontend-design', desc: 'Distinctive, polished web UI', type: 'Imported', enabled: true },
    { id: 'brand-voice', name: 'brand-voice', desc: 'Company tone and writing style', type: 'Imported', enabled: true },
  ])

  // MCP Connections state
  const [mcpServers, setMcpServers] = useState([
    { id: 'figma', name: 'Figma', initial: 'F', tools: '12 tools · HTTP', status: 'Connected', enabled: true },
    { id: 'github', name: 'GitHub', initial: 'G', tools: '28 tools · HTTP', status: 'Connected', enabled: true },
    { id: 'filesystem', name: 'Filesystem', initial: 'F', tools: '8 tools · stdio', status: 'Connected', enabled: true },
    { id: 'postgres', name: 'Postgres', initial: 'P', tools: '6 tools · stdio', status: 'Off', enabled: false },
    { id: 'slack', name: 'Slack', initial: 'S', tools: '14 tools · HTTP', status: 'Needs sign-in', enabled: false },
  ])

  // Chats & Permissions state
  const [permTab, setPermTab] = useState<'all' | 'archived'>('all')
  const [selectedChatId, setSelectedChatId] = useState('1')
  const [chatPermissions, setChatPermissions] = useState<Record<string, any>>({
    '1': {
      runCommands: 'Ask',
      editFiles: 'Conversation',
      outsideWorkspace: 'Ask',
      networkAccess: 'Deny',
      mcpTools: 'Always',
      skills: ['pptx', 'file-reading'],
    },
  })

  // Models tab state
  const [apiKeyInput, setApiKeyInput] = useState('')
  const [activeProviderId, setActiveProviderId] = useState('google')

  if (!isOpen) return null

  const navItems = [
    { id: 'general', label: 'General', icon: SettingsIcon },
    { id: 'appearance', label: 'Appearance', icon: Moon },
    { id: 'updates', label: 'Updates', icon: RefreshCw },
    { id: 'models', label: 'Models', icon: Cpu },
    { id: 'sessions', label: 'Sessions', icon: Layers },
    { id: 'skills', label: 'Skills', icon: Sparkles },
    { id: 'mcp', label: 'MCP connections', icon: Network },
    { id: 'permissions', label: 'Chats & permissions', icon: ShieldCheck },
  ] as const

  const currentPermissions = chatPermissions[selectedChatId] || {
    runCommands: 'Ask',
    editFiles: 'Conversation',
    outsideWorkspace: 'Ask',
    networkAccess: 'Deny',
    mcpTools: 'Always',
    skills: ['pptx', 'file-reading'],
  }

  return (
    <div className="fixed inset-0 z-50 flex bg-slate-900/20 backdrop-blur-md animate-in fade-in duration-200">
      <div className="relative m-auto flex h-[90vh] w-[95vw] max-w-[1240px] rounded-3xl bg-[#f4f6f8] border border-slate-200/80 shadow-[0_25px_70px_rgba(0,0,0,0.15)] overflow-hidden">
        {/* LEFT SETTINGS SIDEBAR */}
        <div className="flex w-64 flex-col justify-between border-r border-slate-200/70 bg-white/70 p-5 backdrop-blur-xl">
          <div>
            <button
              onClick={onClose}
              className="flex items-center gap-2 text-xs font-medium text-slate-600 hover:text-slate-900 transition-colors mb-6"
            >
              <ArrowLeft size={14} />
              <span>Back to app</span>
            </button>

            <h2 className="text-xl font-bold tracking-tight text-slate-900 mb-4 px-2">Settings</h2>

            <div className="space-y-1">
              {navItems.map((item) => {
                const Icon = item.icon
                const isActive = activeTab === item.id
                return (
                  <button
                    key={item.id}
                    onClick={() => setActiveTab(item.id)}
                    className={`flex w-full items-center gap-3 rounded-2xl px-3.5 py-2.5 text-xs font-medium transition-all ${
                      isActive
                        ? 'bg-white text-slate-900 shadow-sm border border-slate-200/80'
                        : 'text-slate-600 hover:bg-white/50 hover:text-slate-900'
                    }`}
                  >
                    <Icon size={15} className={isActive ? 'text-slate-900' : 'text-slate-400'} />
                    <span>{item.label}</span>
                  </button>
                )
              })}
            </div>
          </div>

          <div className="text-[11px] font-mono text-slate-400 px-2">
            Octrex 1.4.0
          </div>
        </div>

        {/* RIGHT SETTINGS CONTENT AREA */}
        <div className="flex-1 overflow-y-auto p-8 custom-scrollbar bg-slate-50/50">
          {/* SESSIONS TAB (FIGMA IMAGE 8) */}
          {activeTab === 'sessions' && (
            <div className="max-w-3xl space-y-6">
              <div>
                <h3 className="text-2xl font-bold tracking-tight text-slate-900">Sessions</h3>
                <p className="mt-1 text-xs text-slate-500">
                  Active and recent sessions across your workspaces.
                </p>
              </div>

              {/* SESSIONS TABLE */}
              <div className="rounded-2xl border border-slate-200/80 bg-white p-2 shadow-sm">
                <div className="grid grid-cols-12 px-4 py-2 text-[11px] font-semibold tracking-wider text-slate-400 uppercase">
                  <div className="col-span-4">Session</div>
                  <div className="col-span-3">Workspace</div>
                  <div className="col-span-2">Started</div>
                  <div className="col-span-3 text-right">Status</div>
                </div>

                <div className="divide-y divide-slate-100">
                  {sessionList.map((s) => (
                    <div
                      key={s.id}
                      className="grid grid-cols-12 items-center px-4 py-3 text-xs hover:bg-slate-50/60 rounded-xl transition-colors"
                    >
                      <div className="col-span-4 font-semibold text-slate-900">{s.title}</div>
                      <div className="col-span-3 font-mono text-[11px] text-slate-500">{s.workspace}</div>
                      <div className="col-span-2 text-slate-500">{s.started}</div>
                      <div className="col-span-3 flex items-center justify-end gap-2">
                        <span
                          className={`rounded-full px-2.5 py-0.5 text-[10px] font-semibold ${
                            s.status === 'Active'
                              ? 'bg-slate-950 text-white'
                              : s.status === 'Idle'
                              ? 'bg-slate-100 text-slate-600'
                              : 'bg-slate-100 text-slate-400'
                          }`}
                        >
                          {s.status}
                        </span>
                        <button
                          onClick={onClose}
                          className="rounded-xl border border-slate-200 bg-white px-3 py-1 text-xs font-medium text-slate-700 hover:bg-slate-100 transition-colors shadow-2xs"
                        >
                          {s.status === 'Ended' ? 'Reopen' : 'Resume'}
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* PREFERENCES TOGGLES */}
              <div className="rounded-2xl border border-slate-200/80 bg-white p-5 shadow-sm space-y-4">
                <div className="flex items-center justify-between">
                  <div>
                    <div className="text-xs font-semibold text-slate-900">Restore sessions on launch</div>
                    <div className="text-[11px] text-slate-500">Reopen last active chats when Octrex starts</div>
                  </div>
                  <button
                    onClick={() => setRestoreOnLaunch(!restoreOnLaunch)}
                    className={`relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out ${
                      restoreOnLaunch ? 'bg-slate-900' : 'bg-slate-200'
                    }`}
                  >
                    <span
                      className={`inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${
                        restoreOnLaunch ? 'translate-x-5' : 'translate-x-0'
                      }`}
                    />
                  </button>
                </div>

                <div className="flex items-center justify-between pt-3 border-t border-slate-100">
                  <div>
                    <div className="text-xs font-semibold text-slate-900">Keep tasks running in background</div>
                    <div className="text-[11px] text-slate-500">Continue terminal tasks when a chat is closed</div>
                  </div>
                  <button
                    onClick={() => setKeepTasksRunning(!keepTasksRunning)}
                    className={`relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out ${
                      keepTasksRunning ? 'bg-slate-900' : 'bg-slate-200'
                    }`}
                  >
                    <span
                      className={`inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${
                        keepTasksRunning ? 'translate-x-5' : 'translate-x-0'
                      }`}
                    />
                  </button>
                </div>
              </div>
            </div>
          )}

          {/* SKILLS TAB (FIGMA IMAGE 9) */}
          {activeTab === 'skills' && (
            <div className="max-w-3xl space-y-6">
              <div>
                <h3 className="text-2xl font-bold tracking-tight text-slate-900">Skills</h3>
                <p className="mt-1 text-xs text-slate-500">
                  Reusable instructions Octrex reads before doing a task.
                </p>
              </div>

              {/* SEARCH & IMPORT */}
              <div className="flex items-center justify-between gap-4">
                <div className="relative flex-1">
                  <Search size={14} className="absolute left-3.5 top-1/2 -translate-y-1/2 text-slate-400" />
                  <input
                    type="text"
                    value={skillSearch}
                    onChange={(e) => setSkillSearch(e.target.value)}
                    placeholder="Search skills"
                    className="w-full rounded-2xl border border-slate-200 bg-white pl-9 pr-4 py-2.5 text-xs text-slate-800 placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-slate-900/5 shadow-2xs"
                  />
                </div>
                <button className="flex items-center gap-1.5 rounded-2xl bg-slate-950 px-4 py-2.5 text-xs font-medium text-white shadow-sm hover:bg-slate-800 transition-colors">
                  <Plus size={14} />
                  <span>Import skill</span>
                </button>
              </div>

              {/* SKILLS GRID */}
              <div className="grid grid-cols-2 gap-3.5">
                {skillItems
                  .filter((s) => s.name.toLowerCase().includes(skillSearch.toLowerCase()))
                  .map((skill) => (
                    <div
                      key={skill.id}
                      className="flex items-start justify-between rounded-2xl border border-slate-200/80 bg-white p-4 shadow-sm"
                    >
                      <div className="flex items-start gap-3">
                        <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-slate-50 border border-slate-200 text-slate-700 mt-0.5">
                          <Sparkles size={15} />
                        </div>
                        <div>
                          <div className="text-xs font-bold text-slate-900">{skill.name}</div>
                          <div className="text-[11px] text-slate-500 mt-0.5">{skill.desc}</div>
                          <span className="mt-2 inline-block rounded-full bg-slate-100 px-2.5 py-0.5 text-[9px] font-medium text-slate-600 border border-slate-200/60">
                            {skill.type}
                          </span>
                        </div>
                      </div>

                      <button
                        onClick={() => {
                          setSkillItems(
                            skillItems.map((s) =>
                              s.id === skill.id ? { ...s, enabled: !s.enabled } : s
                            )
                          )
                        }}
                        className={`relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out ${
                          skill.enabled ? 'bg-slate-900' : 'bg-slate-200'
                        }`}
                      >
                        <span
                          className={`inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${
                            skill.enabled ? 'translate-x-4' : 'translate-x-0'
                          }`}
                        />
                      </button>
                    </div>
                  ))}
              </div>
            </div>
          )}

          {/* MCP CONNECTIONS TAB (FIGMA IMAGE 10) */}
          {activeTab === 'mcp' && (
            <div className="max-w-3xl space-y-6">
              <div>
                <h3 className="text-2xl font-bold tracking-tight text-slate-900">MCP connections</h3>
                <p className="mt-1 text-xs text-slate-500">
                  Connect tools and data sources through MCP servers.
                </p>
              </div>

              <div className="flex justify-end">
                <button className="flex items-center gap-1.5 rounded-2xl bg-slate-950 px-4 py-2 text-xs font-medium text-white shadow-sm hover:bg-slate-800 transition-colors">
                  <Plus size={14} />
                  <span>Add server</span>
                </button>
              </div>

              <div className="space-y-2.5 rounded-2xl border border-slate-200/80 bg-white p-3 shadow-sm">
                {mcpServers.map((srv) => (
                  <div
                    key={srv.id}
                    className="flex items-center justify-between rounded-xl border border-slate-150 p-3 hover:bg-slate-50/50 transition-colors"
                  >
                    <div className="flex items-center gap-3.5">
                      <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-slate-100 font-bold text-xs text-slate-800 border border-slate-200">
                        {srv.initial}
                      </div>
                      <div>
                        <div className="text-xs font-bold text-slate-900">{srv.name}</div>
                        <div className="text-[11px] text-slate-500">{srv.tools}</div>
                      </div>
                    </div>

                    <div className="flex items-center gap-3">
                      <span
                        className={`rounded-full px-2.5 py-0.5 text-[10px] font-semibold ${
                          srv.status === 'Connected'
                            ? 'bg-slate-950 text-white'
                            : srv.status === 'Needs sign-in'
                            ? 'bg-amber-100 text-amber-800'
                            : 'bg-slate-100 text-slate-500'
                        }`}
                      >
                        {srv.status}
                      </span>
                      <button className="rounded-xl border border-slate-200 bg-white px-3 py-1 text-xs font-medium text-slate-700 hover:bg-slate-100 transition-colors shadow-2xs">
                        Configure
                      </button>
                      <button
                        onClick={() => {
                          setMcpServers(
                            mcpServers.map((s) =>
                              s.id === srv.id
                                ? { ...s, enabled: !s.enabled, status: !s.enabled ? 'Connected' : 'Off' }
                                : s
                            )
                          )
                        }}
                        className={`relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out ${
                          srv.enabled ? 'bg-slate-900' : 'bg-slate-200'
                        }`}
                      >
                        <span
                          className={`inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${
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
            <div className="max-w-4xl space-y-6">
              <div>
                <h3 className="text-2xl font-bold tracking-tight text-slate-900">Chats & permissions</h3>
                <p className="mt-1 text-xs text-slate-500">
                  Rename, archive or remove chats and set what each chat may do.
                </p>
              </div>

              {/* TABS PILL */}
              <div className="inline-flex rounded-xl bg-slate-200/80 p-1 text-xs font-medium text-slate-600">
                <button
                  onClick={() => setPermTab('all')}
                  className={`rounded-lg px-3 py-1 transition-all ${
                    permTab === 'all' ? 'bg-white font-semibold text-slate-900 shadow-2xs' : ''
                  }`}
                >
                  All chats 14
                </button>
                <button
                  onClick={() => setPermTab('archived')}
                  className={`rounded-lg px-3 py-1 transition-all ${
                    permTab === 'archived' ? 'bg-white font-semibold text-slate-900 shadow-2xs' : ''
                  }`}
                >
                  Archived 3
                </button>
              </div>

              {/* 2-COLUMN LAYOUT */}
              <div className="grid grid-cols-12 gap-6">
                {/* CHATS LIST */}
                <div className="col-span-6 space-y-2">
                  {[
                    { id: '1', title: 'Q4 roadmap deck', ws: 'octrex-web' },
                    { id: '2', title: 'Fix login redirect', ws: 'octrex-web' },
                    { id: '3', title: 'Refactor API client', ws: 'api-gateway' },
                    { id: '4', title: 'Weekly report', ws: 'Global' },
                    { id: '5', title: 'Landing copy', ws: 'Global' },
                    { id: '6', title: 'DB migration plan', ws: 'api-gateway' },
                  ].map((c) => (
                    <div
                      key={c.id}
                      onClick={() => setSelectedChatId(c.id)}
                      className={`flex items-center justify-between rounded-2xl p-3.5 border transition-all cursor-pointer ${
                        selectedChatId === c.id
                          ? 'bg-white border-slate-900 ring-1 ring-slate-900 shadow-xs'
                          : 'bg-white/80 border-slate-200/80 hover:bg-white'
                      }`}
                    >
                      <div>
                        <div className="text-xs font-bold text-slate-900">{c.title}</div>
                        <div className="text-[10px] font-mono text-slate-400 mt-0.5">▣ {c.ws}</div>
                      </div>
                      <div className="flex items-center gap-1.5 text-slate-400">
                        <button className="p-1 hover:text-slate-700">
                          <Edit2 size={12} />
                        </button>
                        <button className="p-1 hover:text-slate-700">
                          <Archive size={12} />
                        </button>
                        <button className="p-1 hover:text-red-600">
                          <Trash2 size={12} />
                        </button>
                      </div>
                    </div>
                  ))}
                </div>

                {/* PERMISSION DETAILS */}
                <div className="col-span-6 rounded-2xl border border-slate-200/80 bg-white p-5 shadow-sm flex flex-col justify-between">
                  <div className="space-y-4">
                    <div className="flex items-center justify-between">
                      <div>
                        <h4 className="text-sm font-bold text-slate-900">Q4 roadmap deck</h4>
                        <div className="text-[11px] text-slate-400">octrex-web · Started today, 10:42</div>
                      </div>
                      <button className="rounded-xl border border-slate-200 bg-white px-3 py-1 text-xs font-medium text-slate-700 hover:bg-slate-50 shadow-2xs">
                        Rename
                      </button>
                    </div>

                    <div>
                      <div className="text-[11px] font-semibold tracking-wider text-slate-400 uppercase mb-2">
                        Imported Skills
                      </div>
                      <div className="flex flex-wrap items-center gap-1.5">
                        <span className="rounded-full bg-slate-100 px-3 py-0.5 text-xs text-slate-700 font-medium">
                          pptx
                        </span>
                        <span className="rounded-full bg-slate-100 px-3 py-0.5 text-xs text-slate-700 font-medium">
                          file-reading
                        </span>
                        <button className="rounded-full border border-dashed border-slate-300 px-3 py-0.5 text-xs text-slate-500 hover:border-slate-400">
                          + Import
                        </button>
                      </div>
                    </div>

                    <div className="space-y-3 pt-2 border-t border-slate-100">
                      <div className="text-[11px] font-semibold tracking-wider text-slate-400 uppercase">
                        Permissions for this chat
                      </div>

                      {[
                        { key: 'runCommands', label: 'Run commands', sub: 'Terminal', val: 'Ask' },
                        { key: 'editFiles', label: 'Edit files', sub: 'Create and modify', val: 'Conversation' },
                        { key: 'outsideWorkspace', label: 'Outside workspace', sub: 'Read other folders', val: 'Ask' },
                        { key: 'networkAccess', label: 'Network access', sub: 'Web requests', val: 'Deny' },
                        { key: 'mcpTools', label: 'MCP tools', sub: 'Connected servers', val: 'Always' },
                      ].map((perm) => (
                        <div key={perm.key} className="flex items-center justify-between py-1">
                          <div>
                            <div className="text-xs font-semibold text-slate-900">{perm.label}</div>
                            <div className="text-[10px] text-slate-400">{perm.sub}</div>
                          </div>
                          <select
                            defaultValue={perm.val}
                            className="rounded-xl border border-slate-200 bg-slate-50 px-2.5 py-1 text-xs text-slate-800 focus:outline-none"
                          >
                            <option value="Always">Always</option>
                            <option value="Conversation">Conversation</option>
                            <option value="Ask">Ask</option>
                            <option value="Deny">Deny</option>
                          </select>
                        </div>
                      ))}
                    </div>
                  </div>

                  <div className="mt-6 flex items-center justify-between gap-3 pt-4 border-t border-slate-100">
                    <button className="flex-1 rounded-xl border border-slate-200 bg-slate-50 py-2 text-xs font-medium text-slate-700 hover:bg-slate-100">
                      Archive chat
                    </button>
                    <button className="flex-1 rounded-xl border border-rose-200 bg-rose-50/50 py-2 text-xs font-medium text-rose-600 hover:bg-rose-100/50">
                      Delete chat
                    </button>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* MODELS TAB */}
          {activeTab === 'models' && (
            <div className="max-w-3xl space-y-6">
              <div>
                <h3 className="text-2xl font-bold tracking-tight text-slate-900">AI Models & Providers</h3>
                <p className="mt-1 text-xs text-slate-500">
                  Configure Google Gemini, NVIDIA NIM, OpenAI, Anthropic, OpenRouter, Groq, Ollama, and Free OpenCode Gateway.
                </p>
              </div>

              <div className="space-y-3 rounded-2xl border border-slate-200/80 bg-white p-4 shadow-sm">
                {providerRegistry.map((prov) => (
                  <div
                    key={prov.id}
                    onClick={() => setActiveProviderId(prov.id)}
                    className={`flex items-center justify-between rounded-xl p-3 border transition-all cursor-pointer ${
                      activeProviderId === prov.id
                        ? 'bg-slate-50 border-slate-900'
                        : 'bg-white border-slate-150 hover:bg-slate-50/50'
                    }`}
                  >
                    <div>
                      <div className="text-xs font-bold text-slate-900">{prov.name}</div>
                      <div className="text-[11px] text-slate-500">{prov.description}</div>
                    </div>
                    <button
                      onClick={(e) => {
                        e.stopPropagation()
                        if (onConnectProvider) {
                          onConnectProvider(prov.id, apiKeyInput || 'auto-key')
                        }
                      }}
                      className="rounded-full bg-slate-900 px-3.5 py-1 text-xs font-medium text-white hover:bg-slate-800 transition-colors"
                    >
                      Connect
                    </button>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* GENERAL / APPEARANCE / UPDATES */}
          {activeTab === 'general' && (
            <div className="max-w-2xl space-y-4">
              <h3 className="text-2xl font-bold tracking-tight text-slate-900">General Preferences</h3>
              <div className="rounded-2xl border border-slate-200/80 bg-white p-5 shadow-sm space-y-3 text-xs">
                <div className="flex justify-between items-center py-2 border-b border-slate-100">
                  <span className="font-semibold text-slate-800">Application Language</span>
                  <span className="text-slate-500">English (US)</span>
                </div>
                <div className="flex justify-between items-center py-2 border-b border-slate-100">
                  <span className="font-semibold text-slate-800">Default Workspace Folder</span>
                  <span className="font-mono text-[11px] text-slate-500">~/dev</span>
                </div>
              </div>
            </div>
          )}

          {activeTab === 'appearance' && (
            <div className="max-w-2xl space-y-4">
              <h3 className="text-2xl font-bold tracking-tight text-slate-900">Appearance</h3>
              <div className="grid grid-cols-3 gap-3">
                <div className="rounded-2xl border border-slate-900 bg-white p-4 shadow-sm text-center">
                  <Sun size={20} className="mx-auto mb-2 text-slate-900" />
                  <span className="text-xs font-bold text-slate-900">Light (Figma Default)</span>
                </div>
                <div className="rounded-2xl border border-slate-200 bg-slate-900 p-4 shadow-sm text-center text-white">
                  <Moon size={20} className="mx-auto mb-2 text-white" />
                  <span className="text-xs font-bold">Dark Obsidian</span>
                </div>
                <div className="rounded-2xl border border-slate-200 bg-slate-100 p-4 shadow-sm text-center">
                  <Sliders size={20} className="mx-auto mb-2 text-slate-600" />
                  <span className="text-xs font-bold text-slate-700">System Sync</span>
                </div>
              </div>
            </div>
          )}

          {activeTab === 'updates' && (
            <div className="max-w-2xl space-y-4">
              <h3 className="text-2xl font-bold tracking-tight text-slate-900">Software Updates</h3>
              <div className="rounded-2xl border border-slate-200/80 bg-white p-5 shadow-sm space-y-2">
                <div className="flex items-center gap-2">
                  <Check size={16} className="text-emerald-600" />
                  <span className="text-xs font-bold text-slate-900">Octrex is up to date (v1.4.0)</span>
                </div>
                <p className="text-xs text-slate-500">You are on the latest production desktop release.</p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
