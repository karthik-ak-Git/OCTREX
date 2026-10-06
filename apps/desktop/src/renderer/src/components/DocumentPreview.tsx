import { useState } from 'react';
import { Download, Sparkles, ChevronLeft, ChevronRight } from 'lucide-react';

export type DocumentPreviewProps = {
  fileName?: string;
  totalSlides?: number;
  currentSlideIndex?: number;
  size?: string;
  createdBy?: string;
  onAskOctrexToEdit?: (editInstruction: string) => void;
  onDownload?: () => void;
};

export const DocumentPreview: React.FC<DocumentPreviewProps> = ({
  fileName = 'roadmap-q4.pptx',
  totalSlides = 6,
  currentSlideIndex = 1,
  size = '1.2 MB',
  createdBy = 'Main agent',
  onAskOctrexToEdit,
  onDownload
}) => {
  const [activeSlide, setActiveSlide] = useState(currentSlideIndex);

  const slides = [
    { title: 'Q4 Product Roadmap', subtitle: 'Strategic milestones, priorities & ownership', tag: 'OCTREX · 2026' },
    { title: 'Executive Summary', subtitle: 'Unified Multi-Agent System & Provider Catalog', tag: 'STRATEGY' },
    { title: 'Autonomous Engine Architecture', subtitle: 'Reactive contracts, sandboxing & verification checks', tag: 'ENGINEERING' },
    { title: 'Enterprise Provider Gateway', subtitle: 'Ultra-fast inference via NIM, Groq, Gemini & local Ollama', tag: 'INFRASTRUCTURE' },
    { title: 'Platform Security Controls', subtitle: 'Explicit interactive permission matrices & audit logging', tag: 'SECURITY' },
    { title: 'Rollout Schedule & Milestones', subtitle: 'Key release dates and deployment stages', tag: 'DELIVERABLES' }
  ];

  const currentSlide = slides[activeSlide - 1] ?? slides[0] ?? {
    title: 'Slide Title',
    subtitle: 'Slide Subtitle',
    tag: 'OCTREX'
  };

  const handleDownload = () => {
    if (onDownload) {
      onDownload();
      return;
    }
    // Browser download fallback
    const blob = new Blob([`OCTREX Generated Deck: ${fileName}\n\n${slides.map((s, i) => `Slide ${i + 1}: ${s.title}\n${s.subtitle}`).join('\n\n')}`], {
      type: 'text/plain;charset=utf-8'
    });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = fileName;
    link.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="flex flex-col h-full justify-between p-4 text-slate-800 text-xs select-none overflow-y-auto custom-scrollbar">
      <div className="space-y-4">
        {/* SLIDE PREVIEW CANVAS (FIGMA IMAGE 5) */}
        <div className="relative aspect-16/10 rounded-2xl glass-dark p-6 flex flex-col justify-between shadow-xl overflow-hidden group">
          <div className="ambient-glow ambient-cyan w-48 h-48 -top-10 -right-10 opacity-30" />

          <div className="flex items-center justify-between text-[11px] font-mono text-cyan-400">
            <span className="bg-cyan-500/10 border border-cyan-500/20 px-2 py-0.5 rounded-md">
              {currentSlide.tag}
            </span>
            <span className="text-slate-400">
              {activeSlide} / {totalSlides}
            </span>
          </div>

          <div className="my-auto space-y-2 py-4">
            <h3 className="text-xl font-bold tracking-tight text-white leading-tight">
              {currentSlide.title}
            </h3>
            <p className="text-xs text-slate-300 font-light max-w-sm">
              {currentSlide.subtitle}
            </p>
          </div>

          <div className="flex items-center justify-between text-[10px] text-slate-500 font-mono pt-2 border-t border-white/10">
            <span>OCTREX CODE V4</span>
            <span>Slide {activeSlide}</span>
          </div>
        </div>

        {/* SLIDES THUMBNAIL STRIP */}
        <div>
          <div className="flex items-center justify-between text-[11px] font-semibold text-slate-400 uppercase tracking-wider mb-2">
            <span>Slides Overview ({totalSlides})</span>
            <div className="flex items-center gap-1">
              <button
                onClick={() => setActiveSlide((prev) => Math.max(1, prev - 1))}
                disabled={activeSlide === 1}
                className="p-1 rounded hover:bg-slate-100 disabled:opacity-30 cursor-pointer"
              >
                <ChevronLeft size={14} />
              </button>
              <button
                onClick={() => setActiveSlide((prev) => Math.min(totalSlides, prev + 1))}
                disabled={activeSlide === totalSlides}
                className="p-1 rounded hover:bg-slate-100 disabled:opacity-30 cursor-pointer"
              >
                <ChevronRight size={14} />
              </button>
            </div>
          </div>

          <div className="grid grid-cols-3 gap-2">
            {slides.map((slide, idx) => (
              <div
                key={idx}
                onClick={() => setActiveSlide(idx + 1)}
                className={`aspect-16/10 rounded-xl p-2 flex flex-col justify-between border transition-all cursor-pointer ${
                  activeSlide === idx + 1
                    ? 'bg-slate-900 text-white border-slate-900 shadow-md ring-2 ring-cyan-500/30'
                    : 'glass-card text-slate-700 border-slate-200/80 hover:border-slate-400'
                }`}
              >
                <span className="text-[10px] font-bold font-mono opacity-60">0{idx + 1}</span>
                <span className="text-[10px] font-semibold truncate leading-tight">{slide.title}</span>
              </div>
            ))}
          </div>
        </div>

        {/* METADATA DETAILS */}
        <div className="rounded-2xl glass-card border border-white/80 p-3.5 space-y-2 text-xs shadow-2xs">
          <div className="flex justify-between items-center py-0.5">
            <span className="text-slate-400">File name</span>
            <span className="font-semibold text-slate-900 font-mono text-[11px]">{fileName}</span>
          </div>
          <div className="flex justify-between items-center py-0.5">
            <span className="text-slate-400">Format & Size</span>
            <span className="font-semibold text-slate-900 font-mono text-[11px]">{size}</span>
          </div>
          <div className="flex justify-between items-center py-0.5">
            <span className="text-slate-400">Generated by</span>
            <span className="font-semibold text-slate-900">{createdBy}</span>
          </div>
        </div>
      </div>

      {/* FOOTER ACTIONS */}
      <div className="pt-4 space-y-2 border-t border-slate-200/60">
        <button
          onClick={() => {
            if (onAskOctrexToEdit) {
              onAskOctrexToEdit(`Please modify slide ${activeSlide} ("${currentSlide.title}"): `);
            }
          }}
          className="w-full flex items-center justify-center gap-2 rounded-2xl bg-slate-950 hover:bg-slate-800 text-white font-medium py-3 text-xs shadow-sm hover:scale-[1.01] active:scale-[0.99] transition-all cursor-pointer"
        >
          <Sparkles size={14} className="text-cyan-400" />
          <span>Ask Octrex to edit slide {activeSlide}</span>
        </button>

        <button
          onClick={handleDownload}
          className="w-full flex items-center justify-center gap-2 rounded-2xl glass-card hover:bg-white text-slate-700 hover:text-slate-900 border border-slate-200/80 font-medium py-2.5 text-xs shadow-2xs transition-all cursor-pointer"
        >
          <Download size={14} />
          <span>Download {fileName}</span>
        </button>
      </div>
    </div>
  );
};
