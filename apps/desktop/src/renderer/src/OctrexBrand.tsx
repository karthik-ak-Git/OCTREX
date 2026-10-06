import React from 'react'

export type BrandAssetProps = {
  size?: number
  className?: string
}

/**
 * High-definition vector OCTREX Brand Logo.
 * Features an 8-sided quantum/neural matrix icon with sleek gradient accents.
 */
export function OctrexLogo({ size = 24, className = '' }: BrandAssetProps): React.JSX.Element {
  return (
    <svg
      className={`octrex-logo ${className}`}
      width={size}
      height={size}
      viewBox="0 0 32 32"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
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
      {/* Octagonal Outer Ring */}
      <path
        d="M10 3.5L22 3.5L28.5 10L28.5 22L22 28.5L10 28.5L3.5 22L3.5 10L10 3.5Z"
        stroke="url(#octrexGrad)"
        strokeWidth="2.2"
        strokeLinejoin="round"
        fill="#131517"
      />
      {/* Inner Quantum Core / Code Nexus */}
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

/**
 * Hero OCTREX Code Symbol for Home screen and splash cards.
 */
export function OctrexCodeSymbol({ size = 52, className = '' }: BrandAssetProps): React.JSX.Element {
  return (
    <div
      className={`octrex-code-symbol-wrap ${className}`}
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        justifyContent: 'center',
        width: size,
        height: size,
      }}
    >
      <svg
        className="octrex-code-symbol"
        width={size}
        height={size}
        viewBox="0 0 64 64"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        aria-hidden="true"
      >
        <defs>
          <linearGradient id="octrexHeroGrad" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#38bdf8" />
            <stop offset="45%" stopColor="#6366f1" />
            <stop offset="100%" stopColor="#10b981" />
          </linearGradient>
          <filter id="octrexGlow" x="-20%" y="-20%" width="140%" height="140%">
            <feGaussianBlur stdDeviation="3" result="blur" />
            <feComposite in="SourceGraphic" in2="blur" operator="over" />
          </filter>
        </defs>
        {/* Outer Hex/Octagon Background */}
        <path
          d="M20 7L44 7L57 20L57 44L44 57L20 57L7 44L7 20L20 7Z"
          fill="#181a1c"
          stroke="url(#octrexHeroGrad)"
          strokeWidth="2.5"
          strokeLinejoin="round"
          filter="url(#octrexGlow)"
        />
        {/* Inner Tech Lines */}
        <path
          d="M26 23L18 32L26 41"
          stroke="#38bdf8"
          strokeWidth="3.2"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
        <path
          d="M38 23L46 32L38 41"
          stroke="#10b981"
          strokeWidth="3.2"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
        <path
          d="M35 21L29 43"
          stroke="#818cf8"
          strokeWidth="2.6"
          strokeLinecap="round"
        />
      </svg>
    </div>
  )
}

// Backward-compatibility aliases
export const AltrexLogo = OctrexLogo
export const AltrexCodeSymbol = OctrexCodeSymbol
