// packages/ui/src/components/SrujaLoader.tsx
import React from 'react';

export interface SrujaLoaderProps {
  size?: number;
  className?: string;
}

export function SrujaLoader({ size = 48, className = '' }: SrujaLoaderProps) {
  const uniqueId = React.useId();
  const infGradId = `infGrad-${uniqueId}`;
  const sGradId = `sGrad-${uniqueId}`;

  return (
    <div 
      className={`sruja-loader ${className}`}
      style={{ 
        width: size, 
        height: size,
        display: 'inline-flex',
        alignItems: 'center',
        justifyContent: 'center'
      }}
    >
      <svg 
        width={size} 
        height={size} 
        viewBox="0 0 300 300" 
        xmlns="http://www.w3.org/2000/svg"
        className="sruja-loader-svg"
      >
        <defs>
          <linearGradient id={infGradId} x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#7C3AED"/>
            <stop offset="100%" stopColor="#2563EB"/>
          </linearGradient>

          <linearGradient id={sGradId} x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%" stopColor="#DB2777"/>
            <stop offset="100%" stopColor="#F472B6"/>
          </linearGradient>
        </defs>

        <g className="sruja-loader-horizontal">
          <path 
            d="M50,150 C50,100 135,100 150,150 C165,200 250,200 250,150 C250,100 165,100 150,150 C135,200 50,200 50,150" 
            fill="none" 
            stroke={`url(#${infGradId})`} 
            strokeWidth="15" 
            strokeLinecap="round"
          />
        </g>

        <g className="sruja-loader-vertical">
          <path 
            d="M150,50 C100,50 100,135 150,150 C200,165 200,250 150,250 C100,250 100,165 150,150 C200,135 200,50 150,50" 
            fill="none" 
            stroke={`url(#${infGradId})`} 
            strokeWidth="15" 
            strokeLinecap="round"
          />
        </g>

        <g className="sruja-loader-s-highlight-vertical">
          <path 
            d="M150,50 C100,50 100,135 150,150 C200,165 200,250 150,250" 
            fill="none" 
            stroke={`url(#${sGradId})`} 
            strokeWidth="18" 
            strokeLinecap="round"
          />
        </g>

        <g className="sruja-loader-s-highlight-horizontal">
          <path 
            d="M50,150 C50,200 135,200 150,150 C165,100 250,100 250,150" 
            fill="none" 
            stroke={`url(#${sGradId})`} 
            strokeWidth="18" 
            strokeLinecap="round"
          />
        </g>

        <circle cx="150" cy="150" r="12" fill="#2563EB" className="sruja-loader-center"/>
        <circle cx="150" cy="150" r="10" fill="white" className="sruja-loader-center-inner"/>
      </svg>
    </div>
  );
}

