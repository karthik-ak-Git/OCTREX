'use client';

import React from 'react';
import { TaskPlan } from '../lib/backend/types';
import { TaskStepTimeline } from './TaskStepTimeline';
import { Layers, Shield, Clock, Hash, CheckCircle, RefreshCw } from 'lucide-react';

interface TaskPlanViewerProps {
  plan?: TaskPlan | null;
  onReplan?: () => void;
  isReplanning?: boolean;
}

export const TaskPlanViewer: React.FC<TaskPlanViewerProps> = ({ plan, onReplan, isReplanning = false }) => {
  if (!plan) {
    return (
      <div className="p-8 text-center bg-zinc-900/40 rounded-xl border border-zinc-800">
        <Layers className="w-10 h-10 mx-auto text-zinc-600 mb-3" />
        <h3 className="text-base font-semibold text-zinc-300">No Execution Plan Created</h3>
        <p className="text-xs text-zinc-500 max-w-sm mx-auto mt-1">
          An execution plan will be synthesized once the task is submitted to the Orchestrator.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Plan Header & Constraints Card */}
      <div className="p-5 rounded-xl bg-zinc-900/60 border border-zinc-800 space-y-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="flex items-center gap-2">
            <div className="p-2 rounded-lg bg-blue-950/60 border border-blue-800/60 text-blue-400">
              <Layers className="w-5 h-5" />
            </div>
            <div>
              <h3 className="text-base font-bold text-zinc-100 flex items-center gap-2">
                Execution Plan v{plan.version}
                <span className="text-xs font-mono font-normal px-2 py-0.5 rounded bg-zinc-800 text-zinc-400 border border-zinc-700">
                  {plan.status}
                </span>
              </h3>
              <p className="text-xs text-zinc-400 mt-0.5">
                Target Objective: <strong className="text-zinc-200">{plan.objective}</strong>
              </p>
            </div>
          </div>

          {onReplan && (
            <button
              onClick={onReplan}
              disabled={isReplanning}
              className="inline-flex items-center px-3 py-1.5 rounded-lg text-xs font-medium bg-zinc-800 hover:bg-zinc-700 text-zinc-200 border border-zinc-700 transition-colors disabled:opacity-50"
            >
              <RefreshCw className={`w-3.5 h-3.5 mr-1.5 ${isReplanning ? 'animate-spin' : ''}`} />
              Replan Objective
            </button>
          )}
        </div>

        {/* Plan Security Constraints */}
        {plan.constraints && plan.constraints.length > 0 && (
          <div className="pt-3 border-t border-zinc-800/80">
            <h4 className="text-xs font-semibold text-zinc-400 uppercase tracking-wider mb-2 flex items-center gap-1.5">
              <Shield className="w-3.5 h-3.5 text-cyan-400" />
              Security & Execution Constraints
            </h4>
            <div className="flex flex-wrap gap-2">
              {plan.constraints.map((c, i) => (
                <span key={i} className="inline-flex items-center text-xs px-2.5 py-1 rounded-md bg-zinc-950 text-cyan-300 border border-zinc-800 font-mono">
                  🔒 {c}
                </span>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Plan Steps Timeline */}
      <div className="space-y-3">
        <h4 className="text-sm font-semibold text-zinc-300 flex items-center justify-between">
          <span>Execution Steps ({plan.steps.length})</span>
          <span className="text-xs font-normal text-zinc-500 font-mono">Current Step: {plan.current_step + 1} / {plan.steps.length}</span>
        </h4>

        <TaskStepTimeline steps={plan.steps} currentStepIndex={plan.current_step} />
      </div>
    </div>
  );
};
