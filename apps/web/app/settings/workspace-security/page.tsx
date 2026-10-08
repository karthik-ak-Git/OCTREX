import React from 'react';
import { WorkspaceSecuritySettings } from '@/components/WorkspaceSecuritySettings';

export default function WorkspaceSecurityPage() {
  return (
    <div className="min-h-screen bg-slate-950 p-6 text-slate-100">
      <div className="max-w-4xl mx-auto space-y-6">
        <div>
          <h1 className="text-2xl font-bold text-slate-100">Workspace Security & Sandbox</h1>
          <p className="text-xs text-slate-400 mt-1">
            Configure local data access boundaries, filesystem policies, traversal protection, and file protections for Octrex.
          </p>
        </div>

        <WorkspaceSecuritySettings workspaceId="default" />
      </div>
    </div>
  );
}
