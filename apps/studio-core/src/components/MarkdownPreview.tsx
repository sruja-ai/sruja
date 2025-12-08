// apps/studio-core/src/components/MarkdownPreview.tsx
import React, { useEffect } from 'react';
import mermaid from 'mermaid';
import { MarkdownPreview as SharedMarkdownPreview } from '@sruja/ui/components';

let mermaidInitialized = false;
function initMermaid() {
  if (mermaidInitialized) return;
  try {
    mermaid.initialize({
      startOnLoad: false,
      theme: 'default',
      securityLevel: 'loose',
      flowchart: { useMaxWidth: true, htmlLabels: true },
    });
    mermaidInitialized = true;
  } catch (err) {
    logger.error('Failed to initialize Mermaid', {
      component: 'studio',
      action: 'init_mermaid',
      errorType: err instanceof Error ? err.constructor.name : 'unknown',
      error: err instanceof Error ? err.message : String(err),
    });
  }
}

export function MarkdownPreview({ content }: { content: string }) {
  useEffect(() => {
    initMermaid();
  }, []);

  return (
    <div
      style={{
        flex: 1,
        overflow: 'auto',
        padding: '24px',
        backgroundColor: 'var(--color-background)',
      }}
      className="markdown-preview"
    >
      <SharedMarkdownPreview content={content} />
    </div>
  );
}



