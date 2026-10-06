import { useEffect, useMemo, useRef, useState } from "react";
import {
  ArrowUp,
  ChevronRight,
  ChevronDown,
  Folder,
  FolderOpen,
  Globe,
  PanelLeftClose,
  PanelLeftOpen,
  Plus,
  Search,
  Settings as SettingsIcon,
  Square,
  Terminal,
  X,
  Files,
  Sparkles,
  Paperclip,
  Activity,
  FileText,
  Image as ImageIcon,
  Check,
  Shield,
  Layers,
  Code2,
  FileCode,
  Sliders,
} from "lucide-react";
import type {
  AltrexCoreBridge,
  AltrexEvent,
  RoutingMode,
  TaskMode,
} from "@altrex/contracts";
import { OctrexLogo, OctrexCodeSymbol } from "../OctrexBrand";
import { Dialog } from "../components/primitives";
import { useWorkspace } from "./useWorkspace";
import { label, taskView, terminalStates } from "./state";
import { TaskCard } from "./TaskCard";
import { WorkPanel, type PanelTab } from "./WorkPanel";
import { Settings } from "./Settings";
import { SettingsHub } from "./SettingsHub";
import { OnboardingModal } from "./OnboardingModal";
import { ActivityInspector } from "./ActivityInspector";
import { DocumentPreview } from "./DocumentPreview";
import { ImagePreview } from "./ImagePreview";
import { TerminalPanel } from "./TerminalPanel";
import { Approvals } from "./Approvals";
import { Palette, type PaletteAction } from "./Palette";
import { SetupPrompt } from "./SetupPrompt";
import { RouteBadge } from "./RouteBadge";
import "./workspace.css";

function readPreference(key: string): string | null {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}
function savePreference(key: string, value: string) {
  try {
    localStorage.setItem(key, value);
  } catch {
    /* Safe fallback */
  }
}

export function App({
  core = window.altrexCore,
  demo = false,
}: {
  core?: AltrexCoreBridge | undefined;
  demo?: boolean;
}) {
  const app = useWorkspace(core);
  const [prompt, setPrompt] = useState("");
  const [mode, setMode] = useState<TaskMode>("AGENT");
  const [routing, setRouting] = useState<RoutingMode>("AUTO");
  const [customModel, setCustomModel] = useState("");
  const [candidates, setCandidates] = useState(1);
  const [settings, setSettings] = useState(false);
  const [settingsHubOpen, setSettingsHubOpen] = useState(false);
  const [onboardingOpen, setOnboardingOpen] = useState(false);
  const [palette, setPalette] = useState(false);
  const [panel, setPanel] = useState<PanelTab | null>(null);
  const [inspectorTab, setInspectorTab] = useState<
    "activity" | "files" | "terminal" | "changes" | "doc-preview" | "img-preview"
  >("activity");
  const [activePreviewDoc, setActivePreviewDoc] = useState<string | null>(null);
  const [activePreviewImg, setActivePreviewImg] = useState<string | null>(null);
  const [collapsed, setCollapsed] = useState(
    readPreference("altrex.v4.sidebar") === "collapsed",
  );
  const [starting, setStarting] = useState(false);
  const [consent, setConsent] = useState(false);
  const [setupDismissed, setSetupDismissed] = useState(false);
  const [setupOpen, setSetupOpen] = useState(false);
  const [settingsProvider, setSettingsProvider] = useState<string | undefined>();
  const [engine, setEngine] = useState<"OCTREX" | "CODEX">("OCTREX");
  const [selectedModel, setSelectedModel] = useState("Claude Sonnet");

  const codexAvailable = app.consents.some((endpoint) => endpoint.providerId === "codex");
  const useCodex = engine === "CODEX" && mode === "AGENT";
  const relevantConsents = app.consents.filter((endpoint) =>
    useCodex ? endpoint.providerId === "codex" : endpoint.providerId !== "codex",
  );
  const needsProvider = !app.loading && !!core && !app.providers.length;
  const showSetup = needsProvider && (setupOpen || !setupDismissed);

  const [taskFilter, setTaskFilter] = useState("");
  const [allProjects, setAllProjects] = useState(false);
  const [historyLimit, setHistoryLimit] = useState(60);
  const [conversationLimit, setConversationLimit] = useState(12);
  const [prompts, setPrompts] = useState<Record<string, string>>({});

  const composer = useRef<HTMLTextAreaElement>(null);
  const scroller = useRef<HTMLDivElement>(null);
  const nearBottom = useRef(true);
  const startLock = useRef(false);

  const conversation = app.tasks
    .filter(
      (task) =>
        task.sessionId === app.sessionId ||
        (!task.sessionId && task.taskId === app.sessionId),
    )
    .sort((a, b) => a.createdAt.localeCompare(b.createdAt));

  const selected =
    app.tasks.find((task) => task.taskId === app.selectedId) ??
    conversation.at(-1);

  const groupedEvents = useMemo(() => {
    const map = new Map<string | null, AltrexEvent[]>();
    for (const event of app.events) {
      const list = map.get(event.taskId) ?? [];
      list.push(event);
      map.set(event.taskId, list);
    }
    return map;
  }, [app.events]);

  const visibleEvents = useMemo(
    () =>
      [
        ...(groupedEvents.get(null) ?? []),
        ...(selected ? (groupedEvents.get(selected.taskId) ?? []) : []),
      ].sort((a, b) => a.ts.localeCompare(b.ts) || a.seq - b.seq),
    [groupedEvents, selected?.taskId],
  );

  const runningTask = conversation.find(
    (task) =>
      !terminalStates.has(
        taskView(task, groupedEvents.get(task.taskId) ?? []).state,
      ),
  );

  const busyProject = app.activeTasks.some(
    (t) => t.projectPath === app.project?.path,
  );

  const fresh = () => {
    if (starting) return;
    app.newSession();
    setPrompt("");
    setPanel(null);
    setConversationLimit(12);
    composer.current?.focus();
  };

  useEffect(() => {
    const key = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setPalette(true);
      }
      if (
        (event.ctrlKey || event.metaKey) &&
        event.key.toLowerCase() === "n" &&
        !settings &&
        !settingsHubOpen &&
        !palette
      ) {
        event.preventDefault();
        fresh();
      }
    };
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  }, [starting, settings, settingsHubOpen, palette]);

  useEffect(() => {
    if (nearBottom.current && scroller.current)
      scroller.current.scrollTop = scroller.current.scrollHeight;
  }, [app.events, conversation.length]);

  const send = async (confirmed = false) => {
    if (
      startLock.current ||
      !prompt.trim() ||
      runningTask ||
      busyProject ||
      app.historyLoading ||
      app.loading ||
      !core
    )
      return;
    if (!useCodex && !app.providers.length && !setupDismissed) {
      setSetupOpen(true);
      return;
    }
    if (!app.project && mode !== "ASK") {
      await app.openProject();
      return;
    }
    if (routing === "CUSTOM" && !customModel.trim()) {
      setSettings(true);
      return;
    }

    const missing = relevantConsents.filter((endpoint) => !endpoint.granted);
    if (
      !confirmed &&
      app.project &&
      (useCodex || routing !== "LOCAL_ONLY") &&
      mode !== "LOCAL" &&
      missing.length
    ) {
      setConsent(true);
      return;
    }
    if (confirmed) {
      for (const endpoint of missing)
        await app.invoke("consent.grant", {
          providerId: endpoint.providerId,
          baseUrl: endpoint.baseUrl,
        });
      const next = await app.invoke("consent.list", {});
      if (next) app.setConsents(next);
    }
    setConsent(false);
    startLock.current = true;
    setStarting(true);
    app.setError(null);

    const history = conversation.slice(-80).flatMap((task) => {
      const view = taskView(task, groupedEvents.get(task.taskId) ?? []);
      return [
        {
          role: "user" as const,
          content: (prompts[task.taskId] ?? task.title).slice(0, 200000),
        },
        ...(view.text
          ? [
              {
                role: "assistant" as const,
                content: view.text.slice(0, 200000),
              },
            ]
          : []),
      ];
    });

    const text = prompt.trim();
    const result = await app.invoke("task.start", {
      projectPath: app.project?.path ?? null,
      mode,
      prompt: text,
      sessionId: app.sessionId,
      history,
      routingMode: mode === "LOCAL" ? "LOCAL_ONLY" : useCodex ? "AUTO" : routing,
      modelSelection: useCodex ? "CODEX" : routing === "CUSTOM" ? customModel : "AUTO",
      candidates: mode === "AGENT" ? candidates : 1,
    });

    if (result) {
      setPrompts((previous) => ({ ...previous, [result.taskId]: text }));
      setPrompt("");
      nearBottom.current = true;
      await app.acceptTask(result.taskId);
    }
    setStarting(false);
    startLock.current = false;
  };

  const retry = (text: string) => {
    setPrompt(text);
    composer.current?.focus();
  };

  const actions: PaletteAction[] = [
    {
      name: "Open project",
      run: () => void app.openProject(),
      disabled: starting,
    },
    { name: "New task / session", run: fresh, disabled: starting },
    { name: "Provider settings", run: () => setSettings(true) },
    { name: "Settings Hub", run: () => setSettingsHubOpen(true) },
    { name: "Welcome Onboarding", run: () => setOnboardingOpen(true) },
    {
      name: "Change AI mode",
      run: () => document.getElementById("v4-routing")?.focus(),
    },
    ...(
      [
        "Terminal",
        "Changes",
        "Tests",
        "Problems",
        "Agents",
        "Checkpoints",
        "Context",
        "Developer",
      ] as PanelTab[]
    ).map((tab) => ({
      name:
        tab === "Checkpoints"
          ? "Restore checkpoint"
          : `Open ${tab.toLowerCase()}`,
      run: () => {
        setPanel(tab);
        if (tab === "Terminal") setInspectorTab("terminal");
        if (tab === "Changes") setInspectorTab("changes");
      },
    })),
    ...app.projects.map((project) => ({
      name: `Switch project: ${project.name}`,
      run: () => {
        app.setProject(project);
        fresh();
      },
      disabled: starting,
    })),
  ];

  const taskHistory = app.tasks.filter(
    (task) =>
      (allProjects || !app.project || task.projectPath === app.project.path) &&
      task.title.toLowerCase().includes(taskFilter.toLowerCase()),
  );

  const health = app.providers.some((p) => p.health === "HEALTHY")
    ? "AI available"
    : app.providers.length
      ? "Check AI setup"
      : "Set up AI";

  return (
    <div className={`v4-shell ${collapsed ? "v4-collapsed" : ""}`}>
      {/* ========================================================================= */}
      {/* 1. LEFT SIDEBAR                                                           */}
      {/* ========================================================================= */}
      <aside className="v4-sidebar flex flex-col justify-between bg-white/85 border-r border-slate-200/80 backdrop-blur-xl">
        <div className="flex flex-col min-h-0 flex-1">
          {/* BRAND HEADER */}
          <div className="v4-brand flex items-center justify-between p-4 border-b border-slate-100">
            <div className="flex items-center gap-2.5">
              <div className="flex h-8 w-8 items-center justify-center rounded-xl bg-slate-900 text-white shadow-2xs">
                <OctrexLogo size={18} />
              </div>
              <strong className="text-sm font-bold tracking-tight text-slate-900">Octrex</strong>
            </div>
            <button
              className="quiet rounded-lg p-1.5 text-slate-400 hover:text-slate-700 hover:bg-slate-100 transition-colors"
              aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
              onClick={() => {
                setCollapsed(!collapsed);
                savePreference("altrex.v4.sidebar", collapsed ? "expanded" : "collapsed");
              }}
            >
              {collapsed ? <PanelLeftOpen size={15} /> : <PanelLeftClose size={15} />}
            </button>
          </div>

          {/* PRIMARY ACTIONS: NEW CHAT & SEARCH */}
          <div className="p-3 space-y-2">
            <button
              className="v4-nav w-full flex items-center justify-center gap-2 rounded-2xl bg-white border border-slate-200/90 py-2.5 px-4 text-xs font-semibold text-slate-900 shadow-2xs hover:bg-slate-50 hover:border-slate-300 active:scale-[0.99] transition-all"
              title="New task"
              disabled={starting}
              onClick={fresh}
            >
              <Plus size={15} className="text-slate-900" />
              <span>New chat</span>
              <kbd className="hidden">⌘N</kbd>
            </button>

            <button
              className="v4-nav w-full flex items-center justify-between rounded-xl bg-slate-100/70 px-3 py-2 text-xs text-slate-500 border border-slate-200/50 hover:bg-slate-100 hover:text-slate-800 transition-colors"
              title="Commands"
              onClick={() => setPalette(true)}
            >
              <div className="flex items-center gap-2">
                <Search size={13} />
                <span>Search chats</span>
              </div>
              <kbd className="rounded bg-white px-1.5 py-0.5 text-[10px] font-mono font-medium text-slate-400 border border-slate-200">
                ⌘K
              </kbd>
            </button>

            <button
              className="v4-nav w-full flex items-center gap-2 rounded-xl px-3 py-1.5 text-xs text-slate-600 hover:bg-slate-100 hover:text-slate-900 transition-colors"
              title="Open project"
              disabled={starting}
              onClick={() => void app.openProject()}
            >
              <FolderOpen size={14} className="text-slate-400" />
              <span>Open project</span>
            </button>
          </div>

          {/* SIDEBAR CONTENT */}
          <div className="v4-sidebar-content flex-1 overflow-y-auto px-3 space-y-3 custom-scrollbar">
            {/* GLOBAL */}
            <div>
              <div className="v4-section-label text-[10px] font-semibold tracking-wider text-slate-400 uppercase px-2 mb-1">
                Global
              </div>
              <button
                onClick={() => {
                  app.setProject(null);
                  fresh();
                }}
                className={`w-full flex items-center gap-2 rounded-xl px-2.5 py-1.5 text-xs text-left transition-colors ${
                  !app.project
                    ? "bg-slate-200/80 font-semibold text-slate-900"
                    : "text-slate-600 hover:bg-slate-100 hover:text-slate-900"
                }`}
              >
                <Globe size={13} className="text-slate-400" />
                <span>Global workspace</span>
              </button>
            </div>

            {/* PROJECTS */}
            <div>
              <div className="flex items-center justify-between px-2 mb-1">
                <span className="v4-section-label text-[10px] font-semibold tracking-wider text-slate-400 uppercase">
                  Projects
                </span>
                <button
                  onClick={() => void app.openProject()}
                  className="text-slate-400 hover:text-slate-700 p-0.5"
                  title="Connect folder"
                >
                  <Plus size={13} />
                </button>
              </div>

              <div className="space-y-0.5">
                {app.projects.map((project) => (
                  <button
                    className="v4-project w-full flex items-center gap-2 rounded-xl px-2.5 py-1.5 text-xs text-left transition-colors text-slate-700 hover:bg-slate-100/70"
                    aria-current={app.project?.path === project.path ? "page" : undefined}
                    key={project.path}
                    title={project.path}
                    disabled={starting}
                    onClick={() => {
                      app.setProject(project);
                      fresh();
                    }}
                  >
                    <FolderOpen size={13} className="text-slate-400" />
                    <span className="truncate">{project.name}</span>
                  </button>
                ))}
              </div>
            </div>

            {/* SESSIONS & TASK HISTORY */}
            <div>
              <div className="flex items-center justify-between px-2 mb-1">
                <span className="v4-section-label text-[10px] font-semibold tracking-wider text-slate-400 uppercase">
                  Task History
                </span>
                <select
                  aria-label="History project filter"
                  value={allProjects ? "all" : "current"}
                  onChange={(e) => setAllProjects(e.target.value === "all")}
                  className="text-[10px] bg-transparent border-0 p-0 text-slate-400 focus:outline-none"
                >
                  <option value="current">Current</option>
                  <option value="all">All</option>
                </select>
              </div>

              <input
                className="v4-history-search hidden"
                aria-label="Filter task history"
                value={taskFilter}
                placeholder="Find a task…"
                onChange={(e) => setTaskFilter(e.target.value)}
              />

              <div className="space-y-0.5">
                {taskHistory.slice(0, historyLimit).map((task) => (
                  <button
                    className="v4-history w-full flex items-center justify-between rounded-lg px-2 py-1 text-[11px] text-left text-slate-600 hover:bg-slate-100"
                    key={task.taskId}
                    title={task.title}
                    aria-current={app.selectedId === task.taskId ? "true" : undefined}
                    disabled={starting}
                    onClick={() => {
                      void app.selectTask(task);
                      setPrompt("");
                      setConversationLimit(12);
                    }}
                  >
                    <span className="truncate">{task.title}</span>
                    <small className="text-[9px] text-slate-400 shrink-0 ml-1">{label(task.state)}</small>
                  </button>
                ))}
              </div>
            </div>
          </div>
        </div>

        {/* SIDEBAR FOOTER */}
        <div className="p-3 border-t border-slate-200/80 bg-white/60 space-y-2">
          <button
            className="v4-nav v4-settings-trigger w-full flex items-center gap-2.5 rounded-xl px-2.5 py-2 text-xs font-medium text-slate-700 hover:bg-slate-100 hover:text-slate-900 transition-colors"
            title="Settings"
            onClick={() => setSettings(true)}
          >
            <SettingsIcon size={15} className="text-slate-500" />
            <span>Settings</span>
          </button>

          <div
            onClick={() => setSettingsHubOpen(true)}
            className="flex items-center gap-2.5 rounded-xl bg-slate-50 p-2 border border-slate-150 cursor-pointer hover:bg-slate-100 transition-colors"
          >
            <div className="flex h-7 w-7 items-center justify-center rounded-lg bg-slate-900 font-bold text-xs text-white">
              K
            </div>
            <div className="min-w-0 flex-1">
              <div className="text-xs font-semibold text-slate-900 truncate">Karthik</div>
              <div className="text-[10px] text-slate-400 truncate">Pro plan</div>
            </div>
          </div>
        </div>
      </aside>

      {/* ========================================================================= */}
      {/* 2. CENTER CHAT CANVAS                                                     */}
      {/* ========================================================================= */}
      <main className={`v4-main ${panel ? "v4-has-panel" : ""} flex-1 flex flex-col min-w-0 bg-[#f4f6f8] relative overflow-hidden`}>
        {/* TOPBAR */}
        <header className="v4-topbar flex h-14 items-center justify-between px-6 bg-white/60 border-b border-slate-200/80 backdrop-blur-md shrink-0">
          <div className="v4-breadcrumb flex items-center gap-2.5 min-w-0">
            <span className="font-semibold text-xs text-slate-900">{app.project?.name ?? "Your workspace"}</span>
            {app.project?.branch && <small className="text-[10px] text-slate-400 font-mono">({app.project.branch})</small>}
            <ChevronRight size={13} className="text-slate-400" />
            <span className="text-xs text-slate-500">{selected ? selected.title : "New task"}</span>
          </div>

          <div className="v4-top-actions flex items-center gap-3 shrink-0">
            {demo && <span className="v4-demo text-xs text-amber-600 bg-amber-50 px-2 py-0.5 rounded border border-amber-200">DEMO · no real execution</span>}

            <button
              className="v4-health flex items-center gap-2 rounded-full bg-white border border-slate-200/90 px-3 py-1 text-xs font-medium text-slate-700 hover:bg-slate-50"
              onClick={() => (needsProvider ? setSetupOpen(true) : setSettings(true))}
            >
              <span className={`v4-dot h-2 w-2 rounded-full ${app.providers.some((p) => p.health === "HEALTHY") ? "good bg-emerald-500" : "bg-slate-300"}`} />
              {health}
            </button>

            <button
              className="quiet rounded-lg p-2 text-slate-500 hover:text-slate-800 hover:bg-slate-100"
              aria-label="View changes"
              onClick={() => setPanel(panel === "Changes" ? null : "Changes")}
            >
              <Files size={16} />
            </button>
            <button
              className="quiet rounded-lg p-2 text-slate-500 hover:text-slate-800 hover:bg-slate-100"
              aria-label="Open terminal"
              onClick={() => setPanel(panel === "Terminal" ? null : "Terminal")}
            >
              <Terminal size={16} />
            </button>
          </div>
        </header>

        {/* TRANSCRIPT & SCROLL AREA */}
        <div className="v4-conversation-area flex-1 flex flex-col min-h-0 relative">
          <div
            ref={scroller}
            className="v4-conversation flex-1 overflow-y-auto p-6 space-y-6 custom-scrollbar"
            onScroll={() => {
              const el = scroller.current;
              if (el) nearBottom.current = el.scrollHeight - el.scrollTop - el.clientHeight < 90;
            }}
          >
            {app.loading ? (
              <div className="v4-home" role="status">
                Opening your workspace…
              </div>
            ) : !core ? (
              <div className="v4-home">
                <OctrexCodeSymbol />
                <h1>Connect to your workspace</h1>
                <p>Open OCTREX CODE on your desktop to use the secure core.</p>
              </div>
            ) : !conversation.length ? (
              /* FIGMA WELCOME VIEW */
              <div className="v4-home max-w-2xl mx-auto my-8 space-y-8 text-center animate-in fade-in duration-300">
                <div className="inline-flex h-16 w-16 items-center justify-center rounded-3xl bg-white border border-slate-200/80 shadow-sm">
                  <OctrexLogo size={32} className="text-slate-900" />
                </div>

                <div>
                  <span className="eyebrow text-[10px] font-mono font-bold tracking-widest text-slate-400 uppercase">
                    YOUR IDEAS. WORKING SOFTWARE.
                  </span>
                  <h1 className="text-3xl font-bold tracking-tight text-slate-900 mt-2">
                    What should we build?
                  </h1>
                  <p className="mt-2 text-xs text-slate-500 max-w-md mx-auto leading-relaxed">
                    Describe what you need. Octrex reads skills, plans tasks, writes code, builds slides, and verifies everything.
                  </p>
                </div>

                {!app.project && (
                  <button
                    className="primary rounded-2xl bg-slate-950 px-6 py-2.5 text-xs font-semibold text-white shadow-sm hover:bg-slate-800"
                    onClick={() => void app.openProject()}
                  >
                    <FolderOpen size={15} className="mr-2 inline" />
                    Open a project
                  </button>
                )}

                <div className="v4-starters grid grid-cols-3 gap-3 text-left">
                  {[
                    {
                      title: "Build something",
                      text: "Build a login system for this project.",
                      desc: "Turn an idea into working code with automated tests.",
                    },
                    {
                      title: "Find the problem",
                      text: "Find and fix a bug in this project. Explain the cause and verify the fix.",
                      desc: "Trace, fix, and verify root causes.",
                    },
                    {
                      title: "Understand the code",
                      text: "Explain the architecture and key entry points in this project.",
                      desc: "Explore your codebase and architecture.",
                      ask: true,
                    },
                  ].map((item) => (
                    <button
                      key={item.title}
                      onClick={() => {
                        setMode(item.ask ? "ASK" : "AGENT");
                        retry(item.text);
                      }}
                      className="group flex flex-col justify-between rounded-2xl border border-slate-200/80 bg-white p-4 shadow-2xs hover:border-slate-300 hover:shadow-sm cursor-pointer transition-all"
                    >
                      <div>
                        <div className="text-xs font-bold text-slate-900">{item.title}</div>
                        <div className="text-[11px] text-slate-500 mt-1">{item.desc}</div>
                      </div>
                      <span className="mt-3 text-[10px] font-semibold text-slate-900 group-hover:underline">
                        Start task →
                      </span>
                    </button>
                  ))}
                </div>
              </div>
            ) : (
              /* REAL CONVERSATION TRANSCRIPT WITH TASK CARDS */
              <div className="v4-transcript max-w-3xl mx-auto space-y-6">
                {app.historyNotice && <p className="v4-warning text-xs text-amber-700 bg-amber-50 p-3 rounded-xl border border-amber-200">{app.historyNotice}</p>}
                {app.historyLoading && <p role="status" className="text-xs text-slate-400">Restoring session history…</p>}
                {conversation.length > conversationLimit && (
                  <button onClick={() => setConversationLimit((n) => n + 12)} className="text-xs text-slate-500 hover:text-slate-800">
                    Load earlier tasks ({conversation.length - conversationLimit})
                  </button>
                )}
                {conversation.slice(-conversationLimit).map((task) => (
                  <TaskCard
                    key={task.taskId}
                    task={task}
                    events={groupedEvents.get(task.taskId) ?? []}
                    prompt={prompts[task.taskId]}
                    onSelect={() => app.setSelectedId(task.taskId)}
                    onChanges={() => {
                      app.setSelectedId(task.taskId);
                      setPanel("Changes");
                    }}
                    onCancel={() => void app.invoke("task.cancel", { taskId: task.taskId })}
                    onRetry={retry}
                    providers={app.providers}
                  />
                ))}
              </div>
            )}
          </div>

          {/* COMPOSER DOCK */}
          <div className="v4-composer-wrap p-5 max-w-3xl w-full mx-auto shrink-0">
            {selected && !app.project && selected.projectPath && (
              <p className="v4-warning text-xs text-amber-700 bg-amber-50 p-2.5 rounded-xl border border-amber-200 mb-2">
                Reopen {selected.projectPath} before continuing work on this project.
              </p>
            )}

            <form
              className="v4-composer rounded-3xl bg-white border border-slate-200/90 p-3.5 shadow-[0_10px_30px_rgba(0,0,0,0.06)] focus-within:border-slate-400 transition-all"
              onSubmit={(e) => {
                e.preventDefault();
                void send();
              }}
            >
              <textarea
                ref={composer}
                aria-label="Ask OCTREX"
                placeholder={
                  app.project
                    ? "Ask Octrex to build, write or edit anything..."
                    : "Describe what you want to work on…"
                }
                value={prompt}
                maxLength={200000}
                rows={2}
                onChange={(e) => setPrompt(e.target.value)}
                onKeyDown={(e) => {
                  if (
                    e.key === "Enter" &&
                    (e.ctrlKey || e.metaKey) &&
                    !e.nativeEvent.isComposing
                  ) {
                    e.preventDefault();
                    void send();
                  }
                }}
                className="w-full resize-none border-0 bg-transparent px-2 py-1 text-xs text-slate-900 placeholder-slate-400 focus:outline-none leading-relaxed"
              />

              <div className="v4-composer-controls flex items-center justify-between pt-2 border-t border-slate-100">
                <div className="v4-composer-options flex items-center gap-2">
                  <select
                    aria-label="Task mode"
                    value={mode}
                    disabled={starting || !!runningTask}
                    onChange={(e) => {
                      const next = e.target.value as TaskMode;
                      setMode(next);
                      if (next !== "AGENT") setEngine("OCTREX");
                    }}
                    className="rounded-xl border border-slate-200 bg-slate-50 px-2.5 py-1 text-xs text-slate-800 focus:outline-none"
                  >
                    <option value="AGENT">Build</option>
                    <option value="ASK">Ask</option>
                    <option value="LOCAL">Local AI</option>
                    <option value="MULTI">Multi-AI</option>
                  </select>

                  <select
                    id="v4-routing"
                    aria-label="AI mode"
                    value={useCodex ? "CODEX" : routing}
                    disabled={starting || !!runningTask || mode === "LOCAL"}
                    onChange={(e) => {
                      if (e.target.value === "CODEX") {
                        setEngine("CODEX");
                        return;
                      }
                      setEngine("OCTREX");
                      setRouting(e.target.value as RoutingMode);
                    }}
                    className="rounded-xl border border-slate-200 bg-slate-50 px-2.5 py-1 text-xs text-slate-800 focus:outline-none"
                  >
                    {mode === "AGENT" && (
                      <option value="CODEX" disabled={!codexAvailable}>
                        {codexAvailable ? "ChatGPT Codex" : "ChatGPT Codex (install Codex CLI)"}
                      </option>
                    )}
                    {["AUTO", "FAST", "POWERFUL", "FREE_ONLY", "LOCAL_ONLY", "CUSTOM"].map((value) => (
                      <option key={value} value={value}>
                        {label(value)}
                      </option>
                    ))}
                  </select>

                  {(runningTask ?? conversation.at(-1)) && (
                    <RouteBadge
                      events={groupedEvents.get((runningTask ?? conversation.at(-1))!.taskId) ?? []}
                      providers={app.providers}
                    />
                  )}

                  <button
                    type="button"
                    onClick={() => setSettingsHubOpen(true)}
                    className="flex items-center gap-1 rounded-xl bg-slate-100 px-2.5 py-1 text-xs font-medium text-slate-700 hover:bg-slate-200 transition-colors"
                  >
                    <Sparkles size={12} />
                    <span>Skills</span>
                  </button>

                  <details className="v4-run-options hidden">
                    <summary>Options</summary>
                    <div>
                      <select
                        aria-label="Solution candidates"
                        disabled={mode !== "AGENT"}
                        value={candidates}
                        onChange={(e) => setCandidates(Number(e.target.value))}
                      >
                        <option value={1}>One solution</option>
                        <option value={2}>Compare two</option>
                      </select>
                    </div>
                  </details>
                </div>

                {runningTask ? (
                  <button
                    type="button"
                    className="v4-send flex h-8 w-8 items-center justify-center rounded-full bg-slate-900 text-white hover:bg-slate-800 shadow-2xs transition-colors"
                    aria-label="Stop task"
                    onClick={() => void app.invoke("task.cancel", { taskId: runningTask.taskId })}
                  >
                    <Square size={13} className="fill-white" />
                  </button>
                ) : (
                  <button
                    className="v4-send flex h-8 w-8 items-center justify-center rounded-full bg-slate-950 text-white hover:bg-slate-800 disabled:opacity-30 disabled:hover:bg-slate-950 shadow-2xs transition-all"
                    aria-label="Run task"
                    disabled={!prompt.trim() || starting || app.loading || app.historyLoading || busyProject || !core}
                  >
                    <ArrowUp size={15} />
                  </button>
                )}
              </div>
            </form>

            <div className="v4-composer-foot flex items-center justify-between text-[11px] text-slate-400 px-2 pt-2">
              <span>
                {app.project ? app.project.name : "Open a project to edit files"}
                {useCodex ? " · ChatGPT Codex (your subscription)" : mode === "LOCAL" || routing === "LOCAL_ONLY" ? " · Local only" : ` · ${routing === "AUTO" ? "Auto-routed AI" : label(routing)}`}
              </span>
              <span>{starting ? "Starting task…" : "Ctrl + Enter to run"}</span>
            </div>
          </div>
        </div>

        {/* WORK PANEL (WHEN OPENED FOR TERMINAL, CHANGES, TESTS, ETC.) */}
        {panel && (
          <WorkPanel
            key={app.project?.path ?? "no-project"}
            app={app}
            tab={panel}
            setTab={setPanel}
            task={selected}
            events={visibleEvents}
            onClose={() => setPanel(null)}
          />
        )}
      </main>

      {/* ========================================================================= */}
      {/* 3. RIGHT INSPECTOR PANEL (FIGMA MATCHING)                                 */}
      {/* ========================================================================= */}
      {!panel && (
        <aside className="w-[340px] border-l border-slate-200/80 bg-white/80 flex flex-col justify-between backdrop-blur-xl shrink-0 overflow-hidden">
          <div className="p-3 border-b border-slate-150">
            <div className="flex items-center gap-1 rounded-2xl bg-slate-100/80 p-1 text-[11px] font-semibold text-slate-500 overflow-x-auto custom-scrollbar">
              <button
                onClick={() => setInspectorTab("activity")}
                className={`rounded-xl px-3 py-1.5 transition-all shrink-0 ${
                  inspectorTab === "activity" ? "bg-white font-bold text-slate-900 shadow-2xs" : "hover:text-slate-800"
                }`}
              >
                Activity
              </button>

              <button
                onClick={() => {
                  setInspectorTab("files");
                  setPanel("Context");
                }}
                className={`rounded-xl px-3 py-1.5 transition-all shrink-0 ${
                  inspectorTab === "files" ? "bg-white font-bold text-slate-900 shadow-2xs" : "hover:text-slate-800"
                }`}
              >
                Files
              </button>

              {activePreviewDoc && (
                <button
                  onClick={() => setInspectorTab("doc-preview")}
                  className={`flex items-center gap-1 rounded-xl px-2.5 py-1.5 transition-all shrink-0 ${
                    inspectorTab === "doc-preview" ? "bg-white font-bold text-slate-900 shadow-2xs" : "hover:text-slate-800"
                  }`}
                >
                  <span>{activePreviewDoc}</span>
                  <X
                    size={11}
                    className="text-slate-400 hover:text-slate-700 ml-0.5"
                    onClick={(e) => {
                      e.stopPropagation();
                      setActivePreviewDoc(null);
                      setInspectorTab("activity");
                    }}
                  />
                </button>
              )}

              <button
                onClick={() => {
                  setInspectorTab("terminal");
                  setPanel("Terminal");
                }}
                className={`rounded-xl px-3 py-1.5 transition-all shrink-0 ${
                  inspectorTab === "terminal" ? "bg-white font-bold text-slate-900 shadow-2xs" : "hover:text-slate-800"
                }`}
              >
                Terminal
              </button>

              <button
                onClick={() => {
                  setInspectorTab("changes");
                  setPanel("Changes");
                }}
                className={`rounded-xl px-3 py-1.5 transition-all shrink-0 ${
                  inspectorTab === "changes" ? "bg-white font-bold text-slate-900 shadow-2xs" : "hover:text-slate-800"
                }`}
              >
                Changes
              </button>
            </div>
          </div>

          <div className="flex-1 overflow-y-auto custom-scrollbar">
            {inspectorTab === "activity" && (
              <ActivityInspector
                onOpenFile={(fileName) => {
                  if (fileName.endsWith(".pptx")) {
                    setActivePreviewDoc(fileName);
                    setInspectorTab("doc-preview");
                  } else if (fileName.endsWith(".png") || fileName.endsWith(".jpg")) {
                    setActivePreviewImg(fileName);
                    setInspectorTab("img-preview");
                  } else {
                    setPanel("Changes");
                  }
                }}
                onOpenTerminalTask={() => setPanel("Terminal")}
              />
            )}

            {inspectorTab === "doc-preview" && (
              <DocumentPreview
                fileName={activePreviewDoc || "roadmap-q4.pptx"}
                onAskOctrexToEdit={() => {
                  setPrompt("Update slide 2 of roadmap-q4.pptx with the revised Q4 milestones.");
                  composer.current?.focus();
                }}
              />
            )}

            {inspectorTab === "img-preview" && (
              <ImagePreview
                fileName={activePreviewImg || "brand-guide.png"}
                onUseInChat={() => {
                  setPrompt("Use the brand guide palette and icon in the new UI mockup.");
                  composer.current?.focus();
                }}
              />
            )}
          </div>
        </aside>
      )}

      {/* ========================================================================= */}
      {/* 4. MODALS, ALERTS & DIALOGS                                               */}
      {/* ========================================================================= */}
      {app.error && (
        <div role="alert" className="v4-error fixed bottom-4 right-4 z-50 rounded-2xl bg-white border border-rose-200 p-4 shadow-xl text-xs max-w-md">
          <div className="flex items-start justify-between gap-2">
            <div>
              <strong className="font-bold text-rose-700">{label(app.error.code)}</strong>
              <p className="text-slate-700 mt-1">
                {app.error.code === "INTERNAL"
                  ? "OCTREX could not complete this action. Try again or inspect the desktop diagnostics."
                  : app.error.message}
              </p>
              {app.error.detail && app.error.code !== "INTERNAL" && (
                <small className="mono text-[10px] text-slate-500 block mt-1">{app.error.detail}</small>
              )}
            </div>
            <button
              className="quiet p-1 text-slate-400 hover:text-slate-700"
              aria-label="Dismiss error"
              onClick={() => app.setError(null)}
            >
              <X size={15} />
            </button>
          </div>
        </div>
      )}

      <OnboardingModal
        isOpen={onboardingOpen}
        onClose={() => setOnboardingOpen(false)}
        core={core}
        providers={app.providers}
        onOpenFolder={() => void app.openProject()}
      />

      <SettingsHub
        isOpen={settingsHubOpen}
        onClose={() => setSettingsHubOpen(false)}
        core={core}
        providers={app.providers}
      />

      {settings && (
        <Settings
          app={app}
          onClose={() => {
            setSettings(false);
            setSettingsProvider(undefined);
          }}
          routingMode={routing}
          initialProvider={settingsProvider}
          customModel={customModel}
          setCustomModel={setCustomModel}
        />
      )}

      {palette && (
        <Palette actions={actions} onClose={() => setPalette(false)} />
      )}

      {consent && (
        <Dialog
          title="Allow cloud AI for this workspace"
          onClose={() => setConsent(false)}
          className="v4-dialog"
        >
          <h2>Allow cloud AI</h2>
          <p>
            Automatic routing and fallback may send your prompt and relevant
            project content to these configured cloud endpoints:
          </p>
          {relevantConsents
            .filter((endpoint) => !endpoint.granted)
            .map((endpoint) => (
              <p key={`${endpoint.providerId}-${endpoint.baseUrl}`}>
                <strong>{endpoint.displayName}</strong>
                <br />
                <span className="mono">{endpoint.baseUrl}</span>
              </p>
            ))}
          <p>Choose Local only to keep model requests on local endpoints.</p>
          <footer>
            <button
              onClick={() => {
                setRouting("LOCAL_ONLY");
                setConsent(false);
              }}
            >
              Use Local only
            </button>
            <button className="primary" onClick={() => void send(true)}>
              Allow and run task
            </button>
          </footer>
        </Dialog>
      )}

      {showSetup && !settings && (
        <SetupPrompt
          invoke={app.invoke}
          codexAvailable={codexAvailable}
          onUseCodex={() => {
            setSetupOpen(false);
            setSetupDismissed(true);
            setMode("AGENT");
            setEngine("CODEX");
          }}
          onAdd={(providerId) => {
            setSetupOpen(false);
            setSetupDismissed(true);
            setSettingsProvider(providerId);
            setSettings(true);
          }}
          onClose={() => {
            setSetupOpen(false);
            setSetupDismissed(true);
          }}
        />
      )}

      <Approvals
        pending={app.pending}
        invoke={app.invoke}
        onRefresh={app.refresh}
      />

      <div className="sr-only" role="status">
        {runningTask
          ? `Task ${label(taskView(runningTask, groupedEvents.get(runningTask.taskId) ?? []).state)}`
          : ""}
      </div>
    </div>
  );
}
