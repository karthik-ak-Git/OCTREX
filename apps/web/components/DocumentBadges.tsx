'use client';

import React from 'react';
import { Shield, ShieldAlert, Lock, Eye } from 'lucide-react';

const CLASS_STYLES: Record<string, string> = {
  PUBLIC: 'bg-slate-100 text-slate-700 border-slate-300',
  INTERNAL: 'bg-sky-100 text-sky-800 border-sky-300',
  CONFIDENTIAL: 'bg-amber-100 text-amber-800 border-amber-300',
  RESTRICTED: 'bg-orange-100 text-orange-800 border-orange-300',
  SECRET: 'bg-rose-100 text-rose-800 border-rose-400',
};

export function DocumentClassificationBadge({ classification }: { classification?: string }) {
  const c = (classification || 'PUBLIC').toUpperCase();
  const style = CLASS_STYLES[c] || CLASS_STYLES.PUBLIC;
  const Icon = c === 'SECRET' ? Lock : c === 'PUBLIC' ? Eye : Shield;
  return (
    <span className={`inline-flex items-center space-x-1 px-2.5 py-0.5 rounded-full text-[11px] font-extrabold border font-mono ${style}`}>
      <Icon className="w-3 h-3" />
      <span>{c}</span>
    </span>
  );
}

export function DocumentSecurityBadge({ findings }: { findings?: Array<{ category: string; severity: string; summary: string }> }) {
  const count = findings?.length || 0;
  if (count === 0) {
    return (
      <span className="inline-flex items-center space-x-1 px-2.5 py-0.5 rounded-full text-[11px] font-bold bg-emerald-100 text-emerald-800 border border-emerald-300">
        <Shield className="w-3 h-3" />
        <span>No findings</span>
      </span>
    );
  }
  const critical = findings?.some((f) => f.severity === 'CRITICAL');
  return (
    <span
      className={`inline-flex items-center space-x-1 px-2.5 py-0.5 rounded-full text-[11px] font-bold border ${
        critical ? 'bg-rose-100 text-rose-800 border-rose-400' : 'bg-amber-100 text-amber-800 border-amber-300'
      }`}
      title={findings?.map((f) => `[${f.category}] ${f.summary}`).join('\n')}
    >
      <ShieldAlert className="w-3 h-3" />
      <span>{count} finding{count === 1 ? '' : 's'}</span>
    </span>
  );
}
