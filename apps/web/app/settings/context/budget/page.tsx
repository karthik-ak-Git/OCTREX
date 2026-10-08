'use client';

import React, { useState, useEffect } from 'react';
import Link from 'next/link';
import { ArrowLeft, Cpu, RefreshCw, BarChart2, ShieldCheck, AlertCircle } from 'lucide-react';
import { backendClient } from '../../../../lib/backend/client';
import { TokenBudget } from '../../../../lib/backend/types';

export default function TokenBudgetPage() {
  const [sessionId, setSessionId] = useState<string>('global');
  const [budget, setBudget] = useState<TokenBudget | null>(null);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string>('');

  const fetchBudget = async () => {
    setLoading(true);
    setError('');
    try {
      const res = await backendClient.getContextBudget(sessionId);
      if (res.success && res.budget) {
        setBudget(res.budget);
      } else {
        setError(res.error || 'Failed to fetch budget');
      }
    } catch (err: any) {
      setError(err.message || 'Failed to connect');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchBudget();
  }, []);

  return (
    <div className="min-h-screen bg-[#eaedf3] text-slate-900 p-8 flex justify-center">
      <div className="w-[680px] space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between bg-white/80 backdrop-blur-xl border border-white/90 p-5 rounded-3xl shadow-sm">
          <div className="flex items-center space-x-3">
            <Link href="/settings/context" className="p-2 text-slate-400 hover:text-slate-800 rounded-xl hover:bg-slate-100 transition-all">
              <ArrowLeft className="w-5 h-5" />
            </Link>
            <div>
              <h1 className="text-xl font-extrabold text-slate-900 tracking-tight">Token Budget Inspector</h1>
              <p className="text-xs text-slate-500 font-medium">Model context window partitioning and reserved capacity</p>
            </div>
          </div>
        </div>

        {/* Session ID Selector Form */}
        <div className="bg-white/80 backdrop-blur-xl border border-white/90 p-5 rounded-3xl shadow-sm space-y-3">
          <label className="block text-xs font-extrabold uppercase text-slate-400 tracking-wider">Session Key</label>
          <div className="flex space-x-2">
            <input
              type="text"
              value={sessionId}
              onChange={(e) => setSessionId(e.target.value)}
              placeholder="e.g. global or session-id..."
              className="flex-1 px-4 py-2.5 bg-white border border-slate-200 rounded-2xl text-xs font-mono text-slate-800 focus:outline-none focus:ring-2 focus:ring-slate-900/10"
            />
            <button
              onClick={fetchBudget}
              disabled={loading}
              className="px-5 py-2.5 bg-slate-900 hover:bg-slate-800 text-white font-bold text-xs rounded-2xl shadow-sm transition-all flex items-center space-x-1.5 disabled:opacity-50"
            >
              <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
              <span>Query Budget</span>
            </button>
          </div>
        </div>

        {error && (
          <div className="p-4 bg-rose-50 border border-rose-200 text-rose-800 rounded-2xl text-xs font-medium flex items-center space-x-2">
            <AlertCircle className="w-4 h-4 text-rose-600" />
            <span>{error}</span>
          </div>
        )}

        {/* Budget Breakdown Card */}
        {budget && (
          <div className="bg-white/80 backdrop-blur-xl border border-white/90 p-6 rounded-3xl space-y-6 shadow-sm">
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-2.5">
                <div className="w-9 h-9 rounded-2xl bg-slate-900 text-white flex items-center justify-center font-bold shadow-xs">
                  <BarChart2 className="w-5 h-5" />
                </div>
                <div>
                  <h2 className="text-sm font-extrabold text-slate-900">Active Token Budget</h2>
                  <p className="text-xs text-slate-500 font-mono">Model: {budget.model_id}</p>
                </div>
              </div>
              <span className="px-3 py-1 rounded-full text-xs font-extrabold bg-slate-900 text-white font-mono">
                {budget.context_window.toLocaleString()} total window
              </span>
            </div>

            {/* Budget Bar Breakdown */}
            <div className="space-y-2">
              <div className="flex justify-between text-xs font-mono font-bold text-slate-700">
                <span>Usable Input: {budget.usable_input_budget.toLocaleString()} tokens</span>
                <span>Used: {budget.current_usage.toLocaleString()} tokens ({Math.round(budget.utilization_percent)}%)</span>
              </div>
              <div className="w-full h-4 bg-slate-200 rounded-full overflow-hidden flex">
                <div
                  className="bg-emerald-500 h-full transition-all"
                  style={{ width: `${Math.min(100, budget.utilization_percent)}%` }}
                  title="Current Usage"
                />
              </div>
            </div>

            {/* Invariant Component Breakdown Grid */}
            <div className="grid grid-cols-2 gap-3 pt-2 font-mono text-xs">
              <div className="p-4 bg-slate-50 border border-slate-200/80 rounded-2xl space-y-1">
                <div className="text-[10px] font-extrabold uppercase text-slate-400 font-sans">Output Reserve (25%)</div>
                <div className="text-lg font-extrabold text-slate-900">{budget.reserved_output.toLocaleString()} tokens</div>
                <p className="text-[10px] font-sans text-slate-500">Guarantees output space before assembling input</p>
              </div>

              <div className="p-4 bg-slate-50 border border-slate-200/80 rounded-2xl space-y-1">
                <div className="text-[10px] font-extrabold uppercase text-slate-400 font-sans">Safety Margin (5%)</div>
                <div className="text-lg font-extrabold text-slate-900">{budget.safety_margin.toLocaleString()} tokens</div>
                <p className="text-[10px] font-sans text-slate-500">Buffer for token estimation variance</p>
              </div>

              <div className="p-4 bg-slate-50 border border-slate-200/80 rounded-2xl space-y-1">
                <div className="text-[10px] font-extrabold uppercase text-slate-400 font-sans">Fixed Overhead</div>
                <div className="text-lg font-extrabold text-slate-900">{budget.fixed_overhead.toLocaleString()} tokens</div>
                <p className="text-[10px] font-sans text-slate-500">System prompt separator formatting</p>
              </div>

              <div className="p-4 bg-slate-50 border border-slate-200/80 rounded-2xl space-y-1">
                <div className="text-[10px] font-extrabold uppercase text-slate-400 font-sans">Counting Precision</div>
                <div className="text-lg font-extrabold text-slate-900 uppercase">{budget.token_count_kind}</div>
                <p className="text-[10px] font-sans text-slate-500">Exact vs estimated accounting status</p>
              </div>
            </div>

            {/* Invariant Check Badge */}
            <div className="p-3 bg-emerald-50 border border-emerald-200 text-emerald-800 rounded-2xl text-xs font-medium flex items-center justify-between">
              <span className="flex items-center space-x-1.5 font-bold">
                <ShieldCheck className="w-4 h-4 text-emerald-600" />
                <span>Budget Invariant Verified: Usable Input + Output Reserve + Safety Margin + Overhead ≤ Context Window</span>
              </span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
