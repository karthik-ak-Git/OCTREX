'use client';

import React, { useCallback, useEffect, useState } from 'react';
import Link from 'next/link';
import { ArrowLeft, RefreshCw, PlayCircle } from 'lucide-react';
import { backendClient } from '@/lib/backend/client';
import type {
  TaskCompletionResponse,
  VerificationResult,
} from '@/lib/backend/types';
import { VerificationStatus } from '@/components/VerificationStatus';
import { VerificationChecklist } from '@/components/VerificationChecklist';
import { VerificationEvidence } from '@/components/VerificationEvidence';
import { CompletionGateStatus } from '@/components/CompletionGateStatus';
import { RepairAttemptTimeline } from '@/components/RepairAttemptTimeline';
import { VerificationWarning } from '@/components/VerificationWarning';
import { VerificationFailure } from '@/components/VerificationFailure';

interface TaskVerificationClientProps {
  taskId: string;
}

export default function TaskVerificationClient({ taskId }: TaskVerificationClientProps) {
  const [runs, setRuns] = useState<VerificationResult[]>([]);
  const [selected, setSelected] = useState<VerificationResult | null>(null);
  const [completion, setCompletion] = useState<TaskCompletionResponse | null>(null);
  const [gate, setGate] = useState<string | null>(null);
  const [action, setAction] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isRetrying, setIsRetrying] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchAll = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const [listRes, completionRes] = await Promise.all([
        backendClient.listTaskVerifications(taskId),
        backendClient.getTaskCompletion(taskId),
      ]);
      if (listRes.success && listRes.runs) {
        setRuns(listRes.runs);
        setSelected((prev) => {
          if (prev) {
            const stillThere = listRes.runs?.find((r) => r.verification_id === prev.verification_id);
            if (stillThere) return stillThere;
          }
          return listRes.runs && listRes.runs.length > 0 ? listRes.runs[0] : null;
        });
        const latest = listRes.runs[0];
        if (latest) {
          try {
            const detail = await backendClient.getTaskVerification(taskId, latest.verification_id);
            if (detail.success) {
              setGate(detail.completion_gate ?? null);
              setAction(detail.orchestrator_action ?? null);
            }
          } catch {
            /* gate detail is best-effort */
          }
        }
      } else if (!listRes.success) {
        setError(listRes.error || 'Failed to load verification runs');
      }
      if (completionRes.success) {
        setCompletion(completionRes);
      }
    } catch (e: any) {
      setError(e.message || 'Failed to reach verification backend');
    } finally {
      setIsLoading(false);
    }
  }, [taskId]);

  useEffect(() => {
    fetchAll();
  }, [fetchAll]);

  const handleSelect = async (verificationId: string) => {
    try {
      const detail = await backendClient.getTaskVerification(taskId, verificationId);
      if (detail.success && detail.verification) {
        setSelected(detail.verification);
        setGate(detail.completion_gate ?? null);
        setAction(detail.orchestrator_action ?? null);
      }
    } catch (e: any) {
      setError(e.message || 'Failed to load verification detail');
    }
  };

  const handleRetry = async () => {
    if (!selected) return;
    setIsRetrying(true);
    setError(null);
    try {
      const res = await backendClient.retryTaskVerification(taskId, selected.verification_id);
      if (!res.success) {
        setError(res.error || 'Repair budget exhausted or retry rejected');
      }
      await fetchAll();
    } catch (e: any) {
      setError(e.message || 'Failed to register repair attempt');
    } finally {
      setIsRetrying(false);
    }
  };

  return (
    <div className="p-8 max-w-5xl mx-auto space-y-6 text-zinc-100">
      <div className="border-b border-zinc-800 pb-5 space-y-2">
        <Link href="/" className="inline-flex items-center gap-1.5 text-xs text-zinc-400 hover:text-zinc-200">
          <ArrowLeft className="w-3.5 h-3.5" />
          Back to workspace
        </Link>
        <h1 className="text-2xl font-bold tracking-tight text-white flex items-center gap-3">
          <span>Task Verification</span>
          <span className="text-xs px-2.5 py-1 rounded-full font-mono bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
            Phase 13
          </span>
        </h1>
        <p className="text-xs font-mono text-zinc-500 break-all">task: {taskId}</p>
        <p className="text-sm text-zinc-400">
          Independent completion evidence. A model stating “done” never counts — only files,
          artifacts, commands, tools, schemas, constraints, and requirements verified against
          authoritative state.
        </p>
      </div>

      {error && (
        <div className="p-4 rounded-lg bg-rose-500/10 border border-rose-500/20 text-rose-400 text-sm">
          {error}
        </div>
      )}

      {isLoading ? (
        <div className="p-8 text-center text-xs text-zinc-500">Loading verification state…</div>
      ) : (
        <>
          {completion && (
            <div className="p-4 rounded-xl border border-zinc-800 bg-zinc-900/40 flex flex-wrap items-center gap-3">
              <VerificationStatus status={completion.status} confidence={completion.confidence} />
              <span className="text-xs text-zinc-400">
                {completion.verified
                  ? 'Completion independently established.'
                  : 'Completion is NOT established.'}
              </span>
              <div className="ml-auto flex items-center gap-2">
                <button
                  onClick={fetchAll}
                  className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-zinc-800 hover:bg-zinc-700 text-zinc-200 border border-zinc-700"
                >
                  <RefreshCw className="w-3.5 h-3.5" />
                  Refresh
                </button>
                <button
                  onClick={handleRetry}
                  disabled={isRetrying || !selected}
                  className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-blue-600 hover:bg-blue-500 text-white disabled:opacity-50"
                >
                  <PlayCircle className="w-3.5 h-3.5" />
                  {isRetrying ? 'Registering…' : 'Register repair attempt'}
                </button>
              </div>
            </div>
          )}

          {gate && (
            <CompletionGateStatus decision={gate} orchestratorAction={action ?? undefined} />
          )}

          <div className="grid grid-cols-1 lg:grid-cols-5 gap-4">
            <div className="lg:col-span-2">
              <RepairAttemptTimeline
                runs={runs}
                selectedId={selected?.verification_id}
                onSelect={handleSelect}
              />
            </div>
            <div className="lg:col-span-3 space-y-4">
              {!selected ? (
                <div className="p-8 rounded-xl border border-zinc-800 bg-zinc-900/40 text-center text-xs text-zinc-500">
                  No verification runs for this task yet. Run verification to establish completion.
                </div>
              ) : (
                <>
                  <VerificationFailure failures={selected.failures} />
                  <VerificationWarning warnings={selected.warnings} />
                  <div className="space-y-2">
                    <h3 className="text-xs font-bold uppercase tracking-wider text-zinc-400">
                      Checks ({selected.checks.length})
                    </h3>
                    <VerificationChecklist checks={selected.checks} />
                  </div>
                  <VerificationEvidence evidenceRefs={selected.evidence_refs} />
                </>
              )}
            </div>
          </div>
        </>
      )}
    </div>
  );
}
