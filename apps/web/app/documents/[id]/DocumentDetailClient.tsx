'use client';

import React, { useState } from 'react';
import Link from 'next/link';
import { ArrowLeft } from 'lucide-react';
import { backendClient } from '@/lib/backend/client';
import { DocumentDetail } from '@/lib/backend/types';
import { WorkspaceIdInput } from '@/components/DocumentBrowser';
import {
  DocumentMetadataPanel,
  DocumentProvenance,
  DocumentStructureViewer,
  DocumentChunkInspector,
  DocumentSearch,
} from '@/components/DocumentPanels';

function loadStoredWorkspace(): string {
  try {
    return localStorage.getItem('octrex_workspace_id') || '';
  } catch {
    return '';
  }
}

export function DocumentDetailClient({ documentId }: { documentId: string }) {
  const [workspaceId, setWorkspaceId] = useState<string>('');
  const [doc, setDoc] = useState<DocumentDetail | null>(null);
  const [error, setError] = useState<string>('');
  const [assistMsg, setAssistMsg] = useState<string>('');
  const [ingestMsg, setIngestMsg] = useState<string>('');

  React.useEffect(() => {
    setWorkspaceId(loadStoredWorkspace());
  }, []);

  React.useEffect(() => {
    if (!workspaceId || !documentId) return;
    backendClient
      .getDocument(documentId, workspaceId)
      .then((res) => {
        if (res.success && res.document) setDoc(res.document);
        else setError(res.error || 'Not found');
      })
      .catch((e: unknown) => setError(e instanceof Error ? e.message : 'Load failed'));
  }, [workspaceId, documentId]);

  const handleIngest = async () => {
    setIngestMsg('Ingesting…');
    try {
      const res = await backendClient.ingestDocument(documentId, {
        workspace_id: workspaceId,
        session_id: 'session-default',
        model_id: 'local-default',
        context_window: 8192,
      });
      setIngestMsg(res.success ? `Ingested ${res.items_ingested} FileContent items (untrusted, budget-checked).` : res.error || 'Ingest failed');
    } catch (e: unknown) {
      setIngestMsg(e instanceof Error ? e.message : 'Ingest failed');
    }
  };

  const handleAssist = async (op: string) => {
    setAssistMsg('Requesting local-only assist (no cloud fallback)…');
    try {
      const res = await backendClient.assistDocument({ workspace_id: workspaceId, document_id: documentId, operation: op });
      if (res.success) setAssistMsg(JSON.stringify(res.assist?.result_preview || res.assist, null, 2).slice(0, 2000));
      else setAssistMsg(res.error || 'Assist unavailable');
    } catch (e: unknown) {
      setAssistMsg(e instanceof Error ? e.message : 'Assist failed');
    }
  };

  return (
    <div className="min-h-screen bg-[#eaedf3] text-slate-900 p-8 flex justify-center">
      <div className="w-[720px] space-y-6">
        <div className="flex items-center space-x-3 bg-white/80 border border-white/90 p-5 rounded-3xl shadow-sm">
          <Link href="/documents" className="p-2 text-slate-400 hover:text-slate-800 rounded-xl hover:bg-slate-100">
            <ArrowLeft className="w-5 h-5" />
          </Link>
          <div>
            <h1 className="text-xl font-extrabold tracking-tight font-mono">{documentId}</h1>
            <p className="text-xs text-slate-500 font-medium">Provenance · classification · chunks · retrieval</p>
          </div>
        </div>

        <WorkspaceIdInput workspaceId={workspaceId} onChange={setWorkspaceId} />
        {error && <div className="text-xs text-rose-600 bg-white/70 rounded-2xl p-3">{error}</div>}

        {doc && (
          <>
            <DocumentMetadataPanel document={doc} />
            <DocumentProvenance document={doc} />
            <DocumentStructureViewer documentId={documentId} workspaceId={workspaceId} />
            <DocumentSearch documentId={documentId} workspaceId={workspaceId} />
            <DocumentChunkInspector documentId={documentId} workspaceId={workspaceId} />

            <div className="p-5 bg-white/80 border border-slate-200/80 rounded-2xl space-y-2">
              <h3 className="text-sm font-extrabold">ContextEngine ingest</h3>
              <button onClick={handleIngest} className="px-3 py-2 rounded-xl bg-slate-900 text-white text-xs font-bold">
                Ingest as FileContent (untrusted)
              </button>
              {ingestMsg && <div className="text-xs text-slate-600 whitespace-pre-wrap">{ingestMsg}</div>}
            </div>

            <div className="p-5 bg-white/80 border border-slate-200/80 rounded-2xl space-y-2">
              <h3 className="text-sm font-extrabold">Local-only AI assist (ModelRouter → ModelRuntime)</h3>
              <div className="flex space-x-2">
                {['summarize', 'extract', 'label'].map((op) => (
                  <button key={op} onClick={() => handleAssist(op)} className="px-3 py-2 rounded-xl bg-slate-100 text-xs font-bold hover:bg-slate-200">
                    {op}
                  </button>
                ))}
              </div>
              {assistMsg && <div className="text-xs text-slate-600 whitespace-pre-wrap max-h-64 overflow-y-auto">{assistMsg}</div>}
            </div>

            <Link href={`/documents/${encodeURIComponent(documentId)}/search`} className="block text-center text-xs font-bold text-cyan-700 hover:underline">
              Open full search view →
            </Link>
          </>
        )}
      </div>
    </div>
  );
}
