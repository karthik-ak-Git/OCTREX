import { useState } from 'react'
import {
  Globe,
  PanelLeftClose,
  PanelLeftOpen,
  Plus,
  Search,
  Settings as SettingsIcon,
  ChevronDown,
} from 'lucide-react'
import { OctrexLogo } from '../OctrexBrand'

export type SidebarProps = {
  currentProject?: { name: string; path: string } | null
  projects?: Array<{ name: string; path: string }>
  activeWorkspace?: string
  onSelectWorkspace?: (workspace: string) => void
  activeSessionTitle?: string
  activeSessionId?: string
  sessions?: Array<{ id: string; title: string }>
  onNewChat?: () => void
  onOpenSearch?: () => void
  onOpenProject?: () => void
  onSelectProject?: (project: { name: string; path: string } | null) => void
  onSelectSession?: (id: string) => void
  onOpenSettings?: (tab?: 'sessions' | 'skills' | 'mcp' | 'permissions' | 'chats') => void
}

export function Sidebar({
  currentProject,
  projects = [
    { name: 'octrex-web', path: '~/dev/octrex-web' },
    { name: 'api-gateway', path: '~/dev/api-gateway' },
    { name: 'docs-site', path: '~/dev/docs-site' },
  ],
  activeWorkspace = 'octrex-web',
  onSelectWorkspace,
  activeSessionTitle = 'Q4 roadmap deck',
  activeSessionId = 'session-1',
  sessions = [
    { id: 'session-1', title: 'Q4 roadmap deck' },
    { id: 'session-2', title: 'Fix login redirect' },
    { id: 'session-3', title: 'Refactor API client' },
  ],
  onNewChat,
  onOpenSearch,
  onOpenProject,
  onSelectProject,
  onSelectSession,
  onOpenSettings,
}: SidebarProps) {
  const [collapsed, setCollapsed] = useState(false)

  return (
    <aside
      className={`flex flex-col justify-between bg-white/85 border-r border-slate-200/80 backdrop-blur-xl transition-all duration-200 select-none ${
        collapsed ? 'w-16' : 'w-64'
      }`}
    >
      <div className="flex flex-col min-h-0 flex-1">
        {/* BRAND HEADER (FIGMA MATCHING) */}
        <div className="flex items-center justify-between p-4 border-b border-slate-100">
          <div className="flex items-center gap-2.5 min-w-0">
            <div className="flex h-8 w-8 items-center justify-center rounded-xl bg-slate-900 text-white shadow-2xs shrink-0">
              <OctrexLogo size={18} />
            </div>
            {!collapsed && (
              <strong className="text-sm font-bold tracking-tight text-slate-900 truncate">
                Octrex
              </strong>
            )}
          </div>
          <button
            className="rounded-lg p-1.5 text-slate-400 hover:text-slate-700 hover:bg-slate-100 transition-colors cursor-pointer"
            aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
            onClick={() => setCollapsed(!collapsed)}
          >
            {collapsed ? <PanelLeftOpen size={15} /> : <PanelLeftClose size={15} />}
          </button>
        </div>

        {/* PRIMARY ACTIONS */}
        {!collapsed ? (
          <div className="p-3 space-y-2">
            <button
              onClick={onNewChat}
              className="w-full flex items-center justify-center gap-2 rounded-2xl bg-white border border-slate-200/90 py-2.5 px-4 text-xs font-semibold text-slate-900 shadow-2xs hover:bg-slate-50 hover:border-slate-300 active:scale-[0.99] transition-all cursor-pointer"
            >
              <Plus size={15} className="text-slate-900" />
              <span>New chat</span>
            </button>

            <div
              onClick={onOpenSearch}
              className="flex items-center justify-between rounded-xl bg-slate-100/70 px-3 py-2 text-xs text-slate-400 border border-slate-200/50 hover:bg-slate-100 hover:text-slate-600 cursor-pointer transition-colors"
            >
              <div className="flex items-center gap-2">
                <Search size={13} />
                <span>Search chats</span>
              </div>
              <kbd className="rounded bg-white px-1.5 py-0.5 text-[10px] font-mono font-medium text-slate-400 border border-slate-200">
                ⌘K
              </kbd>
            </div>
          </div>
        ) : (
          <div className="p-2 space-y-2 text-center">
            <button
              onClick={onNewChat}
              className="h-10 w-10 mx-auto flex items-center justify-center rounded-xl bg-slate-900 text-white shadow-2xs cursor-pointer"
              title="New chat"
            >
              <Plus size={16} />
            </button>
            <button
              onClick={onOpenSearch}
              className="h-10 w-10 mx-auto flex items-center justify-center rounded-xl bg-slate-100 text-slate-600 cursor-pointer"
              title="Search"
            >
              <Search size={16} />
            </button>
          </div>
        )}

        {/* TREE SECTIONS */}
        {!collapsed && (
          <div className="flex-1 overflow-y-auto px-3 space-y-4 custom-scrollbar">
            {/* GLOBAL */}
            <div>
              <div className="text-[10px] font-semibold tracking-wider text-slate-400 uppercase px-2 mb-1.5">
                Global
              </div>
              <button
                onClick={() => {
                  onSelectProject?.(null)
                  onSelectWorkspace?.('Global')
                }}
                className={`w-full flex items-center gap-2.5 rounded-xl px-2.5 py-2 text-xs text-left transition-colors cursor-pointer ${
                  activeWorkspace === 'Global' || (!currentProject && activeWorkspace !== 'Global')
                    ? 'bg-slate-200/80 font-semibold text-slate-900'
                    : 'text-slate-600 hover:bg-slate-100 hover:text-slate-900'
                }`}
              >
                <Globe size={14} className="text-slate-400" />
                <span>Global workspace</span>
              </button>
            </div>

            {/* PROJECTS */}
            <div>
              <div className="flex items-center justify-between px-2 mb-1.5">
                <span className="text-[10px] font-semibold tracking-wider text-slate-400 uppercase">
                  Projects
                </span>
                <button
                  onClick={() => onOpenProject?.()}
                  className="text-slate-400 hover:text-slate-700 p-0.5 cursor-pointer"
                  title="Connect folder"
                >
                  <Plus size={13} />
                </button>
              </div>

              <div className="space-y-1">
                {projects.map((project) => {
                  const isCurrent = activeWorkspace === project.name || currentProject?.path === project.path
                  return (
                    <div key={project.path} className="space-y-0.5">
                      <button
                        onClick={() => {
                          onSelectProject?.(project)
                          onSelectWorkspace?.(project.name)
                        }}
                        className={`w-full flex items-center gap-2 rounded-xl px-2.5 py-1.5 text-xs text-left transition-colors cursor-pointer ${
                          isCurrent
                            ? 'font-semibold text-slate-900'
                            : 'text-slate-700 hover:bg-slate-100/70'
                        }`}
                      >
                        <ChevronDown size={13} className="text-slate-400" />
                        <span className="truncate">{project.name}</span>
                      </button>

                      {/* ACTIVE SESSIONS LIST UNDER ACTIVE PROJECT */}
                      {isCurrent && (
                        <div className="pl-6 pr-1 space-y-0.5 border-l-2 border-slate-200 ml-3.5 my-1">
                          {sessions.map((sess) => (
                            <button
                              key={sess.id}
                              onClick={() => onSelectSession?.(sess.id)}
                              className={`w-full flex items-center gap-2 rounded-lg px-2 py-1 text-[11px] text-left truncate transition-colors cursor-pointer ${
                                sess.id === activeSessionId || sess.title === activeSessionTitle
                                  ? 'bg-slate-200/80 font-semibold text-slate-900 shadow-2xs'
                                  : 'text-slate-600 hover:bg-slate-100 hover:text-slate-900'
                              }`}
                            >
                              <span className="h-1.5 w-1.5 rounded-full bg-slate-400 shrink-0"></span>
                              <span className="truncate">{sess.title}</span>
                            </button>
                          ))}
                        </div>
                      )}
                    </div>
                  )
                })}

                <button
                  onClick={() => onOpenProject?.()}
                  className="w-full flex items-center gap-2 rounded-xl px-2.5 py-1.5 text-xs text-slate-400 hover:text-slate-700 hover:bg-slate-100/60 transition-colors cursor-pointer"
                >
                  <Plus size={13} />
                  <span>Connect folder...</span>
                </button>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* FOOTER */}
      <div className="p-3 border-t border-slate-200/80 bg-white/60 space-y-2">
        <button
          onClick={() => onOpenSettings?.()}
          className={`w-full flex items-center gap-2.5 rounded-xl px-2.5 py-2 text-xs font-medium text-slate-700 hover:bg-slate-100 hover:text-slate-900 transition-colors cursor-pointer ${
            collapsed ? 'justify-center px-0' : ''
          }`}
          title="Settings"
        >
          <SettingsIcon size={15} className="text-slate-500 shrink-0" />
          {!collapsed && <span>Settings</span>}
        </button>

        {!collapsed && (
          <div
            onClick={() => onOpenSettings?.()}
            className="flex items-center gap-2.5 rounded-xl bg-slate-50 p-2 border border-slate-150 cursor-pointer hover:bg-slate-100 transition-colors"
          >
            <div className="flex h-7 w-7 items-center justify-center rounded-lg bg-slate-900 font-bold text-xs text-white shrink-0">
              K
            </div>
            <div className="min-w-0 flex-1">
              <div className="text-xs font-semibold text-slate-900 truncate">Karthik</div>
              <div className="text-[10px] text-slate-400 truncate">Pro plan</div>
            </div>
          </div>
        )}
      </div>
    </aside>
  )
}
