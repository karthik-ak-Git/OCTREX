'use client';

import React, { useState, useEffect } from 'react';
import Link from 'next/link';
import { Shield, Lock, EyeOff, Globe, ArrowLeft, Check, AlertCircle, Save } from 'lucide-react';
import { PrivacyBadge } from '../../../components/PrivacyBadge';
import { PrivacyRoutingPreview } from '../../../components/PrivacyRoutingPreview';

export default function PrivacySettingsPage() {
  const [privacyMode, setPrivacyMode] = useState<string>('LOCAL_ONLY');
  const [confidentialMode, setConfidentialMode] = useState<boolean>(false);
  const [workspaceClassification, setWorkspaceClassification] = useState<string>('PUBLIC');
  const [statusMessage, setStatusMessage] = useState<string>('');
  const [isSaving, setIsSaving] = useState<boolean>(false);

  const fetchPrivacySettings = async () => {
    try {
      const res = await fetch('/api/privacy/settings');
      if (res.ok) {
        const data = await res.json();
        setPrivacyMode(data.privacy_mode || 'LOCAL_ONLY');
        setConfidentialMode(data.confidential_mode || false);
      }
    } catch (e) {
      console.log('Error fetching privacy settings:', e);
    }
  };

  useEffect(() => {
    fetchPrivacySettings();
  }, []);

  const handleSave = async () => {
    setIsSaving(true);
    setStatusMessage('Saving privacy settings...');
    try {
      const res = await fetch('/api/privacy/settings', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          privacy_mode: privacyMode,
          confidential_mode: confidentialMode,
        }),
      });
      if (res.ok) {
        setStatusMessage('✓ Privacy settings updated successfully.');
      } else {
        setStatusMessage('Failed to save settings.');
      }
    } catch (e: any) {
      setStatusMessage(`Error: ${e.message}`);
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="min-h-screen bg-[#eaedf3] text-slate-900 p-8 flex justify-center">
      <div className="w-[680px] space-y-6">
        
        {/* Header */}
        <div className="flex items-center justify-between bg-white/80 backdrop-blur-xl border border-white/90 p-5 rounded-3xl shadow-sm">
          <div className="flex items-center space-x-3">
            <Link href="/" className="p-2 text-slate-400 hover:text-slate-800 rounded-xl hover:bg-slate-100 transition-all">
              <ArrowLeft className="w-5 h-5" />
            </Link>
            <div>
              <h1 className="text-xl font-extrabold text-slate-900 tracking-tight">Privacy Settings</h1>
              <p className="text-xs text-slate-500 font-medium">Configure Octrex Privacy Gate and routing policies</p>
            </div>
          </div>
          <PrivacyBadge mode={privacyMode} classification={workspaceClassification} />
        </div>

        {/* Status Alert */}
        {statusMessage && (
          <div className="p-3.5 bg-slate-900 text-white rounded-2xl text-xs font-mono font-medium flex items-center justify-between shadow-sm">
            <span>{statusMessage}</span>
            <button onClick={() => setStatusMessage('')} className="text-slate-400 hover:text-white">✕</button>
          </div>
        )}

        {/* Privacy Modes Selector */}
        <div className="bg-white/80 backdrop-blur-xl border border-white/90 p-6 rounded-3xl space-y-4 shadow-sm">
          <div>
            <h2 className="text-sm font-extrabold text-slate-900">Privacy Execution Mode</h2>
            <p className="text-xs text-slate-500 font-medium">Select default execution policy boundary for prompts and workspaces</p>
          </div>

          <div className="grid grid-cols-2 gap-3">
            {/* LOCAL ONLY */}
            <div
              onClick={() => setPrivacyMode('LOCAL_ONLY')}
              className={`p-4 rounded-2xl border cursor-pointer transition-all ${
                privacyMode === 'LOCAL_ONLY'
                  ? 'bg-slate-900 text-white border-slate-900 shadow-md'
                  : 'bg-white text-slate-800 border-slate-200/80 hover:bg-slate-50'
              }`}
            >
              <div className="flex items-center justify-between mb-1">
                <span className="font-extrabold text-sm flex items-center space-x-1.5">
                  <Lock className="w-4 h-4" />
                  <span>LOCAL ONLY</span>
                </span>
                {privacyMode === 'LOCAL_ONLY' && <Check className="w-4 h-4 text-emerald-400" />}
              </div>
              <p className={`text-xs ${privacyMode === 'LOCAL_ONLY' ? 'text-slate-300' : 'text-slate-500'}`}>
                Prohibits any online or cloud model execution. 100% offline local processing.
              </p>
            </div>

            {/* AUTO */}
            <div
              onClick={() => setPrivacyMode('AUTO')}
              className={`p-4 rounded-2xl border cursor-pointer transition-all ${
                privacyMode === 'AUTO'
                  ? 'bg-slate-900 text-white border-slate-900 shadow-md'
                  : 'bg-white text-slate-800 border-slate-200/80 hover:bg-slate-50'
              }`}
            >
              <div className="flex items-center justify-between mb-1">
                <span className="font-extrabold text-sm flex items-center space-x-1.5">
                  <Shield className="w-4 h-4" />
                  <span>AUTO</span>
                </span>
                {privacyMode === 'AUTO' && <Check className="w-4 h-4 text-emerald-400" />}
              </div>
              <p className={`text-xs ${privacyMode === 'AUTO' ? 'text-slate-300' : 'text-slate-500'}`}>
                Evaluates rules, workspace classification & compatibility to allow online when safe.
              </p>
            </div>

            {/* CONFIDENTIAL */}
            <div
              onClick={() => setPrivacyMode('CONFIDENTIAL')}
              className={`p-4 rounded-2xl border cursor-pointer transition-all ${
                privacyMode === 'CONFIDENTIAL'
                  ? 'bg-slate-900 text-white border-slate-900 shadow-md'
                  : 'bg-white text-slate-800 border-slate-200/80 hover:bg-slate-50'
              }`}
            >
              <div className="flex items-center justify-between mb-1">
                <span className="font-extrabold text-sm flex items-center space-x-1.5">
                  <EyeOff className="w-4 h-4" />
                  <span>CONFIDENTIAL</span>
                </span>
                {privacyMode === 'CONFIDENTIAL' && <Check className="w-4 h-4 text-amber-400" />}
              </div>
              <p className={`text-xs ${privacyMode === 'CONFIDENTIAL' ? 'text-slate-300' : 'text-slate-500'}`}>
                Strict industrial privacy. Zero remote calls, zero cloud fallback, zero telemetries.
              </p>
            </div>

            {/* ONLINE ONLY */}
            <div
              onClick={() => setPrivacyMode('ONLINE_ONLY')}
              className={`p-4 rounded-2xl border cursor-pointer transition-all ${
                privacyMode === 'ONLINE_ONLY'
                  ? 'bg-slate-900 text-white border-slate-900 shadow-md'
                  : 'bg-white text-slate-800 border-slate-200/80 hover:bg-slate-50'
              }`}
            >
              <div className="flex items-center justify-between mb-1">
                <span className="font-extrabold text-sm flex items-center space-x-1.5">
                  <Globe className="w-4 h-4" />
                  <span>ONLINE ONLY</span>
                </span>
                {privacyMode === 'ONLINE_ONLY' && <Check className="w-4 h-4 text-sky-400" />}
              </div>
              <p className={`text-xs ${privacyMode === 'ONLINE_ONLY' ? 'text-slate-300' : 'text-slate-500'}`}>
                Requests online model execution. Note: Company policy still overrides this!
              </p>
            </div>
          </div>
        </div>

        {/* Confidential Mode Toggle */}
        <div className="bg-white/80 backdrop-blur-xl border border-white/90 p-5 rounded-3xl flex items-center justify-between shadow-sm">
          <div className="space-y-0.5">
            <div className="text-sm font-extrabold text-slate-900 flex items-center space-x-2">
              <EyeOff className="w-4 h-4 text-amber-600" />
              <span>Global Confidential Mode</span>
            </div>
            <p className="text-xs text-slate-500 font-medium max-w-sm">
              Enforces hard zero-network boundary on all model runtimes regardless of user selection.
            </p>
          </div>
          <button
            onClick={() => setConfidentialMode(!confidentialMode)}
            className={`w-14 h-8 rounded-full p-1 transition-colors ${confidentialMode ? 'bg-amber-600' : 'bg-slate-300'}`}
          >
            <div className={`w-6 h-6 rounded-full bg-white transition-transform ${confidentialMode ? 'translate-x-6' : 'translate-x-0'}`} />
          </button>
        </div>

        {/* Workspace Classification Selector */}
        <div className="bg-white/80 backdrop-blur-xl border border-white/90 p-6 rounded-3xl space-y-3 shadow-sm">
          <div>
            <h2 className="text-sm font-extrabold text-slate-900">Active Workspace Classification</h2>
            <p className="text-xs text-slate-500 font-medium">Security signal level assigned to current workspace files</p>
          </div>

          <div className="flex flex-wrap gap-2">
            {['PUBLIC', 'INTERNAL', 'CONFIDENTIAL', 'RESTRICTED', 'SECRET'].map((cls) => (
              <button
                key={cls}
                onClick={() => setWorkspaceClassification(cls)}
                className={`px-3.5 py-2 rounded-xl text-xs font-mono font-bold transition-all ${
                  workspaceClassification === cls
                    ? 'bg-slate-900 text-white shadow-xs'
                    : 'bg-white border border-slate-200 text-slate-700 hover:bg-slate-50'
                }`}
              >
                {cls}
              </button>
            ))}
          </div>
        </div>

        {/* Routing Preview */}
        <PrivacyRoutingPreview
          classification={workspaceClassification as any}
          mode={privacyMode}
          allowedModes={privacyMode === 'ONLINE_ONLY' || (privacyMode === 'AUTO' && workspaceClassification === 'PUBLIC') ? ['local', 'on_premise', 'cloud'] : ['local', 'on_premise']}
        />

        {/* Save Button */}
        <div className="flex justify-end pt-2">
          <button
            onClick={handleSave}
            disabled={isSaving}
            className="px-6 py-3 bg-slate-900 hover:bg-slate-800 text-white font-bold text-xs rounded-2xl shadow-md transition-all flex items-center space-x-2 disabled:opacity-50"
          >
            <Save className="w-4 h-4" />
            <span>{isSaving ? 'Saving...' : 'Save Privacy Settings'}</span>
          </button>
        </div>

      </div>
    </div>
  );
}
