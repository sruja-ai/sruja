// apps/studio-core/src/components/DocumentationPanel.tsx
import React, { useEffect, useRef, useMemo, useCallback } from 'react';
import { ExternalLink } from 'lucide-react';
import { C4Overview } from './C4Overview';
import { getAllConcepts } from '@sruja/shared';

interface DocumentationPanelProps {
  isOpen: boolean;
  onClose?: () => void; // Optional, used when panel has its own close button
  selectedNodeType: string | null;
  selectedNodeId?: string;
  selectedNodeLabel?: string;
}

export const DocumentationPanel: React.FC<DocumentationPanelProps> = React.memo(({
  isOpen,
  selectedNodeType,
  selectedNodeId,
  selectedNodeLabel,
}) => {
  const panelRef = useRef<HTMLDivElement>(null);
  const sectionRefs = useRef<Record<string, HTMLDivElement | null>>({});

  // Memoize documentation sections (expensive computation)
  const docSections = useMemo(() => getAllConcepts(), []);

  // Scroll to selected section when node type changes
  useEffect(() => {
    if (!selectedNodeType) return;
    
    const element = sectionRefs.current[selectedNodeType];
    if (!element || !panelRef.current) return;

    // Use requestAnimationFrame and setTimeout to ensure DOM is ready and panel is visible
    const scrollTimeout = setTimeout(() => {
      if (!panelRef.current || !element) return;
      
      // Check if panel is actually visible (not just mounted)
      const panelRect = panelRef.current.getBoundingClientRect();
      if (panelRect.width === 0 || panelRect.height === 0) {
        // Panel not visible yet, try again
        setTimeout(() => {
          if (!panelRef.current || !element) return;
          scrollToSection(element, panelRef.current);
        }, 200);
        return;
      }
      
      scrollToSection(element, panelRef.current);
    }, 150);

    return () => clearTimeout(scrollTimeout);
  }, [selectedNodeType]);

  // Memoize scroll function
  const scrollToSection = useCallback((element: HTMLDivElement, panel: HTMLDivElement) => {
    const elementRect = element.getBoundingClientRect();
    const panelRect = panel.getBoundingClientRect();
    
    // Check if element header (top) is visible in viewport
    const headerVisible = 
      elementRect.top >= panelRect.top &&
      elementRect.top <= panelRect.bottom;

    if (!headerVisible) {
      // Calculate scroll position to show the header near the top
      // Get the element's position relative to the scrollable container
      const elementOffsetTop = element.offsetTop;
      
      // Account for panel padding (p-4 = 16px)
      // Scroll so the element header appears near the top with some padding
      const targetScrollTop = elementOffsetTop - 16; // 16px padding from top

      panel.scrollTo({
        top: Math.max(0, targetScrollTop), // Ensure we don't scroll to negative
        behavior: 'smooth',
      });
    } else {
      // Header is visible but might be too low, ensure it's near the top
      const currentScrollTop = panel.scrollTop;
      const elementOffsetTop = element.offsetTop;
      const idealScrollTop = elementOffsetTop - 16;
      
      // If header is visible but not at ideal position, adjust
      if (Math.abs(currentScrollTop - idealScrollTop) > 50) {
        panel.scrollTo({
          top: Math.max(0, idealScrollTop),
          behavior: 'smooth',
        });
      }
    }
  }, []);

  // When used in sidebar, don't check isOpen (parent handles visibility)
  // When used standalone, check isOpen
  if (!isOpen && typeof isOpen === 'boolean') return null;

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <div
        ref={panelRef}
        className="flex-1 overflow-y-auto p-4 space-y-6 min-h-0"
      >
        <C4Overview />
        {docSections.map((section) => {
          const isSelected = selectedNodeType === section.id;
          return (
            <div
              key={section.id}
              ref={(el) => {
                sectionRefs.current[section.id] = el;
              }}
              id={`doc-section-${section.id}`}
              className={`pb-6 border-b border-[var(--color-border)] last:border-b-0 relative ${
                isSelected ? 'bg-[var(--color-primary-50)]/30' : ''
              }`}
            >
              {isSelected && (
                <div className="absolute left-0 top-0 bottom-0 w-1 bg-[var(--color-primary)] rounded-r" />
              )}
              <div className={`flex items-start justify-between mb-2 ${isSelected ? 'pl-3' : ''}`}>
                <h4 className="m-0 text-base font-semibold text-[var(--color-text-primary)]">
                  {section.title}
                </h4>
                <a
                  href={section.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-[var(--color-info-500)] hover:underline flex items-center gap-1 text-xs"
                  title="View full documentation"
                >
                  <ExternalLink size={12} />
                </a>
              </div>
              <div className={isSelected ? 'pl-3' : ''}>
                {isSelected && selectedNodeLabel && (
                  <div className="mb-2 px-2 py-1 bg-[var(--color-primary-50)] rounded text-xs font-mono text-[var(--color-text-secondary)]">
                    {selectedNodeId || selectedNodeLabel}
                  </div>
                )}
                {section.summary && (
                  <p className="text-sm text-[var(--color-text-secondary)] mb-3 italic">
                    {section.summary}
                  </p>
                )}
                <div className="text-sm leading-relaxed text-[var(--color-text-primary)]">
                {section.description.split('\n').map((line, idx) => {
                  // Handle code blocks
                  if (line.trim().startsWith('```')) {
                    return <div key={idx} className="my-2" />;
                  }
                  // Handle headings
                  if (line.startsWith('## ')) {
                    return (
                      <h5 key={idx} className="text-sm font-semibold mt-4 mb-2 text-[var(--color-text-primary)]">
                        {line.replace('## ', '')}
                      </h5>
                    );
                  }
                  if (line.startsWith('### ')) {
                    return (
                      <h6 key={idx} className="text-xs font-semibold mt-3 mb-1.5 text-[var(--color-text-primary)]">
                        {line.replace('### ', '')}
                      </h6>
                    );
                  }
                  // Handle code inline
                  if (line.includes('`')) {
                    const parts = line.split(/(`[^`]+`)/g);
                    return (
                      <p key={idx} className="mb-2">
                        {parts.map((part, pIdx) => {
                          if (part.startsWith('`') && part.endsWith('`')) {
                            return (
                              <code
                                key={pIdx}
                                className="bg-[var(--color-surface)] px-1.5 py-0.5 rounded text-xs font-mono text-[var(--color-text-primary)]"
                              >
                                {part.slice(1, -1)}
                              </code>
                            );
                          }
                          // Handle bold
                          const boldParts = part.split(/(\*\*[^*]+\*\*)/g);
                          return (
                            <span key={pIdx}>
                              {boldParts.map((boldPart, bIdx) => {
                                if (boldPart.startsWith('**') && boldPart.endsWith('**')) {
                                  return (
                                    <strong key={bIdx} className="font-semibold">
                                      {boldPart.slice(2, -2)}
                                    </strong>
                                  );
                                }
                                return <span key={bIdx}>{boldPart}</span>;
                              })}
                            </span>
                          );
                        })}
                      </p>
                    );
                  }
                  // Handle bold text
                  if (line.includes('**')) {
                    const parts = line.split(/(\*\*[^*]+\*\*)/g);
                    return (
                      <p key={idx} className="mb-2">
                        {parts.map((part, pIdx) => {
                          if (part.startsWith('**') && part.endsWith('**')) {
                            return (
                              <strong key={pIdx} className="font-semibold">
                                {part.slice(2, -2)}
                              </strong>
                            );
                          }
                          return <span key={pIdx}>{part}</span>;
                        })}
                      </p>
                    );
                  }
                  // Regular text
                  if (line.trim()) {
                    return (
                      <p key={idx} className="mb-2">
                        {line}
                      </p>
                    );
                  }
                  return <br key={idx} />;
                })}
              </div>
                {section.keyPoints && section.keyPoints.length > 0 && (
                  <div className="mt-4">
                    <h6 className="text-xs font-semibold mb-2 text-[var(--color-text-primary)]">Key Points:</h6>
                    <ul className="list-disc list-inside space-y-1 text-xs text-[var(--color-text-secondary)]">
                      {section.keyPoints.map((point, idx) => (
                        <li key={idx}>{point}</li>
                      ))}
                    </ul>
                  </div>
                )}
                {section.examples && section.examples.length > 0 && (
                  <div className="mt-4">
                    <h6 className="text-xs font-semibold mb-2 text-[var(--color-text-primary)]">Examples:</h6>
                    <div className="space-y-1">
                      {section.examples.map((example, idx) => (
                        <code
                          key={idx}
                          className="block bg-[var(--color-surface)] px-2 py-1 rounded text-xs font-mono text-[var(--color-text-primary)]"
                        >
                          {example}
                        </code>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
});
