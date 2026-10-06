import React, { useState } from 'react';
import {
  Globe,
  PanelLeftClose,
  PanelLeftOpen,
  Plus,
  Search,
  Settings as SettingsIcon,
  ChevronDown,
  FolderPlus,
  X
} from 'lucide-react';
import { OctrexLogo } from '../OctrexBrand';
import { WorkspaceProject, ChatSession } from '../state';

export type SidebarProps = {
  projects: WorkspaceProject[];
  activeWorkspaceId: string;
  onSelectWorkspace: (workspaceId: string) => void;
  activeSessionId: string;
  sessions: ChatSession[];
  onSelectSession: (id: string) => void;
  onNewChat: () => void;
  onOpenProject: () => void;
  onOpenSettings: (tab?: 'sessions' | 'skills' | 'mcp' | 'permissions' | 'chats' | 'appearance' | 'models' | 'general') => void;
};

export const Sidebar: React.FC<SidebarProps> = ({
  projects,
  activeWorkspaceId,
  onSelectWorkspace,
  activeSessionId,
  sessions,
  onSelectSession,
  onNewChat,
  onOpenProject,
  onOpenSettings
}) => {
  const [collapsed, setCollapsed] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [isSearchOpen, setIsSearchOpen] = useState(false);

  const filteredSessions = sessions.filter((s) =>
    s.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
    s.workspaceName.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const activeProject = projects.find((p) => p.id === activeWorkspaceId);

  return (
    <aside
      className={`relative z-20 flex flex-col justify-between glass-panel transition-all duration-300 select-none ${
        collapsed ? 'w-16' : 'w-64'
      }`}
    >
      <div className="flex-1 flex flex-col min-h-0">
        {/* HEADER BRAND & COLLAPSE */}
        <div className="flex h-14 items-center justify-between px-3.5 border-b border-slate-200/60">
          <div
            onClick={onNewChat}
            className="flex items-center gap-2.5 min-w-0 cursor-pointer group"
            title="OCTREX CODE - New Chat"
          >
            <div className="flex h-8 w-8 items-center justify-center rounded-xl bg-slate-950 text-white shrink-0 shadow-sm group-hover:scale-105 transition-transform">
              <OctrexLogo size={18} />
            </div>
            {!collapsed && (
              <span className="font-bold tracking-tight text-sm text-slate-900 group-hover:text-cyan-600 transition-colors">
                OCTREX
              </span>
            )}
          </div>

          <button
            onClick={() => setCollapsed(!collapsed)}
            className="text-slate-400 hover:text-slate-700 hover:bg-slate-100/80 p-1.5 rounded-lg transition-colors cursor-pointer"
            title={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
          >
            {collapsed ? <PanelLeftOpen size={16} /> : <PanelLeftClose size={16} />}
          </button>
        </div>

        {/* TOP ACTION BUTTONS: NEW CHAT & SEARCH */}
        <div className="p-3 space-y-1.5">
          <button
            onClick={onNewChat}
            className={`w-full flex items-center gap-2.5 rounded-xl bg-slate-900 hover:bg-slate-800 text-white font-medium py-2.5 shadow-sm active:scale-[0.99] transition-all cursor-pointer ${
              collapsed ? 'justify-center px-0' : 'px-3 text-xs'
            }`}
            title="New chat"
          >
            <Plus size={15} className="shrink-0" />
            {!collapsed && <span>New chat</span>}
          </button>

          {!collapsed && (
            <button
              onClick={() => setIsSearchOpen(true)}
              className="w-full flex items-center justify-between rounded-xl bg-white/70 hover:bg-white border border-slate-200/80 px-3 py-1.5 text-xs text-slate-400 hover:text-slate-700 shadow-2xs transition-all cursor-pointer"
              title="Search chats (⌘K)"
            >
              <div className="flex items-center gap-2">
                <Search size={13} />
                <span>Search chats</span>
              </div>
              <kbd className="font-mono text-[10px] bg-slate-100 text-slate-500 px-1.5 py-0.5 rounded border border-slate-200">
                ⌘K
              </kbd>
            </button>
          )}
        </div>

        {/* SEARCH MODAL POPUP */}
        {isSearchOpen && (
          <div className="fixed inset-0 z-50 bg-slate-900/30 backdrop-blur-sm flex items-start justify-center pt-20 p-4">
            <div className="w-full max-w-lg glass-modal rounded-3xl p-4 shadow-2xl space-y-3">
              <div className="flex items-center gap-2 border-b border-slate-200/80 pb-2 px-1">
                <Search size={16} className="text-slate-400" />
                <input
                  type="text"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  placeholder="Search conversations, workspaces, or prompts..."
                  autoFocus
                  className="flex-1 text-xs text-slate-900 placeholder-slate-400 bg-transparent focus:outline-none"
                />
                <button
                  onClick={() => setIsSearchOpen(false)}
                  className="p-1 rounded-lg hover:bg-slate-100 text-slate-400 hover:text-slate-600"
                >
                  <X size={14} />
                </button>
              </div>

              <div className="max-h-64 overflow-y-auto space-y-1 custom-scrollbar">
                {filteredSessions.length === 0 ? (
                  <div className="p-4 text-center text-xs text-slate-400">
                    No matching conversations found
                  </div>
                ) : (
                  filteredSessions.map((s) => (
                    <div
                      key={s.id}
                      onClick={() => {
                        onSelectSession(s.id);
                        onSelectWorkspace(s.workspaceId);
                        setIsSearchOpen(false);
                      }}
                      className="flex items-center justify-between p-2.5 rounded-xl hover:bg-slate-100/80 cursor-pointer transition-colors"
                    >
                      <div className="min-w-0 flex-1">
                        <div className="text-xs font-semibold text-slate-900 truncate">
                          {s.title}
                        </div>
                        <div className="text-[11px] text-slate-400 font-mono">
                          {s.workspaceName} • {s.startedAt}
                        </div>
                      </div>
                      <span className="text-[10px] rounded-full bg-slate-100 px-2 py-0.5 text-slate-600 font-medium">
                        {s.model}
                      </span>
                    </div>
                  ))
                )}
              </div>
            </div>
          </div>
        )}

        {/* WORKSPACE & PROJECTS TREE */}
        {!collapsed && (
          <div className="flex-1 overflow-y-auto px-3 space-y-4 custom-scrollbar">
            {/* GLOBAL WORKSPACE */}
            <div>
              <div className="text-[10px] font-semibold tracking-wider text-slate-400 uppercase px-2 mb-1">
                Global
              </div>
              <button
                onClick={() => {
                  const globalProj = projects.find((p) => p.isGlobal) || projects[0];
                  if (globalProj) onSelectWorkspace(globalProj.id);
                }}
                className={`w-full flex items-center gap-2.5 rounded-xl px-2.5 py-2 text-xs text-left transition-all cursor-pointer ${
                  activeProject?.isGlobal
                    ? 'glass-card font-semibold text-slate-900 shadow-2xs border-slate-300'
                    : 'text-slate-600 hover:bg-white/60 hover:text-slate-900'
                }`}
              >
                <Globe size={14} className={activeProject?.isGlobal ? 'text-cyan-600' : 'text-slate-400'} />
                <span>Global workspace</span>
              </button>
            </div>

            {/* PROJECTS LIST */}
            <div>
              <div className="flex items-center justify-between px-2 mb-1">
                <span className="text-[10px] font-semibold tracking-wider text-slate-400 uppercase">
                  Projects
                </span>
                <button
                  onClick={onOpenProject}
                  className="text-slate-400 hover:text-slate-900 p-1 rounded hover:bg-slate-100 transition-colors cursor-pointer"
                  title="Connect local folder..."
                >
                  <Plus size={13} />
                </button>
              </div>

              <div className="space-y-1">
                {projects
                  .filter((p) => !p.isGlobal)
                  .map((project) => {
                    const isCurrent = activeWorkspaceId === project.id;
                    const projectSessions = sessions.filter(
                      (s) => s.workspaceId === project.id || s.workspaceName === project.name
                    );

                    return (
                      <div key={project.id} className="space-y-0.5">
                        <button
                          onClick={() => onSelectWorkspace(project.id)}
                          className={`w-full flex items-center gap-2 rounded-xl px-2.5 py-1.5 text-xs text-left transition-all cursor-pointer ${
                            isCurrent
                              ? 'font-semibold text-slate-900 bg-white/70 border border-slate-200/80 shadow-2xs'
                              : 'text-slate-700 hover:bg-white/50'
                          }`}
                        >
                          <ChevronDown size={13} className={isCurrent ? 'text-cyan-600' : 'text-slate-400'} />
                          <span className="truncate">{project.name}</span>
                        </button>

                        {/* SESSIONS UNDER ACTIVE PROJECT */}
                        {isCurrent && (
                          <div className="pl-5 pr-1 space-y-0.5 border-l-2 border-slate-200/80 ml-3.5 my-1">
                            {projectSessions.map((sess) => (
                              <button
                                key={sess.id}
                                onClick={() => onSelectSession(sess.id)}
                                className={`w-full flex items-center gap-2 rounded-lg px-2 py-1.5 text-[11px] text-left truncate transition-all cursor-pointer ${
                                  sess.id === activeSessionId
                                    ? 'bg-slate-900 text-white font-medium shadow-2xs'
                                    : 'text-slate-600 hover:bg-white hover:text-slate-900'
                                }`}
                              >
                                <span
                                  className={`h-1.5 w-1.5 rounded-full shrink-0 ${
                                    sess.id === activeSessionId ? 'bg-cyan-400' : 'bg-slate-400'
                                  }`}
                                />
                                <span className="truncate">{sess.title}</span>
                              </button>
                            ))}
                          </div>
                        )}
                      </div>
                    );
                  })}

                <button
                  onClick={onOpenProject}
                  className="w-full flex items-center gap-2 rounded-xl px-2.5 py-1.5 text-xs text-slate-400 hover:text-slate-800 hover:bg-white/60 transition-colors cursor-pointer"
                >
                  <FolderPlus size={13} />
                  <span>Connect folder...</span>
                </button>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* FOOTER CONTROLS & PROFILE */}
      <div className="p-3 border-t border-slate-200/60 bg-white/40 space-y-2">
        <button
          onClick={() => onOpenSettings('sessions')}
          className={`w-full flex items-center gap-2.5 rounded-xl px-2.5 py-2 text-xs font-medium text-slate-700 hover:bg-white hover:text-slate-900 shadow-2xs transition-all cursor-pointer ${
            collapsed ? 'justify-center px-0' : ''
          }`}
          title="Open Settings Hub"
        >
          <SettingsIcon size={15} className="text-slate-500 shrink-0" />
          {!collapsed && <span>Settings</span>}
        </button>

        {!collapsed && (
          <div
            onClick={() => onOpenSettings('general')}
            className="flex items-center gap-2.5 rounded-2xl glass-card p-2 cursor-pointer hover:border-slate-300 transition-all group"
            title="User Profile & Preferences"
          >
            <div className="flex h-7 w-7 items-center justify-center rounded-xl bg-gradient-to-tr from-slate-950 to-slate-800 font-bold text-xs text-white shrink-0 shadow-2xs group-hover:scale-105 transition-transform">
              K
            </div>
            <div className="min-w-0 flex-1">
              <div className="text-xs font-semibold text-slate-900 truncate">Karthik</div>
              <div className="text-[10px] text-slate-400 truncate font-medium">Pro plan</div>
            </div>
          </div>
        )}
      </div>
    </aside>
  );
};
