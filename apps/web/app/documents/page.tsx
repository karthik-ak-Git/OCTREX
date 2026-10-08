'use client';

import React, { useState } from 'react';
import Link from 'next/link';
import { ArrowLeft, FileText, Package } from 'lucide-react';
import { DocumentBrowser, WorkspaceIdInput } from '../../components/DocumentBrowser';

function loadStoredWorkspace(): string {
  try {
    return localStorage.getItem('octrex_workspace_id') || '';
  } catch {
    return '';
  }
}

export default function DocumentsPage() {
  const [workspaceId, setWorkspaceId] = useState<string>('');
  const [selected, setSelected] = useState<string>('');

  React.useEffect(() => {
    setWorkspaceId(loadStoredWorkspace());
  }, []);

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
                <FileText className="w-5 h-5 text-cyan-700" />
                <span>Documents</span>
              </h1>
              <p className="text-xs text-slate-500 font-medium">Local-first intake · provenance · classification · chunking</p>
            </div>
          </div>
          <Link href="/artifacts" className="px-3 py-2 rounded-xl bg-slate-900 text-white text-xs font-bold flex items-center space-x-1">
            <Package className="w-3.5 h-3.5" />
            <span>Artifacts</span>
          </Link>
        </div>

        <WorkspaceIdInput workspaceId={workspaceId} onChange={setWorkspaceId} />

        {workspaceId ? (
          <DocumentBrowser workspaceId={workspaceId} onSelect={(id) => setSelected(id)} />
        ) : (
          <div className="p-4 text-center text-xs text-slate-400 bg-white/60 border border-slate-200/60 rounded-2xl">
            Connect a workspace to browse documents.
          </div>
        )}

        {selected && (
          <Link
            href={`/documents/${encodeURIComponent(selected)}`}
            className="block text-center text-xs font-bold text-cyan-700 hover:underline"
          >
            Open selected document →
          </Link>
        )}
      </div>
    </div>
  );
}
