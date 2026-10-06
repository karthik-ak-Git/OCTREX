import { Download, Sparkles, Image as ImageIcon } from 'lucide-react'
import { OctrexLogo } from '../OctrexBrand'

export type ImagePreviewProps = {
  fileName?: string
  uploadedBy?: string
  dimensions?: string
  format?: string
  size?: string
  source?: string
  onDownload?: () => void
  onUseInChat?: () => void
}

export function ImagePreview({
  fileName = 'brand-guide.png',
  uploadedBy = 'Uploaded by you',
  dimensions = '1280 × 1280',
  format = 'PNG',
  size = '240 KB',
  source = 'Uploaded',
  onDownload,
  onUseInChat,
}: ImagePreviewProps) {
  return (
    <div className="flex flex-col h-full justify-between p-4 text-slate-800 text-xs select-none">
      <div className="space-y-4">
        {/* HEADER BAR */}
        <div className="flex items-center justify-between">
          <div>
            <div className="text-xs font-semibold text-slate-900">{fileName}</div>
            <div className="text-[11px] text-slate-400">{uploadedBy} · {dimensions}</div>
          </div>
          <button
            onClick={onDownload}
            className="flex items-center gap-1.5 rounded-xl bg-slate-100 px-3 py-1.5 text-xs font-medium text-slate-700 hover:bg-slate-200 transition-colors"
          >
            <Download size={13} />
            <span>Download</span>
          </button>
        </div>

        {/* IMAGE PREVIEW CANVAS */}
        <div className="relative aspect-square w-full rounded-2xl bg-white border border-slate-200/80 shadow-sm p-8 flex flex-col items-center justify-center overflow-hidden">
          <div className="flex h-32 w-32 items-center justify-center rounded-full bg-slate-100/80 border border-slate-200 shadow-inner">
            <OctrexLogo size={64} className="text-slate-900" />
          </div>

          <div className="mt-8 flex flex-col items-center gap-1.5 w-full max-w-[180px]">
            <div className="h-2 w-full rounded-full bg-slate-200" />
            <div className="h-2 w-2/3 rounded-full bg-slate-200/80" />
          </div>
        </div>

        {/* DETAILS TABLE */}
        <div className="space-y-2 pt-2 border-t border-slate-100">
          <div className="text-[11px] font-semibold tracking-wider text-slate-400 uppercase">
            Details
          </div>
          <div className="space-y-1.5 text-xs">
            <div className="flex justify-between text-slate-600">
              <span>Format</span>
              <span className="font-semibold text-slate-900">{format}</span>
            </div>
            <div className="flex justify-between text-slate-600">
              <span>Dimensions</span>
              <span className="font-semibold text-slate-900">{dimensions}</span>
            </div>
            <div className="flex justify-between text-slate-600">
              <span>Size</span>
              <span className="font-semibold text-slate-900">{size}</span>
            </div>
            <div className="flex justify-between text-slate-600">
              <span>Source</span>
              <span className="font-semibold text-slate-900">{source}</span>
            </div>
          </div>
        </div>
      </div>

      {/* BOTTOM ACTION BUTTON */}
      <button
        onClick={onUseInChat}
        className="mt-6 w-full rounded-2xl bg-slate-950 py-3.5 text-xs font-medium text-white shadow-sm hover:bg-slate-800 active:scale-[0.99] transition-all flex items-center justify-center gap-2"
      >
        <Sparkles size={14} />
        <span>Use in this chat</span>
      </button>
    </div>
  )
}
