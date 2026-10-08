'use client';

import React, { useEffect, useState } from 'react';
import { ToolExecutionActivity } from '@/lib/backend/types';
import { backendClient } from '@/lib/backend/client';
import { ToolSecurityBadge } from '@/components/ToolSecurityBadge';
import Link from 'next/link';

export default function ToolActivityLogPage() {
  const [activities, setActivities] = useState<ToolExecutionActivity[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchActivity = async () => {
    try {
      setLoading(true);
      const res = await backendClient.getToolActivity();
      if (res.success) {
        setActivities(res.activity);
      }
    } catch (e: any) {
      setError(e.message || 'Failed loading tool activity');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchActivity();
  }, []);

  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-100 p-8 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-zinc-800 pb-5">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-white">Tool Execution Audit Log</h1>
          <p className="text-sm text-zinc-400 mt-1">
            Immutable audit trail of all tool execution requests, capability decisions, and security enforcements.
          </p>
        </div>
        <div className="flex items-center space-x-3">
          <Link
            href="/settings/tools"
            className="px-3 py-1.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-200 rounded text-xs font-medium transition"
          >
            Tool Settings
          </Link>
          <Link
            href="/settings/mcp"
            className="px-3 py-1.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-200 rounded text-xs font-medium transition"
          >
            MCP Connections
          </Link>
        </div>
      </div>

      {loading ? (
        <div className="text-center py-12 text-zinc-500 text-sm">Loading activity logs...</div>
      ) : error ? (
        <div className="p-4 bg-red-950/40 border border-red-800/50 rounded text-red-300 text-sm">{error}</div>
      ) : activities.length === 0 ? (
        <div className="bg-zinc-900 border border-zinc-800 rounded-lg p-12 text-center text-zinc-500 text-sm">
          No tool executions logged yet. Execute tools from the workbench to generate audit events.
        </div>
      ) : (
        <div className="bg-zinc-900 border border-zinc-800 rounded-lg overflow-hidden">
          <div className="px-5 py-4 border-b border-zinc-800 bg-zinc-950/50 flex items-center justify-between">
            <h3 className="text-sm font-semibold text-zinc-200">Execution Events ({activities.length})</h3>
            <span className="text-xs text-zinc-500 font-mono">SQLite Audit Trail</span>
          </div>

          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs">
              <thead className="bg-zinc-950/80 text-zinc-400 font-mono uppercase border-b border-zinc-800">
                <tr>
                  <th className="px-4 py-3">Timestamp</th>
                  <th className="px-4 py-3">Tool</th>
                  <th className="px-4 py-3">Actor</th>
                  <th className="px-4 py-3">Policy Source</th>
                  <th className="px-4 py-3">Decision</th>
                  <th className="px-4 py-3">Reason / Details</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-zinc-800/60 font-mono">
                {activities.map((act) => (
                  <tr key={act.id} className="hover:bg-zinc-800/30 transition">
                    <td className="px-4 py-3 text-zinc-400">
                      {new Date(act.timestamp * 1000).toLocaleString()}
                    </td>
                    <td className="px-4 py-3 text-emerald-400 font-semibold">{act.tool || 'N/A'}</td>
                    <td className="px-4 py-3 text-zinc-300">{act.actor}</td>
                    <td className="px-4 py-3 text-zinc-400">{act.policy_source || 'system'}</td>
                    <td className="px-4 py-3">
                      <ToolSecurityBadge status={act.success ? 'ALLOWED' : 'BLOCKED'} />
                    </td>
                    <td className="px-4 py-3 text-zinc-300 max-w-md truncate">
                      {act.reason || 'Completed successfully'}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
}
