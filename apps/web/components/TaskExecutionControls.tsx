'use client';

import React from 'react';
import { Play, Pause, XCircle, RotateCcw, ShieldAlert } from 'lucide-react';

interface TaskExecutionControlsProps {
  status: string;
  onStart: () => void;
  onPause: () => void;
  onResume: () => void;
  onCancel: () => void;
  isLoading?: boolean;
}

export const TaskExecutionControls: React.FC<TaskExecutionControlsProps> = ({
  status,
  onStart,
  onPause,
  onResume,
  onCancel,
  isLoading = false,
}) => {
  const normStatus = (status || '').toLowerCase();

  const isTerminal = ['completed', 'failed', 'cancelled', 'blocked'].includes(normStatus);
  const isExecuting = ['executing', 'running', 'in_progress', 'retrying', 'verifying'].includes(normStatus);
  const isPaused = normStatus === 'paused';
  const isWaitingUser = normStatus === 'waitingforuser';
  const isReady = ['created', 'planning', 'planready', 'pending'].includes(normStatus);

  return (
    <div className="flex flex-wrap items-center gap-2 p-3 rounded-xl bg-zinc-900 border border-zinc-800">
      {isReady && (
        <button
          onClick={onStart}
          disabled={isLoading}
          className="inline-flex items-center px-4 py-2 rounded-lg text-xs font-semibold bg-blue-600 hover:bg-blue-500 text-white shadow-lg shadow-blue-950/40 transition-all disabled:opacity-50"
        >
          <Play className="w-4 h-4 mr-1.5" />
          Start Task Execution
        </button>
      )}

      {isExecuting && (
        <button
          onClick={onPause}
          disabled={isLoading}
          className="inline-flex items-center px-4 py-2 rounded-lg text-xs font-semibold bg-amber-600 hover:bg-amber-500 text-white shadow-lg shadow-amber-950/40 transition-all disabled:opacity-50"
        >
          <Pause className="w-4 h-4 mr-1.5" />
          Pause Execution
        </button>
      )}

      {isPaused && (
        <button
          onClick={onResume}
          disabled={isLoading}
          className="inline-flex items-center px-4 py-2 rounded-lg text-xs font-semibold bg-emerald-600 hover:bg-emerald-500 text-white shadow-lg shadow-emerald-950/40 transition-all disabled:opacity-50"
        >
          <Play className="w-4 h-4 mr-1.5" />
          Resume Execution
        </button>
      )}

      {!isTerminal && (
        <button
          onClick={onCancel}
          disabled={isLoading}
          className="inline-flex items-center px-3.5 py-2 rounded-lg text-xs font-medium bg-zinc-800 hover:bg-red-950 hover:text-red-300 text-zinc-300 border border-zinc-700 transition-all disabled:opacity-50"
        >
          <XCircle className="w-4 h-4 mr-1.5" />
          Cancel Task
        </button>
      )}

      {isTerminal && (
        <div className="text-xs text-zinc-500 font-mono flex items-center gap-1.5 px-2">
          <span>Execution state terminal ({status})</span>
        </div>
      )}
    </div>
  );
};
