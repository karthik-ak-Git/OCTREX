'use client';

import React, { useState, useEffect } from 'react';
import Link from 'next/link';
import { Cpu, ArrowLeft, Layers, Sliders, CheckCircle, RefreshCw, BarChart2, Shield } from 'lucide-react';
import { backendClient } from '../../../lib/backend/client';
import { ContextStatusResponse } from '../../../lib/backend/types';

export default function ContextSettingsPage() {
  const [status, setStatus] = useState<ContextStatusResponse | null>(null);
  const [loading, setLoading] = useState<boolean>(true);

  const fetchStatus = async () => {
    setLoading(true);
    try {
      const res = await backendClient.getContextStatus();
      setStatus(res);
    } catch (err) {
      console.log('Error fetching context status:', err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchStatus();
  }, []);

  return (
    <div className="min-h-screen bg-[#eaedf3] text-slate-900 p-8 flex justify-center">
      <div className="w-[680px] space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between bg-white/80 backdrop-blur-xl border border-white/90 p-5 rounded-3xl shadow-sm">
          <div className="flex items-center space-x-3">
            <Link href="/" className="p-2 text-slate-400 hover:text-slate-800 rounded-xl hover:bg-slate-100 transition-all">
              <ArrowLeft className="w-5 h-5" />
            </Link>
            <div>
              <h1 className="text-xl font-extrabold text-slate-900 tracking-tight">Context Engine Settings</h1>
              <p className="text-xs text-slate-500 font-medium">Token budgeting, context assembly & bounded compaction</p>
            </div>
          </div>
          <button
            onClick={fetchStatus}
            disabled={loading}
            className="p-2 text-slate-500 hover:text-slate-800 hover:bg-slate-100 rounded-xl transition-all"
            title="Refresh status"
          >
            <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
          </button>
        </div>

        {/* Runtime Status Overview Card */}
        <div className="bg-white/80 backdrop-blur-xl border border-white/90 p-6 rounded-3xl space-y-4 shadow-sm">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2.5">
              <div className="w-9 h-9 rounded-2xl bg-slate-900 text-white flex items-center justify-center font-bold shadow-xs">
                <Cpu className="w-5 h-5" />
              </div>
              <div>
                <h2 className="text-sm font-extrabold text-slate-900">Engine Runtime Status</h2>
                <p className="text-xs text-slate-500 font-medium">{status?.service || 'ContextEngineService'}</p>
              </div>
            </div>
            <span className="px-3 py-1 rounded-full text-xs font-extrabold bg-emerald-100 text-emerald-800 border border-emerald-300 flex items-center space-x-1 font-mono">
              <CheckCircle className="w-3.5 h-3.5 text-emerald-600" />
              <span>{status?.status || 'READY'}</span>
            </span>
          </div>

          <div className="grid grid-cols-3 gap-3 pt-2">
            <div className="p-4 bg-slate-50 border border-slate-200/80 rounded-2xl">
              <div className="text-[10px] font-extrabold uppercase text-slate-400">Active Sessions</div>
              <div className="text-xl font-extrabold font-mono text-slate-900 mt-1">{status?.active_sessions ?? 0}</div>
            </div>
            <div className="p-4 bg-slate-50 border border-slate-200/80 rounded-2xl">
              <div className="text-[10px] font-extrabold uppercase text-slate-400">Context Items Tracked</div>
              <div className="text-xl font-extrabold font-mono text-slate-900 mt-1">{status?.total_items_tracked ?? 0}</div>
            </div>
            <div className="p-4 bg-slate-50 border border-slate-200/80 rounded-2xl">
              <div className="text-[10px] font-extrabold uppercase text-slate-400">Checkpoints Saved</div>
              <div className="text-xl font-extrabold font-mono text-slate-900 mt-1">{status?.total_checkpoints ?? 0}</div>
            </div>
          </div>
        </div>

        {/* Sub-page Navigation Cards */}
        <div className="grid grid-cols-2 gap-4">
          <Link
            href="/settings/context/budget"
            className="p-6 bg-white/80 backdrop-blur-xl border border-white/90 rounded-3xl shadow-sm hover:bg-white transition-all space-y-3 block group"
          >
            <div className="w-10 h-10 rounded-2xl bg-slate-900 text-white flex items-center justify-center font-bold shadow-sm group-hover:scale-105 transition-transform">
              <BarChart2 className="w-5 h-5" />
            </div>
            <div>
              <h3 className="text-sm font-extrabold text-slate-900">Token Budget Inspector</h3>
              <p className="text-xs text-slate-500 font-medium mt-1">
                Inspect 25% output reservation, input budgets and live session token accounting.
              </p>
            </div>
          </Link>

          <Link
            href="/settings/context/compaction"
            className="p-6 bg-white/80 backdrop-blur-xl border border-white/90 rounded-3xl shadow-sm hover:bg-white transition-all space-y-3 block group"
          >
            <div className="w-10 h-10 rounded-2xl bg-purple-900 text-white flex items-center justify-center font-bold shadow-sm group-hover:scale-105 transition-transform">
              <Layers className="w-5 h-5" />
            </div>
            <div>
              <h3 className="text-sm font-extrabold text-slate-900">Compaction Engine</h3>
              <p className="text-xs text-slate-500 font-medium mt-1">
                Trigger bounded structured compaction to reduce context overhead safely.
              </p>
            </div>
          </Link>
        </div>

        {/* Security Invariants Card */}
        <div className="bg-white/80 backdrop-blur-xl border border-white/90 p-6 rounded-3xl space-y-3 shadow-sm">
          <div className="flex items-center space-x-2 text-slate-900 font-extrabold text-sm">
            <Shield className="w-4 h-4 text-emerald-600" />
            <span>Context Security Invariants Enforced</span>
          </div>
          <ul className="text-xs text-slate-600 space-y-2 font-medium list-disc pl-4">
            <li><strong>Role Safety:</strong> FileContent and ToolResult items CANNOT assume System role.</li>
            <li><strong>Classification Non-Downgrade:</strong> Compaction summaries maintain max(source classifications).</li>
            <li><strong>Control Plane Protection:</strong> Policy instructions remain separate from user/tool data plane content.</li>
            <li><strong>Zero Remote Leakage:</strong> SECRET items never expose raw contents in diagnostics or previews.</li>
          </ul>
        </div>
      </div>
    </div>
  );
}
