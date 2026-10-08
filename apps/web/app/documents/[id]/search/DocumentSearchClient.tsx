'use client';

import React, { useState } from 'react';
import Link from 'next/link';
import { ArrowLeft } from 'lucide-react';
import { DocumentSearch } from '@/components/DocumentPanels';
import { WorkspaceIdInput } from '@/components/DocumentBrowser';

function loadStoredWorkspace(): string {
  try {
    return localStorage.getItem('octrex_workspace_id') || '';
  } catch {
    return '';
  }
}

export function DocumentSearchClient({ documentId }: { documentId: string }) {
  const [workspaceId, setWorkspaceId] = useState<string>('');

  React.useEffect(() => {
    setWorkspaceId(loadStoredWorkspace());
  }, []);

  return (
    <div className="min-h-screen bg-[#eaedf3] text-slate-900 p-8 flex justify-center">
      <div className="w-[720px] space-y-6">
        <div className="flex items-center space-x-3 bg-white/80 border border-white/90 p-5 rounded-3xl shadow-sm">
          <Link href={`/documents/${encodeURIComponent(documentId)}`} className="p-2 text-slate-400 hover:text-slate-800 rounded-xl hover:bg-slate-100">
            <ArrowLeft className="w-5 h-5" />
          </Link>
          <div>
            <h1 className="text-xl font-extrabold tracking-tight">Document search</h1>
            <p className="text-xs text-slate-500 font-mono">{documentId}</p>
          </div>
        </div>
        <WorkspaceIdInput workspaceId={workspaceId} onChange={setWorkspaceId} />
        {workspaceId && <DocumentSearch documentId={documentId} workspaceId={workspaceId} />}
      </div>
    </div>
  );
}
