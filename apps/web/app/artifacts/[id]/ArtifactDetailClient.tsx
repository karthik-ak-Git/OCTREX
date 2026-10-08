'use client';

import React, { useState } from 'react';
import Link from 'next/link';
import { ArrowLeft } from 'lucide-react';
import { backendClient } from '@/lib/backend/client';
import { ArtifactSummary, VerificationResult } from '@/lib/backend/types';
import { WorkspaceIdInput } from '@/components/DocumentBrowser';
import {
  ArtifactDetails,
  ArtifactExportDialog,
  ArtifactLineage,
  ArtifactVerificationStatus,
} from '@/components/ArtifactPanels';

function loadStoredWorkspace(): string {
  try {
    return localStorage.getItem('octrex_workspace_id') || '';
  } catch {
    return '';
  }
}

export function ArtifactDetailClient({ artifactId }: { artifactId: string }) {
  const [workspaceId, setWorkspaceId] = useState<string>('');
  const [artifact, setArtifact] = useState<ArtifactSummary | null>(null);
  const [taskId, setTaskId] = useState<string>('');
  const [verification, setVerification] = useState<VerificationResult | null>(null);
  const [message, setMessage] = useState<string>('');

  React.useEffect(() => {
    setWorkspaceId(loadStoredWorkspace());
  }, []);

  React.useEffect(() => {
    backendClient
      .getArtifact(artifactId)
      .then((res) => {
        if (res.success && res.artifact) {
          setArtifact(res.artifact);
          if (res.artifact.task_id) setTaskId(res.artifact.task_id);
        } else {
          setMessage(res.error || 'Not found');
        }
      })
      .catch((e: unknown) => setMessage(e instanceof Error ? e.message : 'Load failed'));
  }, [artifactId]);

  const handleVerify = async () => {
    if (!workspaceId || !taskId.trim()) {
      setMessage('Workspace and task id are required for verification.');
      return;
    }
    setMessage('Verifying…');
    try {
      const res = await backendClient.verifyArtifact(artifactId, { workspace_id: workspaceId, task_id: taskId.trim() });
      if (res.success && res.verification) {
        setVerification(res.verification);
        const updated = await backendClient.getArtifact(artifactId);
        if (updated.success && updated.artifact) setArtifact(updated.artifact);
        setMessage(`Verification ${res.verification.status}. Only VERIFIED artifacts may be claimed complete.`);
      } else {
        setMessage(res.error || 'Verify failed');
      }
    } catch (e: unknown) {
      setMessage(e instanceof Error ? e.message : 'Verify failed');
    }
  };

  return (
    <div className="min-h-screen bg-[#eaedf3] text-slate-900 p-8 flex justify-center">
      <div className="w-[720px] space-y-6">
        <div className="flex items-center space-x-3 bg-white/80 border border-white/90 p-5 rounded-3xl shadow-sm">
          <Link href="/artifacts" className="p-2 text-slate-400 hover:text-slate-800 rounded-xl hover:bg-slate-100">
            <ArrowLeft className="w-5 h-5" />
          </Link>
          <div>
            <h1 className="text-xl font-extrabold tracking-tight font-mono">{artifactId}</h1>
            <p className="text-xs text-slate-500 font-medium">Lineage · verification gate · export</p>
          </div>
        </div>

        <WorkspaceIdInput workspaceId={workspaceId} onChange={setWorkspaceId} />
        {message && <div className="text-xs text-slate-600 bg-white/70 rounded-2xl p-3">{message}</div>}

        {artifact && (
          <>
            <ArtifactDetails artifact={artifact} />
            <ArtifactLineage artifactId={artifactId} />
            <div className="p-5 bg-white/80 border border-slate-200/80 rounded-2xl space-y-2">
              <h3 className="text-sm font-extrabold">Verification gate</h3>
              <div className="flex items-center space-x-2">
                <input
                  value={taskId}
                  onChange={(e) => setTaskId(e.target.value)}
                  placeholder="task id for verification evidence"
                  className="flex-1 px-3 py-2 rounded-xl text-xs font-mono bg-slate-50 border border-slate-200 focus:outline-none"
                />
                <button onClick={handleVerify} className="px-3 py-2 rounded-xl bg-slate-900 text-white text-xs font-bold">
                  Verify
                </button>
              </div>
              {verification && (
                <div className="text-xs space-y-1">
                  <ArtifactVerificationStatus status={verification.status} />
                  <div className="font-mono text-slate-500">{verification.checks.length} checks · confidence {verification.confidence}</div>
                  {verification.failures.map((f, i) => (
                    <div key={i} className="text-rose-700 font-medium">{f}</div>
                  ))}
                </div>
              )}
            </div>
            {workspaceId && <ArtifactExportDialog artifactId={artifactId} workspaceId={workspaceId} />}
          </>
        )}
      </div>
    </div>
  );
}
