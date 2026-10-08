'use client';

import React from 'react';
import Link from 'next/link';
import { ArrowLeft, FileText, Shield, Database, Cpu } from 'lucide-react';

const FORMATS = [
  { name: 'TXT / plain text', detail: 'Paragraphs with line ranges', mime: 'text/plain' },
  { name: 'Markdown', detail: 'Headings, sections, code fences, lists', mime: 'text/markdown' },
  { name: 'JSON', detail: 'Validated; flattened per key/item', mime: 'application/json' },
  { name: 'CSV', detail: 'Headers + rows with source lines', mime: 'text/csv' },
  { name: 'XML', detail: 'Safe subset; DOCTYPE/ENTITY rejected (XXE)', mime: 'application/xml' },
  { name: 'PDF', detail: 'Heuristic text + Flate streams; page numbers kept', mime: 'application/pdf' },
  { name: 'DOCX', detail: 'Paragraphs, headings, flattened tables (OOXML)', mime: 'wordprocessingml' },
  { name: 'XLSX', detail: 'Sheets, rows, columns, formulas (OOXML)', mime: 'spreadsheetml' },
];

const INVARIANTS = [
  'Workspace isolation: documents never cross workspace boundaries.',
  'Filesystem boundary: intake only through FilesystemSecurityService; traversal and symlink escapes rejected.',
  'Classification monotonicity: processing may raise, never silently lower, classification.',
  'No document-driven policy changes, capability grants, or privacy decisions.',
  'No automatic cloud upload; no local → online fallback. Confidential+ stays local.',
  'Secrets redacted from logs, audit, events, and model prompts.',
  'Document chunks are FileContent/untrusted — never System role.',
  'Artifacts start UNVERIFIED; VerificationEngine must pass before claiming completion.',
  'Model claims are not evidence. Unknown authorization fails closed.',
  'Unsupported formats fail explicitly; parser failures do not bypass security.',
];

export default function DocumentsSettingsPage() {
  return (
    <div className="min-h-screen bg-[#eaedf3] text-slate-900 p-8 flex justify-center">
      <div className="w-[720px] space-y-6">
        <div className="flex items-center space-x-3 bg-white/80 backdrop-blur-xl border border-white/90 p-5 rounded-3xl shadow-sm">
          <Link href="/" className="p-2 text-slate-400 hover:text-slate-800 rounded-xl hover:bg-slate-100 transition-all">
            <ArrowLeft className="w-5 h-5" />
          </Link>
          <div>
            <h1 className="text-xl font-extrabold tracking-tight flex items-center space-x-2">
              <FileText className="w-5 h-5 text-cyan-700" />
              <span>Document Intelligence Settings</span>
            </h1>
            <p className="text-xs text-slate-500 font-medium">Parser boundaries · privacy behavior · local model policy</p>
          </div>
        </div>

        <div className="bg-white/80 border border-white/90 p-6 rounded-3xl shadow-sm space-y-3">
          <h2 className="text-sm font-extrabold flex items-center space-x-2">
            <Database className="w-4 h-4 text-slate-500" />
            <span>Supported formats (honest parsing, no guessing)</span>
          </h2>
          <div className="grid grid-cols-2 gap-2">
            {FORMATS.map((f) => (
              <div key={f.name} className="p-3 bg-slate-50 border border-slate-200/70 rounded-2xl">
                <div className="text-xs font-extrabold">{f.name}</div>
                <div className="text-[11px] text-slate-500">{f.detail}</div>
                <div className="text-[10px] font-mono text-slate-400 mt-1">{f.mime}</div>
              </div>
            ))}
          </div>
          <p className="text-[11px] text-slate-400">Dependencies: none added for parsing (dependency-free DEFLATE/ZIP/OOXML readers). PDFs that are scanned images report no-text explicitly.</p>
        </div>

        <div className="bg-white/80 border border-white/90 p-6 rounded-3xl shadow-sm space-y-3">
          <h2 className="text-sm font-extrabold flex items-center space-x-2">
            <Shield className="w-4 h-4 text-emerald-600" />
            <span>Security invariants (fail-closed)</span>
          </h2>
          <ul className="text-xs text-slate-600 space-y-2 font-medium list-disc pl-4">
            {INVARIANTS.map((v) => (
              <li key={v}>{v}</li>
            ))}
          </ul>
        </div>

        <div className="bg-white/80 border border-white/90 p-6 rounded-3xl shadow-sm space-y-2">
          <h2 className="text-sm font-extrabold flex items-center space-x-2">
            <Cpu className="w-4 h-4 text-slate-500" />
            <span>AI assistance policy</span>
          </h2>
          <p className="text-xs text-slate-600 font-medium">
            Summarization, extraction, and labeling route strictly <span className="font-mono font-bold">LocalOnly</span> through
            ModelRouter → ModelRuntime with PrivacyGate checks. If no permitted local model exists, the backend returns an
            explicit unavailable/blocked result — it never falls back to cloud automatically.
          </p>
          <div className="flex space-x-2">
            <Link href="/documents" className="px-3 py-2 rounded-xl bg-slate-900 text-white text-xs font-bold">Documents</Link>
            <Link href="/artifacts" className="px-3 py-2 rounded-xl bg-slate-100 text-xs font-bold hover:bg-slate-200">Artifacts</Link>
          </div>
        </div>
      </div>
    </div>
  );
}
