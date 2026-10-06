import { useState } from 'react'
import { Download, Sparkles } from 'lucide-react'

export type DocumentPreviewProps = {
  fileName?: string
  totalSlides?: number
  currentSlideIndex?: number
  size?: string
  created?: string
  createdBy?: string
  onAskOctrexToEdit?: () => void
  onDownload?: () => void
}

export function DocumentPreview({
  fileName = 'roadmap-q4.pptx',
  totalSlides = 6,
  currentSlideIndex = 1,
  size = '1.2 MB',
  created = 'just now',
  createdBy = 'Main agent',
  onAskOctrexToEdit,
  onDownload,
}: DocumentPreviewProps) {
  const [activeSlide, setActiveSlide] = useState(currentSlideIndex)

  const slides = [
    { title: 'Q4 Roadmap', subtitle: 'Priorities, milestones and owners', tag: 'OCTREX · Q4 2026' },
    { title: 'Executive Summary', subtitle: 'Key objectives & core deliverables', tag: 'OCTREX · HIGHLIGHTS' },
    { title: 'Multi-Agent Framework', subtitle: 'Architecture & verification pipeline', tag: 'ENGINEERING' },
    { title: 'Provider Gateway', subtitle: 'Latency benchmarks & local models', tag: 'INFRASTRUCTURE' },
    { title: 'Security & Sandboxing', subtitle: 'Explicit permission controls', tag: 'PLATFORM' },
    { title: 'Next Steps & Milestones', subtitle: 'Q4 Rollout schedule', tag: 'ROADMAP' },
  ]

  const current = slides[activeSlide - 1] || slides[0]

  return (
    <div className="flex flex-col h-full justify-between p-4 text-slate-800 text-xs select-none">
      <div className="space-y-4">
        {/* HEADER BAR (FIGMA IMAGE 5) */}
        <div className="flex items-center justify-between">
          <div>
            <div className="text-xs font-semibold text-slate-900">{fileName}</div>
            <div className="text-[11px] text-slate-400">Slide {activeSlide} of {totalSlides}</div>
          </div>
          <button
            onClick={onDownload}
            className="flex items-center gap-1.5 rounded-xl bg-slate-100 px-3 py-1.5 text-xs font-medium text-slate-700 hover:bg-slate-200 transition-colors cursor-pointer"
          >
            <Download size={13} />
            <span>Download</span>
          </button>
        </div>

        {/* SLIDE CANVAS DISPLAY */}
        <div className="relative aspect-16/10 w-full rounded-2xl bg-white border border-slate-200/80 shadow-sm p-6 flex flex-col justify-between overflow-hidden">
          <div className="text-[10px] font-mono font-bold tracking-widest text-slate-400 uppercase">
            {current?.tag}
          </div>

          <div className="my-auto py-2">
            <h3 className="text-xl font-bold tracking-tight text-slate-900">
              {current?.title}
            </h3>
            <p className="text-xs text-slate-500 mt-1">
              {current?.subtitle}
            </p>
            <div className="mt-3 h-1 w-8 rounded-full bg-slate-900" />
          </div>

          <div className="flex gap-1.5 pt-2">
            <div className="h-1.5 w-6 rounded-full bg-slate-900" />
            <div className="h-1.5 w-4 rounded-full bg-slate-200" />
            <div className="h-1.5 w-4 rounded-full bg-slate-200" />
            <div className="h-1.5 w-4 rounded-full bg-slate-200" />
          </div>
        </div>

        {/* THUMBNAILS STRIP */}
        <div className="grid grid-cols-5 gap-1.5">
          {[1, 2, 3, 4, 5].map((s) => (
            <div
              key={s}
              onClick={() => setActiveSlide(s)}
              className={`aspect-16/10 rounded-lg p-1.5 border transition-all cursor-pointer flex flex-col justify-between ${
                activeSlide === s
                  ? 'bg-slate-50 border-slate-900 ring-1 ring-slate-900'
                  : 'bg-white border-slate-200 hover:border-slate-300'
              }`}
            >
              <div className="space-y-1">
                <div className="h-1 w-full rounded bg-slate-300" />
                <div className="h-1 w-3/4 rounded bg-slate-200" />
              </div>
              <span className="text-[8px] font-mono text-slate-400 text-right">{s}</span>
            </div>
          ))}
        </div>

        {/* DETAILS TABLE */}
        <div className="space-y-2 pt-2 border-t border-slate-100">
          <div className="text-[11px] font-semibold tracking-wider text-slate-400 uppercase">
            Details
          </div>
          <div className="space-y-1.5 text-xs">
            <div className="flex justify-between text-slate-600">
              <span>Slides</span>
              <span className="font-semibold text-slate-900">{totalSlides}</span>
            </div>
            <div className="flex justify-between text-slate-600">
              <span>Size</span>
              <span className="font-semibold text-slate-900">{size}</span>
            </div>
            <div className="flex justify-between text-slate-600">
              <span>Created</span>
              <span className="font-semibold text-slate-900">{created}</span>
            </div>
            <div className="flex justify-between text-slate-600">
              <span>Created by</span>
              <span className="font-semibold text-slate-900">{createdBy}</span>
            </div>
          </div>
        </div>
      </div>

      {/* BOTTOM ACTION BUTTON */}
      <button
        onClick={onAskOctrexToEdit}
        className="mt-6 w-full rounded-2xl bg-slate-950 py-3.5 text-xs font-medium text-white shadow-sm hover:bg-slate-800 active:scale-[0.99] transition-all flex items-center justify-center gap-2 cursor-pointer"
      >
        <Sparkles size={14} />
        <span>Ask Octrex to edit</span>
      </button>
    </div>
  )
}
