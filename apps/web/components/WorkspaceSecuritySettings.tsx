'use client';

import React, { useState, useEffect } from 'react';
import { Shield, Lock, FileText, Download, EyeOff, AlertOctagon, Save, RefreshCw } from 'lucide-react';
import { WorkspaceSecurityPolicy, WorkspaceSecurityStatus } from '@/lib/backend/types';
import { backendClient } from '@/lib/backend/client';
import { ProtectedFilesList } from './ProtectedFilesList';
import { FilesystemSecurityBadge } from './FilesystemSecurityBadge';

interface Props {
  workspaceId?: string;
}

export const WorkspaceSecuritySettings: React.FC<Props> = ({ workspaceId = 'default' }) => {
  const [status, setStatus] = useState<WorkspaceSecurityStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [readOnly, setReadOnly] = useState(false);
  const [allowExport, setAllowExport] = useState(false);
  const [blockHidden, setBlockHidden] = useState(true);
  const [blockBinary, setBlockBinary] = useState(true);
  const [requireDeleteConfirm, setRequireDeleteConfirm] = useState(true);
  const [notice, setNotice] = useState<string | null>(null);

  useEffect(() => {
    fetchSecurityStatus();
  }, [workspaceId]);

  const fetchSecurityStatus = async () => {
    setLoading(true);
    try {
      const res = await backendClient.getWorkspaceSecurity(workspaceId);
      if (res.success && res.status) {
        setStatus(res.status);
        const p: WorkspaceSecurityPolicy = res.status.policy;
        setReadOnly(p.read_only);
        setAllowExport(p.allow_external_export);
        setBlockHidden(p.block_hidden_files);
        setBlockBinary(p.limits.block_binary_files);
        setRequireDeleteConfirm(p.require_confirmation_for_delete);
      }
    } catch (e) {
      console.error('Failed to fetch workspace security status', e);
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    setSaving(true);
    setNotice(null);
    try {
      const updateData = {
        read_only: readOnly,
        allow_external_export: allowExport,
        block_hidden_files: blockHidden,
        require_confirmation_for_delete: requireDeleteConfirm,
        limits: {
          block_binary_files: blockBinary,
        },
      };

      const res = await backendClient.updateWorkspacePermissions(workspaceId, updateData);
      if (res.success) {
        setNotice('Workspace security permissions updated successfully.');
        await fetchSecurityStatus();
      }
    } catch (e) {
      setNotice('Failed to update workspace security permissions.');
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="space-y-6 max-w-4xl mx-auto p-6 bg-slate-950 text-slate-100 rounded-xl border border-slate-800">
      <div className="flex items-center justify-between border-b border-slate-800 pb-4">
        <div>
          <h2 className="text-lg font-bold flex items-center gap-2 text-slate-100">
            <Shield className="w-5 h-5 text-emerald-400" />
            Workspace Filesystem Security & Boundary
          </h2>
          <p className="text-xs text-slate-400 mt-1">
            Enforces strict local sandbox boundaries, traversal prevention, and protected file policies.
          </p>
        </div>
        <FilesystemSecurityBadge status={status} readOnly={readOnly} />
      </div>

      {notice && (
        <div className="p-3 rounded-lg bg-emerald-950/40 border border-emerald-800/50 text-emerald-300 text-xs flex items-center justify-between">
          <span>{notice}</span>
          <button onClick={() => setNotice(null)} className="text-emerald-400 hover:text-emerald-200">Dismiss</button>
        </div>
      )}

      {loading ? (
        <div className="p-8 text-center text-xs text-slate-500 flex items-center justify-center gap-2">
          <RefreshCw className="w-4 h-4 animate-spin text-emerald-400" /> Loading security policies...
        </div>
      ) : (
        <div className="space-y-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="p-4 rounded-lg border border-slate-800 bg-slate-900/60 space-y-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2 text-sm font-semibold">
                  <Lock className="w-4 h-4 text-amber-400" />
                  Read-Only Workspace Mode
                </div>
                <input
                  type="checkbox"
                  checked={readOnly}
                  onChange={(e) => setReadOnly(e.target.checked)}
                  className="w-4 h-4 accent-amber-500 rounded cursor-pointer"
                />
              </div>
              <p className="text-xs text-slate-400 leading-relaxed">
                Blocks all write, create, rename, and delete operations inside the workspace root.
              </p>
            </div>

            <div className="p-4 rounded-lg border border-slate-800 bg-slate-900/60 space-y-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2 text-sm font-semibold">
                  <Download className="w-4 h-4 text-blue-400" />
                  Allow External File Export
                </div>
                <input
                  type="checkbox"
                  checked={allowExport}
                  onChange={(e) => setAllowExport(e.target.checked)}
                  className="w-4 h-4 accent-blue-500 rounded cursor-pointer"
                />
              </div>
              <p className="text-xs text-slate-400 leading-relaxed">
                Permits copying files outside the workspace boundary only when explicitly authorized.
              </p>
            </div>

            <div className="p-4 rounded-lg border border-slate-800 bg-slate-900/60 space-y-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2 text-sm font-semibold">
                  <EyeOff className="w-4 h-4 text-purple-400" />
                  Block Hidden Files & Dotfiles
                </div>
                <input
                  type="checkbox"
                  checked={blockHidden}
                  onChange={(e) => setBlockHidden(e.target.checked)}
                  className="w-4 h-4 accent-purple-500 rounded cursor-pointer"
                />
              </div>
              <p className="text-xs text-slate-400 leading-relaxed">
                Prevents agents from reading dotfiles like <code className="text-slate-200">.env</code> or <code className="text-slate-200">.git</code>.
              </p>
            </div>

            <div className="p-4 rounded-lg border border-slate-800 bg-slate-900/60 space-y-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2 text-sm font-semibold">
                  <AlertOctagon className="w-4 h-4 text-rose-400" />
                  Require Confirmation for Deletion
                </div>
                <input
                  type="checkbox"
                  checked={requireDeleteConfirm}
                  onChange={(e) => setRequireDeleteConfirm(e.target.checked)}
                  className="w-4 h-4 accent-rose-500 rounded cursor-pointer"
                />
              </div>
              <p className="text-xs text-slate-400 leading-relaxed">
                Forces explicit user approval modal before executing file or directory deletions.
              </p>
            </div>
          </div>

          <ProtectedFilesList protectedPaths={status?.policy?.custom_protected_paths} />

          <div className="flex justify-end pt-4 border-t border-slate-800">
            <button
              onClick={handleSave}
              disabled={saving}
              className="px-5 py-2 rounded-lg bg-emerald-600 text-white hover:bg-emerald-500 disabled:opacity-50 text-xs font-semibold shadow-lg shadow-emerald-600/20 transition-colors flex items-center gap-2"
            >
              {saving ? <RefreshCw className="w-3.5 h-3.5 animate-spin" /> : <Save className="w-3.5 h-3.5" />}
              Save Workspace Permissions
            </button>
          </div>
        </div>
      )}
    </div>
  );
};
