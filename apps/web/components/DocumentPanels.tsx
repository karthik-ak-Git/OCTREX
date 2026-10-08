'use client';

import React, { useState } from 'react';
import { FileSearch, Layers, ListTree, ScrollText } from 'lucide-react';
import { backendClient } from '../lib/backend/client';
import { DocumentChunkView, DocumentDetail, DocumentSearchResult } from '../lib/backend/types';
import { DocumentClassificationBadge, DocumentSecurityBadge } from './DocumentBadges';

export function DocumentMetadataPanel({ document }: { document: DocumentDetail }) {
  return (
    <div className="p-5 bg-white/80 border border-slate-200/80 rounded-2xl space-y-3">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-extrabold text-slate-900 font-mono truncate">{document.rel_path}</h3>
        <DocumentClassificationBadge classification={document.classification} />
      </div>
      <div className="grid grid-cols-2 gap-2 text-xs">
        <div className="p-2.5 bg-slate-50 rounded-xl border border-slate-200/70">
          <div className="text-[10px] font-extrabold uppercase text-slate-400">Format</div>
          <div className="font-mono font-bold">{document.format}</div>
        </div>
        <div className="p-2.5 bg-slate-50 rounded-xl border border-slate-200/70">
          <div className="text-[10px] font-extrabold uppercase text-slate-400">Size</div>
          <div className="font-mono font-bold">{(document.size_bytes / 1024).toFixed(1)} KB</div>
        </div>
        <div className="p-2.5 bg-slate-50 rounded-xl border border-slate-200/70">
          <div className="text-[10px] font-extrabold uppercase text-slate-400">Status</div>
          <div className="font-mono font-bold">{document.extraction_status}</div>
        </div>
        <div className="p-2.5 bg-slate-50 rounded-xl border border-slate-200/70">
          <div className="text-[10px] font-extrabold uppercase text-slate-400">Hash</div>
          <div className="font-mono font-bold truncate" title={document.content_hash}>{document.content_hash.slice(0, 20)}…</div>
        </div>
      </div>
      <DocumentSecurityBadge findings={document.findings} />
      {document.warnings.length > 0 && (
        <div className="text-xs text-amber-700 bg-amber-50 border border-amber-200 rounded-xl p-2.5 space-y-1">
          {document.warnings.map((w, i) => (
            <div key={i} className="font-mono">[{w.code}] {w.message}</div>
          ))}
        </div>
      )}
    </div>
  );
}

export function DocumentProvenance({ document }: { document: DocumentDetail }) {
  const prov = document.provenance as Record<string, string>;
  return (
    <div className="p-5 bg-white/80 border border-slate-200/80 rounded-2xl space-y-2">
      <h3 className="text-sm font-extrabold text-slate-900">Provenance</h3>
      <div className="text-xs font-mono text-slate-600 space-y-1">
        <div>document: <span className="font-bold">{document.id}</span></div>
        <div>workspace: <span className="font-bold">{document.workspace_id}</span></div>
        <div>source: <span className="font-bold">{document.rel_path}</span></div>
        {prov.trust && <div>trust: <span className="font-bold text-orange-700">{prov.trust} (never System)</span></div>}
      </div>
      <p className="text-[11px] text-slate-400">Every extracted block traces back to this source. Provenance is never destroyed during normalization.</p>
    </div>
  );
}

export function DocumentStructureViewer({
  documentId,
  workspaceId,
}: {
  documentId: string;
  workspaceId: string;
}) {
  const [sections, setSections] = useState<Array<{ section_id: string }>>([]);
  const [pages, setPages] = useState<number[]>([]);
  const [loading, setLoading] = useState<boolean>(false);

  const load = async () => {
    setLoading(true);
    try {
      const res = await backendClient.getDocumentSections(documentId, workspaceId);
      if (res.success) {
        setSections(res.sections || []);
        setPages(res.pages || []);
      }
    } finally {
      setLoading(false);
    }
  };

  React.useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [documentId]);

  return (
    <div className="p-5 bg-white/80 border border-slate-200/80 rounded-2xl space-y-2">
      <div className="flex items-center space-x-2 text-sm font-extrabold text-slate-900">
        <ListTree className="w-4 h-4 text-slate-500" />
        <span>Structure</span>
        {loading && <span className="text-[11px] text-slate-400 font-medium">loading…</span>}
      </div>
      {sections.length > 0 && (
        <div className="text-xs font-mono text-slate-600 space-y-1">
          {sections.map((s) => (
            <div key={s.section_id} className="px-2 py-1 bg-slate-50 rounded-lg border border-slate-200/60">§ {s.section_id}</div>
          ))}
        </div>
      )}
      {pages.length > 0 && (
        <div className="text-xs text-slate-600">Pages: <span className="font-mono font-bold">{pages.join(', ')}</span></div>
      )}
      {sections.length === 0 && pages.length === 0 && !loading && (
        <div className="text-xs text-slate-400">No sections or pages recorded.</div>
      )}
    </div>
  );
}

export function DocumentChunkInspector({
  documentId,
  workspaceId,
}: {
  documentId: string;
  workspaceId: string;
}) {
  const [chunks, setChunks] = useState<DocumentChunkView[]>([]);
  const [loading, setLoading] = useState<boolean>(false);

  const load = async () => {
    setLoading(true);
    try {
      const res = await backendClient.getDocumentChunks(documentId, workspaceId);
      if (res.success) setChunks(res.chunks || []);
    } finally {
      setLoading(false);
    }
  };

  React.useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [documentId]);

  return (
    <div className="p-5 bg-white/80 border border-slate-200/80 rounded-2xl space-y-3">
      <div className="flex items-center space-x-2 text-sm font-extrabold text-slate-900">
        <Layers className="w-4 h-4 text-slate-500" />
        <span>Chunks ({chunks.length})</span>
        {loading && <span className="text-[11px] text-slate-400 font-medium">loading…</span>}
      </div>
      <div className="space-y-2 max-h-96 overflow-y-auto">
        {chunks.map((c) => (
          <div key={c.chunk_id} className="p-3 bg-slate-50 border border-slate-200/70 rounded-xl">
            <div className="flex items-center justify-between mb-1">
              <span className="text-[11px] font-mono font-bold text-slate-700">
                #{c.chunk_index} · {c.token_estimate} tok · {c.source}
              </span>
              <DocumentClassificationBadge classification={c.classification} />
            </div>
            <div className="text-xs text-slate-600 whitespace-pre-wrap font-medium">{c.text}</div>
          </div>
        ))}
      </div>
    </div>
  );
}

export function DocumentSearch({
  documentId,
  workspaceId,
}: {
  documentId: string;
  workspaceId: string;
}) {
  const [query, setQuery] = useState<string>('');
  const [results, setResults] = useState<DocumentSearchResult[]>([]);
  const [loading, setLoading] = useState<boolean>(false);

  const search = async () => {
    if (!query.trim()) return;
    setLoading(true);
    try {
      const res = await backendClient.searchDocument(documentId, { workspace_id: workspaceId, query: query.trim(), limit: 10 });
      if (res.success) setResults(res.results || []);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="p-5 bg-white/80 border border-slate-200/80 rounded-2xl space-y-3">
      <div className="flex items-center space-x-2 text-sm font-extrabold text-slate-900">
        <FileSearch className="w-4 h-4 text-slate-500" />
        <span>Lexical retrieval (workspace-isolated)</span>
      </div>
      <div className="flex items-center space-x-2">
        <input
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && search()}
          placeholder="Search within this document…"
          className="flex-1 px-3 py-2 rounded-xl text-xs bg-slate-50 border border-slate-200 focus:outline-none focus:ring-2 focus:ring-slate-900/10"
        />
        <button onClick={search} disabled={loading} className="px-3 py-2 rounded-xl bg-slate-900 text-white text-xs font-bold disabled:opacity-50">
          Search
        </button>
      </div>
      <div className="space-y-2">
        {results.map((r) => (
          <div key={r.chunk_id} className="p-3 bg-slate-50 border border-slate-200/70 rounded-xl">
            <div className="flex items-center justify-between mb-1">
              <span className="text-[11px] font-mono text-slate-500">relevance {r.relevance} · {r.source}</span>
              <DocumentClassificationBadge classification={r.classification} />
            </div>
            <div className="text-xs text-slate-600 whitespace-pre-wrap">{r.text}</div>
          </div>
        ))}
      </div>
      <div className="flex items-center space-x-2 text-[11px] text-slate-400">
        <ScrollText className="w-3.5 h-3.5" />
        <span>Results carry provenance and never cross workspace boundaries.</span>
      </div>
    </div>
  );
}
