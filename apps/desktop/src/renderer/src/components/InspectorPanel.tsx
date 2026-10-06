import React, { useState } from 'react';
import {
  Activity,
  Terminal,
  GitBranch,
  X,
  FileText,
  Image as ImageIcon,
  Minimize2,
  Maximize2
} from 'lucide-react';
import { ActivityInspector } from './ActivityInspector';
import { DocumentPreview } from './DocumentPreview';
import { ImagePreview } from './ImagePreview';
import { TerminalPanel } from './TerminalPanel';
import { ChatSession, BackgroundTask } from '../state';

export type InspectorTab = 'activity' | 'document' | 'image' | 'terminal' | 'changes';

interface InspectorPanelProps {
  activeTab: InspectorTab;
  onTabChange: (tab: InspectorTab) => void;
  onClosePreview?: (tab: InspectorTab) => void;
  openPreviews: Array<{ id: string; name: string; type: 'document' | 'image' }>;
  activeSession: ChatSession;
  tasks: BackgroundTask[];
  onStopTask: (taskId: string) => void;
  onAskEdit: (instruction: string) => void;
}

export const InspectorPanel: React.FC<InspectorPanelProps> = ({
  activeTab,
  onTabChange,
  onClosePreview,
  openPreviews,
  activeSession,
  tasks,
  onStopTask,
  onAskEdit
}) => {
  const [isExpanded, setIsExpanded] = useState(false);

  return (
    <aside
      className={`relative z-20 glass-panel border-l border-white/60 flex flex-col h-full transition-all duration-300 ${
        isExpanded ? 'w-[640px]' : 'w-[420px]'
      }`}
    >
      {/* Tab Navigation Header */}
      <div className="h-14 border-b border-slate-200/60 px-3 flex items-center justify-between bg-white/40 select-none">
        <div className="flex items-center gap-1.5 overflow-x-auto no-scrollbar py-1">
          {/* Main Activity Tab */}
          <button
            onClick={() => onTabChange('activity')}
            className={`px-3 py-1.5 rounded-xl text-xs font-semibold flex items-center gap-1.5 transition-all cursor-pointer ${
              activeTab === 'activity'
                ? 'bg-white text-slate-900 shadow-2xs border border-slate-200/80 font-bold'
                : 'text-slate-500 hover:text-slate-800 hover:bg-white/60'
            }`}
          >
            <Activity className="w-3.5 h-3.5 text-cyan-600" />
            <span>Activity</span>
          </button>

          {/* Terminal Tab */}
          <button
            onClick={() => onTabChange('terminal')}
            className={`px-3 py-1.5 rounded-xl text-xs font-semibold flex items-center gap-1.5 transition-all cursor-pointer ${
              activeTab === 'terminal'
                ? 'bg-white text-slate-900 shadow-2xs border border-slate-200/80 font-bold'
                : 'text-slate-500 hover:text-slate-800 hover:bg-white/60'
            }`}
          >
            <Terminal className="w-3.5 h-3.5 text-emerald-600" />
            <span>Terminal</span>
            {tasks.some((t) => t.status === 'Running') && (
              <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
            )}
          </button>

          {/* Dynamic Open Previews Tabs */}
          {openPreviews.map((preview) => {
            const isActive =
              (preview.type === 'document' && activeTab === 'document') ||
              (preview.type === 'image' && activeTab === 'image');

            return (
              <div
                key={preview.id}
                className={`flex items-center gap-1.5 pl-2.5 pr-1.5 py-1.5 rounded-xl text-xs font-medium transition-all ${
                  isActive
                    ? 'bg-white text-slate-900 shadow-2xs border border-slate-200/80 font-semibold'
                    : 'text-slate-500 hover:text-slate-800 hover:bg-white/60'
                }`}
              >
                <button
                  onClick={() => onTabChange(preview.type)}
                  className="flex items-center gap-1.5 truncate max-w-[130px] cursor-pointer"
                  title={preview.name}
                >
                  {preview.type === 'document' ? (
                    <FileText className="w-3.5 h-3.5 text-amber-500 shrink-0" />
                  ) : (
                    <ImageIcon className="w-3.5 h-3.5 text-purple-500 shrink-0" />
                  )}
                  <span className="truncate">{preview.name}</span>
                </button>
                {onClosePreview && (
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      onClosePreview(preview.type);
                    }}
                    className="p-0.5 rounded hover:bg-slate-200/70 text-slate-400 hover:text-slate-700 cursor-pointer"
                    title="Close tab"
                  >
                    <X className="w-3 h-3" />
                  </button>
                )}
              </div>
            );
          })}

          {/* Changes / Working Tree Tab */}
          <button
            onClick={() => onTabChange('changes')}
            className={`px-3 py-1.5 rounded-xl text-xs font-semibold flex items-center gap-1.5 transition-all cursor-pointer ${
              activeTab === 'changes'
                ? 'bg-white text-slate-900 shadow-2xs border border-slate-200/80 font-bold'
                : 'text-slate-500 hover:text-slate-800 hover:bg-white/60'
            }`}
          >
            <GitBranch className="w-3.5 h-3.5 text-indigo-500" />
            <span>Changes</span>
          </button>
        </div>

        {/* Expand / Minimize Toggle */}
        <div className="flex items-center pl-2">
          <button
            onClick={() => setIsExpanded(!isExpanded)}
            className="p-1.5 rounded-lg hover:bg-white/80 text-slate-400 hover:text-slate-700 transition-colors cursor-pointer"
            title={isExpanded ? 'Collapse width' : 'Expand width'}
          >
            {isExpanded ? <Minimize2 className="w-3.5 h-3.5" /> : <Maximize2 className="w-3.5 h-3.5" />}
          </button>
        </div>
      </div>

      {/* Tab Body View */}
      <div className="flex-1 overflow-hidden">
        {activeTab === 'activity' && (
          <ActivityInspector
            activeSession={activeSession}
            tasks={tasks}
            onOpenTerminalTask={() => onTabChange('terminal')}
            onOpenFile={(fileName) => {
              if (fileName.endsWith('.png') || fileName.endsWith('.jpg') || fileName.endsWith('.svg')) {
                onTabChange('image');
              } else {
                onTabChange('document');
              }
            }}
          />
        )}
        {activeTab === 'document' && (
          <DocumentPreview
            onAskOctrexToEdit={onAskEdit}
          />
        )}
        {activeTab === 'image' && (
          <ImagePreview
            onUseInChat={(img) => {
              onAskEdit(`Analyze and reference ${img} in this solution: `);
            }}
          />
        )}
        {activeTab === 'terminal' && (
          <TerminalPanel
            tasks={tasks}
            onStopTask={onStopTask}
          />
        )}
        {activeTab === 'changes' && (
          <div className="p-6 h-full flex flex-col items-center justify-center text-center text-slate-400 space-y-3">
            <div className="w-14 h-14 rounded-3xl glass-card flex items-center justify-center shadow-sm">
              <GitBranch className="w-7 h-7 text-indigo-500" />
            </div>
            <h4 className="text-sm font-bold text-slate-800">Workspace Working Tree</h4>
            <p className="text-xs text-slate-500 max-w-xs leading-relaxed">
              All modified files, staged diffs, and verification checkpoints appear here in real-time.
            </p>
            <div className="px-3.5 py-1.5 rounded-xl glass-card border border-emerald-200/80 text-emerald-700 text-xs font-mono font-semibold shadow-2xs">
              ✓ Clean working directory ({activeSession.workspaceName} / main)
            </div>
          </div>
        )}
      </div>
    </aside>
  );
};
