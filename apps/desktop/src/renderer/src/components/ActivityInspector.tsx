import { FileCode, Image as ImageIcon } from 'lucide-react'

export type ActivityInspectorProps = {
  agents?: Array<{ agentId: string; label: string; role: string; status: string; summary: string }>
  skillsRead?: string[]
  toolsUsed?: Record<string, number>
  createdFiles?: Array<{ name: string; size?: string; slides?: number }>
  uploadedFiles?: Array<{ name: string; size?: string }>
  backgroundTasks?: Array<{ id: string; command: string; status: string; duration: string }>
  contextUsedPercent?: number
  contextTokens?: { used: number; total: number }
  onOpenFile?: (fileName: string) => void
  onOpenTerminalTask?: (taskId: string) => void
}

export function ActivityInspector({
  agents = [
    { agentId: 'main', label: 'Main agent', role: 'Architect', status: 'running', summary: 'Building deck' },
    { agentId: 'sub', label: 'Sub-agent', role: 'Tester', status: 'running', summary: 'Running build check' },
  ],
  skillsRead = ['pptx', 'file-reading', 'frontend-design'],
  toolsUsed = {
    'Read file': 4,
    'Write file': 2,
    Terminal: 3,
    'Web search': 1,
  },
  createdFiles = [
    { name: 'roadmap-q4.pptx', slides: 6 },
    { name: 'outline.md', size: '2 KB' },
  ],
  uploadedFiles = [
    { name: 'brand-guide.png', size: '240 KB' },
    { name: 'notes.pdf', size: '1.1 MB' },
  ],
  backgroundTasks = [
    { id: 'task-1', command: 'npm run build', status: 'Running', duration: '00:42' },
  ],
  contextUsedPercent = 42,
  contextTokens = { used: 84000, total: 200000 },
  onOpenFile,
  onOpenTerminalTask,
}: ActivityInspectorProps) {
  return (
    <div className="flex flex-col gap-6 p-4 text-slate-800 text-xs select-none">
      {/* AGENTS (FIGMA IMAGE 4) */}
      <div>
        <div className="flex items-center justify-between mb-2.5">
          <span className="text-[11px] font-semibold tracking-wider text-slate-400 uppercase">Agents</span>
          <span className="rounded-full bg-slate-100 px-2 py-0.5 text-[10px] font-medium text-slate-600 border border-slate-200/60">
            {agents.length} active
          </span>
        </div>
        <div className="space-y-1.5">
          {agents.map((agent) => (
            <div
              key={agent.agentId}
              className="flex items-center gap-2.5 rounded-xl bg-white p-2.5 border border-slate-150 shadow-2xs"
            >
              <span className="relative flex h-2 w-2">
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
              </span>
              <div className="flex-1 min-w-0">
                <div className="text-xs font-semibold text-slate-900 truncate">{agent.label}</div>
                <div className="text-[11px] text-slate-500 truncate">{agent.summary}</div>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* SKILLS READ */}
      <div>
        <div className="text-[11px] font-semibold tracking-wider text-slate-400 uppercase mb-2">
          Skills Read
        </div>
        <div className="flex flex-wrap gap-1.5">
          {skillsRead.map((skill) => (
            <span
              key={skill}
              className="rounded-full bg-white px-3 py-1 text-[11px] font-medium text-slate-700 border border-slate-200 shadow-2xs"
            >
              {skill}
            </span>
          ))}
        </div>
      </div>

      {/* TOOLS USED */}
      <div>
        <div className="text-[11px] font-semibold tracking-wider text-slate-400 uppercase mb-2">
          Tools Used
        </div>
        <div className="space-y-1 bg-white rounded-xl p-2.5 border border-slate-150 shadow-2xs">
          {Object.entries(toolsUsed).map(([tool, count]) => (
            <div key={tool} className="flex items-center justify-between py-1 text-slate-600 border-b border-slate-50 last:border-0">
              <span className="text-xs font-medium text-slate-700">{tool}</span>
              <span className="text-xs font-mono text-slate-500 font-semibold">{count}</span>
            </div>
          ))}
        </div>
      </div>

      {/* CREATED FILES */}
      {createdFiles.length > 0 && (
        <div>
          <div className="text-[11px] font-semibold tracking-wider text-slate-400 uppercase mb-2">
            Created Files
          </div>
          <div className="space-y-1.5">
            {createdFiles.map((file) => (
              <div
                key={file.name}
                onClick={() => onOpenFile?.(file.name)}
                className="flex items-center justify-between rounded-xl bg-white p-2.5 border border-slate-150 shadow-2xs hover:border-slate-300 cursor-pointer transition-colors"
              >
                <div className="flex items-center gap-2 truncate">
                  <FileCode size={14} className="text-slate-500 shrink-0" />
                  <span className="text-xs font-medium text-slate-800 truncate">{file.name}</span>
                </div>
                <span className="text-[11px] text-slate-400 shrink-0">
                  {file.slides ? `${file.slides} slides` : file.size}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* UPLOADED FILES */}
      {uploadedFiles.length > 0 && (
        <div>
          <div className="text-[11px] font-semibold tracking-wider text-slate-400 uppercase mb-2">
            Uploaded
          </div>
          <div className="space-y-1.5">
            {uploadedFiles.map((file) => (
              <div
                key={file.name}
                onClick={() => onOpenFile?.(file.name)}
                className="flex items-center justify-between rounded-xl bg-white p-2.5 border border-slate-150 shadow-2xs hover:border-slate-300 cursor-pointer transition-colors"
              >
                <div className="flex items-center gap-2 truncate">
                  <ImageIcon size={14} className="text-slate-500 shrink-0" />
                  <span className="text-xs font-medium text-slate-800 truncate">{file.name}</span>
                </div>
                <span className="text-[11px] text-slate-400 shrink-0">{file.size}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* BACKGROUND TASKS */}
      {backgroundTasks.length > 0 && (
        <div>
          <div className="text-[11px] font-semibold tracking-wider text-slate-400 uppercase mb-2">
            Background Tasks
          </div>
          <div className="space-y-1.5">
            {backgroundTasks.map((task) => (
              <div
                key={task.id}
                onDoubleClick={() => onOpenTerminalTask?.(task.id)}
                className="flex items-center gap-2 rounded-xl bg-white p-2.5 border border-slate-150 shadow-2xs hover:border-slate-300 cursor-pointer transition-colors"
              >
                <span className="h-2 w-2 rounded-full bg-slate-900"></span>
                <div className="flex-1 truncate font-mono text-xs font-semibold text-slate-900">{task.command}</div>
                <span className="text-[11px] text-slate-500">{task.status} · {task.duration}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* CONTEXT GAUGE */}
      <div>
        <div className="flex items-center justify-between mb-1.5 text-[11px] text-slate-500">
          <span className="font-semibold tracking-wider text-slate-400 uppercase">Context</span>
          <span>{contextUsedPercent}% used</span>
        </div>
        <div className="h-1.5 w-full overflow-hidden rounded-full bg-slate-200">
          <div
            className="h-full bg-slate-900 transition-all duration-500"
            style={{ width: `${contextUsedPercent}%` }}
          />
        </div>
        <div className="mt-1 text-[10px] text-slate-400">
          {Math.round(contextTokens.used / 1000)}k / {Math.round(contextTokens.total / 1000)}k tokens
        </div>
      </div>
    </div>
  )
}
