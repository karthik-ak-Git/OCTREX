import React, { useState } from 'react';
import { Check, Square, Trash2 } from 'lucide-react';
import { BackgroundTask } from '../state';

export type TerminalPanelProps = {
  tasks?: BackgroundTask[];
  activeTaskId?: string;
  onStopTask?: (taskId: string) => void;
  onSelectTask?: (taskId: string) => void;
};

export const TerminalPanel: React.FC<TerminalPanelProps> = ({
  tasks = [
    {
      id: 'task-1',
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
  ],
  activeTaskId,
  onStopTask,
  onSelectTask
}) => {
  const [selectedId, setSelectedId] = useState<string>(activeTaskId || tasks[0]?.id || 'task-1');
  const [clearedIds, setClearedIds] = useState<Set<string>>(new Set());

  const currentTask = tasks.find((t) => t.id === selectedId) || tasks[0];
  const isCleared = clearedIds.has(currentTask?.id || '');

  const otherTasks = tasks.filter((t) => t.id !== currentTask?.id);

  const handleClear = () => {
    if (currentTask) {
      setClearedIds((prev: Set<string>) => new Set(prev).add(currentTask.id));
    }
  };

  return (
    <div className="flex flex-col h-full justify-between p-4 text-slate-800 text-xs select-none overflow-hidden">
      <div className="space-y-3 flex-1 flex flex-col min-h-0">
        {/* CURRENT TASK STATUS BAR (FIGMA IMAGE 7) */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            {currentTask?.status === 'Running' ? (
              <span className="relative flex h-2 w-2">
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75" />
                <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500" />
              </span>
            ) : (
              <span className="h-2 w-2 rounded-full bg-slate-400" />
            )}
            <span className="font-mono text-xs font-semibold text-slate-900 truncate max-w-[200px]">
              {currentTask?.command || 'npm run build'}
            </span>
          </div>

          <div className="flex items-center gap-2">
            <span className="font-mono text-[11px] text-slate-500">
              {currentTask?.status === 'Running' ? currentTask.duration : currentTask?.status}
            </span>
            <button
              onClick={handleClear}
              className="p-1 rounded hover:bg-slate-100 text-slate-400 hover:text-slate-600"
              title="Clear log stream"
            >
              <Trash2 size={13} />
            </button>
          </div>
        </div>

        {/* TERMINAL STREAM OUTPUT BOX */}
        <div className="flex-1 rounded-2xl glass-dark p-4 font-mono text-[11px] text-emerald-400/90 overflow-y-auto custom-scrollbar border border-white/10 shadow-lg leading-relaxed">
          {isCleared ? (
            <div className="text-slate-500 italic">$ Console buffer cleared</div>
          ) : (
            currentTask?.logs.map((line, idx) => (
              <div
                key={idx}
                className={
                  line.startsWith('$')
                    ? 'text-white font-bold'
                    : line.startsWith('✓')
                    ? 'text-emerald-400 font-semibold'
                    : line.includes('error')
                    ? 'text-rose-400 font-semibold'
                    : 'text-slate-300'
                }
              >
                {line || '\u00A0'}
              </div>
            ))
          )}
        </div>

        {/* OTHER TASKS LIST */}
        <div className="space-y-1.5 pt-1">
          <div className="text-[11px] font-semibold tracking-wider text-slate-400 uppercase">
            Other Tasks
          </div>
          {otherTasks.length > 0 ? (
            otherTasks.map((ot) => (
              <div
                key={ot.id}
                onClick={() => {
                  setSelectedId(ot.id);
                  if (onSelectTask) onSelectTask(ot.id);
                }}
                className="flex items-center justify-between rounded-xl glass-card border border-white/80 p-2.5 hover:bg-slate-50/80 cursor-pointer transition-all shadow-2xs"
              >
                <div className="flex items-center gap-2">
                  <Check size={13} className="text-emerald-600" />
                  <span className="font-mono text-slate-700 font-medium">{ot.command}</span>
                </div>
                <span className="text-[11px] text-slate-400 font-mono">
                  {ot.status} • {ot.duration}
                </span>
              </div>
            ))
          ) : (
            <div className="flex items-center justify-between rounded-xl glass-card border border-white/80 p-2.5 shadow-2xs">
              <div className="flex items-center gap-2">
                <Check size={13} className="text-emerald-600" />
                <span className="font-mono text-slate-700 font-medium">pytest</span>
              </div>
              <span className="text-[11px] text-slate-400 font-mono">done in 12s</span>
            </div>
          )}
        </div>
      </div>

      {/* STOP BUTTON (FIGMA IMAGE 7) */}
      <div className="pt-3 border-t border-slate-200/60">
        <button
          onClick={() => {
            if (currentTask && onStopTask) {
              onStopTask(currentTask.id);
            }
          }}
          disabled={currentTask?.status !== 'Running'}
          className="w-full flex items-center justify-center gap-2 rounded-2xl bg-white hover:bg-rose-50 border border-slate-200/90 hover:border-rose-200 text-slate-800 hover:text-rose-600 font-semibold py-2.5 text-xs shadow-2xs disabled:opacity-40 disabled:hover:bg-white disabled:hover:text-slate-800 disabled:hover:border-slate-200 transition-all cursor-pointer"
        >
          <Square size={12} className="fill-current" />
          <span>Stop</span>
        </button>
      </div>
    </div>
  );
};
