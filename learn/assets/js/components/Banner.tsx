// Site-wide banner component for pre-alpha and fork messaging
import React, { useState } from 'react';
import { getStorageItem, setStorageItem } from '../utils/storage';

const BANNER_DISMISSED_KEY = 'sruja-banner-dismissed';

export function Banner() {
  const [dismissed, setDismissed] = useState(false);

  // Check if user has dismissed the banner before (using safe storage)
  React.useEffect(() => {
    const isDismissed = getStorageItem(BANNER_DISMISSED_KEY) === 'true';
    if (isDismissed) {
      setDismissed(true);
    }
  }, []);


  const handleDismiss = () => {
    setDismissed(true);
    setStorageItem(BANNER_DISMISSED_KEY, 'true');
  };

  if (dismissed) {
    return null;
  }

  return (
    <div className="sruja-corner-banner fixed top-4 right-4 z-[1100] shadow-2xl">
      <div className="corner-banner-wrapper relative">
        {/* Ribbon tail effect */}
        <div className="corner-banner-tail absolute -bottom-2 right-0 w-0 h-0 border-l-[12px] border-l-transparent border-t-[12px] border-t-violet-700"></div>
        
        {/* Main banner */}
        <div className="corner-banner-content bg-gradient-to-br from-violet-600 via-purple-600 to-violet-600 text-white px-4 py-2.5 sm:px-5 sm:py-3 rounded-lg relative overflow-hidden">
          {/* Animated background pattern */}
          <div className="absolute inset-0 opacity-20">
            <div className="absolute inset-0" style={{
              backgroundImage: 'repeating-linear-gradient(45deg, transparent, transparent 10px, rgba(255,255,255,0.1) 10px, rgba(255,255,255,0.1) 20px)'
            }}></div>
          </div>
          
          <div className="relative flex items-center gap-2.5 sm:gap-3">
            <span className="inline-flex items-center gap-1.5 px-2 py-0.5 bg-white/20 backdrop-blur-sm rounded-full text-xs sm:text-sm font-bold whitespace-nowrap">
              <span className="w-1.5 h-1.5 sm:w-2 sm:h-2 bg-white rounded-full animate-pulse"></span>
              PRE-ALPHA
            </span>
            <a
              href="https://github.com/sruja-ai/sruja"
              target="_blank"
              rel="noopener noreferrer"
              className="text-xs sm:text-sm font-semibold hover:opacity-90 transition-opacity whitespace-nowrap underline decoration-white/50 hover:decoration-white"
            >
              Fork on GitHub
            </a>
          </div>
          
          {/* Close button */}
          <button
            onClick={handleDismiss}
            className="banner-close absolute -top-1 -right-1 w-5 h-5 sm:w-6 sm:h-6 flex items-center justify-center rounded-full bg-white/30 hover:bg-white/40 transition-colors text-white focus:outline-none focus:ring-2 focus:ring-white/50 shadow-lg"
            aria-label="Dismiss banner"
          >
            <svg
              className="w-3 h-3 sm:w-4 sm:h-4"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={3}
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        </div>
      </div>
    </div>
  );
}

