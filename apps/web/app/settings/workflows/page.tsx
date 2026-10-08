'use client';

import React, { useEffect, useState } from 'react';
import { WorkflowDefinition } from '@/lib/backend/types';
import { backendClient } from '@/lib/backend/client';

export default function WorkflowsPage() {
  const [workflows, setWorkflows] = useState<WorkflowDefinition[]>([]);
  const [selected, setSelected] = useState<WorkflowDefinition | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [runResult, setRunResult] = useState<string | null>(null);

  const fetchAll = async () => {
    try {
      setLoading(true);
      const res = await backendClient.listWorkflows();
      if (res.success) setWorkflows(res.workflows);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : 'Failed loading workflows');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchAll();
  }, []);

  const validate = async (id: string) => {
    try {
      const r = await backendClient.validateWorkflow(id);
      setNotice(r.valid ? 'Workflow DAG is VALID' : `Invalid: ${r.error ?? 'unknown'}`);
    } catch (e: unknown) {
      setNotice(e instanceof Error ? e.message : 'Validation failed');
    }
  };

  const run = async (id: string) => {
    try {
      setRunResult(null);
      const r = await backendClient.runWorkflow(id, { inputs: {} });
      if (r.success) setRunResult(`Run started: ${(r.run as { id: string }).id} (pinned ${(r.run as { workflow_version: string }).workflow_version})`);
      else setRunResult(`Run rejected: ${r.error ?? 'unknown'}`);
    } catch (e: unknown) {
      setRunResult(e instanceof Error ? e.message : 'Run failed');
    }
  };

  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-100 p-8 space-y-6">
      <div className="border-b border-zinc-800 pb-5">
        <h1 className="text-2xl font-bold text-white">Workflows</h1>
        <p className="text-sm text-zinc-400 mt-1">
          Structured step graphs validated for cycles, depth, and references. Execution delegates to the Phase 11 Orchestrator — selection is never authorization.
        </p>
      </div>
      {notice && <div className="p-3 bg-sky-950/40 border border-sky-800/50 rounded text-sky-300 text-xs">{notice}</div>}
      {runResult && <div className="p-3 bg-zinc-900 border border-zinc-800 rounded text-xs">{runResult}</div>}
      {loading ? (
        <div className="text-sm text-zinc-500">Loading workflows…</div>
      ) : error ? (
        <div className="p-4 bg-red-950/40 border border-red-800/50 rounded text-red-300 text-sm">{error}</div>
      ) : (
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
          <div className="lg:col-span-6 bg-zinc-900 border border-zinc-800 rounded-lg overflow-hidden">
            <div className="px-5 py-3 border-b border-zinc-800">Workflows ({workflows.length})</div>
            <div className="divide-y divide-zinc-800/60 max-h-[600px] overflow-y-auto">
              {workflows.map((w) => (
                <div key={`${w.id}@${w.version}`} onClick={() => setSelected(w)} className="p-4 hover:bg-zinc-800/30 cursor-pointer">
                  <div className="font-semibold text-sm">{w.name} <span className="font-mono text-xs text-zinc-500">{w.id}@{w.version}</span></div>
                  <p className="text-xs text-zinc-400">{w.description}</p>
                  <div className="flex items-center space-x-2 pt-1">
                    <span className="text-[10px] font-mono px-1.5 py-0.5 bg-zinc-800 rounded border border-zinc-700">{w.status}</span>
                    <span className="text-[10px] font-mono px-1.5 py-0.5 bg-zinc-800 rounded border border-zinc-700">{w.source}</span>
                    <button onClick={(e) => { e.stopPropagation(); validate(w.id); }} className="px-2 py-0.5 bg-zinc-800 rounded text-[11px]">Validate</button>
                    <button onClick={(e) => { e.stopPropagation(); run(w.id); }} className="px-2 py-0.5 bg-emerald-900 rounded text-[11px]">Run</button>
                  </div>
                </div>
              ))}
            </div>
          </div>
          <div className="lg:col-span-6 bg-zinc-900 border border-zinc-800 rounded-lg p-4">
            <h3 className="text-sm font-semibold mb-2">Workflow Details & Steps</h3>
            {selected ? (
              <div className="space-y-2 text-xs">
                <div className="space-y-1">
                  {selected.steps.map((s) => (
                    <div key={s.id} className="p-2 bg-zinc-950 border border-zinc-800 rounded">
                      <div className="font-mono text-zinc-300">{s.id} · {s.kind} → {s.reference}</div>
                      <div className="text-zinc-500">{s.description}</div>
                      <div className="text-zinc-600 font-mono">deps: [{s.dependencies.join(', ')}] · verify: {s.verification_required ? 'yes' : 'no'}</div>
                    </div>
                  ))}
                </div>
                <pre className="p-2 bg-zinc-950 border border-zinc-800 rounded text-[11px] overflow-auto">{JSON.stringify(selected, null, 2)}</pre>
              </div>
            ) : (
              <div className="text-xs text-zinc-500">Select a workflow.</div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
