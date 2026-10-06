import React, { useState } from 'react';
import { Download, Sparkles, ZoomIn, ZoomOut, RotateCcw, Check, Image as ImageIcon } from 'lucide-react';

export type ImagePreviewProps = {
  imageName?: string;
  dimensions?: string;
  size?: string;
  format?: string;
  onUseInChat?: (imageName: string) => void;
  onDownload?: () => void;
};

export const ImagePreview: React.FC<ImagePreviewProps> = ({
  imageName = 'brand-guide.png',
  dimensions = '1280 × 1280',
  size = '240 KB',
  format = 'PNG Image',
  onUseInChat,
  onDownload
}) => {
  const [zoom, setZoom] = useState(1);
  const [used, setUsed] = useState(false);

  const handleDownload = () => {
    if (onDownload) {
      onDownload();
      return;
    }
    // SVG Canvas fallback download
    const canvas = document.createElement('canvas');
    canvas.width = 400;
    canvas.height = 400;
    const ctx = canvas.getContext('2d');
    if (ctx) {
      ctx.fillStyle = '#0f172a';
      ctx.fillRect(0, 0, 400, 400);
      ctx.fillStyle = '#38bdf8';
      ctx.font = 'bold 24px sans-serif';
      ctx.fillText('OCTREX BRAND', 100, 200);
      const url = canvas.toDataURL('image/png');
      const link = document.createElement('a');
      link.href = url;
      link.download = imageName;
      link.click();
    }
  };

  return (
    <div className="flex flex-col h-full justify-between p-4 text-slate-800 text-xs select-none overflow-y-auto custom-scrollbar">
      <div className="space-y-4">
        {/* IMAGE PREVIEW CANVAS (FIGMA IMAGE 6) */}
        <div className="relative aspect-square rounded-2xl glass-dark flex items-center justify-center p-6 shadow-xl overflow-hidden group">
          <div className="ambient-glow ambient-cyan w-64 h-64 opacity-25" />

          {/* ZOOM CONTROLS OVERLAY */}
          <div className="absolute top-3 right-3 flex items-center gap-1 glass-dark rounded-xl p-1 z-10 opacity-80 group-hover:opacity-100 transition-opacity">
            <button
              onClick={() => setZoom((z) => Math.min(2, z + 0.25))}
              className="p-1 rounded hover:bg-white/10 text-white/80 hover:text-white"
              title="Zoom in"
            >
              <ZoomIn size={13} />
            </button>
            <button
              onClick={() => setZoom((z) => Math.max(0.5, z - 0.25))}
              className="p-1 rounded hover:bg-white/10 text-white/80 hover:text-white"
              title="Zoom out"
            >
              <ZoomOut size={13} />
            </button>
            <button
              onClick={() => setZoom(1)}
              className="p-1 rounded hover:bg-white/10 text-white/80 hover:text-white"
              title="Reset zoom"
            >
              <RotateCcw size={13} />
            </button>
          </div>

          <div
            className="flex flex-col items-center justify-center transition-transform duration-200"
            style={{ transform: `scale(${zoom})` }}
          >
            <div className="w-24 h-24 rounded-3xl bg-gradient-to-tr from-cyan-500/20 to-emerald-500/20 border border-cyan-400/30 flex items-center justify-center shadow-lg mb-3">
              <ImageIcon size={42} className="text-cyan-400" />
            </div>
            <div className="font-bold text-sm text-white tracking-wide">OCTREX BRAND ASSET</div>
            <div className="text-[11px] text-slate-400 mt-1 font-mono">{dimensions}</div>
          </div>
        </div>

        {/* METADATA INFO CARD */}
        <div className="rounded-2xl glass-card border border-white/80 p-3.5 space-y-2 text-xs shadow-2xs">
          <div className="flex justify-between items-center py-0.5">
            <span className="text-slate-400">File name</span>
            <span className="font-semibold text-slate-900 font-mono text-[11px]">{imageName}</span>
          </div>
          <div className="flex justify-between items-center py-0.5">
            <span className="text-slate-400">Resolution</span>
            <span className="font-semibold text-slate-900 font-mono text-[11px]">{dimensions}</span>
          </div>
          <div className="flex justify-between items-center py-0.5">
            <span className="text-slate-400">Size & Format</span>
            <span className="font-semibold text-slate-900 font-mono text-[11px]">{size} • {format}</span>
          </div>
        </div>
      </div>

      {/* FOOTER ACTION BUTTONS */}
      <div className="pt-4 space-y-2 border-t border-slate-200/60">
        <button
          onClick={() => {
            if (onUseInChat) onUseInChat(imageName);
            setUsed(true);
            setTimeout(() => setUsed(false), 2000);
          }}
          className="w-full flex items-center justify-center gap-2 rounded-2xl bg-slate-950 hover:bg-slate-800 text-white font-medium py-3 text-xs shadow-sm hover:scale-[1.01] active:scale-[0.99] transition-all cursor-pointer"
        >
          {used ? (
            <>
              <Check size={14} className="text-emerald-400" />
              <span>Attached to chat!</span>
            </>
          ) : (
            <>
              <Sparkles size={14} className="text-cyan-400" />
              <span>Use in this chat</span>
            </>
          )}
        </button>

        <button
          onClick={handleDownload}
          className="w-full flex items-center justify-center gap-2 rounded-2xl glass-card hover:bg-white text-slate-700 hover:text-slate-900 border border-slate-200/80 font-medium py-2.5 text-xs shadow-2xs transition-all cursor-pointer"
        >
          <Download size={14} />
          <span>Download {imageName}</span>
        </button>
      </div>
    </div>
  );
};
