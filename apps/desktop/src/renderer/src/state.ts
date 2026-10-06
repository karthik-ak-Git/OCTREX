import { useState, useEffect } from 'react';

export interface WorkspaceProject {
  id: string;
  name: string;
  path: string;
  isGlobal?: boolean;
}

export interface ChatMessage {
  id: string;
  sender: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: string;
  executionSteps?: ExecutionStep[];
  artifacts?: ArtifactFile[];
  pendingPermission?: PermissionRequest;
}

export interface ExecutionStep {
  id: string;
  type: 'skill' | 'read' | 'write' | 'terminal';
  label: string;
  target: string;
  status: 'completed' | 'running' | 'failed';
  duration?: string;
}

export interface ArtifactFile {
  id: string;
  name: string;
  type: 'document' | 'image' | 'code';
  size: string;
  metadata?: string;
  url?: string;
  slidesCount?: number;
}

export interface PermissionRequest {
  id: string;
  command: string;
  description: string;
  status: 'pending' | 'approved' | 'denied';
  scope?: 'once' | 'conversation' | 'all';
}

export interface ChatSession {
  id: string;
  title: string;
  workspaceId: string;
  workspaceName: string;
  model: string;
  status: 'Active' | 'Idle' | 'Ended';
  startedAt: string;
  updatedAt: string;
  messages: ChatMessage[];
  skillsUsed: string[];
  toolsCount: Record<string, number>;
  tokensUsed: number;
}

export interface SkillItem {
  id: string;
  name: string;
  desc: string;
  type: 'Built-in' | 'Imported';
  enabled: boolean;
  systemPrompt?: string | undefined;
}

export interface McpServerItem {
  id: string;
  name: string;
  initial: string;
  urlOrCmd: string;
  transport: 'HTTP' | 'stdio';
  toolsCount: number;
  status: 'Connected' | 'Off' | 'Needs sign-in';
  enabled: boolean;
}

export interface BackgroundTask {
  id: string;
  sessionId: string;
  command: string;
  status: 'Running' | 'Completed' | 'Stopped' | 'Failed';
  duration: string;
  logs: string[];
}

const DEFAULT_PROJECTS: WorkspaceProject[] = [
  { id: 'ws-global', name: 'Global', path: '~/dev/global', isGlobal: true },
  { id: 'ws-octrex-web', name: 'octrex-web', path: '~/dev/octrex-web' },
  { id: 'ws-api-gateway', name: 'api-gateway', path: '~/dev/api-gateway' },
  { id: 'ws-docs-site', name: 'docs-site', path: '~/dev/docs-site' }
];

const DEFAULT_SKILLS: SkillItem[] = [
  { id: 'pptx', name: 'pptx', desc: 'Create and edit PowerPoint decks', type: 'Built-in', enabled: true },
  { id: 'docx', name: 'docx', desc: 'Write and edit Word documents', type: 'Built-in', enabled: true },
  { id: 'xlsx', name: 'xlsx', desc: 'Build spreadsheets and financial models', type: 'Built-in', enabled: true },
  { id: 'pdf', name: 'pdf', desc: 'Read, parse, merge and generate PDFs', type: 'Built-in', enabled: true },
  { id: 'frontend-design', name: 'frontend-design', desc: 'Distinctive, modern web UI & Tailwind styling', type: 'Imported', enabled: true },
  { id: 'brand-voice', name: 'brand-voice', desc: 'OCTREX tone of voice and technical documentation style', type: 'Imported', enabled: true }
];

const DEFAULT_MCP_SERVERS: McpServerItem[] = [
  { id: 'figma', name: 'Figma', initial: 'F', urlOrCmd: 'http://localhost:3845', transport: 'HTTP', toolsCount: 12, status: 'Connected', enabled: true },
  { id: 'github', name: 'GitHub', initial: 'G', urlOrCmd: 'npx @modelcontextprotocol/server-github', transport: 'stdio', toolsCount: 28, status: 'Connected', enabled: true },
  { id: 'filesystem', name: 'Filesystem', initial: 'F', urlOrCmd: 'npx @modelcontextprotocol/server-filesystem', transport: 'stdio', toolsCount: 8, status: 'Connected', enabled: true },
  { id: 'postgres', name: 'Postgres', initial: 'P', urlOrCmd: 'npx @modelcontextprotocol/server-postgres', transport: 'stdio', toolsCount: 6, status: 'Off', enabled: false },
  { id: 'slack', name: 'Slack', initial: 'S', urlOrCmd: 'https://mcp.slack.internal', transport: 'HTTP', toolsCount: 14, status: 'Needs sign-in', enabled: false }
];

const INITIAL_SESSION: ChatSession = {
  id: 'session-1',
  title: 'Q4 roadmap deck',
  workspaceId: 'ws-octrex-web',
  workspaceName: 'octrex-web',
  model: 'Claude Sonnet 3.5',
  status: 'Active',
  startedAt: 'Today, 10:42',
  updatedAt: 'Just now',
  skillsUsed: ['pptx', 'file-reading', 'frontend-design'],
  toolsCount: { 'Read file': 4, 'Write file': 2, Terminal: 3, 'Web search': 1 },
  tokensUsed: 84200,
  messages: [
    {
      id: 'msg-1',
      sender: 'user',
      content: 'Create a 6-slide Q4 roadmap deck from the notes in /docs, then run the build check.',
      timestamp: '10:42 AM'
    },
    {
      id: 'msg-2',
      sender: 'assistant',
      content: "I'll read your notes and the pptx skill, then build the deck.",
      timestamp: '10:42 AM',
      executionSteps: [
        { id: 'step-1', type: 'skill', label: 'Read skill', target: 'pptx', status: 'completed' },
        { id: 'step-2', type: 'read', label: 'Read file', target: 'docs/notes.md', status: 'completed' },
        { id: 'step-3', type: 'write', label: 'Created file', target: 'roadmap-q4.pptx', status: 'completed' },
        { id: 'step-4', type: 'terminal', label: 'Running', target: 'npm run build', status: 'running', duration: '00:42' }
      ],
      artifacts: [
        {
          id: 'art-1',
          name: 'roadmap-q4.pptx',
          type: 'document',
          size: '1.2 MB',
          metadata: '6 slides · 1.2 MB · Just now',
          slidesCount: 6
        }
      ],
      pendingPermission: {
        id: 'perm-1',
        command: 'npm install pptxgenjs',
        description: 'Octrex wants to run a command',
        status: 'pending'
      }
    }
  ]
};

export function useOctrexStore() {
  // Projects & Workspaces
  const [projects, setProjects] = useState<WorkspaceProject[]>(() => {
    const saved = localStorage.getItem('octrex_projects');
    return saved ? JSON.parse(saved) : DEFAULT_PROJECTS;
  });

  const [activeWorkspaceId, setActiveWorkspaceId] = useState<string>(() => {
    return localStorage.getItem('octrex_active_ws') || 'ws-octrex-web';
  });

  // Sessions
  const [sessions, setSessions] = useState<ChatSession[]>(() => {
    const saved = localStorage.getItem('octrex_sessions');
    return saved ? JSON.parse(saved) : [INITIAL_SESSION];
  });

  const [activeSessionId, setActiveSessionId] = useState<string>(() => {
    return localStorage.getItem('octrex_active_session') || 'session-1';
  });

  // Skills
  const [skills, setSkills] = useState<SkillItem[]>(() => {
    const saved = localStorage.getItem('octrex_skills');
    return saved ? JSON.parse(saved) : DEFAULT_SKILLS;
  });

  // MCP Servers
  const [mcpServers, setMcpServers] = useState<McpServerItem[]>(() => {
    const saved = localStorage.getItem('octrex_mcp_servers');
    return saved ? JSON.parse(saved) : DEFAULT_MCP_SERVERS;
  });

  // Provider Connections
  const [connectedProviders, setConnectedProviders] = useState<Record<string, { apiKey?: string; connected: boolean }>>(() => {
    const saved = localStorage.getItem('octrex_connected_providers');
    return saved ? JSON.parse(saved) : { google: { connected: true }, openai: { connected: true } };
  });

  // Background Tasks
  const [tasks, setTasks] = useState<BackgroundTask[]>([
    {
      id: 'task-build-1',
      sessionId: 'session-1',
      command: 'npm run build',
      status: 'Running',
      duration: '00:42',
      logs: [
        '$ npm run build',
        '',
        '> octrex-web@1.4.0 build',
        '> vite build',
        '',
        'vite v5.4 building for production...',
        '✓ 214 modules transformed.',
        'dist/index.html   0.46 kB',
        'dist/assets/app.js  182.30 kB',
        'dist/assets/app.css 14.12 kB',
        '',
        'running type check...'
      ]
    }
  ]);

  // Persist state updates
  useEffect(() => {
    localStorage.setItem('octrex_projects', JSON.stringify(projects));
  }, [projects]);

  useEffect(() => {
    localStorage.setItem('octrex_active_ws', activeWorkspaceId);
  }, [activeWorkspaceId]);

  useEffect(() => {
    localStorage.setItem('octrex_sessions', JSON.stringify(sessions));
  }, [sessions]);

  useEffect(() => {
    localStorage.setItem('octrex_active_session', activeSessionId);
  }, [activeSessionId]);

  useEffect(() => {
    localStorage.setItem('octrex_skills', JSON.stringify(skills));
  }, [skills]);

  useEffect(() => {
    localStorage.setItem('octrex_mcp_servers', JSON.stringify(mcpServers));
  }, [mcpServers]);

  useEffect(() => {
    localStorage.setItem('octrex_connected_providers', JSON.stringify(connectedProviders));
  }, [connectedProviders]);

  const activeWorkspace = projects.find((p) => p.id === activeWorkspaceId) || projects[0];
  const activeSession = sessions.find((s) => s.id === activeSessionId) || sessions[0] || INITIAL_SESSION;

  // Actions
  const createNewSession = (workspaceId?: string) => {
    const wsId = workspaceId || activeWorkspaceId;
    const ws = projects.find((p) => p.id === wsId) || activeWorkspace || DEFAULT_PROJECTS[1];
    const newId = `session-${Date.now()}`;
    const newSession: ChatSession = {
      id: newId,
      title: 'New conversation',
      workspaceId: ws?.id || 'ws-octrex-web',
      workspaceName: ws?.name || 'octrex-web',
      model: activeSession?.model || 'Claude Sonnet 3.5',
      status: 'Active',
      startedAt: 'Just now',
      updatedAt: 'Just now',
      skillsUsed: [],
      toolsCount: {},
      tokensUsed: 0,
      messages: []
    };

    setSessions((prev) => [newSession, ...prev]);
    setActiveSessionId(newId);
    return newSession;
  };

  const deleteSession = (sessionId: string) => {
    setSessions((prev) => prev.filter((s) => s.id !== sessionId));
    if (activeSessionId === sessionId) {
      const remaining = sessions.filter((s) => s.id !== sessionId);
      if (remaining.length > 0 && remaining[0]) {
        setActiveSessionId(remaining[0].id);
      } else {
        createNewSession();
      }
    }
  };

  const addProject = (name: string, path: string) => {
    const newProject: WorkspaceProject = {
      id: `ws-${Date.now()}`,
      name,
      path
    };
    setProjects((prev) => [...prev, newProject]);
    setActiveWorkspaceId(newProject.id);
    return newProject;
  };

  const updateModel = (model: string) => {
    setSessions((prev) =>
      prev.map((s) => (s.id === activeSessionId ? { ...s, model } : s))
    );
  };

  const sendMessage = async (prompt: string) => {
    if (!prompt.trim()) return;

    const userMsg: ChatMessage = {
      id: `msg-${Date.now()}`,
      sender: 'user',
      content: prompt,
      timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
    };

    // Auto update title if new
    const updatedTitle =
      activeSession.title === 'New conversation' || activeSession.messages.length === 0
        ? prompt.slice(0, 30) + (prompt.length > 30 ? '…' : '')
        : activeSession.title;

    // Add assistant response with dynamic steps
    const assistantMsgId = `msg-asst-${Date.now()}`;
    const isDeckRequest = prompt.toLowerCase().includes('deck') || prompt.toLowerCase().includes('slide') || prompt.toLowerCase().includes('pptx');
    const isBuildRequest = prompt.toLowerCase().includes('build') || prompt.toLowerCase().includes('check') || prompt.toLowerCase().includes('run');
    const wsName = activeWorkspace?.name || 'octrex-web';

    const generatedArtifacts: ArtifactFile[] = [];
    if (isDeckRequest) {
      generatedArtifacts.push({
        id: `art-${Date.now()}`,
        name: 'roadmap-q4.pptx',
        type: 'document',
        size: '1.2 MB',
        metadata: '6 slides · 1.2 MB · Just now',
        slidesCount: 6
      });
    }

    const assistantMsg: ChatMessage = {
      id: assistantMsgId,
      sender: 'assistant',
      content: isDeckRequest
        ? "I'll read your notes and the pptx skill, then build the deck."
        : `Analyzing workspace **${wsName}** and executing your request: "${prompt}".`,
      timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
      executionSteps: [
        { id: `step-${Date.now()}-1`, type: 'skill', label: 'Read skill', target: isDeckRequest ? 'pptx' : 'frontend-design', status: 'completed' },
        { id: `step-${Date.now()}-2`, type: 'read', label: 'Read file', target: `${wsName}/src/index.ts`, status: 'completed' },
        { id: `step-${Date.now()}-3`, type: 'write', label: 'Created file', target: isDeckRequest ? 'roadmap-q4.pptx' : 'solution.ts', status: 'completed' }
      ],
      artifacts: generatedArtifacts,
      pendingPermission: {
        id: `perm-${Date.now()}`,
        command: isBuildRequest ? 'npm run build' : 'npx vitest run',
        description: 'Octrex wants to run a command',
        status: 'pending'
      }
    };

    setSessions((prev) =>
      prev.map((s) => {
        if (s.id === activeSessionId) {
          return {
            ...s,
            title: updatedTitle,
            updatedAt: 'Just now',
            tokensUsed: s.tokensUsed + Math.floor(prompt.length * 1.8 + 420),
            toolsCount: {
              ...s.toolsCount,
              'Read file': (s.toolsCount['Read file'] || 0) + 1,
              'Write file': (s.toolsCount['Write file'] || 0) + 1,
              Terminal: (s.toolsCount.Terminal || 0) + 1
            },
            messages: [...s.messages, userMsg, assistantMsg]
          };
        }
        return s;
      })
    );
  };

  const respondPermission = (msgId: string, decision: 'approve' | 'deny', scope: 'once' | 'conversation' | 'all') => {
    setSessions((prev) =>
      prev.map((s) => {
        if (s.id === activeSessionId) {
          return {
            ...s,
            messages: s.messages.map((m) => {
              if (m.id === msgId && m.pendingPermission) {
                return {
                  ...m,
                  pendingPermission: {
                    ...m.pendingPermission,
                    status: decision === 'approve' ? 'approved' : 'denied',
                    scope
                  }
                };
              }
              return m;
            })
          };
        }
        return s;
      })
    );
  };

  const toggleSkill = (skillId: string) => {
    setSkills((prev) =>
      prev.map((sk) => (sk.id === skillId ? { ...sk, enabled: !sk.enabled } : sk))
    );
  };

  const addSkill = (name: string, desc: string, systemPrompt?: string) => {
    const newSkill: SkillItem = {
      id: name.toLowerCase().replace(/\s+/g, '-'),
      name,
      desc,
      type: 'Imported',
      enabled: true,
      systemPrompt
    };
    setSkills((prev) => [...prev, newSkill]);
    return newSkill;
  };

  const toggleMcpServer = (serverId: string) => {
    setMcpServers((prev) =>
      prev.map((srv) =>
        srv.id === serverId
          ? {
              ...srv,
              enabled: !srv.enabled,
              status: !srv.enabled ? 'Connected' : 'Off'
            }
          : srv
      )
    );
  };

  const addMcpServer = (name: string, urlOrCmd: string, transport: 'HTTP' | 'stdio', toolsCount = 5) => {
    const newServer: McpServerItem = {
      id: name.toLowerCase().replace(/\s+/g, '-'),
      name,
      initial: name.charAt(0).toUpperCase(),
      urlOrCmd,
      transport,
      toolsCount,
      status: 'Connected',
      enabled: true
    };
    setMcpServers((prev) => [...prev, newServer]);
    return newServer;
  };

  const connectProvider = async (providerId: string, apiKey: string) => {
    setConnectedProviders((prev) => ({
      ...prev,
      [providerId]: { apiKey, connected: true }
    }));
    return true;
  };

  const stopTask = (taskId: string) => {
    setTasks((prev) =>
      prev.map((t) => (t.id === taskId ? { ...t, status: 'Stopped' } : t))
    );
  };

  return {
    projects,
    activeWorkspace,
    activeWorkspaceId,
    setActiveWorkspaceId,
    addProject,
    sessions,
    activeSession,
    activeSessionId,
    setActiveSessionId,
    createNewSession,
    deleteSession,
    updateModel,
    sendMessage,
    respondPermission,
    skills,
    toggleSkill,
    addSkill,
    mcpServers,
    toggleMcpServer,
    addMcpServer,
    connectedProviders,
    connectProvider,
    tasks,
    stopTask
  };
}
