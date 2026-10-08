'use client';

import React from 'react';
import { Clock, Play, Pause, CheckCircle2, XCircle, AlertTriangle, ShieldAlert } from 'lucide-react';

interface TaskStatusBadgeProps {
  status: string;
}

export const TaskStatusBadge: React.FC<TaskStatusBadgeProps> = ({ status }) => {
  const normalized = (status || 'created').toLowerCase();

  let color = 'bg-gray-800 text-gray-300 border-gray-700';
  let icon = <Clock className="w-3.5 h-3.5 mr-1.5 text-gray-400" />;
  let label = status;

  switch (normalized) {
    case 'created':
    case 'planning':
    case 'planready':
      color = 'bg-blue-950/60 text-blue-400 border-blue-800/60';
      icon = <Clock className="w-3.5 h-3.5 mr-1.5 text-blue-400" />;
      label = status === 'planready' ? 'Plan Ready' : status;
      break;
    case 'running':
    case 'executing':
      color = 'bg-emerald-950/60 text-emerald-400 border-emerald-800/60 animate-pulse';
      icon = <Play className="w-3.5 h-3.5 mr-1.5 text-emerald-400" />;
      label = 'Executing';
      break;
    case 'paused':
      color = 'bg-amber-950/60 text-amber-400 border-amber-800/60';
      icon = <Pause className="w-3.5 h-3.5 mr-1.5 text-amber-400" />;
      label = 'Paused';
      break;
    case 'waitingforuser':
      color = 'bg-purple-950/60 text-purple-300 border-purple-800/60 animate-bounce';
      icon = <AlertTriangle className="w-3.5 h-3.5 mr-1.5 text-purple-400" />;
      label = 'Waiting for User';
      break;
    case 'waitingfortool':
    case 'verifying':
    case 'retrying':
      color = 'bg-cyan-950/60 text-cyan-300 border-cyan-800/60';
      icon = <Clock className="w-3.5 h-3.5 mr-1.5 text-cyan-400" />;
      label = status;
      break;
    case 'completed':
      color = 'bg-green-950/60 text-green-400 border-green-800/60';
      icon = <CheckCircle2 className="w-3.5 h-3.5 mr-1.5 text-green-400" />;
      label = 'Completed';
      break;
    case 'failed':
      color = 'bg-red-950/60 text-red-400 border-red-800/60';
      icon = <XCircle className="w-3.5 h-3.5 mr-1.5 text-red-400" />;
      label = 'Failed';
      break;
    case 'blocked':
      color = 'bg-rose-950/60 text-rose-400 border-rose-800/60';
      icon = <ShieldAlert className="w-3.5 h-3.5 mr-1.5 text-rose-400" />;
      label = 'Blocked';
      break;
    case 'cancelled':
      color = 'bg-zinc-800 text-zinc-400 border-zinc-700';
      icon = <XCircle className="w-3.5 h-3.5 mr-1.5 text-zinc-400" />;
      label = 'Cancelled';
      break;
  }

  return (
    <span className={`inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium border ${color}`}>
      {icon}
      <span className="capitalize">{label}</span>
    </span>
  );
};
