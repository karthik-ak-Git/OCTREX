'use client';

import React, { useState } from 'react';
import { FolderOpen, FileText, Upload } from 'lucide-react';
import { backendClient } from '../lib/backend/client';
import { DocumentSummary } from '../lib/backend/types';
import { DocumentClassificationBadge, DocumentSecurityBadge } from './DocumentBadges';

export function WorkspaceIdInput({
  workspaceId,
  onChange,
}: {
  workspaceId: string;
  onChange: (id: string) => void;
}) {
  const [path, setPath] = useState<string>('');
  const [resolving, setResolving] = useState<boolean>(false);
  const [error, setError] = useState<string>('');

  const handleInspect = async () => {
    if (!path.trim()) return;
    setResolving(true);
    setError('');
    try {
      const data = await backendClient.inspectWorkspace(path.trim());
      if (data.workspace_id) {
        onChange(data.workspace_id);
        try {
          localStorage.setItem('octrex_workspace_id', data.workspace_id);
        } catch {
          /* storage unavailable */
        }
      } else {
        setError('Backend did not return a workspace id.');
      }
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : 'Inspect failed');
    } finally {
      setResolving(false);
    }
  };

  return (
    <div className="p-4 bg-white/80 border border-slate-200/80 rounded-2xl space-y-2">
      <label className="text-[11px] font-extrabold uppercase text-slate-400 tracking-wider">
        Active Workspace
      </label>
      <div className="flex items-center space-x-2">
        <input
          value={workspaceId}
          onChange={(e) => onChange(e.target.value)}
          placeholder="ws-… (from Connect folder)"
          className="flex-1 px-3 py-2 rounded-xl text-xs font-mono bg-slate-50 border border-slate-200 focus:outline-none focus:ring-2 focus:ring-slate-900/10"
        />
      </div>
      <div className="flex items-center space-x-2">
        <input
          value={path}
          onChange={(e) => setPath(e.target.value)}
          placeholder="Or paste a folder path, e.g. D:\project"
          className="flex-1 px-3 py-2 rounded-xl text-xs font-medium bg-slate-50 border border-slate-200 focus:outline-none focus:ring-2 focus:ring-slate-900/10"
        />
        <button
          onClick={handleInspect}
          disabled={resolving}
          className="px-3 py-2 rounded-xl bg-slate-900 text-white text-xs font-bold hover:bg-slate-800 disabled:opacity-50 flex items-center space-x-1"
        >
          <FolderOpen className="w-3.5 h-3.5" />
          <span>{resolving ? '…' : 'Connect'}</span>
        </button>
      </div>
      {error && <div className="text-xs text-rose-600 font-medium">{error}</div>}
    </div>
  );
}

export function DocumentBrowser({
  workspaceId,
  onSelect,
}: {
  workspaceId: string;
  onSelect: (id: string) => void;
}) {
  const [docs, setDocs] = useState<DocumentSummary[]>([]);
  const [loading, setLoading] = useState<boolean>(false);
  const [relPath, setRelPath] = useState<string>('');
  const [message, setMessage] = useState<string>('');

  const fetchDocs = async () => {
    if (!workspaceId) return;
    setLoading(true);
    try {
      const res = await backendClient.listDocuments(workspaceId);
      if (res.success) setDocs(res.documents || []);
      else setMessage(res.error || 'List failed');
    } catch (e: unknown) {
      setMessage(e instanceof Error ? e.message : 'List failed');
    } finally {
      setLoading(false);
    }
  };

  React.useEffect(() => {
    if (workspaceId) fetchDocs();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [workspaceId]);

  const handleImport = async () => {
    if (!workspaceId || !relPath.trim()) return;
    setLoading(true);
    setMessage('');
    try {
      const res = await backendClient.importDocument({ workspace_id: workspaceId, rel_path: relPath.trim() });
      if (res.success && res.document_id) {
        setMessage(`Imported ${res.blocks || 0} blocks, ${res.chunks || 0} chunks (${res.classification}).`);
        setRelPath('');
        fetchDocs();
        onSelect(res.document_id);
      } else {
        setMessage(res.error || 'Import failed');
      }
    } catch (e: unknown) {
      setMessage(e instanceof Error ? e.message : 'Import failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-4">
      <div className="p-4 bg-white/80 border border-slate-200/80 rounded-2xl space-y-2">
        <label className="text-[11px] font-extrabold uppercase text-slate-400 tracking-wider">
          Import workspace-relative document
        </label>
        <div className="flex items-center space-x-2">
          <input
            value={relPath}
            onChange={(e) => setRelPath(e.target.value)}
            placeholder="e.g. docs/spec.md, data/report.csv"
            className="flex-1 px-3 py-2 rounded-xl text-xs font-mono bg-slate-50 border border-slate-200 focus:outline-none focus:ring-2 focus:ring-slate-900/10"
          />
          <button
            onClick={handleImport}
            disabled={loading || !relPath.trim()}
            className="px-3 py-2 rounded-xl bg-slate-900 text-white text-xs font-bold hover:bg-slate-800 disabled:opacity-50 flex items-center space-x-1"
          >
            <Upload className="w-3.5 h-3.5" />
            <span>Import</span>
          </button>
        </div>
        {message && <div className="text-xs text-slate-600 font-medium">{message}</div>}
        <div className="text-[11px] text-slate-400">Supported: TXT · Markdown · JSON · CSV · XML · PDF · DOCX · XLSX. Others are rejected explicitly.</div>
      </div>

      <div className="space-y-2">
        {docs.map((d) => (
          <button
            key={d.id}
            onClick={() => onSelect(d.id)}
            className="w-full text-left p-3 rounded-2xl bg-white border border-slate-200/80 hover:border-slate-400 shadow-2xs transition-all"
          >
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-2 truncate">
                <FileText className="w-4 h-4 text-cyan-700 flex-shrink-0" />
                <span className="text-xs font-bold font-mono truncate">{d.rel_path}</span>
              </div>
              <DocumentClassificationBadge classification={d.classification} />
            </div>
            <div className="flex items-center justify-between mt-1.5">
              <span className="text-[11px] font-mono text-slate-400">
                {d.format} · {(d.size_bytes / 1024).toFixed(1)} KB · {d.extraction_status}
              </span>
              <DocumentSecurityBadge findings={[]} />
            </div>
          </button>
        ))}
        {!loading && docs.length === 0 && (
          <div className="p-4 text-center text-xs text-slate-400 bg-white/60 border border-slate-200/60 rounded-2xl">
            No documents imported yet.
          </div>
        )}
      </div>
    </div>
  );
}
