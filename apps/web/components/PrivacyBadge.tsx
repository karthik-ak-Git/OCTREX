'use client';

import React from 'react';
import { Shield, Lock, Globe, AlertTriangle, EyeOff } from 'lucide-react';

interface PrivacyBadgeProps {
  mode?: string;
  privacyMode?: string;
  classification?: string;
  compact?: boolean;
  onClick?: () => void;
}

export function PrivacyBadge({ mode, privacyMode, classification = 'PUBLIC', compact = false, onClick }: PrivacyBadgeProps) {
  const activeMode = privacyMode || mode || 'LOCAL_ONLY';
  let badgeClass = 'bg-emerald-100 text-emerald-800 border-emerald-300';
  let Icon = Lock;
  let label = 'PRIVATE';

  const isLocal = activeMode === 'LOCAL_ONLY' || activeMode === 'CONFIDENTIAL' || classification === 'CONFIDENTIAL' || classification === 'RESTRICTED' || classification === 'SECRET';

  if (activeMode === 'CONFIDENTIAL' || classification === 'CONFIDENTIAL') {
    badgeClass = 'bg-amber-100 text-amber-900 border-amber-300';
    Icon = EyeOff;
    label = 'CONFIDENTIAL';
  } else if (classification === 'RESTRICTED' || classification === 'SECRET') {
    badgeClass = 'bg-rose-100 text-rose-900 border-rose-300';
    Icon = Shield;
    label = classification;
  } else if (activeMode === 'ONLINE_ONLY') {
    badgeClass = 'bg-sky-100 text-sky-800 border-sky-300';
    Icon = Globe;
    label = 'ONLINE';
  } else if (activeMode === 'AUTO') {
    badgeClass = isLocal ? 'bg-emerald-100 text-emerald-800 border-emerald-300' : 'bg-blue-100 text-blue-800 border-blue-300';
    Icon = Shield;
    label = isLocal ? 'AUTO (LOCAL)' : 'AUTO (HYBRID)';
  }

  if (compact) {
    return (
      <span className={`px-2 py-0.5 rounded-full text-[10px] font-extrabold uppercase tracking-wider border font-mono flex items-center space-x-1 ${badgeClass}`}>
        <Icon className="w-3 h-3" />
        <span>{label}</span>
      </span>
    );
  }

  return (
    <div onClick={onClick} className={`px-3 py-1.5 rounded-xl border text-xs font-bold flex items-center space-x-1.5 shadow-2xs cursor-pointer ${badgeClass}`}>
      <Icon className="w-3.5 h-3.5" />
      <span>Privacy: {label}</span>
    </div>
  );
}
