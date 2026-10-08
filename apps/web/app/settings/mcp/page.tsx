'use client';

import React, { useEffect, useState } from 'react';
import { McpServerInfo, McpTrustLevel } from '@/lib/backend/types';
import { backendClient } from '@/lib/backend/client';
import { ToolSecurityBadge } from '@/components/ToolSecurityBadge';
import Link from 'next/link';

export default function McpSettingsPage() {
  const [servers, setServers] = useState<McpServerInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // New MCP form state
  const [newServerId, setNewServerId] = useState('');
  const [newName, setNewName] = useState('');
  const [newTransport, setNewTransport] = useState('stdio');
  const [newEndpoint, setNewEndpoint] = useState('');
  const [newTrustLevel, setNewTrustLevel] = useState<McpTrustLevel>('UNTRUSTED');

  const fetchMcpServers = async () => {
    try {
      setLoading(true);
      const res = await backendClient.getMcpConnections();
      if (res.success) {
        setServers(res.servers);
      }
    } catch (e: any) {
      setError(e.message || 'Failed loading MCP servers');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchMcpServers();
  }, []);

  const handleToggleServer = async (srv: McpServerInfo) => {
    try {
      if (srv.enabled) {
        await backendClient.disableMcp(srv.server_id);
      } else {
        await backendClient.enableMcp(srv.server_id);
      }
      fetchMcpServers();
    } catch (e: any) {
      alert(`Action failed: ${e.message}`);
    }
  };

  const handleAddMcpServer = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newServerId || !newName || !newEndpoint) {
      alert('Please fill in server ID, name, and endpoint');
      return;
    }

    try {
      await backendClient.connectMcp({
        server_id: newServerId,
        name: newName,
        transport: newTransport,
        endpoint: newEndpoint,
        trust_level: newTrustLevel,
        enabled: true,
        declared_capabilities: ['EXTERNAL_HTTPS'],
        allowed_tools: [],
        status: 'CONNECTED',
      });
      setNewServerId('');
      setNewName('');
      setNewEndpoint('');
      fetchMcpServers();
    } catch (err: any) {
      alert(`Failed connecting MCP server: ${err.message}`);
    }
  };

  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-100 p-8 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-zinc-800 pb-5">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-white">Model Context Protocol (MCP) Boundary</h1>
          <p className="text-sm text-zinc-400 mt-1">
            Configure external MCP servers with strict trust boundaries and explicit capability rules.
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
            href="/settings/tools/activity"
            className="px-3 py-1.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-200 rounded text-xs font-medium transition"
          >
            Audit Activity Log
          </Link>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
        {/* Connection List */}
        <div className="lg:col-span-7 bg-zinc-900 border border-zinc-800 rounded-lg overflow-hidden">
          <div className="px-5 py-4 border-b border-zinc-800 bg-zinc-950/50 flex items-center justify-between">
            <h3 className="text-sm font-semibold text-zinc-200">Connected MCP Servers ({servers.length})</h3>
            <span className="text-xs text-zinc-500 font-mono">Untrusted By Default</span>
          </div>

          {loading ? (
            <div className="p-8 text-center text-zinc-500 text-sm">Loading connections...</div>
          ) : error ? (
            <div className="p-4 bg-red-950/40 border border-red-800/50 rounded text-red-300 text-sm">{error}</div>
          ) : servers.length === 0 ? (
            <div className="p-8 text-center text-zinc-500 text-sm">
              No MCP servers connected yet. Add one using the form.
            </div>
          ) : (
            <div className="divide-y divide-zinc-800/60">
              {servers.map((srv) => (
                <div key={srv.server_id} className="p-4 flex items-center justify-between hover:bg-zinc-800/30 transition">
                  <div className="space-y-1">
                    <div className="flex items-center space-x-2">
                      <span className="font-semibold text-sm text-zinc-100">{srv.name}</span>
                      <span className="text-xs font-mono text-zinc-500">({srv.server_id})</span>
                    </div>
                    <p className="text-xs text-zinc-400 font-mono">{srv.endpoint}</p>
                    <div className="flex items-center space-x-2 pt-1">
                      <ToolSecurityBadge status={srv.enabled ? 'ALLOWED' : 'BLOCKED'} trustLevel={srv.trust_level} />
                      <span className="text-[10px] font-mono text-zinc-500 px-1.5 py-0.5 bg-zinc-950 rounded border border-zinc-800">
                        {srv.transport}
                      </span>
                    </div>
                  </div>

                  <button
                    onClick={() => handleToggleServer(srv)}
                    className={`px-3 py-1 rounded text-xs font-medium transition ${
                      srv.enabled
                        ? 'bg-rose-950/60 hover:bg-rose-900 border border-rose-800/50 text-rose-300'
                        : 'bg-emerald-950/60 hover:bg-emerald-900 border border-emerald-800/50 text-emerald-300'
                    }`}
                  >
                    {srv.enabled ? 'Disable Server' : 'Enable Server'}
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Add Connection Form */}
        <div className="lg:col-span-5 bg-zinc-900 border border-zinc-800 rounded-lg p-5 space-y-4">
          <h3 className="text-sm font-semibold text-zinc-200 border-b border-zinc-800 pb-3">Register New MCP Server</h3>
          <form onSubmit={handleAddMcpServer} className="space-y-3 text-xs">
            <div>
              <label className="block text-zinc-400 mb-1 font-mono">Server ID</label>
              <input
                type="text"
                placeholder="github-mcp"
                value={newServerId}
                onChange={(e) => setNewServerId(e.target.value)}
                className="w-full px-3 py-2 bg-zinc-950 border border-zinc-800 rounded text-zinc-100 font-mono focus:outline-none focus:border-zinc-700"
              />
            </div>

            <div>
              <label className="block text-zinc-400 mb-1 font-mono">Display Name</label>
              <input
                type="text"
                placeholder="GitHub MCP Server"
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                className="w-full px-3 py-2 bg-zinc-950 border border-zinc-800 rounded text-zinc-100 focus:outline-none focus:border-zinc-700"
              />
            </div>

            <div>
              <label className="block text-zinc-400 mb-1 font-mono">Transport</label>
              <select
                value={newTransport}
                onChange={(e) => setNewTransport(e.target.value)}
                className="w-full px-3 py-2 bg-zinc-950 border border-zinc-800 rounded text-zinc-100 focus:outline-none focus:border-zinc-700"
              >
                <option value="stdio">stdio (Local Process)</option>
                <option value="https">https (Remote Stream)</option>
                <option value="http">http (Local HTTP)</option>
              </select>
            </div>

            <div>
              <label className="block text-zinc-400 mb-1 font-mono">Endpoint / Command</label>
              <input
                type="text"
                placeholder="npx -y @modelcontextprotocol/server-github"
                value={newEndpoint}
                onChange={(e) => setNewEndpoint(e.target.value)}
                className="w-full px-3 py-2 bg-zinc-950 border border-zinc-800 rounded text-zinc-100 font-mono focus:outline-none focus:border-zinc-700"
              />
            </div>

            <div>
              <label className="block text-zinc-400 mb-1 font-mono">Trust Level</label>
              <select
                value={newTrustLevel}
                onChange={(e) => setNewTrustLevel(e.target.value as McpTrustLevel)}
                className="w-full px-3 py-2 bg-zinc-950 border border-zinc-800 rounded text-zinc-100 focus:outline-none focus:border-zinc-700"
              >
                <option value="UNTRUSTED">UNTRUSTED (Strict Capability Policy)</option>
                <option value="RESTRICTED">RESTRICTED (Workspace Scoped Only)</option>
                <option value="TRUSTED">TRUSTED (Full Capability Access)</option>
              </select>
            </div>

            <button
              type="submit"
              className="w-full py-2 bg-emerald-600 hover:bg-emerald-500 text-white font-medium rounded transition"
            >
              Connect MCP Server
            </button>
          </form>
        </div>
      </div>
    </div>
  );
}
