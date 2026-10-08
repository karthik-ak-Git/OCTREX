'use client';

import React, { useState } from 'react';
import Link from 'next/link';
import { ArrowLeft, Package, FileText, Plus } from 'lucide-react';
import { backendClient } from '../../lib/backend/client';
import { WorkspaceIdInput } from '../../components/DocumentBrowser';
import { ArtifactList } from '../../components/ArtifactPanels';

function loadStoredWorkspace(): string {
  try {
    return localStorage.getItem('octrex_workspace_id') || '';
  } catch {
    return '';
  }
}

const ARTIFACT_TYPES = [
  'GENERATED_DOCUMENT',
  'GENERATED_TEXT',
  'GENERATED_JSON',
  'GENERATED_CSV',
  'GENERATED_MARKDOWN',
  'GENERATED_CODE',
  'GENERATED_REPORT',
  'TRANSFORMED_DOCUMENT',
  'EXTRACTED_DATASET',
];

export default function ArtifactsPage() {
  const [workspaceId, setWorkspaceId] = useState<string>('');
  const [selected, setSelected] = useState<string>('');
  const [refreshKey, setRefreshKey] = useState<number>(0);
  const [name, setName] = useState<string>('');
  const [relPath, setRelPath] = useState<string>('');
  const [content, setContent] = useState<string>('');
  const [artifactType, setArtifactType] = useState<string>('GENERATED_MARKDOWN');
  const [sourceDoc, setSourceDoc] = useState<string>('');
  const [message, setMessage] = useState<string>('');

  React.useEffect(() => {
    setWorkspaceId(loadStoredWorkspace());
  }, []);

  const handleCreate = async () => {
    if (!workspaceId || !name.trim() || !relPath.trim() || !content) return;
    setMessage('Creating (UNVERIFIED)…');
    try {
      const res = await backendClient.createArtifact({
        workspace_id: workspaceId,
        rel_path: relPath.trim(),
        name: name.trim(),
        content,
        artifact_type: artifactType,
        source_document_id: sourceDoc.trim() || undefined,
      });
      if (res.success && res.artifact) {
        setMessage(`Created ${res.artifact.id} — UNVERIFIED until VerificationEngine passes.`);
        setRefreshKey((k) => k + 1);
        setSelected(res.artifact.id);
      } else {
        setMessage(res.error || 'Create failed');
      }
    } catch (e: unknown) {
      setMessage(e instanceof Error ? e.message : 'Create failed');
    }
  };

  return (
    <div className="min-h-screen bg-[#eaedf3] text-slate-900 p-8 flex justify-center">
      <div className="w-[720px] space-y-6">
        <div className="flex items-center justify-between bg-white/80 backdrop-blur-xl border border-white/90 p-5 rounded-3xl shadow-sm">
          <div className="flex items-center space-x-3">
            <Link href="/" className="p-2 text-slate-400 hover:text-slate-800 rounded-xl hover:bg-slate-100 transition-all">
              <ArrowLeft className="w-5 h-5" />
            </Link>
            <div>
              <h1 className="text-xl font-extrabold tracking-tight flex items-center space-x-2">
                <Package className="w-5 h-5 text-violet-700" />
                <span>Artifacts</span>
              </h1>
              <p className="text-xs text-slate-500 font-medium">Lineage · verification gate · authorized export</p>
            </div>
          </div>
          <Link href="/documents" className="px-3 py-2 rounded-xl bg-slate-900 text-white text-xs font-bold flex items-center space-x-1">
            <FileText className="w-3.5 h-3.5" />
            <span>Documents</span>
          </Link>
        </div>

        <WorkspaceIdInput workspaceId={workspaceId} onChange={setWorkspaceId} />

        <div className="p-5 bg-white/80 border border-slate-200/80 rounded-2xl space-y-2">
          <h3 className="text-sm font-extrabold flex items-center space-x-2">
            <Plus className="w-4 h-4" />
            <span>Create derived artifact</span>
          </h3>
          <div className="grid grid-cols-2 gap-2">
            <input value={name} onChange={(e) => setName(e.target.value)} placeholder="Artifact name" className="px-3 py-2 rounded-xl text-xs bg-slate-50 border border-slate-200 focus:outline-none" />
            <input value={relPath} onChange={(e) => setRelPath(e.target.value)} placeholder="rel path, e.g. out/report.md" className="px-3 py-2 rounded-xl text-xs font-mono bg-slate-50 border border-slate-200 focus:outline-none" />
            <select value={artifactType} onChange={(e) => setArtifactType(e.target.value)} className="px-3 py-2 rounded-xl text-xs bg-slate-50 border border-slate-200">
              {ARTIFACT_TYPES.map((t) => (
                <option key={t} value={t}>{t}</option>
              ))}
            </select>
            <input value={sourceDoc} onChange={(e) => setSourceDoc(e.target.value)} placeholder="source document id (optional)" className="px-3 py-2 rounded-xl text-xs font-mono bg-slate-50 border border-slate-200 focus:outline-none" />
          </div>
          <textarea value={content} onChange={(e) => setContent(e.target.value)} placeholder="Artifact content…" rows={5} className="w-full px-3 py-2 rounded-xl text-xs font-mono bg-slate-50 border border-slate-200 focus:outline-none" />
          <button onClick={handleCreate} disabled={!workspaceId} className="px-3 py-2 rounded-xl bg-slate-900 text-white text-xs font-bold disabled:opacity-50">
            Create UNVERIFIED artifact
          </button>
          {message && <div className="text-xs text-slate-600">{message}</div>}
        </div>

        {workspaceId && <ArtifactList workspaceId={workspaceId} onSelect={setSelected} refreshKey={refreshKey} />}

        {selected && (
          <Link href={`/artifacts/${encodeURIComponent(selected)}`} className="block text-center text-xs font-bold text-cyan-700 hover:underline">
            Open selected artifact →
          </Link>
        )}
      </div>
    </div>
  );
}
