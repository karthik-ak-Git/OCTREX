'use client';

import React from 'react';
import { HardwareProfile } from '@/lib/backend/types';

interface HardwareCompatibilityCardProps {
  profile: HardwareProfile | null;
  className?: string;
}

export function HardwareCompatibilityCard({
  profile,
  className = '',
}: HardwareCompatibilityCardProps) {
  if (!profile) {
    return (
      <div className={`p-4 rounded-lg border border-zinc-800 bg-zinc-900/50 text-xs text-zinc-500 ${className}`}>
        Hardware profile loading...
      </div>
    );
  }

  const ramMb = Math.round(profile.memory.total_bytes / (1024 * 1024));
  const hasGpu = profile.gpus.length > 0;

  return (
    <div className={`p-4 rounded-lg border border-zinc-800 bg-zinc-900/60 space-y-3 ${className}`}>
      <div className="flex items-center justify-between border-b border-zinc-800/80 pb-2">
        <h4 className="text-xs font-semibold text-zinc-300 uppercase tracking-wider">
          Local Hardware Intelligence Snapshot
        </h4>
        <span className="text-[11px] font-mono px-2 py-0.5 rounded bg-zinc-800 text-zinc-300">
          Confidence: {profile.confidence}
        </span>
      </div>

      <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 text-xs">
        <div>
          <span className="text-zinc-500 block">Operating System</span>
          <span className="font-semibold text-zinc-200">{profile.os} ({profile.arch})</span>
        </div>

        <div>
          <span className="text-zinc-500 block">CPU Cores</span>
          <span className="font-semibold text-zinc-200">{profile.cpu.logical_cores} Cores</span>
        </div>

        <div>
          <span className="text-zinc-500 block">System RAM</span>
          <span className="font-semibold text-zinc-200">{ramMb.toLocaleString()} MB</span>
        </div>

        <div>
          <span className="text-zinc-500 block">GPU Accelerators</span>
          <span className={`font-semibold ${hasGpu ? 'text-emerald-400' : 'text-amber-400'}`}>
            {hasGpu ? `${profile.gpus.length} Detected` : 'CPU Only'}
          </span>
        </div>
      </div>

      {hasGpu && (
        <div className="pt-2 border-t border-zinc-800/60 space-y-1">
          {profile.gpus.map((gpu, i) => {
            const vramMb = gpu.vram_bytes ? Math.round(gpu.vram_bytes / (1024 * 1024)) : 'N/A';
            return (
              <div key={i} className="flex items-center justify-between text-[11px] text-zinc-400 font-mono">
                <span>GPU {gpu.device_index}: {gpu.name} ({gpu.vendor})</span>
                <span className="text-zinc-300">{vramMb} MB VRAM</span>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
