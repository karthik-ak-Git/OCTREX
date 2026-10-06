import React, { useRef, useState, useEffect } from 'react';
import {
  ArrowUp,
  Check,
  ChevronDown,
  ChevronRight,
  FileText,
  Folder,
  Sparkles,
  Paperclip,
  ShieldAlert,
  X,
  Layers
} from 'lucide-react';
import { OctrexLogo } from '../OctrexBrand';
import { ChatSession, SkillItem } from '../state';

export type ChatCanvasProps = {
  activeSession: ChatSession;
  skills: SkillItem[];
  onSelectModel: (model: string) => void;
  onSendMessage: (prompt: string) => void;
  onRespondPermission: (msgId: string, decision: 'approve' | 'deny', scope: 'once' | 'conversation' | 'all') => void;
  onOpenFilePreview: (fileName: string) => void;
  onOpenTerminal: () => void;
  onOpenSkillsSettings: () => void;
};

export const ChatCanvas: React.FC<ChatCanvasProps> = ({
  activeSession,
  skills,
  onSelectModel,
  onSendMessage,
  onRespondPermission,
  onOpenFilePreview,
  onOpenTerminal,
  onOpenSkillsSettings
}) => {
  const [inputPrompt, setInputPrompt] = useState('');
  const [isPlanMode, setIsPlanMode] = useState(false);
  const [showSkillsMenu, setShowSkillsMenu] = useState(false);
  const [permissionScope, setPermissionScope] = useState<'once' | 'conversation' | 'all'>('once');
  const [attachedFiles, setAttachedFiles] = useState<string[]>([]);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Auto scroll to bottom of messages
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [activeSession.messages]);

  const handleSubmit = (e?: React.FormEvent) => {
    e?.preventDefault();
    if (!inputPrompt.trim() && attachedFiles.length === 0) return;

    let fullPrompt = inputPrompt.trim();
    if (isPlanMode) {
      fullPrompt = `[PLAN MODE] ${fullPrompt}`;
    }
    if (attachedFiles.length > 0) {
      fullPrompt += `\n(Attachments: ${attachedFiles.join(', ')})`;
    }

    onSendMessage(fullPrompt);
    setInputPrompt('');
    setAttachedFiles([]);
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
    }
  };

  const handlePickAttachment = async () => {
    try {
      if (window.octrex?.pickAttachments) {
        const paths = await window.octrex.pickAttachments();
        if (paths && paths.length > 0) {
          const fileNames = paths.map((p: string) => p.split(/[\\/]/).pop() || p);
          setAttachedFiles((prev) => [...prev, ...fileNames]);
          return;
        }
      }
    } catch (err) {
      console.warn('Native picker error, fallback:', err);
    }
    // Web fallback
    const input = document.createElement('input');
    input.type = 'file';
    input.multiple = true;
    input.onchange = (e: any) => {
      const files = Array.from(e.target.files || []) as File[];
      setAttachedFiles((prev) => [...prev, ...files.map((f) => f.name)]);
    };
    input.click();
  };

  const insertSkillMention = (skillName: string) => {
    setInputPrompt((prev) => `${prev}@${skillName} `);
    setShowSkillsMenu(false);
    textareaRef.current?.focus();
  };

  return (
    <div className="flex-1 flex flex-col min-w-0 bg-[#f4f6f8] relative overflow-hidden select-none">
      {/* Ambient glassmorphic light glow effects */}
      <div className="ambient-glow ambient-cyan w-96 h-96 -top-24 -left-24" />
      <div className="ambient-glow ambient-emerald w-96 h-96 -bottom-24 -right-24" />

      {/* CANVAS HEADER (FIGMA IMAGE 4) */}
      <header className="relative z-10 flex h-14 items-center justify-between px-6 glass-panel border-b border-white/60 shrink-0">
        <div className="flex items-center gap-3 min-w-0">
          <h2 className="text-sm font-bold text-slate-900 truncate">
            {activeSession.title}
          </h2>
          <div className="flex items-center gap-1.5 rounded-lg bg-white/70 px-2.5 py-1 text-[11px] font-mono font-medium text-slate-700 border border-slate-200/80 shadow-2xs">
            <Folder size={12} className="text-slate-400" />
            <span>{activeSession.workspaceName}</span>
          </div>
        </div>

        <div className="flex items-center gap-3 shrink-0">
          {/* MODEL SELECTOR DROPDOWN */}
          <div className="relative inline-block">
            <select
              value={activeSession.model}
              onChange={(e) => onSelectModel(e.target.value)}
              className="appearance-none rounded-full glass-card pl-3.5 pr-8 py-1.5 text-xs font-semibold text-slate-800 shadow-2xs hover:border-slate-300 focus:outline-none cursor-pointer"
            >
              <option value="Claude Sonnet 3.5">Claude Sonnet 3.5</option>
              <option value="Gemini 2.5 Flash">Gemini 2.5 Flash</option>
              <option value="Gemini 2.5 Pro">Gemini 2.5 Pro</option>
              <option value="GPT-4o">GPT-4o</option>
              <option value="Llama 3.3 70B">Llama 3.3 70B (NVIDIA)</option>
              <option value="Qwen 2.5 Coder">Qwen 2.5 Coder (Ollama)</option>
              <option value="DeepSeek R1">DeepSeek R1</option>
            </select>
            <ChevronDown
              size={12}
              className="absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none text-slate-400"
            />
          </div>
        </div>
      </header>

      {/* MESSAGES & INTERACTION THREAD */}
      <div className="relative z-10 flex-1 overflow-y-auto p-6 space-y-6 custom-scrollbar">
        <div className="max-w-3xl mx-auto space-y-6 pb-28">
          {activeSession.messages.length === 0 ? (
            <div className="pt-24 text-center space-y-3">
              <div className="w-12 h-12 rounded-2xl glass-card mx-auto flex items-center justify-center text-slate-800 shadow-sm">
                <OctrexLogo size={24} />
              </div>
              <h3 className="text-base font-bold text-slate-900">
                What would you like to build today?
              </h3>
              <p className="text-xs text-slate-500 max-w-sm mx-auto">
                OCTREX CODE can plan features, read documentation, create slide decks, edit files, and run commands with verified safety.
              </p>
              <div className="flex flex-wrap items-center justify-center gap-2 pt-2">
                {[
                  'Create a 6-slide Q4 roadmap deck',
                  'Refactor API client to TypeScript',
                  'Inspect workspace for test coverage',
                  'Design modern landing hero in Tailwind'
                ].map((prompt) => (
                  <button
                    key={prompt}
                    onClick={() => onSendMessage(prompt)}
                    className="glass-card hover:bg-white rounded-xl px-3 py-1.5 text-xs text-slate-700 hover:text-slate-900 border border-slate-200/80 shadow-2xs hover:scale-[1.02] transition-all cursor-pointer"
                  >
                    {prompt}
                  </button>
                ))}
              </div>
            </div>
          ) : (
            activeSession.messages.map((msg) => (
              <div key={msg.id} className="space-y-4">
                {msg.sender === 'user' ? (
                  /* USER MESSAGE BUBBLE */
                  <div className="flex justify-end">
                    <div className="rounded-2xl glass-card border border-white/80 px-5 py-3.5 text-xs text-slate-800 shadow-sm max-w-[85%] leading-relaxed font-normal">
                      {msg.content}
                    </div>
                  </div>
                ) : (
                  /* ASSISTANT MESSAGE & EXECUTION BLOCK */
                  <div className="flex items-start gap-3.5">
                    <div className="flex h-8 w-8 items-center justify-center rounded-xl bg-slate-900 text-white shrink-0 mt-1 shadow-sm">
                      <OctrexLogo size={16} />
                    </div>

                    <div className="flex-1 space-y-4 max-w-[92%]">
                      {msg.content && (
                        <div className="text-xs text-slate-700 leading-relaxed font-normal">
                          {msg.content}
                        </div>
                      )}

                      {/* LIVE EXECUTION STEPS CARD */}
                      {msg.executionSteps && msg.executionSteps.length > 0 && (
                        <div className="rounded-2xl glass-card border border-white/80 p-4 shadow-sm space-y-3">
                          {msg.executionSteps.map((step) => (
                            <div
                              key={step.id}
                              onClick={() => {
                                if (step.type === 'terminal') onOpenTerminal();
                                else if (step.type === 'write') onOpenFilePreview(step.target);
                              }}
                              className={`flex items-center justify-between text-xs text-slate-800 ${
                                step.type === 'terminal' || step.type === 'write'
                                  ? 'cursor-pointer hover:bg-slate-50/60 rounded-lg p-1 transition-colors'
                                  : ''
                              }`}
                            >
                              <div className="flex items-center gap-3">
                                {step.status === 'completed' ? (
                                  <Check size={14} className="text-slate-900 shrink-0" />
                                ) : (
                                  <span className="relative flex h-2.5 w-2.5 ml-0.5 shrink-0">
                                    <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-cyan-400 opacity-75"></span>
                                    <span className="relative inline-flex rounded-full h-2.5 w-2.5 bg-cyan-500"></span>
                                  </span>
                                )}
                                <span className="font-semibold text-slate-900 w-24">{step.label}</span>
                                <span className="font-mono text-slate-600 text-[11px] truncate max-w-xs">
                                  {step.target}
                                </span>
                              </div>
                              {step.duration && (
                                <span className="font-mono text-[11px] text-slate-400">
                                  {step.duration}
                                </span>
                              )}
                            </div>
                          ))}
                        </div>
                      )}

                      {/* GENERATED ARTIFACT FILE CARDS */}
                      {msg.artifacts &&
                        msg.artifacts.map((art) => (
                          <div
                            key={art.id}
                            className="flex items-center justify-between rounded-2xl glass-card border border-white/80 p-4 shadow-sm hover:border-slate-300 transition-all"
                          >
                            <div className="flex items-center gap-3.5">
                              <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-slate-100 border border-slate-200 text-slate-800 shadow-2xs">
                                <FileText size={18} />
                              </div>
                              <div>
                                <div className="text-xs font-bold text-slate-900">{art.name}</div>
                                <div className="text-[11px] text-slate-400">
                                  {art.metadata || `${art.size} • Just now`}
                                </div>
                              </div>
                            </div>
                            <button
                              onClick={() => onOpenFilePreview(art.name)}
                              className="flex items-center gap-1 rounded-xl bg-slate-100 hover:bg-slate-200 px-3.5 py-1.5 text-xs font-semibold text-slate-800 transition-all cursor-pointer shadow-2xs"
                            >
                              <span>Open</span>
                              <ChevronRight size={13} />
                            </button>
                          </div>
                        ))}

                      {/* PERMISSION APPROVAL CARD (FIGMA IMAGE 4) */}
                      {msg.pendingPermission && (
                        <div className="rounded-2xl glass-card border border-amber-200/60 bg-amber-50/30 p-5 shadow-sm space-y-4">
                          <div className="flex items-center gap-2">
                            <ShieldAlert size={16} className="text-amber-600" />
                            <div>
                              <div className="text-xs font-bold text-slate-900">Permission needed</div>
                              <div className="text-[11px] text-slate-500">
                                {msg.pendingPermission.description}
                              </div>
                            </div>
                          </div>

                          <div className="rounded-xl bg-slate-950 px-3.5 py-2 font-mono text-xs text-emerald-400 font-medium">
                            $ {msg.pendingPermission.command}
                          </div>

                          {msg.pendingPermission.status === 'pending' ? (
                            <div className="space-y-3">
                              {/* SCOPE SELECTORS */}
                              <div className="space-y-1.5 text-xs text-slate-700">
                                {[
                                  { id: 'once', label: 'Only once' },
                                  { id: 'conversation', label: 'This conversation' },
                                  { id: 'all', label: 'Always for this project' }
                                ].map((opt) => (
                                  <label
                                    key={opt.id}
                                    className="flex items-center gap-2 cursor-pointer select-none"
                                  >
                                    <input
                                      type="radio"
                                      name={`scope-${msg.id}`}
                                      checked={permissionScope === opt.id}
                                      onChange={() => setPermissionScope(opt.id as any)}
                                      className="accent-slate-900 cursor-pointer"
                                    />
                                    <span>{opt.label}</span>
                                  </label>
                                ))}
                              </div>

                              {/* ACTIONS */}
                              <div className="flex items-center gap-2.5 pt-1">
                                <button
                                  onClick={() => onRespondPermission(msg.id, 'approve', permissionScope)}
                                  className="rounded-xl bg-slate-950 hover:bg-slate-800 px-4 py-1.5 text-xs font-semibold text-white shadow-sm transition-all cursor-pointer"
                                >
                                  Allow
                                </button>
                                <button
                                  onClick={() => onRespondPermission(msg.id, 'deny', permissionScope)}
                                  className="rounded-xl bg-white hover:bg-slate-100 border border-slate-200 px-4 py-1.5 text-xs font-semibold text-slate-700 transition-all cursor-pointer"
                                >
                                  Deny
                                </button>
                              </div>
                            </div>
                          ) : (
                            <div className="flex items-center gap-2 text-xs font-medium">
                              {msg.pendingPermission.status === 'approved' ? (
                                <span className="text-emerald-700 font-semibold flex items-center gap-1">
                                  <Check size={13} /> Approved command execution
                                </span>
                              ) : (
                                <span className="text-rose-600 font-semibold flex items-center gap-1">
                                  <X size={13} /> Denied command execution
                                </span>
                              )}
                            </div>
                          )}
                        </div>
                      )}
                    </div>
                  </div>
                )}
              </div>
            ))
          )}
          <div ref={messagesEndRef} />
        </div>
      </div>

      {/* FLOATING GLASS COMPOSER (FIGMA IMAGE 4) */}
      <div className="absolute bottom-6 left-6 right-6 z-20 max-w-3xl mx-auto">
        {/* SKILLS POPOVER MENU */}
        {showSkillsMenu && (
          <div className="mb-2 glass-modal rounded-2xl p-3 shadow-xl border border-slate-200 space-y-2">
            <div className="flex items-center justify-between text-[11px] font-semibold text-slate-500 uppercase px-1">
              <span>Attach Skill</span>
              <button
                onClick={onOpenSkillsSettings}
                className="text-cyan-600 hover:underline cursor-pointer lowercase text-[10px]"
              >
                manage skills
              </button>
            </div>
            <div className="grid grid-cols-2 gap-1.5">
              {skills
                .filter((s) => s.enabled)
                .map((skill) => (
                  <button
                    key={skill.id}
                    onClick={() => insertSkillMention(skill.name)}
                    className="flex items-center gap-2 p-2 rounded-xl hover:bg-slate-100 text-left transition-colors cursor-pointer"
                  >
                    <Layers size={13} className="text-cyan-600 shrink-0" />
                    <div className="truncate">
                      <div className="text-xs font-semibold text-slate-900 truncate">{skill.name}</div>
                      <div className="text-[10px] text-slate-400 truncate">{skill.desc}</div>
                    </div>
                  </button>
                ))}
            </div>
          </div>
        )}

        {/* ATTACHED FILES CHIPS */}
        {attachedFiles.length > 0 && (
          <div className="mb-2 flex flex-wrap gap-1.5 px-2">
            {attachedFiles.map((file, idx) => (
              <div
                key={idx}
                className="flex items-center gap-1.5 rounded-lg bg-slate-900 text-white text-[11px] font-medium px-2.5 py-1 shadow-sm"
              >
                <FileText size={11} />
                <span>{file}</span>
                <button
                  onClick={() => setAttachedFiles((prev) => prev.filter((_, i) => i !== idx))}
                  className="hover:text-rose-300 cursor-pointer"
                >
                  <X size={11} />
                </button>
              </div>
            ))}
          </div>
        )}

        <form
          onSubmit={handleSubmit}
          className="glass-composer rounded-3xl p-3 border border-slate-200/90 shadow-lg space-y-2.5"
        >
          <textarea
            ref={textareaRef}
            rows={2}
            value={inputPrompt}
            placeholder={`Ask Octrex anything in ${activeSession.workspaceName}... (Shift+Enter for newline, Enter to send)`}
            onChange={(e) => {
              setInputPrompt(e.target.value);
              e.target.style.height = 'auto';
              e.target.style.height = `${Math.min(e.target.scrollHeight, 140)}px`;
            }}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && !e.shiftKey && !e.nativeEvent.isComposing) {
                e.preventDefault();
                handleSubmit();
              }
            }}
            className="w-full resize-none border-0 bg-transparent px-2.5 py-1 text-xs text-slate-900 placeholder-slate-400 focus:outline-none leading-relaxed"
          />

          <div className="flex items-center justify-between pt-1 border-t border-slate-100">
            <div className="flex items-center gap-1.5">
              <button
                type="button"
                onClick={handlePickAttachment}
                className="flex h-7 w-7 items-center justify-center rounded-lg text-slate-500 hover:text-slate-900 hover:bg-slate-100 transition-colors cursor-pointer"
                title="Attach files (Local docs/images)"
              >
                <Paperclip size={14} />
              </button>

              <button
                type="button"
                onClick={() => setShowSkillsMenu(!showSkillsMenu)}
                className={`flex items-center gap-1 rounded-lg px-2.5 py-1 text-xs font-medium transition-colors cursor-pointer ${
                  showSkillsMenu
                    ? 'bg-slate-900 text-white'
                    : 'bg-slate-100/80 hover:bg-slate-200 text-slate-700'
                }`}
                title="Select skills to use"
              >
                <Sparkles size={12} />
                <span>@ Skills</span>
              </button>

              <button
                type="button"
                onClick={() => setIsPlanMode(!isPlanMode)}
                className={`rounded-lg px-2.5 py-1 text-xs font-semibold transition-all cursor-pointer ${
                  isPlanMode
                    ? 'bg-indigo-600 text-white shadow-2xs'
                    : 'bg-slate-100 hover:bg-slate-200 text-slate-700'
                }`}
                title="Toggle Plan Mode"
              >
                <span>Plan</span>
              </button>
            </div>

            <button
              type="submit"
              disabled={!inputPrompt.trim() && attachedFiles.length === 0}
              className="flex h-8 w-8 items-center justify-center rounded-full bg-slate-950 text-white hover:bg-slate-800 disabled:opacity-30 disabled:hover:bg-slate-950 shadow-sm active:scale-95 transition-all cursor-pointer"
              title="Send message"
            >
              <ArrowUp size={15} />
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};
