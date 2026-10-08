'use client';

import React, { useState } from 'react';
import { AlertTriangle, Send, ShieldCheck, CheckCircle2 } from 'lucide-react';

interface TaskWaitingForUserProps {
  taskId: string;
  promptText?: string;
  onSubmitInput: (input: string) => Promise<void>;
}

export const TaskWaitingForUser: React.FC<TaskWaitingForUserProps> = ({
  taskId,
  promptText = 'The task execution loop has reached a step requiring explicit user input or consent to proceed.',
  onSubmitInput,
}) => {
  const [input, setInput] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim() || isSubmitting) return;

    try {
      setIsSubmitting(true);
      setError(null);
      await onSubmitInput(input.trim());
      setInput('');
    } catch (err: any) {
      setError(err.message || 'Failed to submit user input.');
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="p-5 rounded-xl bg-purple-950/40 border border-purple-800/60 space-y-4 shadow-xl shadow-purple-950/20">
      <div className="flex items-start gap-3">
        <div className="p-2 rounded-lg bg-purple-900/60 text-purple-300 border border-purple-700/60 mt-0.5">
          <AlertTriangle className="w-5 h-5" />
        </div>
        <div>
          <h4 className="text-sm font-bold text-purple-200 flex items-center gap-2">
            Task Paused — Action & Input Required
            <span className="text-[10px] uppercase font-mono px-2 py-0.5 rounded bg-purple-900 text-purple-300 border border-purple-700">
              WaitingForUser
            </span>
          </h4>
          <p className="text-xs text-purple-300/90 mt-1 leading-relaxed">
            {promptText}
          </p>
        </div>
      </div>

      <form onSubmit={handleSubmit} className="space-y-3 pt-2">
        <div>
          <label className="block text-xs font-semibold text-purple-200 mb-1.5">
            User Response / Decision / Guidance
          </label>
          <textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="Type your decision, clarification, or instruction for the orchestrator..."
            rows={3}
            className="w-full px-3 py-2 rounded-lg bg-zinc-950 border border-purple-800/80 text-zinc-100 placeholder-purple-400/50 text-xs focus:outline-none focus:ring-2 focus:ring-purple-500/50 resize-none font-mono"
          />
        </div>

        {error && (
          <div className="text-xs text-red-400 font-medium">
            ⚠️ {error}
          </div>
        )}

        <div className="flex items-center justify-between">
          <span className="text-[11px] text-purple-400/80 flex items-center gap-1">
            <ShieldCheck className="w-3.5 h-3.5 text-purple-400" />
            Authoritative Orchestrator Security Gate
          </span>

          <button
            type="submit"
            disabled={!input.trim() || isSubmitting}
            className="inline-flex items-center px-4 py-2 rounded-lg text-xs font-semibold bg-purple-600 hover:bg-purple-500 text-white shadow-lg shadow-purple-950/50 transition-all disabled:opacity-50"
          >
            <Send className="w-3.5 h-3.5 mr-1.5" />
            {isSubmitting ? 'Submitting...' : 'Submit Input & Resume'}
          </button>
        </div>
      </form>
    </div>
  );
};
