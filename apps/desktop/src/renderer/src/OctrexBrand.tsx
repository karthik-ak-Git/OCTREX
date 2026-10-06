export function OctrexLogo({ size = 28, className = '' }: { size?: number; className?: string }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 32 32"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className={`octrex-logo ${className}`}
      aria-hidden="true"
    >
      <defs>
        <linearGradient id="octrexGrad" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor="#38bdf8" />
          <stop offset="50%" stopColor="#3b82f6" />
          <stop offset="100%" stopColor="#10b981" />
        </linearGradient>
        <linearGradient id="octrexCoreGrad" x1="0%" y1="100%" x2="100%" y2="0%">
          <stop offset="0%" stopColor="#06b6d4" />
          <stop offset="100%" stopColor="#6366f1" />
        </linearGradient>
      </defs>
      <path
        d="M10 3.5L22 3.5L28.5 10L28.5 22L22 28.5L10 28.5L3.5 22L3.5 10L10 3.5Z"
        fill="#131517"
        stroke="url(#octrexGrad)"
        strokeWidth="2.2"
        strokeLinejoin="round"
      />
      <path
        d="M13 11.5L9 16L13 20.5"
        stroke="#38bdf8"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path
        d="M19 11.5L23 16L19 20.5"
        stroke="#10b981"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <circle cx="16" cy="16" r="2.2" fill="url(#octrexCoreGrad)" />
    </svg>
  )
}

export function OctrexCodeSymbol({ size = 44, className = '' }: { size?: number; className?: string }) {
  return <OctrexLogo size={size} className={className} />
}
