'use client';

import React, { useState, useEffect } from 'react';
import {
  Shield,
  Globe,
  Lock,
  Server,
  Check,
  X,
  Activity as ActivityIcon,
  AlertTriangle,
  Plus,
  Trash2,
  Eye,
  RefreshCw,
  Sliders,
  FileText,
  Key,
  Info,
  ChevronRight
} from 'lucide-react';
import { backendClient } from '../lib/backend/client';
import {
  AllowlistEntry,
  NetworkDecision,
  NetworkMode,
  NetworkRule,
  NetworkSecurityStatus
} from '../lib/backend/types';

export function NetworkSecurityBadge({ mode, blockedCount }: { mode?: NetworkMode; blockedCount?: number }) {
  const currentMode = mode || 'local_only';

  let badgeColor = 'bg-emerald-900/90 text-emerald-300 border-emerald-700/60';
  let label = 'LOCAL ONLY';

  if (currentMode === 'restricted') {
    badgeColor = 'bg-amber-950/90 text-amber-300 border-amber-800/60';
    label = 'RESTRICTED';
  } else if (currentMode === 'online_allowed') {
    badgeColor = 'bg-cyan-950/90 text-cyan-300 border-cyan-800/60';
    label = 'ONLINE ALLOWED';
  } else if (currentMode === 'disabled') {
    badgeColor = 'bg-rose-950/90 text-rose-300 border-rose-800/60';
    label = 'NETWORK BLOCKED';
  }

  return (
    <div className={`inline-flex items-center space-x-1.5 px-2.5 py-1 rounded-full text-[10px] font-bold border font-mono tracking-wider shadow-2xs ${badgeColor}`}>
      <Shield className="w-3 h-3" />
      <span>{label}</span>
      {blockedCount !== undefined && blockedCount > 0 && (
        <span className="ml-1 px-1.5 py-0.2 rounded-full bg-rose-500/20 text-rose-400 text-[9px]">
          {blockedCount} BLOCKED
        </span>
      )}
    </div>
  );
}

interface NetworkSecurityModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export function NetworkSecurityModal({ isOpen, onClose }: NetworkSecurityModalProps) {
  const [activeTab, setActiveTab] = useState<'settings' | 'policy' | 'activity' | 'dry_run'>('settings');
  const [status, setStatus] = useState<NetworkSecurityStatus | null>(null);
  const [rules, setRules] = useState<NetworkRule[]>([]);
  const [allowlist, setAllowlist] = useState<AllowlistEntry[]>([]);
  const [decisions, setDecisions] = useState<NetworkDecision[]>([]);
  const [selectedDecision, setSelectedDecision] = useState<NetworkDecision | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(false);

  // New Allowlist Form State
  const [newDomain, setNewDomain] = useState<string>('');
  const [newDomainDesc, setNewDomainDesc] = useState<string>('');

  // Dry Run Form State
  const [testUrl, setTestUrl] = useState<string>('https://api.openai.com/v1/chat/completions');
  const [testCapability, setTestCapability] = useState<string>('cloud_model_inference');
  const [testResult, setTestResult] = useState<NetworkDecision | null>(null);

  const fetchNetworkData = async () => {
    setIsLoading(true);
    try {
      const [st, pol, dec] = await Promise.all([
        backendClient.getNetworkStatus(),
        backendClient.getNetworkPolicy(),
        backendClient.getNetworkDecisions(),
      ]);

      setStatus(st);
      if (pol.success) {
        setRules(pol.active_rules || []);
        setAllowlist(pol.allowlist?.entries || []);
      }
      if (dec.success) {
        setDecisions(dec.decisions || []);
      }
    } catch (err) {
      console.error('Error fetching network security data:', err);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    if (isOpen) {
      fetchNetworkData();
    }
  }, [isOpen]);

  const handleModeChange = async (newMode: NetworkMode) => {
    try {
      const res = await backendClient.setNetworkMode(newMode);
      if (res.success) {
        setStatus(res.status);
      }
    } catch (err: any) {
      alert(`Failed to set mode: ${err.message}`);
    }
  };

  const handleAddAllowlist = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newDomain.trim()) return;
    try {
      const res = await backendClient.addNetworkAllowlist(newDomain.trim(), newDomainDesc.trim());
      if (res.success) {
        setNewDomain('');
        setNewDomainDesc('');
        fetchNetworkData();
      }
    } catch (err: any) {
      alert(`Error adding allowlist entry: ${err.message}`);
    }
  };

  const handleDeleteAllowlist = async (id: string) => {
    try {
      const res = await backendClient.deleteNetworkAllowlist(id);
      if (res.success) {
        fetchNetworkData();
      }
    } catch (err: any) {
      alert(`Error removing allowlist entry: ${err.message}`);
    }
  };

  const handleDryRunTest = async () => {
    if (!testUrl.trim()) return;
    try {
      const res = await backendClient.evaluateNetworkRequest({
        destination: testUrl.trim(),
        capability: testCapability as any,
        source: 'dry_run_ui_inspector',
      });
      if (res.success) {
        setTestResult(res.decision);
      }
    } catch (err: any) {
      alert(`Dry run evaluation failed: ${err.message}`);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/60 backdrop-blur-md p-4 animate-in fade-in duration-200">
      <div className="w-[880px] max-h-[85vh] bg-slate-900 border border-slate-800 rounded-3xl text-slate-100 shadow-2xl flex flex-col overflow-hidden">
        
        {/* Modal Header */}
        <div className="p-6 border-b border-slate-800/80 flex items-center justify-between bg-slate-950/40">
          <div className="flex items-center space-x-3">
            <div className="w-10 h-10 rounded-2xl bg-cyan-950/80 border border-cyan-700/60 flex items-center justify-center text-cyan-400">
              <Shield className="w-5 h-5" />
            </div>
            <div>
              <div className="flex items-center space-x-2">
                <h2 className="text-lg font-bold tracking-tight text-white">Network Security Boundary</h2>
                <NetworkSecurityBadge mode={status?.mode} blockedCount={status?.blocked_count} />
              </div>
              <p className="text-xs text-slate-400">
                Centralized Outbound Traffic Control & Fail-Closed Policy Engine (Phase 7)
              </p>
            </div>
          </div>

          <div className="flex items-center space-x-2">
            <button
              onClick={fetchNetworkData}
              disabled={isLoading}
              title="Refresh status"
              className="p-2 rounded-xl bg-slate-800/80 hover:bg-slate-700 text-slate-300 transition-all disabled:opacity-50"
            >
              <RefreshCw className={`w-4 h-4 ${isLoading ? 'animate-spin' : ''}`} />
            </button>
            <button
              onClick={onClose}
              className="p-2 rounded-xl bg-slate-800/80 hover:bg-slate-700 text-slate-300 transition-all"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
        </div>

        {/* Tab Bar */}
        <div className="flex border-b border-slate-800 bg-slate-950/20 px-6 pt-2">
          <button
            onClick={() => setActiveTab('settings')}
            className={`px-4 py-3 text-xs font-bold border-b-2 flex items-center space-x-2 transition-all ${
              activeTab === 'settings'
                ? 'border-cyan-500 text-cyan-400 bg-cyan-950/20'
                : 'border-transparent text-slate-400 hover:text-slate-200'
            }`}
          >
            <Sliders className="w-4 h-4" />
            <span>Mode & Allowlist</span>
          </button>
          <button
            onClick={() => setActiveTab('policy')}
            className={`px-4 py-3 text-xs font-bold border-b-2 flex items-center space-x-2 transition-all ${
              activeTab === 'policy'
                ? 'border-cyan-500 text-cyan-400 bg-cyan-950/20'
                : 'border-transparent text-slate-400 hover:text-slate-200'
            }`}
          >
            <FileText className="w-4 h-4" />
            <span>Policy Hierarchy</span>
          </button>
          <button
            onClick={() => setActiveTab('activity')}
            className={`px-4 py-3 text-xs font-bold border-b-2 flex items-center space-x-2 transition-all ${
              activeTab === 'activity'
                ? 'border-cyan-500 text-cyan-400 bg-cyan-950/20'
                : 'border-transparent text-slate-400 hover:text-slate-200'
            }`}
          >
            <ActivityIcon className="w-4 h-4" />
            <span>Network Activity ({decisions.length})</span>
          </button>
          <button
            onClick={() => setActiveTab('dry_run')}
            className={`px-4 py-3 text-xs font-bold border-b-2 flex items-center space-x-2 transition-all ${
              activeTab === 'dry_run'
                ? 'border-cyan-500 text-cyan-400 bg-cyan-950/20'
                : 'border-transparent text-slate-400 hover:text-slate-200'
            }`}
          >
            <Eye className="w-4 h-4" />
            <span>Dry-Run Inspector</span>
          </button>
        </div>

        {/* Tab Content Body */}
        <div className="flex-1 p-6 overflow-y-auto space-y-6">

          {/* TAB 1: SETTINGS & MODE */}
          {activeTab === 'settings' && (
            <div className="space-y-6">
              {/* Security Status Cards */}
              <div className="grid grid-cols-4 gap-3">
                <div className="p-4 rounded-2xl bg-slate-800/50 border border-slate-700/60">
                  <div className="text-[10px] font-bold text-slate-400 uppercase tracking-wider mb-1">NETWORK MODE</div>
                  <div className="text-sm font-extrabold font-mono text-cyan-400 uppercase">{status?.mode}</div>
                </div>
                <div className="p-4 rounded-2xl bg-slate-800/50 border border-slate-700/60">
                  <div className="text-[10px] font-bold text-slate-400 uppercase tracking-wider mb-1">FAIL-CLOSED GUARANTEE</div>
                  <div className="text-sm font-extrabold font-mono text-emerald-400">ACTIVE</div>
                </div>
                <div className="p-4 rounded-2xl bg-slate-800/50 border border-slate-700/60">
                  <div className="text-[10px] font-bold text-slate-400 uppercase tracking-wider mb-1">ALLOWLIST DOMAINS</div>
                  <div className="text-sm font-extrabold font-mono text-white">{status?.allowlist_count}</div>
                </div>
                <div className="p-4 rounded-2xl bg-slate-800/50 border border-slate-700/60">
                  <div className="text-[10px] font-bold text-slate-400 uppercase tracking-wider mb-1">TOTAL DECISIONS</div>
                  <div className="text-sm font-extrabold font-mono text-white">{status?.total_decisions_count}</div>
                </div>
              </div>

              {/* Mode Selection Cards */}
              <div>
                <label className="block text-xs font-bold uppercase tracking-wider text-slate-400 mb-3">
                  Enforcement Mode
                </label>
                <div className="grid grid-cols-2 gap-3">
                  <div
                    onClick={() => handleModeChange('local_only')}
                    className={`p-4 rounded-2xl border cursor-pointer transition-all ${
                      status?.mode === 'local_only'
                        ? 'bg-cyan-950/40 border-cyan-500 shadow-md ring-1 ring-cyan-500'
                        : 'bg-slate-800/40 border-slate-700/70 hover:bg-slate-800/80'
                    }`}
                  >
                    <div className="flex items-center justify-between mb-1">
                      <span className="font-bold text-sm text-white">Local Only (Private by Default)</span>
                      {status?.mode === 'local_only' && <Check className="w-4 h-4 text-cyan-400" />}
                    </div>
                    <p className="text-xs text-slate-400 leading-relaxed">
                      Blocks all external network traffic. Only local loopback endpoints (e.g. Ollama localhost) are permitted.
                    </p>
                  </div>

                  <div
                    onClick={() => handleModeChange('restricted')}
                    className={`p-4 rounded-2xl border cursor-pointer transition-all ${
                      status?.mode === 'restricted'
                        ? 'bg-cyan-950/40 border-cyan-500 shadow-md ring-1 ring-cyan-500'
                        : 'bg-slate-800/40 border-slate-700/70 hover:bg-slate-800/80'
                    }`}
                  >
                    <div className="flex items-center justify-between mb-1">
                      <span className="font-bold text-sm text-white">Restricted Allowlist</span>
                      {status?.mode === 'restricted' && <Check className="w-4 h-4 text-cyan-400" />}
                    </div>
                    <p className="text-xs text-slate-400 leading-relaxed">
                      Strict enforcement allowing outbound calls ONLY to explicitly approved domains in the allowlist.
                    </p>
                  </div>

                  <div
                    onClick={() => handleModeChange('online_allowed')}
                    className={`p-4 rounded-2xl border cursor-pointer transition-all ${
                      status?.mode === 'online_allowed'
                        ? 'bg-cyan-950/40 border-cyan-500 shadow-md ring-1 ring-cyan-500'
                        : 'bg-slate-800/40 border-slate-700/70 hover:bg-slate-800/80'
                    }`}
                  >
                    <div className="flex items-center justify-between mb-1">
                      <span className="font-bold text-sm text-white">Online Allowed (Policy Mediated)</span>
                      {status?.mode === 'online_allowed' && <Check className="w-4 h-4 text-cyan-400" />}
                    </div>
                    <p className="text-xs text-slate-400 leading-relaxed">
                      Outbound traffic enabled subject to SSRF checks, DNS validation, and policy rules. Unknown destinations blocked.
                    </p>
                  </div>

                  <div
                    onClick={() => handleModeChange('disabled')}
                    className={`p-4 rounded-2xl border cursor-pointer transition-all ${
                      status?.mode === 'disabled'
                        ? 'bg-rose-950/40 border-rose-500 shadow-md ring-1 ring-rose-500'
                        : 'bg-slate-800/40 border-slate-700/70 hover:bg-slate-800/80'
                    }`}
                  >
                    <div className="flex items-center justify-between mb-1">
                      <span className="font-bold text-sm text-rose-300">Disabled (Kill Switch)</span>
                      {status?.mode === 'disabled' && <Check className="w-4 h-4 text-rose-400" />}
                    </div>
                    <p className="text-xs text-slate-400 leading-relaxed">
                      Emergency network kill switch. Complete network isolation; all network calls fail closed.
                    </p>
                  </div>
                </div>
              </div>

              {/* Allowlist Manager */}
              <div className="p-5 rounded-2xl bg-slate-950/40 border border-slate-800 space-y-4">
                <div className="flex items-center justify-between">
                  <div>
                    <h3 className="text-sm font-bold text-white">Approved Domain Allowlist</h3>
                    <p className="text-xs text-slate-400">Explicit host/domain patterns permitted for outbound traffic</p>
                  </div>
                </div>

                {/* Add Entry Form */}
                <form onSubmit={handleAddAllowlist} className="flex gap-2">
                  <input
                    type="text"
                    value={newDomain}
                    onChange={(e) => setNewDomain(e.target.value)}
                    placeholder="Domain pattern (e.g. *.example.com)"
                    className="flex-1 px-3.5 py-2 rounded-xl text-xs font-mono bg-slate-900 border border-slate-700/80 text-slate-100 focus:outline-none focus:border-cyan-500"
                  />
                  <input
                    type="text"
                    value={newDomainDesc}
                    onChange={(e) => setNewDomainDesc(e.target.value)}
                    placeholder="Description (optional)"
                    className="w-48 px-3.5 py-2 rounded-xl text-xs bg-slate-900 border border-slate-700/80 text-slate-100 focus:outline-none focus:border-cyan-500"
                  />
                  <button
                    type="submit"
                    className="px-4 py-2 bg-cyan-600 hover:bg-cyan-500 text-white rounded-xl font-bold text-xs flex items-center space-x-1 transition-all"
                  >
                    <Plus className="w-4 h-4" />
                    <span>Add</span>
                  </button>
                </form>

                {/* Allowlist Table */}
                <div className="space-y-2 max-h-48 overflow-y-auto">
                  {allowlist.map((entry) => (
                    <div key={entry.id} className="p-3 rounded-xl bg-slate-900/80 border border-slate-800 flex items-center justify-between">
                      <div>
                        <span className="font-mono text-xs font-bold text-cyan-300">{entry.domain_pattern}</span>
                        <p className="text-[11px] text-slate-400">{entry.description}</p>
                      </div>
                      <button
                        onClick={() => handleDeleteAllowlist(entry.id)}
                        title="Remove entry"
                        className="p-1 text-slate-400 hover:text-rose-400 rounded-lg hover:bg-slate-800 transition-all"
                      >
                        <Trash2 className="w-4 h-4" />
                      </button>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}

          {/* TAB 2: POLICY HIERARCHY */}
          {activeTab === 'policy' && (
            <div className="space-y-6">
              <div className="p-4 rounded-2xl bg-cyan-950/30 border border-cyan-800/50 flex items-start space-x-3">
                <Info className="w-5 h-5 text-cyan-400 flex-shrink-0 mt-0.5" />
                <p className="text-xs text-cyan-200 leading-relaxed">
                  <strong>Policy Hierarchy Priority:</strong> SYSTEM (1) &gt; COMPANY (2) &gt; SECURITY (3) &gt; PRIVACY (4) &gt; PERMISSION (5) &gt; USER (6).
                  A lower-priority rule (e.g. User allow) can NEVER override a higher-priority DENY rule.
                </p>
              </div>

              <div className="space-y-3">
                {rules.map((rule) => (
                  <div key={rule.id} className="p-4 rounded-2xl bg-slate-800/40 border border-slate-700/60 flex items-center justify-between">
                    <div className="space-y-1">
                      <div className="flex items-center space-x-2">
                        <span className="text-xs font-bold text-white">{rule.name}</span>
                        <span className="px-2 py-0.5 rounded text-[10px] font-mono font-bold bg-slate-900 border border-slate-700 text-cyan-300 uppercase">
                          {rule.source}
                        </span>
                        <span className={`px-2 py-0.5 rounded text-[10px] font-bold uppercase font-mono ${
                          rule.action === 'allow' ? 'bg-emerald-950 text-emerald-300 border border-emerald-800' : 'bg-rose-950 text-rose-300 border border-rose-800'
                        }`}>
                          {rule.action}
                        </span>
                      </div>
                      <p className="text-xs text-slate-400">{rule.description}</p>
                      <div className="flex items-center space-x-3 text-[11px] font-mono text-slate-400 pt-1">
                        <span>Pattern: <strong className="text-slate-200">{rule.domain_pattern}</strong></span>
                        {rule.capability && <span>Capability: <strong className="text-slate-200">{rule.capability}</strong></span>}
                        {rule.port && <span>Port: <strong className="text-slate-200">{rule.port}</strong></span>}
                      </div>
                    </div>
                    <div className="text-right font-mono text-xs text-slate-400">
                      Priority: <strong className="text-white">{rule.priority}</strong>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* TAB 3: NETWORK ACTIVITY LOG */}
          {activeTab === 'activity' && (
            <div className="space-y-4">
              <div className="flex justify-between items-center text-xs text-slate-400">
                <span>Showing recent network security enforcement logs</span>
                <span>Total: {decisions.length}</span>
              </div>

              <div className="border border-slate-800 rounded-2xl overflow-hidden">
                <table className="w-full text-left text-xs">
                  <thead className="bg-slate-950 text-slate-400 font-bold uppercase text-[10px] tracking-wider border-b border-slate-800">
                    <tr>
                      <th className="p-3">Time</th>
                      <th className="p-3">Destination</th>
                      <th className="p-3">Disposition</th>
                      <th className="p-3">Policy Source</th>
                      <th className="p-3">Reason</th>
                      <th className="p-3">Inspect</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-slate-800/60 font-mono">
                    {decisions.length === 0 ? (
                      <tr>
                        <td colSpan={6} className="p-6 text-center text-slate-500 italic">
                          No network activity recorded yet.
                        </td>
                      </tr>
                    ) : (
                      decisions.map((dec) => (
                        <tr key={dec.decision_id} className="hover:bg-slate-800/40 transition-colors">
                          <td className="p-3 text-slate-400 whitespace-nowrap">
                            {new Date(dec.timestamp * 1000).toLocaleTimeString()}
                          </td>
                          <td className="p-3 font-bold text-white truncate max-w-[200px]">
                            {dec.endpoint.raw_url || `${dec.endpoint.protocol}://${dec.endpoint.host}:${dec.endpoint.port}`}
                          </td>
                          <td className="p-3">
                            <span className={`px-2 py-0.5 rounded text-[10px] font-bold uppercase ${
                              dec.disposition === 'allow'
                                ? 'bg-emerald-950 text-emerald-300 border border-emerald-800'
                                : 'bg-rose-950 text-rose-300 border border-rose-800'
                            }`}>
                              {dec.disposition}
                            </span>
                          </td>
                          <td className="p-3 text-slate-300">{dec.policy_source}</td>
                          <td className="p-3 text-slate-400 font-sans text-xs truncate max-w-[250px]">{dec.reason}</td>
                          <td className="p-3">
                            <button
                              onClick={() => setSelectedDecision(dec)}
                              className="p-1 text-cyan-400 hover:text-cyan-300 hover:bg-cyan-950/60 rounded transition-all"
                            >
                              <ChevronRight className="w-4 h-4" />
                            </button>
                          </td>
                        </tr>
                      ))
                    )}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {/* TAB 4: DRY-RUN INSPECTOR */}
          {activeTab === 'dry_run' && (
            <div className="space-y-6">
              <div className="p-5 rounded-2xl bg-slate-950/40 border border-slate-800 space-y-4">
                <h3 className="text-sm font-bold text-white">Safe Policy Dry-Run Evaluation</h3>
                <p className="text-xs text-slate-400">Test an outbound destination against current network security policies without executing the actual network request.</p>

                <div className="space-y-3">
                  <div>
                    <label className="block text-xs font-bold text-slate-400 mb-1">Target Destination URL</label>
                    <input
                      type="text"
                      value={testUrl}
                      onChange={(e) => setTestUrl(e.target.value)}
                      className="w-full px-4 py-2.5 rounded-xl bg-slate-900 border border-slate-700 text-xs font-mono text-white focus:outline-none focus:border-cyan-500"
                    />
                  </div>

                  <div>
                    <label className="block text-xs font-bold text-slate-400 mb-1">Capability</label>
                    <select
                      value={testCapability}
                      onChange={(e) => setTestCapability(e.target.value)}
                      className="w-full px-4 py-2.5 rounded-xl bg-slate-900 border border-slate-700 text-xs font-mono text-white focus:outline-none focus:border-cyan-500"
                    >
                      <option value="cloud_model_inference">cloud_model_inference</option>
                      <option value="external_https">external_https</option>
                      <option value="web_fetch">web_fetch</option>
                      <option value="web_search">web_search</option>
                      <option value="remote_mcp">remote_mcp</option>
                      <option value="loopback">loopback</option>
                    </select>
                  </div>

                  <button
                    onClick={handleDryRunTest}
                    className="w-full py-3 bg-cyan-600 hover:bg-cyan-500 text-white rounded-xl font-bold text-xs shadow-sm transition-all"
                  >
                    Evaluate Policy Dry-Run
                  </button>
                </div>
              </div>

              {testResult && (
                <div className="p-5 rounded-2xl bg-slate-950 border border-slate-800 space-y-3 font-mono text-xs">
                  <div className="flex items-center justify-between border-b border-slate-800 pb-2">
                    <span className="font-bold text-slate-300">Dry-Run Decision Result:</span>
                    <span className={`px-2.5 py-1 rounded text-xs font-bold uppercase ${
                      testResult.disposition === 'allow'
                        ? 'bg-emerald-950 text-emerald-300 border border-emerald-800'
                        : 'bg-rose-950 text-rose-300 border border-rose-800'
                    }`}>
                      {testResult.disposition}
                    </span>
                  </div>

                  <div><span className="text-slate-400">Reason:</span> <span className="text-white font-sans">{testResult.reason}</span></div>
                  <div><span className="text-slate-400">Policy Source:</span> <span className="text-cyan-400">{testResult.policy_source}</span></div>
                  <div><span className="text-slate-400">Matched Rule:</span> <span className="text-slate-200">{testResult.matched_rule || 'None (Fail-Closed Default)'}</span></div>
                  <div><span className="text-slate-400">Endpoint:</span> <span className="text-slate-300">{testResult.endpoint.protocol}://{testResult.endpoint.host}:{testResult.endpoint.port}</span></div>
                </div>
              )}
            </div>
          )}
        </div>

        {/* DECISION INSPECTOR MODAL */}
        {selectedDecision && (
          <div className="fixed inset-0 z-60 flex items-center justify-center bg-slate-950/70 backdrop-blur-md p-4">
            <div className="w-[500px] bg-slate-900 border border-slate-700 rounded-3xl p-6 text-slate-100 space-y-4 shadow-2xl">
              <div className="flex justify-between items-start">
                <div>
                  <h3 className="text-base font-bold text-white">Network Decision Inspector</h3>
                  <p className="text-xs text-slate-400 font-mono">{selectedDecision.decision_id}</p>
                </div>
                <button onClick={() => setSelectedDecision(null)} className="p-1 text-slate-400 hover:text-white">
                  <X className="w-5 h-5" />
                </button>
              </div>

              <div className="space-y-3 font-mono text-xs bg-slate-950 p-4 rounded-2xl border border-slate-800">
                <div>
                  <span className="text-slate-400">Disposition:</span>{' '}
                  <strong className={selectedDecision.disposition === 'allow' ? 'text-emerald-400' : 'text-rose-400'}>
                    {selectedDecision.disposition.toUpperCase()}
                  </strong>
                </div>
                <div><span className="text-slate-400">Destination:</span> <span className="text-white">{selectedDecision.endpoint.raw_url || selectedDecision.endpoint.host}</span></div>
                <div><span className="text-slate-400">Policy Source:</span> <span className="text-cyan-400">{selectedDecision.policy_source}</span></div>
                <div><span className="text-slate-400">Matched Rule:</span> <span className="text-slate-200">{selectedDecision.matched_rule || 'Default Fail-Closed'}</span></div>
                <div><span className="text-slate-400 font-sans">Reason:</span> <p className="text-slate-200 font-sans mt-1">{selectedDecision.reason}</p></div>
              </div>
            </div>
          </div>
        )}

      </div>
    </div>
  );
}
