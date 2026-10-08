'use client';

import React, { useState, useEffect } from 'react';
import Link from 'next/link';
import { Shield, ArrowLeft, CheckCircle2, AlertTriangle, Lock, FileText } from 'lucide-react';
import { PolicyRule } from '../../../lib/backend/types';

export default function PolicyStatusPage() {
  const [policies, setPolicies] = useState<PolicyRule[]>([]);
  const [policyVersion, setPolicyVersion] = useState<number>(1);
  const [effectiveStatus, setEffectiveStatus] = useState<any>(null);

  const fetchPolicies = async () => {
    try {
      const res = await fetch('/api/privacy/policies');
      if (res.ok) {
        const data = await res.json();
        setPolicies(data.rules || []);
        setPolicyVersion(data.version || 1);
      }

      const stRes = await fetch('/api/privacy/status');
      if (stRes.ok) {
        const stData = await stRes.json();
        setEffectiveStatus(stData);
      }
    } catch (e) {
      console.log('Error fetching policies:', e);
    }
  };

  useEffect(() => {
    fetchPolicies();
  }, []);

  return (
    <div className="min-h-screen bg-[#eaedf3] text-slate-900 p-8 flex justify-center">
      <div className="w-[720px] space-y-6">
        
        {/* Header */}
        <div className="flex items-center justify-between bg-white/80 backdrop-blur-xl border border-white/90 p-5 rounded-3xl shadow-sm">
          <div className="flex items-center space-x-3">
            <Link href="/" className="p-2 text-slate-400 hover:text-slate-800 rounded-xl hover:bg-slate-100 transition-all">
              <ArrowLeft className="w-5 h-5" />
            </Link>
            <div>
              <h1 className="text-xl font-extrabold text-slate-900 tracking-tight">Policy Status & Rules</h1>
              <p className="text-xs text-slate-500 font-medium">Effective system, company, and security policy hierarchy</p>
            </div>
          </div>
          <span className="px-3 py-1 rounded-full bg-slate-900 text-white font-mono text-xs font-bold">
            Policy v{policyVersion}
          </span>
        </div>

        {/* Policy Hierarchy Card */}
        <div className="bg-white/80 backdrop-blur-xl border border-white/90 p-6 rounded-3xl space-y-3 shadow-sm">
          <h2 className="text-sm font-extrabold text-slate-900">Deterministic Policy Hierarchy</h2>
          <p className="text-xs text-slate-500 font-medium leading-relaxed">
            Higher-level policies strictly override lower-level preferences. User options can never bypass Company or System rules.
          </p>

          <div className="space-y-2 pt-2 text-xs font-mono">
            <div className="p-3 bg-rose-50 border border-rose-200 rounded-2xl flex justify-between items-center text-rose-900 font-bold">
              <span>1. SYSTEM POLICY (Priority 100 - Highest)</span>
              <span className="text-[10px] bg-rose-200 px-2 py-0.5 rounded-full">IMMUTABLE</span>
            </div>
            <div className="p-3 bg-amber-50 border border-amber-200 rounded-2xl flex justify-between items-center text-amber-900 font-bold">
              <span>2. COMPANY POLICY (Priority 80)</span>
              <span className="text-[10px] bg-amber-200 px-2 py-0.5 rounded-full">ENTERPRISE</span>
            </div>
            <div className="p-3 bg-sky-50 border border-sky-200 rounded-2xl flex justify-between items-center text-sky-900 font-bold">
              <span>3. SECURITY POLICY (Priority 60)</span>
              <span className="text-[10px] bg-sky-200 px-2 py-0.5 rounded-full">ACTIVE</span>
            </div>
            <div className="p-3 bg-indigo-50 border border-indigo-200 rounded-2xl flex justify-between items-center text-indigo-900 font-bold">
              <span>4. PRIVACY POLICY (Priority 40)</span>
              <span className="text-[10px] bg-indigo-200 px-2 py-0.5 rounded-full">ACTIVE</span>
            </div>
            <div className="p-3 bg-emerald-50 border border-emerald-200 rounded-2xl flex justify-between items-center text-emerald-900 font-bold">
              <span>5. USER PREFERENCES (Priority 20 - Lowest)</span>
              <span className="text-[10px] bg-emerald-200 px-2 py-0.5 rounded-full">CONFIGURABLE</span>
            </div>
          </div>
        </div>

        {/* Active Policy Rules List */}
        <div className="bg-white/80 backdrop-blur-xl border border-white/90 p-6 rounded-3xl space-y-4 shadow-sm">
          <div className="flex justify-between items-center">
            <div>
              <h2 className="text-sm font-extrabold text-slate-900">Active Policy Rules ({policies.length})</h2>
              <p className="text-xs text-slate-500 font-medium">Evaluated in order before model selection</p>
            </div>
          </div>

          <div className="space-y-2.5">
            {policies.map((rule) => (
              <div key={rule.id} className="p-4 bg-white border border-slate-200/90 rounded-2xl space-y-1.5 shadow-2xs">
                <div className="flex items-center justify-between text-xs">
                  <div className="flex items-center space-x-2">
                    <span className="font-extrabold text-slate-900">{rule.name}</span>
                    <span className="font-mono text-[10px] text-slate-400">[{rule.id}]</span>
                  </div>
                  <div className="flex items-center space-x-2">
                    <span className="px-2 py-0.5 rounded-full bg-slate-100 font-mono text-[10px] font-bold text-slate-700">
                      {rule.source} ({rule.priority})
                    </span>
                    <span className={`px-2 py-0.5 rounded-full font-mono text-[10px] font-bold ${
                      rule.action === 'DENY' ? 'bg-rose-100 text-rose-800' :
                      rule.action === 'REQUIRE_CONSENT' ? 'bg-amber-100 text-amber-800' : 'bg-emerald-100 text-emerald-800'
                    }`}>
                      {rule.action}
                    </span>
                  </div>
                </div>
                <p className="text-xs text-slate-600 font-medium">{rule.reason}</p>
              </div>
            ))}
          </div>
        </div>

      </div>
    </div>
  );
}
