import { Check, Square, Terminal as TerminalIcon, Play, RefreshCw } from 'lucide-react'
import type { CommandView } from './state'

export type TerminalPanelProps = {
  command?: string
  status?: string
  duration?: string
  output?: string
  otherTasks?: Array<{ name: string; duration: string; completed: boolean }>
  onStop?: () => void
  onSelectTask?: (taskName: string) => void
}

export function TerminalPanel({
  command = 'npm run build',
  status = 'Running',
  duration = '00:42',
  output = `$ npm run build\n\n> octrex-web@1.4.0 build\n> vite build\n\nvite v5.4 building for production...\n✓ 214 modules transformed.\ndist/index.html   0.46 kB\ndist/assets/app.js  182.30 kB\ndist/assets/app.css 14.12 kB\n\nrunning type check...`,
  otherTasks = [
    { name: 'pytest', duration: 'done in 12s', completed: true },
  ],
  onStop,
  onSelectTask,
}: TerminalPanelProps) {
  return (
    <div className="flex flex-col h-full justify-between p-4 text-slate-800 text-xs select-none">
      <div className="space-y-3 flex-1 flex flex-col min-h-0">
        {/* CURRENT TASK STATUS BAR */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <span className="relative flex h-2 w-2">
              <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-slate-400 opacity-75"></span>
              <span className="relative inline-flex rounded-full h-2 w-2 bg-slate-900"></span>
            </span>
            <span className="font-mono text-xs font-semibold text-slate-900">{command}</span>
          </div>
          <span className="rounded-full bg-slate-100 px-2.5 py-0.5 text-[11px] font-medium text-slate-600 border border-slate-200/60">
            {status} ⌵
          </span>
        </div>

        {/* TERMINAL CONSOLE VIEW */}
        <div className="flex-1 min-h-[220px] rounded-2xl bg-white border border-slate-200/80 p-4 font-mono text-[11px] leading-relaxed text-slate-700 overflow-y-auto whitespace-pre-wrap shadow-xs custom-scrollbar">
          {output}
        </div>

        {/* BOTTOM RUNNING STATUS & STOP BUTTON */}
        <div className="flex items-center justify-between pt-1">
          <div className="text-[11px] text-slate-500 font-mono">
            {status} · {duration}
          </div>
          <button
            onClick={onStop}
            className="flex items-center gap-1.5 rounded-xl border border-slate-200 bg-white px-3.5 py-1.5 text-xs font-medium text-slate-700 hover:bg-slate-50 hover:border-slate-300 transition-colors shadow-2xs"
          >
            <Square size={11} className="fill-slate-700" />
            <span>Stop</span>
          </button>
        </div>

        {/* OTHER TASKS LIST */}
        <div className="space-y-1.5 pt-3 border-t border-slate-100">
          <div className="text-[11px] font-semibold tracking-wider text-slate-400 uppercase">
            Other Tasks
          </div>
          <div className="space-y-1.5">
            {otherTasks.map((t) => (
              <div
                key={t.name}
                onClick={() => onSelectTask?.(t.name)}
                className="flex items-center gap-2 rounded-xl bg-white p-2.5 border border-slate-150 shadow-2xs hover:border-slate-300 cursor-pointer transition-colors"
              >
                <Check size={13} className="text-emerald-600" />
                <span className="font-mono text-xs font-medium text-slate-800">{t.name}</span>
                <span className="text-[11px] text-slate-400 font-mono ml-auto">· {t.duration}</span>
              </div>
            ))}
          </div>
          <div className="text-[10px] text-slate-400 pt-1">
            Tip: double-click a background task to reopen it here.
          </div>
        </div>
      </div>
    </div>
  )
}
