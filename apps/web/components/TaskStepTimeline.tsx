'use client';

import React from 'react';
import { TaskStep } from '../lib/backend/types';
import { 
  CheckCircle2, 
  Clock, 
  XCircle, 
  AlertCircle, 
  ArrowRight, 
  FileText, 
  Cpu, 
  Shield, 
  Terminal, 
  UserCheck, 
  CheckCheck,
  ChevronRight
} from 'lucide-react';

interface TaskStepTimelineProps {
  steps: TaskStep[];
  currentStepIndex?: number;
}

export const TaskStepTimeline: React.FC<TaskStepTimelineProps> = ({ steps, currentStepIndex = 0 }) => {
  if (!steps || steps.length === 0) {
    return (
      <div className="p-6 text-center text-zinc-500 bg-zinc-900/50 rounded-xl border border-zinc-800">
        No steps available for this execution plan.
      </div>
    );
  }

  const getActionIcon = (actionType: string) => {
    switch (actionType) {
      case 'ThinkAnalyze':
        return <Cpu className="w-4 h-4 text-cyan-400" />;
      case 'RetrieveContext':
        return <FileText className="w-4 h-4 text-blue-400" />;
      case 'ReadFile':
      case 'WriteFile':
        return <FileText className="w-4 h-4 text-emerald-400" />;
      case 'ExecuteTool':
        return <Terminal className="w-4 h-4 text-amber-400" />;
      case 'ModelCall':
        return <Cpu className="w-4 h-4 text-purple-400" />;
      case 'Verify':
        return <CheckCheck className="w-4 h-4 text-emerald-400" />;
      case 'AskUser':
        return <UserCheck className="w-4 h-4 text-rose-400" />;
      case 'Complete':
        return <CheckCircle2 className="w-4 h-4 text-green-400" />;
      default:
        return <Shield className="w-4 h-4 text-zinc-400" />;
    }
  };

  const getStatusBadge = (status: string) => {
    switch (status.toLowerCase()) {
      case 'completed':
        return <span className="inline-flex items-center text-xs text-emerald-400 bg-emerald-950/40 px-2 py-0.5 rounded border border-emerald-800/40"><CheckCircle2 className="w-3 h-3 mr-1" /> Completed</span>;
      case 'inprogress':
      case 'executing':
        return <span className="inline-flex items-center text-xs text-blue-400 bg-blue-950/40 px-2 py-0.5 rounded border border-blue-800/40 animate-pulse"><Clock className="w-3 h-3 mr-1" /> Executing</span>;
      case 'waitingforuser':
        return <span className="inline-flex items-center text-xs text-purple-300 bg-purple-950/40 px-2 py-0.5 rounded border border-purple-800/40"><UserCheck className="w-3 h-3 mr-1" /> Input Needed</span>;
      case 'failed':
        return <span className="inline-flex items-center text-xs text-red-400 bg-red-950/40 px-2 py-0.5 rounded border border-red-800/40"><XCircle className="w-3 h-3 mr-1" /> Failed</span>;
      case 'skipped':
        return <span className="inline-flex items-center text-xs text-zinc-500 bg-zinc-900 px-2 py-0.5 rounded border border-zinc-800">Skipped</span>;
      default:
        return <span className="inline-flex items-center text-xs text-zinc-400 bg-zinc-900 px-2 py-0.5 rounded border border-zinc-800">Pending</span>;
    }
  };

  return (
    <div className="space-y-4">
      <div className="relative pl-6 space-y-6 before:absolute before:left-2.5 before:top-2 before:bottom-2 before:w-0.5 before:bg-zinc-800">
        {steps.map((step, idx) => {
          const isCurrent = idx === currentStepIndex;

          return (
            <div key={step.id} className="relative group">
              {/* Timeline marker node */}
              <div className={`absolute -left-6 top-1.5 w-5 h-5 rounded-full border flex items-center justify-center text-xs font-semibold ${
                step.status === 'Completed'
                  ? 'bg-emerald-950 border-emerald-500 text-emerald-400'
                  : step.status === 'Failed'
                  ? 'bg-red-950 border-red-500 text-red-400'
                  : isCurrent
                  ? 'bg-blue-950 border-blue-500 text-blue-400 ring-2 ring-blue-500/30'
                  : 'bg-zinc-900 border-zinc-700 text-zinc-500'
              }`}>
                {step.order}
              </div>

              {/* Step Card */}
              <div className={`p-4 rounded-xl border transition-all ${
                isCurrent 
                  ? 'bg-zinc-900/90 border-blue-500/40 shadow-lg shadow-blue-950/20' 
                  : 'bg-zinc-900/40 border-zinc-800/80 hover:border-zinc-700'
              }`}>
                <div className="flex items-start justify-between gap-3">
                  <div className="flex items-center gap-2">
                    <div className="p-1.5 rounded-lg bg-zinc-800/80 border border-zinc-700/50">
                      {getActionIcon(step.action_type)}
                    </div>
                    <div>
                      <h4 className="text-sm font-semibold text-zinc-200 flex items-center gap-2">
                        {step.objective}
                        <span className="text-xs font-mono font-normal text-zinc-500">[{step.action_type}]</span>
                      </h4>
                      {step.dependencies && step.dependencies.length > 0 && (
                        <p className="text-xs text-zinc-500 mt-0.5">
                          Depends on: {step.dependencies.join(', ')}
                        </p>
                      )}
                    </div>
                  </div>
                  <div>
                    {getStatusBadge(step.status)}
                  </div>
                </div>

                {/* Step Metadata & Attempts */}
                <div className="mt-3 pt-3 border-t border-zinc-800/60 flex flex-wrap items-center justify-between gap-2 text-xs text-zinc-400">
                  <div className="flex items-center gap-4">
                    <span>Attempts: <strong className="text-zinc-300">{step.attempts}/{step.max_attempts}</strong></span>
                    {step.verification_required && (
                      <span className="text-emerald-400/90 font-medium">✓ Verifier Required</span>
                    )}
                    {step.requires_user_input && (
                      <span className="text-amber-400/90 font-medium">⚠️ User Consent Required</span>
                    )}
                  </div>
                </div>

                {/* Failure Reason */}
                {step.failure_reason && (
                  <div className="mt-3 p-2.5 rounded-lg bg-red-950/40 border border-red-800/50 text-xs text-red-300 flex items-start gap-2">
                    <AlertCircle className="w-4 h-4 text-red-400 flex-shrink-0 mt-0.5" />
                    <div>
                      <span className="font-semibold text-red-400">Step Failure: </span>
                      {step.failure_reason}
                    </div>
                  </div>
                )}

                {/* Step Output Preview (Non-sensitive) */}
                {step.outputs && (
                  <div className="mt-3 p-2.5 rounded-lg bg-zinc-950 border border-zinc-800/80 font-mono text-xs text-zinc-300">
                    <div className="text-[10px] text-zinc-500 uppercase tracking-wider mb-1 font-sans">Step Output Summary</div>
                    <pre className="whitespace-pre-wrap break-all max-h-32 overflow-y-auto font-mono text-[11px] text-zinc-300">
                      {JSON.stringify(step.outputs, null, 2)}
                    </pre>
                  </div>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
