'use client';

import React from 'react';
import { CompatibilityStatus, ExecutionMode } from '@/lib/backend/types';

interface ModelCompatibilityBadgeProps {
  status?: CompatibilityStatus | string;
  executionMode?: ExecutionMode;
  reason?: string;
  className?: string;
}

export function ModelCompatibilityBadge({
  status,
  executionMode,
  reason,
  className = '',
}: ModelCompatibilityBadgeProps) {
  if (executionMode === 'cloud') {
    return (
      <span
        title={reason || 'Cloud model inference (requires privacy authorization)'}
        className={`inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-medium bg-purple-500/10 text-purple-400 border border-purple-500/20 ${className}`}
      >
        <span className="w-1.5 h-1.5 rounded-full bg-purple-400 animate-pulse" />
        Cloud Model
      </span>
    );
  }

  switch (status) {
    case 'compatible':
      return (
        <span
          title={reason || 'Hardware fully compatible'}
          className={`inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 ${className}`}
        >
          <span className="w-1.5 h-1.5 rounded-full bg-emerald-400" />
          Compatible
        </span>
      );
    case 'compatible_with_warnings':
      return (
        <span
          title={reason || 'Compatible with performance warnings'}
          className={`inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-medium bg-amber-500/10 text-amber-400 border border-amber-500/20 ${className}`}
        >
          <span className="w-1.5 h-1.5 rounded-full bg-amber-400" />
          Warnings
        </span>
      );
    case 'incompatible':
      return (
        <span
          title={reason || 'Incompatible hardware'}
          className={`inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-medium bg-rose-500/10 text-rose-400 border border-rose-500/20 ${className}`}
        >
          <span className="w-1.5 h-1.5 rounded-full bg-rose-400" />
          Incompatible
        </span>
      );
    default:
      return (
        <span
          title={reason || 'Unknown hardware status'}
          className={`inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-medium bg-zinc-500/10 text-zinc-400 border border-zinc-500/20 ${className}`}
        >
          <span className="w-1.5 h-1.5 rounded-full bg-zinc-400" />
          Unknown
        </span>
      );
  }
}
