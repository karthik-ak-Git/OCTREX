import { useRef, useState } from 'react'
import {
  ArrowUp,
  Check,
  ChevronDown,
  ChevronRight,
  FileText,
  Folder,
  Plus,
  Square,
} from 'lucide-react'
import { OctrexLogo } from '../OctrexBrand'

export type ChatCanvasProps = {
  sessionTitle?: string
  workspaceName?: string
  activeWorkspace?: string
  selectedModel?: string
  activeModel?: string
  onSelectModel?: (model: string) => void
  onOpenDocPreview?: () => void
  onOpenFilePreview?: (file: string) => void
  onOpenTerminal?: () => void
  onSend?: (prompt: string) => void
  onAttach?: () => void
  onOpenSkills?: () => void
}

export function ChatCanvas({
  sessionTitle = 'Q4 roadmap deck',
  workspaceName,
  activeWorkspace,
  selectedModel,
  activeModel,
  onSelectModel,
  onOpenDocPreview,
  onOpenFilePreview,
  onOpenTerminal,
  onSend,
  onAttach,
  onOpenSkills,
}: ChatCanvasProps) {
  const currentWorkspace = activeWorkspace || workspaceName || 'octrex-web'
  const currentModel = activeModel || selectedModel || 'Claude Sonnet'
  const handleOpenDoc = onOpenFilePreview ? () => onOpenFilePreview('roadmap-q4.pptx') : onOpenDocPreview
  const [inputPrompt, setInputPrompt] = useState('')
  const [approvalScope, setApprovalScope] = useState<'once' | 'conversation' | 'all'>('once')
  const [isRunning, setIsRunning] = useState(false)
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  const handleSubmit = (e?: React.FormEvent) => {
    e?.preventDefault()
    if (!inputPrompt.trim()) return
    onSend?.(inputPrompt)
    setInputPrompt('')
  }

  return (
    <div className="flex-1 flex flex-col min-w-0 bg-[#f4f6f8] relative overflow-hidden select-none">
      {/* CANVAS HEADER (FIGMA IMAGE 4) */}
      <header className="flex h-14 items-center justify-between px-6 bg-white/60 border-b border-slate-200/80 backdrop-blur-md shrink-0">
        <div className="flex items-center gap-3 min-w-0">
          <h2 className="text-sm font-bold text-slate-900 truncate">{sessionTitle}</h2>
          <div className="flex items-center gap-1 rounded-md bg-slate-100 px-2 py-0.5 text-[11px] font-mono text-slate-600 border border-slate-200">
            <Folder size={11} className="text-slate-400" />
            <span>{currentWorkspace}</span>
          </div>
        </div>

        <div className="flex items-center gap-3 shrink-0">
          {/* MODEL SELECTOR DROPDOWN */}
          <div className="relative inline-block">
            <select
              value={currentModel}
              onChange={(e) => onSelectModel?.(e.target.value)}
              className="appearance-none rounded-full bg-white border border-slate-200/90 pl-3.5 pr-8 py-1.5 text-xs font-semibold text-slate-800 shadow-2xs hover:border-slate-300 focus:outline-none cursor-pointer"
            >
              <option value="Claude Sonnet">Claude Sonnet</option>
              <option value="Gemini 2.5 Flash">Gemini 2.5 Flash</option>
              <option value="Gemini 2.5 Pro">Gemini 2.5 Pro</option>
              <option value="GPT-4o">GPT-4o</option>
              <option value="Llama 3.3 70B">Llama 3.3 70B</option>
              <option value="Qwen 2.5 Coder">Qwen 2.5 Coder</option>
              <option value="DeepSeek R1">DeepSeek R1</option>
            </select>
            <ChevronDown size={12} className="absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none text-slate-400" />
          </div>
        </div>
      </header>

      {/* MESSAGES & EXECUTION AREA */}
      <div className="flex-1 overflow-y-auto p-6 space-y-6 custom-scrollbar">
        <div className="max-w-3xl mx-auto space-y-6">
          {/* USER MESSAGE BUBBLE */}
          <div className="flex justify-end">
            <div className="rounded-2xl bg-white border border-slate-200/80 px-5 py-3.5 text-xs text-slate-800 shadow-2xs max-w-[85%] leading-relaxed">
              Create a 6-slide Q4 roadmap deck from the notes in /docs, then run the build check.
            </div>
          </div>

          {/* ASSISTANT RESPONSE */}
          <div className="flex items-start gap-3.5">
            <div className="flex h-8 w-8 items-center justify-center rounded-xl bg-slate-900 text-white shrink-0 mt-1 shadow-2xs">
              <OctrexLogo size={16} />
            </div>

            <div className="flex-1 space-y-4 max-w-[92%]">
              <div className="text-xs text-slate-700 leading-relaxed">
                I'll read your notes and the pptx skill, then build the deck.
              </div>

              {/* LIVE EXECUTION STEPS CARD (FIGMA IMAGE 4) */}
              <div className="rounded-2xl border border-slate-200/80 bg-white p-4 shadow-2xs space-y-3">
                <div className="flex items-center gap-3 text-xs text-slate-800">
                  <Check size={14} className="text-slate-900" />
                  <span className="font-semibold text-slate-900 w-24">Read skill</span>
                  <span className="font-mono text-slate-500 text-[11px]">pptx</span>
                </div>

                <div className="flex items-center gap-3 text-xs text-slate-800">
                  <Check size={14} className="text-slate-900" />
                  <span className="font-semibold text-slate-900 w-24">Read file</span>
                  <span className="font-mono text-slate-500 text-[11px]">docs/notes.md</span>
                </div>

                <div className="flex items-center gap-3 text-xs text-slate-800">
                  <Check size={14} className="text-slate-900" />
                  <span className="font-semibold text-slate-900 w-24">Created file</span>
                  <span className="font-mono text-slate-500 text-[11px]">roadmap-q4.pptx</span>
                </div>

                <div
                  onClick={onOpenTerminal}
                  className="flex items-center justify-between text-xs text-slate-800 pt-1 border-t border-slate-100 cursor-pointer hover:bg-slate-50/50 rounded-lg p-1 transition-colors"
                >
                  <div className="flex items-center gap-3">
                    <span className="relative flex h-2 w-2 ml-0.5">
                      <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-slate-400 opacity-75"></span>
                      <span className="relative inline-flex rounded-full h-2 w-2 bg-slate-900"></span>
                    </span>
                    <span className="font-semibold text-slate-900 w-24">Running</span>
                    <span className="font-mono text-slate-600 text-[11px]">npm run build</span>
                  </div>
                  <span className="font-mono text-[11px] text-slate-400">00:42</span>
                </div>
              </div>

              {/* GENERATED FILE CARD */}
              <div className="flex items-center justify-between rounded-2xl border border-slate-200/80 bg-white p-4 shadow-2xs hover:border-slate-300 transition-colors">
                <div className="flex items-center gap-3.5">
                  <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-slate-100 border border-slate-200 text-slate-800">
                    <FileText size={18} />
                  </div>
                  <div>
                    <div className="text-xs font-bold text-slate-900">roadmap-q4.pptx</div>
                    <div className="text-[11px] text-slate-400">6 slides · 1.2 MB · Just now</div>
                  </div>
                </div>
                <button
                  onClick={handleOpenDoc}
                  className="flex items-center gap-1 rounded-xl bg-slate-100 px-3 py-1.5 text-xs font-semibold text-slate-700 hover:bg-slate-200 transition-colors cursor-pointer"
                >
                  <span>Open</span>
                  <ChevronRight size={12} />
                </button>
              </div>

              <div className="text-xs text-slate-700 leading-relaxed">
                Deck is ready. The build check is running in the background.
              </div>

              {/* PERMISSION APPROVAL CARD (FIGMA IMAGE 4) */}
              <div className="rounded-2xl border border-slate-200/80 bg-white p-5 shadow-2xs space-y-4">
                <div>
                  <div className="text-xs font-bold text-slate-900">Permission needed</div>
                  <div className="text-[11px] text-slate-500">Octrex wants to run a command</div>
                </div>

                <div className="rounded-xl bg-slate-50 border border-slate-200/60 p-3 font-mono text-xs text-slate-800">
                  $ npm install pptxgenjs
                </div>

                <div className="flex items-center justify-between pt-1">
                  <div className="flex items-center gap-1.5">
                    <button
                      type="button"
                      onClick={() => setApprovalScope('once')}
                      className={`rounded-lg px-2.5 py-1 text-xs font-medium transition-all cursor-pointer ${
                        approvalScope === 'once'
                          ? 'bg-slate-900 text-white'
                          : 'bg-slate-100 text-slate-600 hover:bg-slate-200'
                      }`}
                    >
                      Only this action
                    </button>
                    <button
                      type="button"
                      onClick={() => setApprovalScope('conversation')}
                      className={`rounded-lg px-2.5 py-1 text-xs font-medium transition-all cursor-pointer ${
                        approvalScope === 'conversation'
                          ? 'bg-slate-900 text-white'
                          : 'bg-slate-100 text-slate-600 hover:bg-slate-200'
                      }`}
                    >
                      This conversation
                    </button>
                    <button
                      type="button"
                      onClick={() => setApprovalScope('all')}
                      className={`rounded-lg px-2.5 py-1 text-xs font-medium transition-all cursor-pointer ${
                        approvalScope === 'all'
                          ? 'bg-slate-900 text-white'
                          : 'bg-slate-100 text-slate-600 hover:bg-slate-200'
                      }`}
                    >
                      All chats
                    </button>
                  </div>

                  <div className="flex items-center gap-2">
                    <button
                      type="button"
                      className="rounded-xl border border-slate-200 bg-white px-4 py-1.5 text-xs font-semibold text-slate-700 hover:bg-slate-50 transition-colors cursor-pointer"
                    >
                      Deny
                    </button>
                    <button
                      type="button"
                      className="rounded-xl bg-slate-950 px-5 py-1.5 text-xs font-semibold text-white shadow-2xs hover:bg-slate-800 transition-colors cursor-pointer"
                    >
                      Allow
                    </button>
                  </div>
                </div>

                <div className="text-[10px] text-slate-400">
                  Choose how long this approval applies. You can change it in Settings → Permissions.
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* COMPOSER DOCK (FIGMA IMAGE 4) */}
      <div className="p-5 max-w-3xl w-full mx-auto shrink-0">
        <form
          onSubmit={handleSubmit}
          className="rounded-3xl bg-white border border-slate-200/90 p-3.5 shadow-[0_10px_30px_rgba(0,0,0,0.06)] focus-within:border-slate-400 transition-all"
        >
          <textarea
            ref={textareaRef}
            aria-label="Ask OCTREX"
            placeholder="Ask Octrex to build, write or edit anything..."
            value={inputPrompt}
            maxLength={200000}
            rows={2}
            onChange={(e) => setInputPrompt(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && (e.ctrlKey || e.metaKey) && !e.nativeEvent.isComposing) {
                e.preventDefault()
                handleSubmit()
              }
            }}
            className="w-full resize-none border-0 bg-transparent px-2 py-1 text-xs text-slate-900 placeholder-slate-400 focus:outline-none leading-relaxed"
          />

          <div className="flex items-center justify-between pt-2 border-t border-slate-100">
            <div className="flex items-center gap-1.5">
              <button
                type="button"
                onClick={onAttach}
                className="flex h-7 w-7 items-center justify-center rounded-lg text-slate-400 hover:text-slate-700 hover:bg-slate-100 transition-colors cursor-pointer"
                title="Attach file"
              >
                <Plus size={14} />
              </button>

              <button
                type="button"
                onClick={onOpenSkills}
                className="flex items-center gap-1 rounded-lg bg-slate-100/70 px-2.5 py-1 text-xs font-medium text-slate-600 hover:bg-slate-100 transition-colors cursor-pointer"
              >
                <span>@ Skills</span>
              </button>

              <button
                type="button"
                className="rounded-lg px-2.5 py-1 text-xs font-medium bg-slate-100 text-slate-800 transition-colors cursor-pointer"
              >
                <span>Plan</span>
              </button>
            </div>

            {isRunning ? (
              <button
                type="button"
                onClick={() => setIsRunning(false)}
                className="flex h-8 w-8 items-center justify-center rounded-full bg-slate-900 text-white hover:bg-slate-800 shadow-2xs transition-colors cursor-pointer"
              >
                <Square size={13} className="fill-white" />
              </button>
            ) : (
              <button
                type="submit"
                disabled={!inputPrompt.trim()}
                className="flex h-8 w-8 items-center justify-center rounded-full bg-slate-950 text-white hover:bg-slate-800 disabled:opacity-30 disabled:hover:bg-slate-950 shadow-2xs transition-all cursor-pointer"
              >
                <ArrowUp size={15} />
              </button>
            )}
          </div>
        </form>
      </div>
    </div>
  )
}
