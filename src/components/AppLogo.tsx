interface AppLogoProps {
  className?: string;
  size?: number;
}

export function AppLogo({ className = "", size = 32 }: AppLogoProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 1024 1024"
      xmlns="http://www.w3.org/2000/svg"
      className={className}
    >
      <defs>
        <linearGradient id="dbGrad1" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor="#DC2626" />
          <stop offset="100%" stopColor="#991B1B" />
        </linearGradient>
        <linearGradient id="dbGrad2" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor="#2563EB" />
          <stop offset="100%" stopColor="#1D4ED8" />
        </linearGradient>
        <linearGradient id="arrowGrad" x1="0%" y1="0%" x2="100%" y2="0%">
          <stop offset="0%" stopColor="#10B981" />
          <stop offset="100%" stopColor="#059669" />
        </linearGradient>
      </defs>

      {/* Left database (red/source) */}
      <g transform="translate(100, 200)">
        <path d="M0 80v200c0 44.2 89.5 80 200 80s200-35.8 200-80V80" fill="url(#dbGrad1)" />
        <ellipse cx="200" cy="80" rx="200" ry="80" fill="url(#dbGrad1)" />
        <ellipse cx="200" cy="80" rx="160" ry="60" fill="none" stroke="white" strokeWidth="8" opacity="0.3" />
        <ellipse cx="200" cy="180" rx="200" ry="80" fill="none" stroke="white" strokeWidth="6" opacity="0.2" />
        <ellipse cx="200" cy="280" rx="200" ry="80" fill="none" stroke="white" strokeWidth="6" opacity="0.2" />
      </g>

      {/* Right database (blue/target) */}
      <g transform="translate(524, 384)">
        <path d="M0 80v200c0 44.2 89.5 80 200 80s200-35.8 200-80V80" fill="url(#dbGrad2)" />
        <ellipse cx="200" cy="80" rx="200" ry="80" fill="url(#dbGrad2)" />
        <ellipse cx="200" cy="80" rx="160" ry="60" fill="none" stroke="white" strokeWidth="8" opacity="0.3" />
        <ellipse cx="200" cy="180" rx="200" ry="80" fill="none" stroke="white" strokeWidth="6" opacity="0.2" />
        <ellipse cx="200" cy="280" rx="200" ry="80" fill="none" stroke="white" strokeWidth="6" opacity="0.2" />
      </g>

      {/* Sync arrows (green) */}
      <g fill="url(#arrowGrad)">
        <path d="M480 340 L580 340 L580 300 L680 380 L580 460 L580 420 L480 420 Z" />
        <path d="M544 560 L444 560 L444 520 L344 600 L444 680 L444 640 L544 640 Z" />
      </g>
    </svg>
  );
}
