'use client';

import React, { useEffect, useState } from 'react';
import { ToolDescriptor, ToolDecision } from '@/lib/backend/types';
import { backendClient } from '@/lib/backend/client';
import { ToolSecurityBadge } from '@/components/ToolSecurityBadge';
import { ToolPermissionInspector } from '@/components/ToolPermissionInspector';
import Link from 'next/link';

export default function ToolSettingsPage() {
  const [tools, setTools] = useState<ToolDescriptor[]>([]);
  const [selectedTool, setSelectedTool] = useState<ToolDescriptor | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [evaluating, setEvaluating] = useState(false);
  const [lastDecision, setLastDecision] = useState<ToolDecision | null>(null);

  const fetchTools = async () => {
    try {
      setLoading(true);
      const res = await backendClient.getTools();
      if (res.success) {
        setTools(res.tools);
        if (res.tools.length > 0 && !selectedTool) {
          setSelectedTool(res.tools[0]);
        }
      }
    } catch (e: any) {
      setError(e.message || 'Failed loading tools');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchTools();
  }, []);

  const handleToggleEnable = async (tool: ToolDescriptor) => {
    try {
      if (tool.enabled) {
        await backendClient.disableTool(tool.id);
      } else {
        await backendClient.enableTool(tool.id);
      }
      fetchTools();
    } catch (e: any) {
      alert(`Action failed: ${e.message}`);
    }
  };

  const handleTestEvaluation = async (tool: ToolDescriptor) => {
    try {
      setEvaluating(true);
      const res = await backendClient.evaluateTool({
        tool_id: tool.id,
        arguments: {},
        requested_capabilities: tool.capabilities,
      });
      if (res.success) {
        setLastDecision(res.decision);
      }
    } catch (e: any) {
      alert(`Evaluation failed: ${e.message}`);
    } finally {
      setEvaluating(false);
    }
  };

  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-100 p-8 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-zinc-800 pb-5">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-white">Tool Runtime & Security Governance</h1>
          <p className="text-sm text-zinc-400 mt-1">
            Centralized capability enforcement, permission management, and isolated tool runtime.
          </p>
        </div>
        <div className="flex items-center space-x-3">
          <Link
            href="/settings/mcp"
            className="px-3 py-1.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-200 rounded text-xs font-medium transition"
          >
            MCP Connections
          </Link>
          <Link
            href="/settings/tools/activity"
            className="px-3 py-1.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-200 rounded text-xs font-medium transition"
          >
            Audit Activity Log
          </Link>
        </div>
      </div>

      {loading ? (
        <div className="text-center py-12 text-zinc-500 text-sm">Loading tool registry...</div>
      ) : error ? (
        <div className="p-4 bg-red-950/40 border border-red-800/50 rounded text-red-300 text-sm">{error}</div>
      ) : (
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
          {/* Tool List */}
          <div className="lg:col-span-7 bg-zinc-900 border border-zinc-800 rounded-lg overflow-hidden">
            <div className="px-5 py-4 border-b border-zinc-800 bg-zinc-950/50 flex items-center justify-between">
              <h3 className="text-sm font-semibold text-zinc-200">Registered Tools ({tools.length})</h3>
              <span className="text-xs text-zinc-500 font-mono">Fail-Closed Enforcement</span>
            </div>

            <div className="divide-y divide-zinc-800/60 max-h-[600px] overflow-y-auto">
              {tools.map((t) => {
                const isSelected = selectedTool?.id === t.id;
                return (
                  <div
                    key={t.id}
                    onClick={() => {
                      setSelectedTool(t);
                      setLastDecision(null);
                    }}
                    className={`p-4 transition cursor-pointer flex items-center justify-between ${
                      isSelected ? 'bg-zinc-800/60 border-l-2 border-emerald-500' : 'hover:bg-zinc-800/30'
                    }`}
                  >
                    <div className="space-y-1 pr-4">
                      <div className="flex items-center space-x-2">
                        <span className="font-semibold text-sm text-zinc-100">{t.name}</span>
                        <span className="text-xs font-mono text-zinc-500">({t.id})</span>
                      </div>
                      <p className="text-xs text-zinc-400 line-clamp-1">{t.description}</p>
                      <div className="flex items-center space-x-2 pt-1">
                        <ToolSecurityBadge status={t.enabled ? 'ALLOWED' : 'BLOCKED'} riskLevel={t.risk_level} />
                        <span className="text-[10px] font-mono text-zinc-500 px-1.5 py-0.5 bg-zinc-950 rounded border border-zinc-800">
                          {t.source}
                        </span>
                      </div>
                    </div>

                    <div className="flex items-center space-x-2">
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleTestEvaluation(t);
                        }}
                        disabled={evaluating}
                        className="px-2.5 py-1 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 rounded text-xs transition"
                      >
                        Evaluate
                      </button>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleToggleEnable(t);
                        }}
                        className={`px-3 py-1 rounded text-xs font-medium transition ${
                          t.enabled
                            ? 'bg-rose-950/60 hover:bg-rose-900 border border-rose-800/50 text-rose-300'
                            : 'bg-emerald-950/60 hover:bg-emerald-900 border border-emerald-800/50 text-emerald-300'
                        }`}
                      >
                        {t.enabled ? 'Disable' : 'Enable'}
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>

          {/* Inspector Panel */}
          <div className="lg:col-span-5">
            <ToolPermissionInspector tool={selectedTool} decision={lastDecision} />
          </div>
        </div>
      )}
    </div>
  );
}
