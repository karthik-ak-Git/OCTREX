'use client';

import React, { useState, useEffect, useCallback, useRef } from 'react';
import Link from 'next/link';
import {
  Plus,
  RefreshCw,
  ChevronRight,
  ArrowLeft,
  Cpu,
  Activity,
  AlertTriangle,
  CheckCircle2,
  XCircle,
  Clock,
  RotateCcw,
  Shield,
  ListChecks,
  Layers,
  ExternalLink,
} from 'lucide-react';
import { backendClient } from '../../lib/backend/client';
import type {
  TaskSummary,
  TaskPlan,
  TaskStep,
  OrchestratorState,
  ExecutionDecision,
} from '../../lib/backend/types';
import { TaskStatusBadge } from '../../components/TaskStatusBadge';
import { TaskStepTimeline } from '../../components/TaskStepTimeline';
import { TaskPlanViewer } from '../../components/TaskPlanViewer';
import { TaskExecutionControls } from '../../components/TaskExecutionControls';
import { TaskWaitingForUser } from '../../components/TaskWaitingForUser';
import { VerificationStatus } from '../../components/VerificationStatus';
import { CompletionGateStatus } from '../../components/CompletionGateStatus';
import { VerificationFailure } from '../../components/VerificationFailure';
import type { TaskCompletionResponse } from '../../lib/backend/types';

// ─── Helpers ─────────────────────────────────────────────────────────────────

function timeAgo(ts: number): string {
  const seconds = Math.floor((Date.now() - ts * 1000) / 1000);
  if (seconds < 60) return `${seconds}s ago`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  return `${hours}h ago`;
}

function isActiveStatus(status: string): boolean {
  const s = status.toLowerCase();
  return ['executing', 'running', 'in_progress', 'planning', 'retrying', 'verifying', 'waitingfortool'].includes(s);
}

function isWaitingForUser(status: string): boolean {
  return status.toLowerCase() === 'waitingforuser';
}

// ─── Create Task Panel ────────────────────────────────────────────────────────

interface CreateTaskPanelProps {
  onCreated: (task: TaskSummary, plan?: TaskPlan) => void;
}

function CreateTaskPanel({ onCreated }: CreateTaskPanelProps) {
  const [objective, setObjective] = useState('');
  const [workspacePath, setWorkspacePath] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleCreate = async () => {
    if (!objective.trim()) return;
    setIsLoading(true);
    setError(null);
    try {
      const result = await backendClient.createTask(objective.trim());
      if (result.success && result.task) {
        onCreated(result.task, result.plan ?? undefined);
        setObjective('');
        setWorkspacePath('');
      } else {
        setError(result.error ?? 'Failed to create task');
      }
    } catch (e: any) {
      setError(e.message ?? 'Unknown error');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="bg-zinc-900 border border-zinc-800 rounded-2xl p-5 space-y-4">
      <div className="flex items-center gap-2">
        <Layers className="w-4 h-4 text-blue-400" />
        <h3 className="text-sm font-semibold text-zinc-100">Create Orchestration Task</h3>
      </div>

      <div className="space-y-3">
        <div>
          <label className="block text-[10px] font-bold uppercase tracking-widest text-zinc-500 mb-1.5">
            Objective
          </label>
          <textarea
            value={objective}
            onChange={(e) => setObjective(e.target.value)}
            placeholder="Describe what the agent should accomplish..."
            rows={3}
            className="w-full px-3 py-2.5 bg-zinc-800 border border-zinc-700 rounded-xl text-xs text-zinc-100 placeholder-zinc-600 focus:outline-none focus:ring-2 focus:ring-blue-500/50 resize-none"
          />
        </div>

        <div>
          <label className="block text-[10px] font-bold uppercase tracking-widest text-zinc-500 mb-1.5">
            Workspace path <span className="text-zinc-600">(optional)</span>
          </label>
          <input
            value={workspacePath}
            onChange={(e) => setWorkspacePath(e.target.value)}
            placeholder="/path/to/project"
            className="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-xl text-xs text-zinc-100 placeholder-zinc-600 focus:outline-none focus:ring-2 focus:ring-blue-500/50 font-mono"
          />
        </div>
      </div>

      {error && (
        <div className="flex items-center gap-2 px-3 py-2 bg-red-950/50 border border-red-800/50 rounded-lg text-xs text-red-300">
          <XCircle className="w-3.5 h-3.5 flex-shrink-0" />
          {error}
        </div>
      )}

      <button
        onClick={handleCreate}
        disabled={isLoading || !objective.trim()}
        className="w-full flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl bg-blue-600 hover:bg-blue-500 disabled:bg-zinc-800 disabled:text-zinc-600 text-white text-xs font-semibold transition-all shadow-lg shadow-blue-950/40"
      >
        {isLoading ? (
          <>
            <RefreshCw className="w-3.5 h-3.5 animate-spin" />
            Creating task…
          </>
        ) : (
          <>
            <Plus className="w-3.5 h-3.5" />
            Create &amp; Plan Task
          </>
        )}
      </button>
    </div>
  );
}

// ─── Task List Item ───────────────────────────────────────────────────────────

interface TaskListItemProps {
  task: TaskSummary;
  isSelected: boolean;
  onClick: () => void;
}

function TaskListItem({ task, isSelected, onClick }: TaskListItemProps) {
  return (
    <button
      onClick={onClick}
      className={`w-full text-left p-3.5 rounded-xl border transition-all ${
        isSelected
          ? 'bg-zinc-800 border-zinc-600 shadow-lg'
          : 'bg-zinc-900/60 border-zinc-800 hover:bg-zinc-800/60 hover:border-zinc-700'
      }`}
    >
      <div className="flex items-start justify-between gap-2">
        <div className="flex-1 min-w-0">
          <p className="text-xs font-semibold text-zinc-100 truncate">{task.title}</p>
          <p className="text-[10px] text-zinc-500 font-mono mt-0.5 truncate">{task.id}</p>
        </div>
        <ChevronRight className={`w-3.5 h-3.5 flex-shrink-0 mt-0.5 transition-colors ${isSelected ? 'text-blue-400' : 'text-zinc-600'}`} />
      </div>
      <div className="flex items-center justify-between mt-2">
        <TaskStatusBadge status={task.status} />
        <span className="text-[10px] text-zinc-600 font-mono">{timeAgo(task.updated_at)}</span>
      </div>
    </button>
  );
}

// ─── Task Detail Panel ────────────────────────────────────────────────────────

interface TaskDetailPanelProps {
  taskId: string;
  onBack: () => void;
}

function TaskDetailPanel({ taskId, onBack }: TaskDetailPanelProps) {
  const [task, setTask] = useState<TaskSummary | null>(null);
  const [plan, setPlan] = useState<TaskPlan | null>(null);
  const [steps, setSteps] = useState<TaskStep[]>([]);
  const [currentStep, setCurrentStep] = useState<number>(0);
  const [isLoading, setIsLoading] = useState(false);
  const [actionLoading, setActionLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [lastDecision, setLastDecision] = useState<ExecutionDecision | null>(null);
  const [completion, setCompletion] = useState<TaskCompletionResponse | null>(null);
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const loadTask = useCallback(async () => {
    try {
      const [taskRes, stepsRes, completionRes] = await Promise.all([
        backendClient.getTaskById(taskId),
        backendClient.getTaskSteps(taskId),
        backendClient.getTaskCompletion(taskId).catch(() => null),
      ]);

      if (taskRes.success && taskRes.task) {
        setTask(taskRes.task);
        setPlan(taskRes.plan ?? null);
      }
      if (stepsRes.success) {
        setSteps(stepsRes.steps ?? []);
        setCurrentStep(stepsRes.current_step ?? 0);
      }
      if (completionRes && completionRes.success) {
        setCompletion(completionRes);
      }
    } catch (e: any) {
      setError(e.message);
    }
  }, [taskId]);

  // Initial load
  useEffect(() => {
    setTask(null);
    setPlan(null);
    setSteps([]);
    setError(null);
    setLastDecision(null);
    setIsLoading(true);
    loadTask().finally(() => setIsLoading(false));
  }, [taskId, loadTask]);

  // Poll while task is active or waiting for user
  useEffect(() => {
    if (pollRef.current) clearInterval(pollRef.current);

    const shouldPoll = task && (isActiveStatus(task.status) || isWaitingForUser(task.status));
    if (shouldPoll) {
      pollRef.current = setInterval(loadTask, 3000);
    }

    return () => {
      if (pollRef.current) clearInterval(pollRef.current);
    };
  }, [task?.status, loadTask]);

  const handleStart = async () => {
    if (!task) return;
    setActionLoading(true);
    try {
      const res = await backendClient.startTask(task.id);
      if (res.success) await loadTask();
      else setError(res.message ?? 'Failed to start');
    } catch (e: any) {
      setError(e.message);
    } finally {
      setActionLoading(false);
    }
  };

  const handlePause = async () => {
    if (!task) return;
    setActionLoading(true);
    try {
      await backendClient.pauseTask(task.id);
      await loadTask();
    } catch (e: any) {
      setError(e.message);
    } finally {
      setActionLoading(false);
    }
  };

  const handleResume = async () => {
    if (!task) return;
    setActionLoading(true);
    try {
      const res = await backendClient.resumeTask(task.id);
      if (res.success) await loadTask();
      else setError(res.message ?? 'Failed to resume');
    } catch (e: any) {
      setError(e.message);
    } finally {
      setActionLoading(false);
    }
  };

  const handleCancel = async () => {
    if (!task) return;
    setActionLoading(true);
    try {
      await backendClient.cancelTask(task.id);
      await loadTask();
    } catch (e: any) {
      setError(e.message);
    } finally {
      setActionLoading(false);
    }
  };

  const handleReplan = async (revisedObjective: string) => {
    if (!task) return;
    try {
      const res = await backendClient.replanTask(task.id, revisedObjective);
      if (res.success && res.plan) setPlan(res.plan);
      await loadTask();
    } catch (e: any) {
      setError(e.message);
    }
  };

  const handleUserInput = async (input: string) => {
    if (!task) return;
    setActionLoading(true);
    try {
      const res = await backendClient.submitUserInput(task.id, input);
      if (res.success) {
        setLastDecision(res.decision ?? null);
        await loadTask();
      } else {
        setError(res.error ?? 'Failed to submit input');
      }
    } catch (e: any) {
      setError(e.message);
    } finally {
      setActionLoading(false);
    }
  };

  if (isLoading) {
    return (
      <div className="flex-1 flex items-center justify-center text-zinc-500">
        <RefreshCw className="w-5 h-5 animate-spin mr-2" />
        <span className="text-sm">Loading task…</span>
      </div>
    );
  }

  if (!task) {
    return (
      <div className="flex-1 flex items-center justify-center text-zinc-500 text-sm">
        Task not found
      </div>
    );
  }

  return (
    <div className="flex-1 flex flex-col gap-4 min-h-0 overflow-y-auto">

      {/* Header */}
      <div className="flex items-start justify-between gap-3 flex-shrink-0">
        <div className="flex items-start gap-3 min-w-0">
          <button onClick={onBack} className="mt-0.5 p-1.5 rounded-lg hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 transition-colors">
            <ArrowLeft className="w-4 h-4" />
          </button>
          <div className="min-w-0">
            <h2 className="text-sm font-semibold text-zinc-100 leading-tight">{task.title}</h2>
            <p className="text-[10px] font-mono text-zinc-500 mt-0.5 truncate">{task.id}</p>
          </div>
        </div>
        <div className="flex items-center gap-2 flex-shrink-0">
          <TaskStatusBadge status={task.status} />
          <button
            onClick={loadTask}
            className="p-1.5 rounded-lg hover:bg-zinc-800 text-zinc-500 hover:text-zinc-200 transition-colors"
            title="Refresh"
          >
            <RefreshCw className="w-3.5 h-3.5" />
          </button>
          <Link
            href={`/tasks/${encodeURIComponent(task.id)}/verification`}
            className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-zinc-400 hover:text-zinc-100 text-xs transition-colors"
          >
            <Shield className="w-3.5 h-3.5" />
            Verify
            <ExternalLink className="w-3 h-3" />
          </Link>
        </div>
      </div>

      {/* Error banner */}
      {error && (
        <div className="flex items-center gap-2 px-3 py-2 bg-red-950/50 border border-red-800/50 rounded-lg text-xs text-red-300 flex-shrink-0">
          <XCircle className="w-3.5 h-3.5 flex-shrink-0" />
          <span>{error}</span>
          <button onClick={() => setError(null)} className="ml-auto text-red-400 hover:text-red-200">✕</button>
        </div>
      )}

      {/* Last decision */}
      {lastDecision && (
        <div className={`flex items-center gap-2 px-3 py-2 rounded-lg text-xs border flex-shrink-0 ${
          lastDecision.type === 'Complete' ? 'bg-green-950/40 border-green-800/50 text-green-300' :
          lastDecision.type === 'Fail' ? 'bg-red-950/40 border-red-800/50 text-red-300' :
          lastDecision.type === 'Block' ? 'bg-rose-950/40 border-rose-800/50 text-rose-300' :
          'bg-zinc-800/60 border-zinc-700 text-zinc-300'
        }`}>
          <Activity className="w-3.5 h-3.5 flex-shrink-0" />
          <span><strong>Decision:</strong> {lastDecision.type}</span>
          {lastDecision.reason && <span className="text-zinc-400">— {lastDecision.reason}</span>}
        </div>
      )}

      {/* Execution controls */}
      <div className="flex-shrink-0">
        <TaskExecutionControls
          status={task.status}
          onStart={handleStart}
          onPause={handlePause}
          onResume={handleResume}
          onCancel={handleCancel}
          isLoading={actionLoading}
        />
      </div>

      {/* User input form (when waiting) */}
      {isWaitingForUser(task.status) && (
        <div className="flex-shrink-0">
          <TaskWaitingForUser
            taskId={task.id}
            onSubmitInput={handleUserInput}
          />
        </div>
      )}

      {/* Plan viewer */}
      {plan && (
        <div className="flex-shrink-0">
          <TaskPlanViewer
            plan={plan}
            onReplan={() => {
              const revised = window.prompt('Revised objective for replan:');
              if (revised && revised.trim()) void handleReplan(revised.trim());
            }}
          />
        </div>
      )}

      {/* Step timeline */}
      {steps.length > 0 && (
        <div className="flex-shrink-0">
          <div className="flex items-center gap-2 mb-3">
            <ListChecks className="w-4 h-4 text-zinc-400" />
            <h3 className="text-xs font-semibold text-zinc-300">Execution Steps</h3>
            <span className="text-[10px] font-mono text-zinc-600">
              {steps.filter(s => s.status === 'Completed').length}/{steps.length} complete
            </span>
          </div>
          <TaskStepTimeline steps={steps} currentStepIndex={currentStep} />
        </div>
      )}

      {/* Verification (Phase 13) — independent completion evidence */}
      <div className="flex-shrink-0 space-y-3">
        <div className="flex items-center gap-2 mb-1">
          <Shield className="w-4 h-4 text-emerald-400" />
          <h3 className="text-xs font-semibold text-zinc-300">Verification & Completion Gate</h3>
          <Link
            href={`/tasks/verification?taskId=${encodeURIComponent(taskId)}`}
            className="ml-auto inline-flex items-center gap-1 text-[11px] font-semibold text-cyan-400 hover:text-cyan-300 hover:underline"
          >
            Full evidence <ExternalLink className="w-3 h-3" />
          </Link>
        </div>
        {!completion ? (
          <div className="p-4 rounded-xl bg-zinc-900/40 border border-zinc-800 text-xs text-zinc-500 text-center">
            No verification run yet — completion is never assumed from model output.
          </div>
        ) : (
          <div className="space-y-3">
            <div className="flex items-center gap-2">
              <VerificationStatus status={completion.status} confidence={completion.confidence} />
              <span className="text-[11px] text-zinc-500 font-mono">
                {completion.verified ? 'completion independently established' : 'completion NOT established'}
              </span>
            </div>
            <CompletionGateStatus
              decision={completion.completion_gate}
              orchestratorAction={completion.orchestrator_action}
            />
            <VerificationFailure failures={completion.failures} />
          </div>
        )}
      </div>

      {/* Metadata */}
      <div className="flex-shrink-0 grid grid-cols-2 gap-3 pb-4">
        <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-3">
          <p className="text-[10px] font-bold uppercase tracking-widest text-zinc-600 mb-1">Created</p>
          <p className="text-xs text-zinc-300 font-mono">{timeAgo(task.created_at)}</p>
        </div>
        <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-3">
          <p className="text-[10px] font-bold uppercase tracking-widest text-zinc-600 mb-1">Updated</p>
          <p className="text-xs text-zinc-300 font-mono">{timeAgo(task.updated_at)}</p>
        </div>
        {task.workspace_id && (
          <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-3">
            <p className="text-[10px] font-bold uppercase tracking-widest text-zinc-600 mb-1">Workspace</p>
            <p className="text-xs text-zinc-300 font-mono truncate">{task.workspace_id}</p>
          </div>
        )}
        {task.session_id && (
          <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-3">
            <p className="text-[10px] font-bold uppercase tracking-widest text-zinc-600 mb-1">Session</p>
            <p className="text-xs text-zinc-300 font-mono truncate">{task.session_id}</p>
          </div>
        )}
        {task.error && (
          <div className="col-span-2 bg-red-950/30 border border-red-800/50 rounded-xl p-3">
            <p className="text-[10px] font-bold uppercase tracking-widest text-red-600 mb-1">Error</p>
            <p className="text-xs text-red-300 font-mono">{task.error}</p>
          </div>
        )}
      </div>
    </div>
  );
}

// ─── Orchestration Status Header ─────────────────────────────────────────────

function OrchestrationStatusBar() {
  const [status, setStatus] = useState<{
    service: string;
    status: string;
    fail_closed: boolean;
    untrusted_model_actions: boolean;
    authoritative_security: boolean;
  } | null>(null);

  useEffect(() => {
    backendClient.getOrchestrationStatus()
      .then(res => {
        if (res.success) setStatus(res);
      })
      .catch(() => {});
  }, []);

  if (!status) return null;

  return (
    <div className="flex flex-wrap items-center gap-3 px-4 py-2.5 bg-zinc-900/80 border-b border-zinc-800 text-[10px] font-mono">
      <div className="flex items-center gap-1.5">
        <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 inline-block" />
        <span className="text-zinc-400">{status.service}</span>
      </div>
      {status.fail_closed && (
        <span className="flex items-center gap-1 px-2 py-0.5 rounded bg-blue-950/60 border border-blue-800/40 text-blue-300">
          <Shield className="w-3 h-3" /> Fail-closed
        </span>
      )}
      {status.authoritative_security && (
        <span className="flex items-center gap-1 px-2 py-0.5 rounded bg-emerald-950/60 border border-emerald-800/40 text-emerald-300">
          <CheckCircle2 className="w-3 h-3" /> Auth security
        </span>
      )}
      {status.untrusted_model_actions && (
        <span className="flex items-center gap-1 px-2 py-0.5 rounded bg-amber-950/60 border border-amber-800/40 text-amber-300">
          <AlertTriangle className="w-3 h-3" /> Untrusted model actions
        </span>
      )}
    </div>
  );
}

// ─── Main Page ────────────────────────────────────────────────────────────────

export default function TasksPage() {
  const [tasks, setTasks] = useState<TaskSummary[]>([]);
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(null);
  const [isLoadingTasks, setIsLoadingTasks] = useState(false);
  const [showCreatePanel, setShowCreatePanel] = useState(false);
  const [taskError, setTaskError] = useState<string | null>(null);

  const loadTasks = useCallback(async () => {
    setIsLoadingTasks(true);
    try {
      const res = await backendClient.getTasks();
      if (res.success) setTasks(res.tasks ?? []);
    } catch (e: any) {
      setTaskError(e.message);
    } finally {
      setIsLoadingTasks(false);
    }
  }, []);

  useEffect(() => {
    loadTasks();
  }, [loadTasks]);

  const handleTaskCreated = (task: TaskSummary, plan?: TaskPlan) => {
    setTasks(prev => [task, ...prev]);
    setSelectedTaskId(task.id);
    setShowCreatePanel(false);
  };

  return (
    <div className="min-h-screen bg-[#0d0d0f] text-zinc-100 flex flex-col">

      {/* Page header */}
      <div className="border-b border-zinc-800 px-6 py-4 flex items-center justify-between bg-zinc-950/60">
        <div className="flex items-center gap-3">
          <Link
            href="/"
            className="p-1.5 rounded-lg hover:bg-zinc-800 text-zinc-500 hover:text-zinc-200 transition-colors"
          >
            <ArrowLeft className="w-4 h-4" />
          </Link>
          <div>
            <div className="flex items-center gap-2">
              <Cpu className="w-4 h-4 text-blue-400" />
              <h1 className="text-sm font-bold text-zinc-100">Task Orchestration</h1>
              <span className="px-2 py-0.5 rounded text-[9px] font-bold bg-blue-950/60 text-blue-400 border border-blue-800/50 font-mono">
                PHASE 11
              </span>
            </div>
            <p className="text-[10px] text-zinc-500 mt-0.5">
              Autonomous execution engine — plan, execute, verify, recover
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={loadTasks}
            disabled={isLoadingTasks}
            className="p-2 rounded-lg hover:bg-zinc-800 text-zinc-500 hover:text-zinc-200 transition-colors disabled:opacity-40"
            title="Refresh task list"
          >
            <RefreshCw className={`w-4 h-4 ${isLoadingTasks ? 'animate-spin' : ''}`} />
          </button>
          <button
            onClick={() => setShowCreatePanel(v => !v)}
            className={`flex items-center gap-2 px-3.5 py-2 rounded-xl text-xs font-semibold transition-all ${
              showCreatePanel
                ? 'bg-blue-600/20 text-blue-300 border border-blue-700/60'
                : 'bg-blue-600 hover:bg-blue-500 text-white shadow-lg shadow-blue-950/40'
            }`}
          >
            <Plus className="w-4 h-4" />
            New Task
          </button>
        </div>
      </div>

      {/* Orchestration status bar */}
      <OrchestrationStatusBar />

      {/* Body */}
      <div className="flex-1 flex overflow-hidden">

        {/* Left column — task list */}
        <aside className="w-80 flex-shrink-0 border-r border-zinc-800 flex flex-col bg-zinc-950/40 overflow-hidden">
          {/* Create panel */}
          {showCreatePanel && (
            <div className="p-4 border-b border-zinc-800">
              <CreateTaskPanel onCreated={handleTaskCreated} />
            </div>
          )}

          {/* Task list header */}
          <div className="px-4 py-3 border-b border-zinc-800/60 flex items-center justify-between">
            <span className="text-[10px] font-bold uppercase tracking-widest text-zinc-500">
              Tasks ({tasks.length})
            </span>
            {isLoadingTasks && <RefreshCw className="w-3 h-3 text-zinc-600 animate-spin" />}
          </div>

          {/* Error */}
          {taskError && (
            <div className="mx-3 mt-3 px-3 py-2 bg-red-950/40 border border-red-800/40 rounded-lg text-xs text-red-400">
              {taskError}
            </div>
          )}

          {/* Task entries */}
          <div className="flex-1 overflow-y-auto p-3 space-y-2">
            {tasks.length === 0 && !isLoadingTasks && (
              <div className="py-12 text-center space-y-2">
                <Clock className="w-8 h-8 text-zinc-700 mx-auto" />
                <p className="text-xs text-zinc-600">No tasks yet.</p>
                <p className="text-[10px] text-zinc-700">Click "New Task" to create one.</p>
              </div>
            )}
            {tasks.map(task => (
              <TaskListItem
                key={task.id}
                task={task}
                isSelected={selectedTaskId === task.id}
                onClick={() => setSelectedTaskId(task.id)}
              />
            ))}
          </div>
        </aside>

        {/* Right column — task detail */}
        <main className="flex-1 overflow-hidden flex flex-col p-6">
          {selectedTaskId ? (
            <TaskDetailPanel
              key={selectedTaskId}
              taskId={selectedTaskId}
              onBack={() => setSelectedTaskId(null)}
            />
          ) : (
            <div className="flex-1 flex flex-col items-center justify-center text-center space-y-4">
              <div className="w-16 h-16 rounded-2xl bg-zinc-900 border border-zinc-800 flex items-center justify-center">
                <Layers className="w-7 h-7 text-zinc-600" />
              </div>
              <div className="space-y-1">
                <h3 className="text-sm font-semibold text-zinc-300">Select a task</h3>
                <p className="text-xs text-zinc-600 max-w-xs leading-relaxed">
                  Choose a task from the left panel, or create a new one to begin autonomous orchestrated execution.
                </p>
              </div>
              <div className="flex flex-wrap justify-center gap-3 text-[10px] text-zinc-600 font-mono mt-2">
                {['Fail-closed security', 'Authoritative policy', 'Untrusted model actions', 'Bounded execution', 'Verified completion'].map(label => (
                  <span key={label} className="flex items-center gap-1 px-2 py-1 rounded bg-zinc-900 border border-zinc-800">
                    <Shield className="w-2.5 h-2.5" /> {label}
                  </span>
                ))}
              </div>
            </div>
          )}
        </main>
      </div>
    </div>
  );
}
