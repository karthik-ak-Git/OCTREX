import { FileCode, Image as ImageIcon, Terminal, Activity, Layers } from 'lucide-react';
import { ChatSession, BackgroundTask } from '../state';

export type ActivityInspectorProps = {
  activeSession: ChatSession;
  tasks: BackgroundTask[];
  onOpenFile: (fileName: string) => void;
  onOpenTerminalTask: (taskId: string) => void;
};

export const ActivityInspector: React.FC<ActivityInspectorProps> = ({
  activeSession,
  tasks,
  onOpenFile,
  onOpenTerminalTask
}) => {
  const tokenMax = 200000;
  const tokenUsed = activeSession.tokensUsed || 84200;
  const tokenPercent = Math.min(Math.round((tokenUsed / tokenMax) * 100), 100);

  // Derive created files from session artifacts
  const createdArtifacts = activeSession.messages
    .flatMap((m) => m.artifacts || [])
    .filter((v, i, a) => a.findIndex((t) => t.name === v.name) === i);

  const skillsRead = activeSession.skillsUsed.length > 0
    ? activeSession.skillsUsed
    : ['pptx', 'file-reading', 'frontend-design'];

  const toolsCount = Object.keys(activeSession.toolsCount).length > 0
    ? activeSession.toolsCount
    : { 'Read file': 4, 'Write file': 2, Terminal: 3, 'Web search': 1 };

  return (
    <div className="flex flex-col h-full justify-between p-4 space-y-6 text-slate-800 text-xs select-none overflow-y-auto custom-scrollbar">
      <div className="space-y-6">
        {/* ACTIVE AGENTS (FIGMA IMAGE 4) */}
        <div>
          <div className="flex items-center justify-between text-[11px] font-semibold text-slate-400 uppercase tracking-wider mb-2.5">
            <span className="flex items-center gap-1.5">
              <Activity size={13} className="text-cyan-600" />
              <span>Agents</span>
            </span>
            <span className="rounded-full bg-slate-100 text-slate-700 px-2 py-0.5 text-[10px] font-bold">
              2 active
            </span>
          </div>

          <div className="space-y-2">
            <div className="rounded-2xl glass-card border border-white/80 p-3 shadow-2xs space-y-1">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <span className="h-2 w-2 rounded-full bg-emerald-500 animate-pulse" />
                  <span className="font-semibold text-slate-900">Main agent</span>
                </div>
                <span className="text-[10px] rounded-md bg-slate-100 px-1.5 py-0.5 font-medium text-slate-600">
                  Architect
                </span>
              </div>
              <div className="text-[11px] text-slate-500">Executing task in {activeSession.workspaceName}</div>
            </div>

            <div className="rounded-2xl glass-card border border-white/80 p-3 shadow-2xs space-y-1">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <span className="h-2 w-2 rounded-full bg-cyan-500 animate-pulse" />
                  <span className="font-semibold text-slate-900">Sub-agent</span>
                </div>
                <span className="text-[10px] rounded-md bg-slate-100 px-1.5 py-0.5 font-medium text-slate-600">
                  Verifier
                </span>
              </div>
              <div className="text-[11px] text-slate-500">Checking build outputs & verification</div>
            </div>
          </div>
        </div>

        {/* SKILLS READ */}
        <div>
          <div className="flex items-center gap-1.5 text-[11px] font-semibold text-slate-400 uppercase tracking-wider mb-2">
            <Layers size={13} className="text-indigo-500" />
            <span>Skills read</span>
          </div>
          <div className="flex flex-wrap gap-1.5">
            {skillsRead.map((sk) => (
              <span
                key={sk}
                className="rounded-lg glass-card px-2.5 py-1 font-mono text-[11px] font-medium text-slate-700 border border-slate-200/80 shadow-2xs"
              >
                {sk}
              </span>
            ))}
          </div>
        </div>

        {/* TOOLS USED BREAKDOWN TABLE */}
        <div>
          <div className="text-[11px] font-semibold text-slate-400 uppercase tracking-wider mb-2">
            Tools used
          </div>
          <div className="rounded-2xl glass-card border border-white/80 p-2 shadow-2xs divide-y divide-slate-100">
            {Object.entries(toolsCount).map(([tool, count]) => (
              <div key={tool} className="flex items-center justify-between px-2.5 py-1.5 text-xs">
                <span className="text-slate-700 font-medium">{tool}</span>
                <span className="font-mono text-slate-900 font-bold bg-slate-100 px-2 py-0.5 rounded-md text-[11px]">
                  {count}
                </span>
              </div>
            ))}
          </div>
        </div>

        {/* CREATED FILES */}
        <div>
          <div className="text-[11px] font-semibold text-slate-400 uppercase tracking-wider mb-2">
            Created files
          </div>
          <div className="space-y-1.5">
            {createdArtifacts.length > 0 ? (
              createdArtifacts.map((file) => (
                <div
                  key={file.id}
                  onClick={() => onOpenFile(file.name)}
                  className="flex items-center justify-between rounded-xl glass-card border border-white/80 p-2.5 hover:bg-slate-50/80 cursor-pointer transition-all shadow-2xs group"
                >
                  <div className="flex items-center gap-2">
                    <FileCode size={14} className="text-amber-500 group-hover:scale-110 transition-transform" />
                    <span className="font-semibold text-slate-800">{file.name}</span>
                  </div>
                  <span className="text-[11px] font-mono text-slate-400">
                    {file.slidesCount ? `${file.slidesCount} slides` : file.size}
                  </span>
                </div>
              ))
            ) : (
              <div
                onClick={() => onOpenFile('roadmap-q4.pptx')}
                className="flex items-center justify-between rounded-xl glass-card border border-white/80 p-2.5 hover:bg-slate-50/80 cursor-pointer transition-all shadow-2xs group"
              >
                <div className="flex items-center gap-2">
                  <FileCode size={14} className="text-amber-500 group-hover:scale-110 transition-transform" />
                  <span className="font-semibold text-slate-800">roadmap-q4.pptx</span>
                </div>
                <span className="text-[11px] font-mono text-slate-400">6 slides</span>
              </div>
            )}
          </div>
        </div>

        {/* UPLOADED ASSETS */}
        <div>
          <div className="text-[11px] font-semibold text-slate-400 uppercase tracking-wider mb-2">
            Uploaded files
          </div>
          <div className="space-y-1.5">
            {[
              { name: 'brand-guide.png', size: '240 KB' },
              { name: 'notes.pdf', size: '1.1 MB' }
            ].map((file) => (
              <div
                key={file.name}
                onClick={() => onOpenFile(file.name)}
                className="flex items-center justify-between rounded-xl glass-card border border-white/80 p-2.5 hover:bg-slate-50/80 cursor-pointer transition-all shadow-2xs group"
              >
                <div className="flex items-center gap-2">
                  <ImageIcon size={14} className="text-purple-500 group-hover:scale-110 transition-transform" />
                  <span className="font-semibold text-slate-800">{file.name}</span>
                </div>
                <span className="text-[11px] font-mono text-slate-400">{file.size}</span>
              </div>
            ))}
          </div>
        </div>

        {/* BACKGROUND TASKS */}
        <div>
          <div className="text-[11px] font-semibold text-slate-400 uppercase tracking-wider mb-2">
            Background tasks
          </div>
          <div className="space-y-1.5">
            {tasks.map((task) => (
              <div
                key={task.id}
                onClick={() => onOpenTerminalTask(task.id)}
                className="flex items-center justify-between rounded-xl glass-card border border-white/80 p-2.5 hover:bg-slate-50/80 cursor-pointer transition-all shadow-2xs"
              >
                <div className="flex items-center gap-2">
                  <Terminal size={14} className="text-emerald-600" />
                  <span className="font-mono text-xs font-semibold text-slate-900">{task.command}</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-[10px] rounded-md bg-emerald-100 text-emerald-800 font-semibold px-1.5 py-0.5">
                    {task.status}
                  </span>
                  <span className="font-mono text-[11px] text-slate-400">{task.duration}</span>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* CONTEXT BUDGET GAUGE */}
      <div className="rounded-2xl glass-card border border-white/80 p-3.5 space-y-2 shadow-2xs">
        <div className="flex items-center justify-between">
          <span className="text-slate-600 font-semibold">Context window</span>
          <span className="font-mono text-[11px] font-bold text-slate-900">{tokenPercent}% used</span>
        </div>
        <div className="h-1.5 w-full rounded-full bg-slate-100 overflow-hidden">
          <div
            className="h-full rounded-full bg-gradient-to-r from-cyan-500 to-emerald-500 transition-all duration-500"
            style={{ width: `${tokenPercent}%` }}
          />
        </div>
        <div className="flex items-center justify-between text-[10px] text-slate-400 font-mono">
          <span>{tokenUsed.toLocaleString()} tokens</span>
          <span>{tokenMax.toLocaleString()} limit</span>
        </div>
      </div>
    </div>
  );
};
