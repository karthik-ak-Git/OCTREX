import React from 'react';
import { RiskLevel, ToolDecisionState, McpTrustLevel } from '@/lib/backend/types';

interface ToolSecurityBadgeProps {
  status?: ToolDecisionState | 'ALLOWED' | 'BLOCKED' | 'RESTRICTED' | 'UNTRUSTED_MCP';
  riskLevel?: RiskLevel;
  trustLevel?: McpTrustLevel;
  label?: string;
}

export const ToolSecurityBadge: React.FC<ToolSecurityBadgeProps> = ({
  status,
  riskLevel,
  trustLevel,
  label,
}) => {
  let badgeStyle = 'bg-gray-100 text-gray-800 border-gray-300';
  let badgeText = label || status || 'UNKNOWN';

  if (status === 'ALLOW' || status === 'ALLOWED') {
    badgeStyle = 'bg-emerald-900/40 text-emerald-300 border-emerald-500/50';
    badgeText = label || 'TOOL ALLOWED';
  } else if (status === 'BLOCK' || status === 'BLOCKED') {
    badgeStyle = 'bg-rose-900/40 text-rose-300 border-rose-500/50';
    badgeText = label || 'TOOL BLOCKED';
  } else if (status === 'REQUIRE_CONSENT') {
    badgeStyle = 'bg-amber-900/40 text-amber-300 border-amber-500/50';
    badgeText = label || 'CONSENT REQUIRED';
  } else if (status === 'UNTRUSTED_MCP' || trustLevel === 'UNTRUSTED') {
    badgeStyle = 'bg-purple-900/40 text-purple-300 border-purple-500/50';
    badgeText = label || 'UNTRUSTED MCP';
  }

  if (riskLevel) {
    if (riskLevel === 'CRITICAL') {
      badgeStyle = 'bg-red-950 text-red-200 border-red-600 font-bold';
    } else if (riskLevel === 'HIGH') {
      badgeStyle = 'bg-orange-950 text-orange-200 border-orange-600';
    }
  }

  return (
    <span
      className={`inline-flex items-center px-2.5 py-0.5 rounded text-xs font-medium border ${badgeStyle}`}
    >
      <span className="w-1.5 h-1.5 mr-1.5 rounded-full bg-current opacity-75" />
      {badgeText}
    </span>
  );
};
