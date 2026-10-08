'use client';

import React, { useState } from 'react';
import { AlertTriangle, Globe, Lock, Shield, Check, X, Eye } from 'lucide-react';
import { ConsentRequest } from '../lib/backend/types';

interface OnlineConsentDialogProps {
  request?: ConsentRequest | null;
  consentRequest?: ConsentRequest | null;
  onApprove: (consentId?: string, reason?: string) => void;
  onDeny: (consentId?: string, reason?: string) => void;
  onClose?: () => void;
}

export function OnlineConsentDialog({ request, consentRequest, onApprove, onDeny, onClose }: OnlineConsentDialogProps) {
  const [userNote, setUserNote] = useState('');
  const activeReq = consentRequest || request;

  if (!activeReq) return null;

  return (
    <div className="fixed inset-0 z-50 bg-slate-950/50 backdrop-blur-md flex items-center justify-center p-4">
      <div className="w-[520px] bg-white/95 backdrop-blur-2xl border border-slate-200 rounded-3xl p-6 shadow-2xl space-y-5">
        
        {/* Header */}
        <div className="flex items-center justify-between border-b border-slate-200/80 pb-3">
          <div className="flex items-center space-x-2.5">
            <div className="w-9 h-9 rounded-2xl bg-amber-100 border border-amber-300 flex items-center justify-center text-amber-700 font-bold">
              <Globe className="w-5 h-5" />
            </div>
            <div>
              <h3 className="text-base font-extrabold text-slate-900">Online Model Execution Consent</h3>
              <span className="text-[10px] text-slate-400 font-mono">Consent ID: {activeReq.id || activeReq.request_id}</span>
            </div>
          </div>
          {onClose && (
            <button onClick={onClose} className="p-1 text-slate-400 hover:text-slate-700 rounded-lg">
              <X className="w-4 h-4" />
            </button>
          )}
        </div>

        {/* Warning banner */}
        <div className="p-3.5 bg-amber-50 border border-amber-200 rounded-2xl text-xs space-y-1 text-amber-900">
          <div className="flex items-center space-x-1.5 font-bold">
            <AlertTriangle className="w-4 h-4 text-amber-600" />
            <span>Outbound Network Transmission Notice</span>
          </div>
          <p className="text-[11px] leading-relaxed text-amber-800">
            {activeReq.reasoning || 'Sending request data to online provider requires your explicit authorization under Octrex Privacy Policy.'}
          </p>
        </div>

        {/* Outbound Payload Preview Summary */}
        <div className="p-4 bg-slate-100/80 border border-slate-200/80 rounded-2xl space-y-3 text-xs">
          <div className="flex justify-between items-center font-bold text-slate-900">
            <span className="flex items-center space-x-1.5">
              <Eye className="w-4 h-4 text-slate-600" />
              <span>Payload Preview Summary</span>
            </span>
            <span className="px-2.5 py-0.5 rounded-full bg-slate-200 text-slate-700 font-mono text-[10px]">
              {activeReq.payload_preview?.approximate_payload_bytes || 0} bytes
            </span>
          </div>

          <div className="grid grid-cols-2 gap-2 font-mono text-[11px]">
            <div><span className="text-slate-400">Destination Provider:</span> <strong className="text-slate-800">{activeReq.destination_provider}</strong></div>
            <div><span className="text-slate-400">Model:</span> <strong className="text-slate-800">{activeReq.destination_model}</strong></div>
            <div><span className="text-slate-400">Classification:</span> <strong className="text-amber-700">{activeReq.classification}</strong></div>
            <div><span className="text-slate-400">Execution Mode:</span> <strong className="text-slate-800">{activeReq.requested_mode}</strong></div>
          </div>

          {activeReq.payload_preview?.files_included && activeReq.payload_preview.files_included.length > 0 && (
            <div className="pt-1 border-t border-slate-200/60">
              <span className="text-[10px] font-extrabold text-slate-400 uppercase">Included Files</span>
              <div className="flex flex-wrap gap-1 mt-1 font-mono text-[10px]">
                {activeReq.payload_preview.files_included.map((f, i) => (
                  <span key={i} className="px-2 py-0.5 rounded-md bg-white border border-slate-300 text-slate-700">
                    {f}
                  </span>
                ))}
              </div>
            </div>
          )}
        </div>

        {/* User Note Input */}
        <div className="space-y-1.5">
          <label className="block text-[11px] font-extrabold uppercase text-slate-400 tracking-wider">Audit Reason / Note (Optional)</label>
          <input
            type="text"
            value={userNote}
            onChange={(e) => setUserNote(e.target.value)}
            placeholder="Reason for granting/denying online consent..."
            className="w-full px-4 py-2.5 bg-slate-100/80 border border-slate-200 rounded-xl text-xs font-medium text-slate-800 focus:outline-none focus:bg-white focus:ring-2 focus:ring-slate-900/10"
          />
        </div>

        {/* Actions */}
        <div className="flex items-center justify-end space-x-2 pt-2">
          <button
            onClick={() => onDeny(activeReq.id || activeReq.request_id, userNote)}
            className="px-4 py-2.5 rounded-xl text-xs font-bold bg-rose-600 text-white hover:bg-rose-700 transition-all flex items-center space-x-1.5"
          >
            <X className="w-4 h-4" />
            <span>Deny Online Use</span>
          </button>
          <button
            onClick={() => onApprove(activeReq.id || activeReq.request_id, userNote)}
            className="px-5 py-2.5 rounded-xl text-xs font-bold bg-emerald-600 text-white hover:bg-emerald-700 transition-all flex items-center space-x-1.5 shadow-sm"
          >
            <Check className="w-4 h-4" />
            <span>Allow Once (Online)</span>
          </button>
        </div>

      </div>
    </div>
  );
}
