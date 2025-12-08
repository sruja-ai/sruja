import React, { useRef, useState, useEffect } from 'react';
import { createPortal } from 'react-dom';

export function Tooltip({ content, children }: { content: string; children: React.ReactNode }) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState<{ top: number; left: number } | null>(null);
  const [isVisible, setIsVisible] = useState(false);

  const updatePosition = () => {
    if (containerRef.current) {
      // Get the actual button/child element, not just the wrapper
      const childElement = containerRef.current.firstElementChild as HTMLElement;
      const elementToMeasure = childElement || containerRef.current;
      const rect = elementToMeasure.getBoundingClientRect();
      // For fixed positioning, use viewport coordinates directly
      setPosition({
        top: rect.top - 8,
        left: rect.left + rect.width / 2,
      });
    }
  };

  const handleMouseEnter = () => {
    setIsVisible(true);
    // Use requestAnimationFrame to ensure DOM is ready
    requestAnimationFrame(() => {
      updatePosition();
    });
  };

  useEffect(() => {
    if (isVisible) {
      // Double RAF to ensure layout is complete
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          updatePosition();
        });
      });
      window.addEventListener('scroll', updatePosition, true);
      window.addEventListener('resize', updatePosition);
      return () => {
        window.removeEventListener('scroll', updatePosition, true);
        window.removeEventListener('resize', updatePosition);
      };
    } else {
      setPosition(null);
    }
  }, [isVisible]);

  return (
    <>
      <div
        ref={containerRef}
        className="relative group inline-flex"
        onMouseEnter={handleMouseEnter}
        onMouseLeave={() => setIsVisible(false)}
      >
        {children}
      </div>
      {isVisible && position && typeof document !== 'undefined' && createPortal(
        <div
          className="fixed rounded bg-[var(--color-neutral-900)] text-[var(--color-background)] text-xs px-3 py-2 shadow-lg z-[9999] max-w-xs"
          style={{
            top: `${position.top}px`,
            left: `${position.left}px`,
            transform: 'translate(-50%, -100%)',
            pointerEvents: 'auto',
          }}
        >
          <div className="whitespace-pre-line">
            {content.split('\n').map((line, idx) => {
              if (line.startsWith('Docs: ')) {
                const url = line.replace('Docs: ', '');
                return (
                  <div key={idx} className="mt-1 pt-1 border-t border-[var(--color-neutral-700)]">
                    <a
                      href={url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-[var(--color-info-500)] hover:underline"
                      onClick={(e) => e.stopPropagation()}
                    >
                      📖 View Docs
                    </a>
                  </div>
                );
              }
              return <div key={idx}>{line}</div>;
            })}
          </div>
          <div className="absolute left-1/2 top-full -translate-x-1/2 w-0 h-0 border-l-4 border-l-transparent border-r-4 border-r-transparent border-t-4 border-t-[var(--color-neutral-900)]" />
        </div>,
        document.body
      )}
    </>
  );
}

