// packages/ui/src/components/Logo.tsx
import { CSSProperties } from 'react';
import { cn } from '../utils/cn';

export interface LogoProps {
  /** Size of the logo in pixels */
  size?: number;
  /** Whether to show the logo with rotation animation */
  isLoading?: boolean;
  /** Additional CSS classes */
  className?: string;
  /** Additional inline styles */
  style?: CSSProperties;
  /** Alt text for accessibility */
  alt?: string;
}

// Embedded SVG logo component
const LogoSVG = ({ size = 32 }: { size: number }) => (
  <svg 
    width={size} 
    height={size} 
    viewBox="0 0 300 300" 
    xmlns="http://www.w3.org/2000/svg"
    className="inline-block"
  >
    <defs>
      <linearGradient id="infGrad" x1="0%" y1="0%" x2="100%" y2="100%">
        <stop offset="0%" stopColor="#7C3AED" />
        <stop offset="100%" stopColor="#2563EB" />
      </linearGradient>
      <linearGradient id="sGrad" x1="0%" y1="0%" x2="100%" y2="0%">
        <stop offset="0%" stopColor="#DB2777" />
        <stop offset="100%" stopColor="#F472B6" />
      </linearGradient>
    </defs>
    <path
      d="M50,150 C50,100 135,100 150,150 C165,200 250,200 250,150 C250,100 165,100 150,150 C135,200 50,200 50,150"
      fill="none"
      stroke="url(#infGrad)"
      strokeWidth="15"
      strokeLinecap="round"
    />
    <path
      d="M150,50 C100,50 100,135 150,150 C200,165 200,250 150,250 C100,250 100,165 150,150 C200,135 200,50 150,50"
      fill="none"
      stroke="url(#infGrad)"
      strokeWidth="15"
      strokeLinecap="round"
    />
    <path
      d="M150,50 C100,50 100,135 150,150 C200,165 200,250 150,250"
      fill="none"
      stroke="url(#sGrad)"
      strokeWidth="18"
      strokeLinecap="round"
    />
    <path
      d="M50,150 C50,200 135,200 150,150 C165,100 250,100 250,150"
      fill="none"
      stroke="url(#sGrad)"
      strokeWidth="18"
      strokeLinecap="round"
    />
    <circle cx="150" cy="150" r="12" fill="#2563EB" />
    <circle cx="150" cy="150" r="10" fill="white" />
  </svg>
);

export function Logo({
  size = 32,
  isLoading = false,
  className = '',
  style = {},
  alt = 'Sruja Logo',
}: LogoProps) {
  return (
    <span
      className={cn(isLoading && 'logo-rotating', className)}
      style={style}
      role="img"
      aria-label={alt}
    >
      <LogoSVG size={size} />
    </span>
  );
}
