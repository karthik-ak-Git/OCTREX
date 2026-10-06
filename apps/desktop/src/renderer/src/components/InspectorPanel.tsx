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

export type InspectorTab = 'activity' | 'document' | 'image' | 'terminal' | 'changes';

interface InspectorPanelProps {
  activeTab: InspectorTab;
  onTabChange: (tab: InspectorTab) => void;
  onClosePreview?: (tab: InspectorTab) => void;
  openPreviews: Array<{ id: string; name: string; type: 'document' | 'image' }>;
}

export const InspectorPanel: React.FC<InspectorPanelProps> = ({
  activeTab,
  onTabChange,
  onClosePreview,
  openPreviews
}) => {
  const [isExpanded, setIsExpanded] = useState(false);

  return (
    <aside
      className={`bg-white border-l border-slate-200/80 flex flex-col h-full transition-all duration-200 ${
        isExpanded ? 'w-[640px]' : 'w-[420px]'
      }`}
    >
      {/* Tab Navigation Header */}
      <div className="h-12 border-b border-slate-200/80 px-3 flex items-center justify-between bg-slate-50/50 select-none">
        <div className="flex items-center gap-1 overflow-x-auto no-scrollbar py-1">
          {/* Main Activity Tab */}
          <button
            onClick={() => onTabChange('activity')}
            className={`px-3 py-1.5 rounded-lg text-xs font-semibold flex items-center gap-1.5 transition-colors ${
              activeTab === 'activity'
                ? 'bg-white text-slate-900 shadow-sm border border-slate-200/80'
                : 'text-slate-500 hover:text-slate-800 hover:bg-slate-100/60'
            }`}
          >
            <Activity className="w-3.5 h-3.5 text-cyan-600" />
            <span>Activity</span>
          </button>

          {/* Terminal Tab */}
          <button
            onClick={() => onTabChange('terminal')}
            className={`px-3 py-1.5 rounded-lg text-xs font-semibold flex items-center gap-1.5 transition-colors ${
              activeTab === 'terminal'
                ? 'bg-white text-slate-900 shadow-sm border border-slate-200/80'
                : 'text-slate-500 hover:text-slate-800 hover:bg-slate-100/60'
            }`}
          >
            <Terminal className="w-3.5 h-3.5 text-emerald-600" />
            <span>Terminal</span>
            <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
          </button>

          {/* Dynamic Preview Tabs */}
          {openPreviews.map((preview) => {
            const isActive =
              (preview.type === 'document' && activeTab === 'document') ||
              (preview.type === 'image' && activeTab === 'image');

            return (
              <div
                key={preview.id}
                className={`flex items-center gap-1.5 pl-2.5 pr-1.5 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                  isActive
                    ? 'bg-white text-slate-900 shadow-sm border border-slate-200/80'
                    : 'text-slate-500 hover:text-slate-800 hover:bg-slate-100/60'
                }`}
              >
                <button
                  onClick={() => onTabChange(preview.type)}
                  className="flex items-center gap-1.5 truncate max-w-[130px]"
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
                    className="p-0.5 rounded hover:bg-slate-200 text-slate-400 hover:text-slate-600"
                    title="Close tab"
                  >
                    <X className="w-3 h-3" />
                  </button>
                )}
              </div>
            );
          })}

          {/* Changes / Diff Tab */}
          <button
            onClick={() => onTabChange('changes')}
            className={`px-3 py-1.5 rounded-lg text-xs font-semibold flex items-center gap-1.5 transition-colors ${
              activeTab === 'changes'
                ? 'bg-white text-slate-900 shadow-sm border border-slate-200/80'
                : 'text-slate-500 hover:text-slate-800 hover:bg-slate-100/60'
            }`}
          >
            <GitBranch className="w-3.5 h-3.5 text-indigo-500" />
            <span>Changes</span>
          </button>
        </div>

        {/* Expand / Minimize Width */}
        <div className="flex items-center pl-2">
          <button
            onClick={() => setIsExpanded(!isExpanded)}
            className="p-1.5 rounded-md hover:bg-slate-200/70 text-slate-400 hover:text-slate-700 transition-colors"
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
            onOpenTerminalTask={() => onTabChange('terminal')}
            onOpenFile={(fileName: string) => {
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
            onAskOctrexToEdit={() => {
              console.log('Ask edit');
            }}
          />
        )}
        {activeTab === 'image' && (
          <ImagePreview
            onUseInChat={() => {
              console.log('Use image in chat');
            }}
          />
        )}
        {activeTab === 'terminal' && (
          <TerminalPanel
            onStop={() => onTabChange('activity')}
          />
        )}
        {activeTab === 'changes' && (
          <div className="p-6 h-full flex flex-col items-center justify-center text-center text-slate-400">
            <div className="w-12 h-12 rounded-2xl bg-indigo-50 flex items-center justify-center mb-3">
              <GitBranch className="w-6 h-6 text-indigo-500" />
            </div>
            <h4 className="text-sm font-semibold text-slate-700">Workspace Working Tree</h4>
            <p className="text-xs text-slate-500 mt-1 max-w-xs">
              All modified files, diffs, and staged commits will appear here in real-time.
            </p>
            <div className="mt-4 px-3 py-1.5 rounded-lg bg-emerald-50 border border-emerald-200 text-emerald-700 text-xs font-mono font-medium">
              ✓ Clean working directory (main branch)
            </div>
          </div>
        )}
      </div>
    </aside>
  );
};
