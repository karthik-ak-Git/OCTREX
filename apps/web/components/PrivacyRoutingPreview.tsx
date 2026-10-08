'use client';

import React from 'react';
import { Server, Cpu, Globe, CheckCircle, XCircle, ArrowRight, Shield } from 'lucide-react';
import { PrivacyClassification, ExecutionMode } from '../lib/backend/types';

interface PrivacyRoutingPreviewProps {
  classification?: PrivacyClassification;
  mode?: string;
  allowedModes?: ExecutionMode[];
}

export function PrivacyRoutingPreview({
  classification = 'PUBLIC',
  mode = 'LOCAL_ONLY',
  allowedModes = ['local', 'on_premise']
}: PrivacyRoutingPreviewProps) {
  const isCloudAllowed = allowedModes.includes('cloud' as any) || allowedModes.includes('Cloud' as any);

  return (
    <div className="bg-white/80 border border-slate-200/90 rounded-2xl p-4 space-y-3 shadow-2xs">
      <div className="flex justify-between items-center text-xs">
        <div className="flex items-center space-x-1.5 font-bold text-slate-900">
          <Shield className="w-4 h-4 text-slate-700" />
          <span>Execution Path Authorization Preview</span>
        </div>
        <span className="text-[10px] font-mono text-slate-400">CLASSIFICATION: {classification}</span>
      </div>

      <div className="grid grid-cols-3 gap-2">
        {/* Local Path */}
        <div className="p-3 bg-emerald-50/70 border border-emerald-200 rounded-xl text-xs space-y-1">
          <div className="flex justify-between items-center">
            <span className="font-bold text-emerald-900 flex items-center space-x-1">
              <Server className="w-3.5 h-3.5 text-emerald-600" />
              <span>LOCAL</span>
            </span>
            <CheckCircle className="w-4 h-4 text-emerald-600" />
          </div>
          <div className="text-[10px] text-emerald-700">Always Allowed</div>
        </div>

        {/* On-Premise Path */}
        <div className="p-3 bg-emerald-50/70 border border-emerald-200 rounded-xl text-xs space-y-1">
          <div className="flex justify-between items-center">
            <span className="font-bold text-emerald-900 flex items-center space-x-1">
              <Cpu className="w-3.5 h-3.5 text-emerald-600" />
              <span>ON-PREMISE</span>
            </span>
            <CheckCircle className="w-4 h-4 text-emerald-600" />
          </div>
          <div className="text-[10px] text-emerald-700">Internal Network</div>
        </div>

        {/* Cloud Path */}
        <div className={`p-3 rounded-xl border text-xs space-y-1 ${
          isCloudAllowed ? 'bg-sky-50/70 border-sky-200' : 'bg-rose-50/70 border-rose-200'
        }`}>
          <div className="flex justify-between items-center">
            <span className={`font-bold flex items-center space-x-1 ${isCloudAllowed ? 'text-sky-900' : 'text-rose-900'}`}>
              <Globe className={`w-3.5 h-3.5 ${isCloudAllowed ? 'text-sky-600' : 'text-rose-600'}`} />
              <span>ONLINE</span>
            </span>
            {isCloudAllowed ? (
              <CheckCircle className="w-4 h-4 text-sky-600" />
            ) : (
              <XCircle className="w-4 h-4 text-rose-600" />
            )}
          </div>
          <div className={`text-[10px] ${isCloudAllowed ? 'text-sky-700' : 'text-rose-700'}`}>
            {isCloudAllowed ? 'Allowed by policy' : 'Prohibited by policy'}
          </div>
        </div>
      </div>
    </div>
  );
}
