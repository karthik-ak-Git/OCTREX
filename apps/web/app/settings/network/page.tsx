'use client';

import React, { useState, useEffect } from 'react';
import Link from 'next/link';
import { Globe, ArrowLeft, Check, ShieldAlert, Save } from 'lucide-react';

type NetworkStatus = {
  mode: string;
  enabled: boolean;
  service_health: string;
  active_rules_count: number;
  allowlist_count: number;
  blocked_count: number;
  total_decisions_count: number;
};

const MODES = [
  { id: 'local_only', label: 'LOCAL ONLY', desc: 'External network requests are strictly prohibited. Loopback/local runtimes only.' },
  { id: 'restricted', label: 'RESTRICTED', desc: 'Only explicitly allowed destinations pass. Everything else is blocked.' },
  { id: 'online_allowed', label: 'ONLINE ALLOWED', desc: 'Permits approved cloud provider APIs after policy evaluation.' },
  { id: 'disabled', label: 'DISABLED', desc: 'Network boundary disabled. All outbound requests are blocked fail-closed.' },
];

export default function NetworkSettingsPage() {
  const [status, setStatus] = useState<NetworkStatus | null>(null);
  const [mode, setMode] = useState<string>('local_only');
  const [statusMessage, setStatusMessage] = useState<string>('');
  const [isSaving, setIsSaving] = useState<boolean>(false);

  const fetchStatus = async () => {
    try {
      const res = await fetch('/api/network/status');
      if (res.ok) {
        const data = await res.json();
        setStatus(data);
        if (data.mode) setMode(data.mode);
      }
    } catch (e) {
      console.log('Error fetching network status:', e);
    }
  };

  useEffect(() => {
    fetchStatus();
  }, []);

  const handleSave = async () => {
    setIsSaving(true);
    setStatusMessage('Saving network mode...');
    try {
      const res = await fetch('/api/network/mode', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ mode }),
      });
      const data = await res.json();
      if (res.ok && data.success) {
        setStatusMessage('✓ Network mode updated successfully.');
        fetchStatus();
      } else {
        setStatusMessage(`Failed to save: ${data.error || 'unknown error'}`);
      }
    } catch (e: any) {
      setStatusMessage(`Error: ${e.message}`);
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="min-h-screen bg-[#eaedf3] text-slate-900 p-8 flex justify-center">
      <div className="w-[680px] space-y-6">

        <div className="flex items-center justify-between bg-white/80 backdrop-blur-xl border border-white/90 p-5 rounded-3xl shadow-sm">
          <div className="flex items-center space-x-3">
            <Link href="/" className="p-2 text-slate-400 hover:text-slate-800 rounded-xl hover:bg-slate-100 transition-all">
              <ArrowLeft className="w-5 h-5" />
            </Link>
            <div>
              <h1 className="text-xl font-extrabold text-slate-900 tracking-tight">Network Settings</h1>
              <p className="text-xs text-slate-500 font-medium">Authoritative NetworkSecurityService boundary — fail-closed</p>
            </div>
          </div>
          {status && (
            <span className="px-3 py-1.5 rounded-xl text-xs font-mono font-bold bg-slate-900 text-white">
              {status.mode.toUpperCase()} · {status.service_health}
            </span>
          )}
        </div>

        {statusMessage && (
          <div className="p-3.5 bg-slate-900 text-white rounded-2xl text-xs font-mono font-medium flex items-center justify-between shadow-sm">
            <span>{statusMessage}</span>
            <button onClick={() => setStatusMessage('')} className="text-slate-400 hover:text-white">✕</button>
          </div>
        )}

        <div className="bg-white/80 backdrop-blur-xl border border-white/90 p-6 rounded-3xl space-y-4 shadow-sm">
          <div>
            <h2 className="text-sm font-extrabold text-slate-900">Network Execution Mode</h2>
            <p className="text-xs text-slate-500 font-medium">Controls which outbound destinations the backend will permit</p>
          </div>

          <div className="grid grid-cols-2 gap-3">
            {MODES.map((m) => (
              <div
                key={m.id}
                onClick={() => setMode(m.id)}
                className={`p-4 rounded-2xl border cursor-pointer transition-all ${
                  mode === m.id
                    ? 'bg-slate-900 text-white border-slate-900 shadow-md'
                    : 'bg-white text-slate-800 border-slate-200/80 hover:bg-slate-50'
                }`}
              >
                <div className="flex items-center justify-between mb-1">
                  <span className="font-extrabold text-sm flex items-center space-x-1.5">
                    <Globe className="w-4 h-4" />
                    <span>{m.label}</span>
                  </span>
                  {mode === m.id && <Check className="w-4 h-4 text-emerald-400" />}
                </div>
                <p className={`text-xs ${mode === m.id ? 'text-slate-300' : 'text-slate-500'}`}>
                  {m.desc}
                </p>
              </div>
            ))}
          </div>
        </div>

        {status && (
          <div className="bg-white/80 backdrop-blur-xl border border-white/90 p-5 rounded-3xl shadow-sm">
            <div className="text-sm font-extrabold text-slate-900 flex items-center space-x-2 mb-3">
              <ShieldAlert className="w-4 h-4 text-amber-600" />
              <span>Boundary Status</span>
            </div>
            <div className="grid grid-cols-2 gap-2 text-xs font-mono">
              <div className="p-3 bg-slate-50 rounded-xl">Active rules: <b>{status.active_rules_count}</b></div>
              <div className="p-3 bg-slate-50 rounded-xl">Allowlist entries: <b>{status.allowlist_count}</b></div>
              <div className="p-3 bg-slate-50 rounded-xl">Blocked decisions: <b>{status.blocked_count}</b></div>
              <div className="p-3 bg-slate-50 rounded-xl">Total decisions: <b>{status.total_decisions_count}</b></div>
            </div>
            <p className="text-xs text-slate-500 font-medium mt-3">
              Redirects are independently re-evaluated. A redirect to a private address never bypasses validation.
            </p>
          </div>
        )}

        <div className="flex justify-end pt-2">
          <button
            onClick={handleSave}
            disabled={isSaving}
            className="px-6 py-3 bg-slate-900 hover:bg-slate-800 text-white font-bold text-xs rounded-2xl shadow-md transition-all flex items-center space-x-2 disabled:opacity-50"
          >
            <Save className="w-4 h-4" />
            <span>{isSaving ? 'Saving...' : 'Save Network Mode'}</span>
          </button>
        </div>

      </div>
    </div>
  );
}
