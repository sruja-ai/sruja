import { Download, Braces, CodeXml, FileText, Maximize2, Code } from 'lucide-react';
import { Button, MonacoEditor } from '@sruja/ui';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { SrujaLoader } from '@sruja/ui';
import { MermaidDiagram } from '@sruja/ui/components';
import type { PreviewFormat, ArchitectureJSON, ExpandedMermaid } from '../types';

import type { PaneType } from '../types';

interface PreviewPanelsProps {
  previewFormat: PreviewFormat;
  setPreviewFormat: (format: PreviewFormat) => void;
  activePane: PaneType;
  setActivePane: (pane: PaneType) => void;
  savePaneToStorage: (pane: PaneType) => void;
  viewerContainerRef: React.RefObject<HTMLDivElement | null>;
  // JSON Preview
  archData: ArchitectureJSON | null;
  isParsingDsl: boolean;
  onDownloadJson: () => void;
  // HTML Preview
  htmlPreview: string;
  isGeneratingHtml: boolean;
  previewFrameRef: React.RefObject<HTMLIFrameElement | null>;
  onDownloadHtml: () => void;
  previewErrors?: Array<{ type: string; message?: string; filename?: string; lineno?: number; colno?: number; stack?: string }>;
  // Markdown Preview
  markdownPreview: string;
  isGeneratingMarkdown: boolean;
  onDownloadMarkdown: () => void;
  expandedMermaid: ExpandedMermaid | null;
  mermaidZoom: number;
  onMermaidExpand: (svg: string, code: string) => void;
  onMermaidZoomIn: () => void;
  onMermaidZoomOut: () => void;
  onMermaidClose: () => void;
}

export function PreviewPanels({
  previewFormat,
  setPreviewFormat,
  activePane,
  setActivePane,
  savePaneToStorage,
  viewerContainerRef,
  archData,
  isParsingDsl,
  onDownloadJson,
  htmlPreview,
  isGeneratingHtml,
  previewFrameRef,
  onDownloadHtml,
  previewErrors,
  markdownPreview,
  isGeneratingMarkdown,
  onDownloadMarkdown,
  expandedMermaid,
  mermaidZoom,
  onMermaidExpand,
  onMermaidZoomIn,
  onMermaidZoomOut,
  onMermaidClose,
}: PreviewPanelsProps) {
  const title =
    previewFormat === 'diagram' ? 'Diagram Preview' :
      previewFormat === 'json' ? 'JSON Preview' :
        previewFormat === 'preview' ? 'HTML Preview' :
          previewFormat === 'markdown' ? 'Markdown Preview' : 'Preview';

  const canDownload =
    previewFormat === 'json' ||
    previewFormat === 'preview' ||
    previewFormat === 'markdown';

  const handleDownload = () => {
    if (previewFormat === 'json') return onDownloadJson();
    if (previewFormat === 'preview') return onDownloadHtml();
    if (previewFormat === 'markdown') return onDownloadMarkdown();
  };
  return (
    <>
      {/* Preview Toolbar */}
      <div className="flex items-center justify-between vscode-toolbar" style={{ borderBottom: '1px solid var(--vscode-panel-border)' }}>
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium">{title}</span>
        </div>
        <div className="flex items-center gap-1">
          <button
            className={`vscode-toolbar-button ${previewFormat === 'diagram' ? 'primary' : ''}`}
            onClick={() => setPreviewFormat('diagram')}
            title="Diagram"
          >
            <Code className="vscode-icon" />
          </button>
          {/* Format Selection */}
          <button
            className={`vscode-toolbar-button ${previewFormat === 'json' ? 'primary' : ''}`}
            onClick={() => setPreviewFormat('json')}
            title="JSON"
          >
            <Braces className="vscode-icon" />
          </button>
          <button
            className={`vscode-toolbar-button ${previewFormat === 'preview' ? 'primary' : ''}`}
            onClick={() => setPreviewFormat('preview')}
            title="HTML"
          >
            <CodeXml className="vscode-icon" />
          </button>
          <button
            className={`vscode-toolbar-button ${previewFormat === 'markdown' ? 'primary' : ''}`}
            onClick={() => setPreviewFormat('markdown')}
            title="Markdown"
          >
            <FileText className="vscode-icon" />
          </button>

          {/* Toggle to preview only */}
          <div className="vscode-toolbar-separator" />
          <button
            className="vscode-toolbar-button"
            onClick={() => {
              setActivePane(previewFormat);
              savePaneToStorage(previewFormat);
            }}
            title="Open preview in full screen"
          >
            <Maximize2 className="vscode-icon" />
          </button>
          <button
            className="vscode-toolbar-button"
            onClick={handleDownload}
            disabled={!canDownload || (previewFormat === 'preview' && (!htmlPreview || isGeneratingHtml)) || (previewFormat === 'markdown' && isGeneratingMarkdown) || (previewFormat === 'json' && (!archData || isParsingDsl))}
            title="Download current preview"
          >
            <Download className="vscode-icon" />
            <span>Download</span>
          </button>
        </div>
      </div>
      {/* Preview Content */}
      <div className="flex-1 overflow-hidden">
        {previewFormat === 'diagram' && (
          <div className="w-full h-full">
            <div className="w-full h-full" ref={viewerContainerRef} />
          </div>
        )}
        {previewFormat === 'json' && (
          <div className="w-full h-full flex flex-col">
            {isParsingDsl ? (
              <div className="flex-1 flex items-center justify-center">
                <div className="text-center">
                  <SrujaLoader size={48} />
                  <p className="mt-4 text-sm" style={{ color: 'var(--vscode-descriptionForeground)' }}>Parsing DSL to JSON...</p>
                </div>
              </div>
            ) : archData ? (
              <div className="flex-1 overflow-hidden">
                <MonacoEditor
                  value={JSON.stringify(archData, null, 2)}
                  onChange={() => { }}
                  language="json"
                  theme="vs-dark"
                  height="100%"
                  options={{ readOnly: true, wordWrap: 'on', minimap: { enabled: false } }}
                />
              </div>
            ) : (
              <div className="flex-1 flex items-center justify-center" style={{ color: 'var(--vscode-descriptionForeground)' }}>
                No architecture data available. Parse DSL to generate JSON.
              </div>
            )}
          </div>
        )}
        {previewFormat === 'preview' && (
          <div className="w-full h-full flex flex-col">
            {isGeneratingHtml ? (
              <div className="flex-1 flex items-center justify-center">
                <div className="text-center">
                  <SrujaLoader size={48} />
                  <p className="mt-4 text-sm" style={{ color: 'var(--vscode-descriptionForeground)' }}>Generating HTML preview...</p>
                </div>
              </div>
            ) : (
              <iframe
                ref={previewFrameRef}
                className="flex-1 w-full border-0"
                title="HTML Preview"
                sandbox="allow-same-origin allow-scripts allow-forms allow-popups allow-modals"
              />
            )}
            {!isGeneratingHtml && previewErrors && previewErrors.length > 0 && (
              <div className="p-2 border-t bg-background text-xs" style={{ color: 'var(--vscode-descriptionForeground)' }}>
                <div style={{ fontWeight: 600, marginBottom: 6 }}>Preview Diagnostics</div>
                {previewErrors.slice(-3).map((e, idx) => (
                  <div key={idx} style={{ marginBottom: 4 }}>
                    <div>
                      {e.type}: {e.message || ''}
                    </div>
                    {e.filename && (
                      <div style={{ opacity: 0.8 }}>
                        at {e.filename}:{e.lineno}:{e.colno}
                      </div>
                    )}
                    {e.stack && (
                      <div style={{ whiteSpace: 'pre-wrap', opacity: 0.8 }}>{e.stack}</div>
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
        {previewFormat === 'markdown' && (
          <div className="w-full h-full flex flex-col">
            {isGeneratingMarkdown ? (
              <div className="flex-1 flex items-center justify-center">
                <div className="text-center">
                  <SrujaLoader size={48} />
                  <p className="mt-4 text-sm" style={{ color: 'var(--vscode-descriptionForeground)' }}>Generating markdown preview...</p>
                </div>
              </div>
            ) : (
              <div className="flex-1 overflow-auto p-4 markdown-preview" style={{ fontFamily: 'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif' }}>
                <div className="markdown-content" style={{ maxWidth: '800px', margin: '0 auto' }}>
                  <ReactMarkdown
                    remarkPlugins={[remarkGfm]}
                    components={{
                      code({ className, children, ...props }: any) {
                        const match = /language-(\w+)/.exec(className || '');
                        const codeString = String(children).replace(/\n$/, '');
                        const isInline = !className || !match;

                        if (!isInline && match && match[1] === 'mermaid') {
                          const codeHash = codeString.substring(0, 50) + codeString.length;
                          return <MermaidDiagram key={`mermaid-${codeHash}`} code={codeString} onExpand={onMermaidExpand} />;
                        }

                        if (!isInline) {
                          return (
                            <pre className="markdown-code-block">
                              <code className={className} {...props}>
                                {children}
                              </code>
                            </pre>
                          );
                        }

                        return (
                          <code className="markdown-inline-code" {...props}>
                            {children}
                          </code>
                        );
                      },
                      h1: ({ children }) => <h1 className="markdown-h1">{children}</h1>,
                      h2: ({ children }) => <h2 className="markdown-h2">{children}</h2>,
                      h3: ({ children }) => <h3 className="markdown-h3">{children}</h3>,
                      h4: ({ children }) => <h4 className="markdown-h4">{children}</h4>,
                      p: ({ children }) => <p className="markdown-p">{children}</p>,
                      ul: ({ children }) => <ul className="markdown-ul">{children}</ul>,
                      ol: ({ children }) => <ol className="markdown-ol">{children}</ol>,
                      li: ({ children }) => <li className="markdown-li">{children}</li>,
                      blockquote: ({ children }) => <blockquote className="markdown-blockquote">{children}</blockquote>,
                      table: ({ children }) => <div className="markdown-table-wrapper"><table className="markdown-table">{children}</table></div>,
                      thead: ({ children }) => <thead className="markdown-thead">{children}</thead>,
                      tbody: ({ children }) => <tbody className="markdown-tbody">{children}</tbody>,
                      tr: ({ children }) => <tr className="markdown-tr">{children}</tr>,
                      th: ({ children }) => <th className="markdown-th">{children}</th>,
                      td: ({ children }) => <td className="markdown-td">{children}</td>,
                      a: ({ href, children }) => <a href={href} className="markdown-link" target="_blank" rel="noopener noreferrer">{children}</a>,
                      img: ({ src, alt }) => <img src={src} alt={alt} className="markdown-img" />,
                    }}
                  >
                    {markdownPreview || 'No markdown preview available'}
                  </ReactMarkdown>
                </div>
              </div>
            )}
          </div>
        )}

      </div>
    </>
  );
}
