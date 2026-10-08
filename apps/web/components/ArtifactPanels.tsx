'use client';

import React, { useState } from 'react';
import Link from 'next/link';
import { Package, GitBranch, BadgeCheck, Download } from 'lucide-react';
import { backendClient } from '../lib/backend/client';
import { ArtifactLineageEntry, ArtifactSummary } from '../lib/backend/types';

export function ArtifactVerificationStatus({ status }: { status?: string }) {
  const s = (status || 'UNVERIFIED').toUpperCase();
  const style =
    s === 'VERIFIED'
      ? 'bg-emerald-100 text-emerald-800 border-emerald-300'
      : s === 'FAILED'
        ? 'bg-rose-100 text-rose-800 border-rose-300'
        : 'bg-slate-100 text-slate-600 border-slate-300';
  return (
    <span className={`inline-flex items-center space-x-1 px-2.5 py-0.5 rounded-full text-[11px] font-extrabold border font-mono ${style}`}>
      <BadgeCheck className="w-3 h-3" />
      <span>{s}</span>
    </span>
  );
}

export function ArtifactList({
  workspaceId,
  onSelect,
  refreshKey,
}: {
  workspaceId: string;
  onSelect: (id: string) => void;
  refreshKey?: number;
}) {
  const [artifacts, setArtifacts] = useState<ArtifactSummary[]>([]);
  const [loading, setLoading] = useState<boolean>(false);

  const load = async () => {
    if (!workspaceId) return;
    setLoading(true);
    try {
      const res = await backendClient.listArtifacts({ workspace_id: workspaceId });
      if (res.success) setArtifacts(res.artifacts || []);
    } finally {
      setLoading(false);
    }
  };

  React.useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [workspaceId, refreshKey]);

  return (
    <div className="space-y-2">
      {artifacts.map((a) => (
        <button
          key={a.id}
          onClick={() => onSelect(a.id)}
          className="w-full text-left p-3 rounded-2xl bg-white border border-slate-200/80 hover:border-slate-400 shadow-2xs transition-all"
        >
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2 truncate">
              <Package className="w-4 h-4 text-violet-700 flex-shrink-0" />
              <span className="text-xs font-bold truncate">{a.name}</span>
            </div>
            <ArtifactVerificationStatus status={a.verification_status} />
          </div>
          <div className="text-[11px] font-mono text-slate-400 mt-1 truncate">
            {a.artifact_type} · {(a.size / 1024).toFixed(1)} KB · {a.path}
          </div>
        </button>
      ))}
      {!loading && artifacts.length === 0 && (
        <div className="p-4 text-center text-xs text-slate-400 bg-white/60 border border-slate-200/60 rounded-2xl">
          No artifacts yet. Generated artifacts start UNVERIFIED.
        </div>
      )}
    </div>
  );
}

export function ArtifactDetails({ artifact }: { artifact: ArtifactSummary }) {
  return (
    <div className="p-5 bg-white/80 border border-slate-200/80 rounded-2xl space-y-3">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-extrabold text-slate-900 truncate">{artifact.name}</h3>
        <ArtifactVerificationStatus status={artifact.verification_status} />
      </div>
      <div className="grid grid-cols-2 gap-2 text-xs">
        <div className="p-2.5 bg-slate-50 rounded-xl border border-slate-200/70">
          <div className="text-[10px] font-extrabold uppercase text-slate-400">Type</div>
          <div className="font-mono font-bold">{artifact.artifact_type}</div>
        </div>
        <div className="p-2.5 bg-slate-50 rounded-xl border border-slate-200/70">
          <div className="text-[10px] font-extrabold uppercase text-slate-400">Size</div>
          <div className="font-mono font-bold">{(artifact.size / 1024).toFixed(1)} KB</div>
        </div>
        <div className="p-2.5 bg-slate-50 rounded-xl border border-slate-200/70 col-span-2">
          <div className="text-[10px] font-extrabold uppercase text-slate-400">Path</div>
          <div className="font-mono font-bold truncate">{artifact.path}</div>
        </div>
        <div className="p-2.5 bg-slate-50 rounded-xl border border-slate-200/70 col-span-2">
          <div className="text-[10px] font-extrabold uppercase text-slate-400">Checksum</div>
          <div className="font-mono font-bold truncate" title={artifact.checksum || ''}>{artifact.checksum || '—'}</div>
        </div>
      </div>
      <p className="text-[11px] text-slate-400">
        Artifacts never auto-verify. Completion may only be claimed after VerificationEngine passes.
      </p>
    </div>
  );
}

export function ArtifactLineage({ artifactId }: { artifactId: string }) {
  const [lineage, setLineage] = useState<ArtifactLineageEntry[]>([]);

  React.useEffect(() => {
    backendClient.getArtifactLineage(artifactId).then((res) => {
      if (res.success) setLineage(res.lineage || []);
    });
  }, [artifactId]);

  return (
    <div className="p-5 bg-white/80 border border-slate-200/80 rounded-2xl space-y-2">
      <div className="flex items-center space-x-2 text-sm font-extrabold text-slate-900">
        <GitBranch className="w-4 h-4 text-slate-500" />
        <span>Lineage (source document → extraction → artifact → verified)</span>
      </div>
      {lineage.length === 0 && <div className="text-xs text-slate-400">No lineage entries recorded.</div>}
      <div className="space-y-2">
        {lineage.map((l) => (
          <div key={l.id} className="p-3 bg-slate-50 border border-slate-200/70 rounded-xl text-xs font-mono text-slate-600 space-y-1">
            {l.source_document_id && (
              <div>
                source document:{' '}
                <Link href={`/documents/${encodeURIComponent(l.source_document_id)}`} className="text-cyan-700 font-bold hover:underline">
                  {l.source_document_id}
                </Link>
              </div>
            )}
            {l.parent_artifact_id && <div>parent artifact: {l.parent_artifact_id}</div>}
            {l.producing_workflow && <div>workflow: {l.producing_workflow}</div>}
            {l.producing_skill && <div>skill: {l.producing_skill}</div>}
            <div>classification: {l.classification}</div>
          </div>
        ))}
      </div>
    </div>
  );
}

export function ArtifactExportDialog({
  artifactId,
  workspaceId,
}: {
  artifactId: string;
  workspaceId: string;
}) {
  const [dest, setDest] = useState<string>('');
  const [message, setMessage] = useState<string>('');
  const [busy, setBusy] = useState<boolean>(false);

  const doExport = async () => {
    if (!dest.trim()) return;
    setBusy(true);
    setMessage('');
    try {
      const res = await backendClient.exportArtifact(artifactId, {
        workspace_id: workspaceId,
        dest_external_path: dest.trim(),
      });
      setMessage(res.success ? 'Exported via authorized boundary.' : res.error || 'Export blocked');
    } catch (e: unknown) {
      setMessage(e instanceof Error ? e.message : 'Export failed');
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="p-5 bg-white/80 border border-slate-200/80 rounded-2xl space-y-2">
      <div className="flex items-center space-x-2 text-sm font-extrabold text-slate-900">
        <Download className="w-4 h-4 text-slate-500" />
        <span>Authorized export</span>
      </div>
      <div className="flex items-center space-x-2">
        <input
          value={dest}
          onChange={(e) => setDest(e.target.value)}
          placeholder="External destination path…"
          className="flex-1 px-3 py-2 rounded-xl text-xs font-mono bg-slate-50 border border-slate-200 focus:outline-none focus:ring-2 focus:ring-slate-900/10"
        />
        <button onClick={doExport} disabled={busy} className="px-3 py-2 rounded-xl bg-slate-900 text-white text-xs font-bold disabled:opacity-50">
          Export
        </button>
      </div>
      {message && <div className="text-xs text-slate-600">{message}</div>}
      <p className="text-[11px] text-slate-400">SECRET/RESTRICTED artifacts are blocked from export without authorization.</p>
    </div>
  );
}
