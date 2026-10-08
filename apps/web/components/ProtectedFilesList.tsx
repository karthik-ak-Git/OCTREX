'use client';

import React from 'react';
import { Shield, FileCode, Lock, AlertCircle } from 'lucide-react';
import { ProtectedPath } from '@/lib/backend/types';

interface Props {
  protectedPaths?: ProtectedPath[];
}

export const ProtectedFilesList: React.FC<Props> = ({ protectedPaths }) => {
  const defaults: ProtectedPath[] = [
    { id: '1', pattern: '**/.env*', description: 'Environment credentials & secrets', action: 'deny', policy_source: 'system_security', enabled: true },
    { id: '2', pattern: '**/*.pem', description: 'SSL & SSH Private Keys', action: 'deny', policy_source: 'system_security', enabled: true },
    { id: '3', pattern: '**/id_rsa*', description: 'SSH Private Keys', action: 'deny', policy_source: 'system_security', enabled: true },
    { id: '4', pattern: '**/*.key', description: 'Private Encryption Keys', action: 'deny', policy_source: 'system_security', enabled: true },
    { id: '5', pattern: '**/.git/**', description: 'Git Repository Metadata', action: 'deny', policy_source: 'system_security', enabled: true },
  ];

  const list = protectedPaths && protectedPaths.length > 0 ? protectedPaths : defaults;

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-semibold text-slate-200 flex items-center gap-2">
          <Shield className="w-4 h-4 text-emerald-400" />
          Protected File Rules & Sensitive Patterns
        </h3>
        <span className="text-xs text-slate-400">{list.length} active rules</span>
      </div>

      <div className="divide-y divide-slate-800 rounded-lg border border-slate-800 bg-slate-900/60 overflow-hidden text-xs">
        {list.map((rule) => (
          <div key={rule.id} className="p-3 flex items-center justify-between hover:bg-slate-800/40 transition-colors">
            <div className="space-y-1">
              <div className="flex items-center gap-2 font-mono font-medium text-slate-200">
                <FileCode className="w-3.5 h-3.5 text-amber-400 shrink-0" />
                <span>{rule.pattern}</span>
              </div>
              <p className="text-slate-400 text-[11px] font-sans">{rule.description}</p>
            </div>

            <div className="flex items-center gap-2">
              <span className={`px-2 py-0.5 rounded text-[10px] uppercase font-mono font-semibold ${
                rule.action === 'deny'
                  ? 'bg-rose-500/20 text-rose-300 border border-rose-500/30'
                  : 'bg-amber-500/20 text-amber-300 border border-amber-500/30'
              }`}>
                {rule.action}
              </span>
              <span className="text-[10px] text-slate-500 font-mono">{rule.policy_source}</span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
