// apps/website/src/features/viewer/components/InteractiveViewer.tsx
import { ThemeProvider, SrujaMonacoEditor } from '@sruja/ui';
import '@sruja/ui/design-system/styles.css';
import { useState, useEffect, useRef, useCallback } from 'react';
import { createViewer, type ArchitectureJSON, type ViewerInstance } from '@sruja/viewer';
import { Button } from '@sruja/ui';
import { Maximize2, Code, Download, Braces, CodeXml, FileText } from 'lucide-react';
import { SrujaLoader } from '@sruja/ui';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import LZString from 'lz-string';
import mermaid from 'mermaid';

// Import modular components and utilities
import type { InteractiveViewerProps, PaneType, PreviewFormat, ValidationStatus, ExpandedMermaid } from '../types';
import { DEFAULT_SPLIT_PANEL_SIZE } from '../constants';
import { useWasm } from '../hooks/useWasm';
import { useExamples } from '../hooks/useExamples';
import { useDslState } from '../hooks/useDslState';
import { useDslParser } from '../hooks/useDslParser';
import { useExampleLoader } from '../hooks/useExampleLoader';
import { saveDslToStorage, savePaneToStorage, loadPaneFromStorage } from '../utils/storage';
import { updateUrlWithCode, copyShareUrl, parseViewParams } from '../utils/urlState';
import { downloadHtml, downloadMarkdown, downloadJson, downloadPdf } from '../utils/downloads';
import { generateHtmlPreview, generateMarkdownPreview, generatePdfPreview } from '../utils/previews';
import { TopNavBar } from './TopNavBar';
import { ErrorModal } from './ErrorModal';
import { MermaidExpandedModal } from './MermaidExpandedModal';
import { MermaidDiagram } from '@sruja/ui/components';
import { PreviewPanels } from './PreviewPanels';
import { viewerStyles } from '../styles';

export default function InteractiveViewer({ initialDsl, initialData }: InteractiveViewerProps) {
  // State management
  const [dsl, setDsl] = useDslState(initialDsl);
  const [archData, setArchData] = useState<ArchitectureJSON | null>(initialData || null);
  const [htmlPreview, setHtmlPreview] = useState<string>('');
  const [markdownPreview, setMarkdownPreview] = useState<string>('');
  const [pdfPreviewUrl, setPdfPreviewUrl] = useState<string>('');
  const [activePane, setActivePane] = useState<PaneType>(loadPaneFromStorage());
  const [expandedMermaid, setExpandedMermaid] = useState<ExpandedMermaid | null>(null);
  const [mermaidZoom, setMermaidZoom] = useState(1);
  const [showErrorModal, setShowErrorModal] = useState(false);
  const [splitPanelSize, setSplitPanelSize] = useState(DEFAULT_SPLIT_PANEL_SIZE);
  const [isResizing, setIsResizing] = useState(false);
  const [previewFormat, setPreviewFormat] = useState<PreviewFormat>('diagram');
  const [validationStatus, setValidationStatus] = useState<ValidationStatus>({
    isValid: true,
    errors: 0,
    warnings: 0,
    lastError: undefined,
  });
  const [isGeneratingHtml, setIsGeneratingHtml] = useState(false);
  const [isGeneratingMarkdown, setIsGeneratingMarkdown] = useState(false);
  const [isGeneratingPdf, setIsGeneratingPdf] = useState(false);
  const [isParsingDsl, setIsParsingDsl] = useState(false);
  const [urlUpdateTimeout, setUrlUpdateTimeout] = useState<NodeJS.Timeout | null>(null);
  const [shareCopied, setShareCopied] = useState(false);

  // Refs
  const viewerRef = useRef<ViewerInstance | null>(null);
  const viewerContainerRef = useRef<HTMLDivElement>(null);
  const previewFrameRef = useRef<HTMLIFrameElement>(null);
  const mermaidInitialized = useRef(false);
  const previewBlobUrlRef = useRef<string | null>(null);
  const [previewErrors, setPreviewErrors] = useState<Array<{ type: string; message?: string; filename?: string; lineno?: number; colno?: number; stack?: string }>>([]);

  // Hooks
  const { wasmApiRef, isWasmLoading } = useWasm();
  const examples = useExamples();

  // DSL Parser (must be defined before useExampleLoader)
  const parseDslToJson = useDslParser(
    wasmApiRef,
    setArchData,
    setValidationStatus,
    setIsParsingDsl,
    setHtmlPreview,
    setIsGeneratingHtml,
    setMarkdownPreview,
    setIsGeneratingMarkdown,
    setPdfPreviewUrl,
    setIsGeneratingPdf,
    archData
  );

  // Example Loader (uses parseDslToJson)
  const { selectedExample, setSelectedExample, isLoadingExample, loadExample } = useExampleLoader(
    setDsl,
    parseDslToJson,
    setValidationStatus,
    urlUpdateTimeout,
    setUrlUpdateTimeout
  );

  // Initialize Mermaid (only once globally)
  useEffect(() => {
    if (!mermaidInitialized.current && typeof window !== 'undefined' && mermaid) {
      try {
        mermaid.initialize({
          startOnLoad: false,
          theme: 'default',
          securityLevel: 'loose',
        });
        mermaidInitialized.current = true;
      } catch (err) {
        console.error('Failed to initialize Mermaid:', err);
      }
    }
  }, []);

  // Parse URL params for view and preview control on mount
  useEffect(() => {
    if (typeof window === 'undefined') return;
    const viewParams = parseViewParams();
    if (viewParams.pane) {
      setActivePane(viewParams.pane);
    }
    if (viewParams.preview) {
      setPreviewFormat(viewParams.preview);
    }
  }, []);

  // Initialize WASM and parse initial DSL
  useEffect(() => {
    if (!isWasmLoading && wasmApiRef.current) {
      if (dsl) {
        parseDslToJson(dsl);
      } else if (initialData) {
        setArchData(initialData);
      }
    }
  }, [isWasmLoading, dsl, initialData, parseDslToJson]);

  // Load from URL parameters (runs after initial render to override localStorage)
  useEffect(() => {
    if (typeof window === 'undefined') return;
    if (initialDsl) return; // Skip if we already have initialDsl

    const params = new URLSearchParams(window.location.search);
    const dataParam = params.get('data');
    const urlParam = params.get('url');
    const codeParam = params.get('code');
    const hash = window.location.hash;

    // Support compressed code in hash: #code=<base64>
    if (hash.startsWith('#code=')) {
      const b64 = hash.substring('#code='.length);
      try {
        const decompressed = LZString.decompressFromBase64(decodeURIComponent(b64));
        if (decompressed) {
          setDsl(decompressed);
          saveDslToStorage(decompressed);
          return;
        }
      } catch (_) { }
    }

    // Support code in query parameter
    if (codeParam) {
      try {
        const decompressed = LZString.decompressFromBase64(decodeURIComponent(codeParam));
        if (decompressed) {
          setDsl(decompressed);
          saveDslToStorage(decompressed);
          return;
        }
      } catch (e) {
        console.error('Error decompressing code parameter:', e);
      }
      return;
    }

    // Support JSON data in query parameter
    if (dataParam) {
      try {
        const decoded = decodeURIComponent(dataParam);
        const parsed = JSON.parse(decoded);
        setArchData(parsed);
        if (wasmApiRef.current && parsed) {
          wasmApiRef.current.printJsonToDsl(JSON.stringify(parsed))
            .then((dslStr: string) => {
              if (dslStr) {
                setDsl(dslStr);
                saveDslToStorage(dslStr);
              }
            })
            .catch(() => { });
        }
      } catch (e) {
        console.error('Failed to parse data parameter:', e);
      }
      return;
    }

    // Support URL to fetch JSON
    if (urlParam) {
      fetch(decodeURIComponent(urlParam))
        .then(res => res.json())
        .then(json => {
          setArchData(json);
          if (wasmApiRef.current) {
            wasmApiRef.current.printJsonToDsl(JSON.stringify(json))
              .then((dslStr: string) => {
                if (dslStr) {
                  setDsl(dslStr);
                  saveDslToStorage(dslStr);
                }
              })
              .catch(() => { });
          }
        })
        .catch(err => {
          console.error('Failed to load architecture from URL:', err);
        });
      return;
    }
  }, [initialDsl, wasmApiRef]);

  // Handle DSL changes
  const handleDslChange = useCallback((value: string | undefined) => {
    const newDsl = value || '';
    setDsl(newDsl);
    saveDslToStorage(newDsl);
    updateUrlWithCode(newDsl, urlUpdateTimeout, setUrlUpdateTimeout);
    parseDslToJson(newDsl);
  }, [setDsl, parseDslToJson, urlUpdateTimeout, setUrlUpdateTimeout]);

  // Initialize viewer when container and data are ready
  useEffect(() => {
    if (!viewerContainerRef.current || !archData || isWasmLoading) return;

    // Destroy existing viewer if any
    if (viewerRef.current) {
      viewerRef.current.destroy();
    }

    // Create new viewer
    const viewer = createViewer({
      container: viewerContainerRef.current,
      data: archData,
      onSelect: (id: string | null) => {
        // Handle selection if needed
      },
    });

    viewerRef.current = viewer;
    viewer.init();

    // Wait for Cytoscape to be ready
    const waitForCytoscape = () => {
      if (viewer.cy && !viewer.cy.destroyed()) {
        viewer.cy.ready(() => {
          if (viewer.cy && !viewer.cy.destroyed()) {
            viewer.cy.resize();
            viewer.cy.fit(undefined, 80);
          }
        });
      } else {
        setTimeout(waitForCytoscape, 50);
      }
    };

    setTimeout(waitForCytoscape, 50);

    // Handle window resize
    const handleResize = () => {
      if (viewer.cy && !viewer.cy.destroyed()) {
        viewer.cy.resize();
        viewer.cy.fit(undefined, 80);
      }
    };

    window.addEventListener('resize', handleResize);

    return () => {
      window.removeEventListener('resize', handleResize);
      if (viewer) {
        viewer.destroy();
      }
    };
  }, [archData, isWasmLoading]);

  // Update preview iframe when HTML changes or when switching to preview pane
  useEffect(() => {
    // Use a small timeout to ensure iframe is mounted when conditionally rendered
    const timeoutId = setTimeout(() => {
      if (previewFrameRef.current && htmlPreview && (activePane === 'preview' || (activePane === 'split' && previewFormat === 'preview'))) {
        // Clean up previous blob URL if it exists
        if (previewBlobUrlRef.current) {
          URL.revokeObjectURL(previewBlobUrlRef.current);
          previewBlobUrlRef.current = null;
        }

        // Create a blob URL from the HTML content
        // This gives the iframe a proper origin (blob: URL) instead of null, fixing CORS issues
        const blob = new Blob([htmlPreview], { type: 'text/html' });
        const blobUrl = URL.createObjectURL(blob);
        previewBlobUrlRef.current = blobUrl;
        previewFrameRef.current.src = blobUrl;
      }
    }, 0);

    return () => {
      clearTimeout(timeoutId);
      // Clean up blob URL on unmount
      if (previewBlobUrlRef.current) {
        URL.revokeObjectURL(previewBlobUrlRef.current);
        previewBlobUrlRef.current = null;
      }
    };
  }, [htmlPreview, activePane, previewFormat]);

  // Capture diagnostics from the preview iframe
  useEffect(() => {
    const handler = (event: MessageEvent) => {
      const srcWin = previewFrameRef.current?.contentWindow;
      if (!srcWin) return;
      if (event.source !== srcWin) return;
      const data: any = event.data;
      if (!data || typeof data !== 'object') return;
      if (data.type === 'sruja:html-preview:error') {
        console.error('HTML preview error:', data);
        setPreviewErrors(prev => [...prev, data].slice(-50));
      } else if (data.type === 'sruja:html-preview:console') {
        console.error('HTML preview console:', data);
        setPreviewErrors(prev => [...prev, data].slice(-50));
      }
    };
    window.addEventListener('message', handler);
    return () => window.removeEventListener('message', handler);
  }, []);

  // Handle resizing panels
  useEffect(() => {
    if (!isResizing) return;

    const handleMouseMove = (e: MouseEvent) => {
      if (!isResizing) return;
      const container = viewerContainerRef.current?.parentElement;
      if (!container) return;

      const containerRect = container.getBoundingClientRect();
      const newSize = ((e.clientX - containerRect.left) / containerRect.width) * 100;
      const clampedSize = Math.max(20, Math.min(80, newSize));
      setSplitPanelSize(clampedSize);
    };

    const handleMouseUp = () => {
      setIsResizing(false);
    };

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);

    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isResizing]);

  // Cleanup timeout on unmount
  useEffect(() => {
    return () => {
      if (urlUpdateTimeout) {
        clearTimeout(urlUpdateTimeout);
      }
    };
  }, [urlUpdateTimeout]);

  // Share URL handler
  const handleShare = useCallback(async () => {
    const success = await copyShareUrl(dsl);
    if (success) {
      setShareCopied(true);
      setTimeout(() => setShareCopied(false), 2000);
      try { window.dispatchEvent(new CustomEvent('sruja:event', { detail: { type: 'viewer.share' } })) } catch { }
    }
  }, [dsl]);

  // Open in Studio handler
  const handleOpenStudio = useCallback(() => {
    if (!dsl || !dsl.trim()) {
      window.open('/studio', '_blank');
      return;
    }
    const compressed = LZString.compressToBase64(dsl);
    const studioUrl = `/studio#code=${encodeURIComponent(compressed)}`;
    window.open(studioUrl, '_blank');
  }, [dsl]);

  // Example change handler
  const handleExampleChange = useCallback((exampleFile: string) => {
    const example = examples.find(ex => ex.file === exampleFile);
    if (example) {
      setSelectedExample(example.file);
      loadExample(example.file);
    }
  }, [examples, loadExample, setSelectedExample]);

  if (isWasmLoading) {
    return (
      <div className="flex items-center justify-center h-full w-full">
        <div className="text-center">
          <SrujaLoader size={48} />
          <p className="text-[var(--color-text-secondary)] mt-4">Initializing Sruja Viewer...</p>
        </div>
      </div>
    );
  }

  return (
    <ThemeProvider defaultMode="system">
      <style>{viewerStyles}</style>
      <div className="flex flex-col h-full w-full" style={{ background: 'var(--vscode-editor-background)', color: 'var(--vscode-foreground)' }}>
        {/* Top Navigation Bar */}
        <TopNavBar
          examples={examples}
          selectedExample={selectedExample}
          isLoadingExample={isLoadingExample}
          isWasmLoading={isWasmLoading}
          onExampleChange={handleExampleChange}
          onOpenStudio={handleOpenStudio}
          onShare={handleShare}
          shareCopied={shareCopied}
          validationStatus={validationStatus}
          onErrorClick={() => setShowErrorModal(true)}
        />

        {/* Content Area */}
        <div className="flex-1 flex overflow-hidden relative">
          {/* Editor Panel with Controls */}
          <div
            className={`flex flex-col border-r ${activePane === 'split' ? '' : 'w-full'}`}
            style={activePane === 'split' ? { width: `${splitPanelSize}%` } : {}}
          >
            {/* Editor Toolbar */}
            <div className="flex items-center justify-between vscode-toolbar" style={{ borderBottom: '1px solid var(--vscode-panel-border)' }}>
              <div className="flex items-center gap-2">
                <Code className="vscode-icon" />
                <span className="text-sm font-medium">Editor</span>
              </div>
              <div className="flex items-center gap-1">
                <button
                  className={`vscode-toolbar-button ${activePane === 'split' ? 'primary' : ''}`}
                  onClick={() => {
                    setActivePane('split');
                    savePaneToStorage('split');
                  }}
                  title="Split view"
                >
                  <Maximize2 className="vscode-icon" />
                </button>
                <button
                  className={`vscode-toolbar-button ${activePane === 'editor' ? 'primary' : ''}`}
                  onClick={() => {
                    setActivePane('editor');
                    savePaneToStorage('editor');
                  }}
                  title="Editor only"
                >
                  <Code className="vscode-icon" />
                </button>
              </div>
            </div>
            {/* Editor Content */}
            <div className="flex-1 overflow-hidden">
              <SrujaMonacoEditor
                value={dsl}
                onChange={handleDslChange}
                theme="vs-dark"
                options={{
                  minimap: { enabled: false },
                  fontSize: 14,
                  lineNumbers: 'on',
                  wordWrap: 'on',
                }}
              />
            </div>
          </div>

          {/* Resizer Handle */}
          {activePane === 'split' && (
            <div
              className="w-1 bg-border hover:bg-primary cursor-col-resize transition-colors relative z-10"
              onMouseDown={(e) => {
                setIsResizing(true);
                e.preventDefault();
              }}
              style={{ cursor: 'col-resize' }}
            />
          )}

          {/* Preview Panel (includes Diagram, JSON, Markdown, HTML) */}
          {activePane === 'split' && (
            <div
              className="flex flex-col"
              style={{ width: `${100 - splitPanelSize}%` }}
            >
              <PreviewPanels
                previewFormat={previewFormat}
                setPreviewFormat={setPreviewFormat}
                activePane={activePane}
                setActivePane={setActivePane}
                savePaneToStorage={savePaneToStorage}
                viewerContainerRef={viewerContainerRef}
                archData={archData}
                isParsingDsl={isParsingDsl}
                onDownloadJson={() => downloadJson(archData)}
                htmlPreview={htmlPreview}
                isGeneratingHtml={isGeneratingHtml}
                previewFrameRef={previewFrameRef}
                onDownloadHtml={() => downloadHtml(htmlPreview, archData)}
                previewErrors={previewErrors}
                markdownPreview={markdownPreview}
                isGeneratingMarkdown={isGeneratingMarkdown}
                onDownloadMarkdown={() => downloadMarkdown(markdownPreview, archData)}
                expandedMermaid={expandedMermaid}
                mermaidZoom={mermaidZoom}
                onMermaidExpand={(svg, code) => {
                  setMermaidZoom(1);
                  setExpandedMermaid({ svg, code });
                }}
                onMermaidZoomIn={() => setMermaidZoom(prev => Math.min(prev + 0.25, 3))}
                onMermaidZoomOut={() => setMermaidZoom(prev => Math.max(prev - 0.25, 0.5))}
                onMermaidClose={() => {
                  setExpandedMermaid(null);
                  setMermaidZoom(1);
                }}
              />
            </div>
          )}

          {/* Full-screen panes */}
          {activePane === 'json' && (
            <div className="w-full h-full flex flex-col">
              <div className="p-2 border-b bg-background text-sm flex items-center justify-between">
                <span>JSON Export</span>
                <button
                  className="vscode-toolbar-button"
                  onClick={() => downloadJson(archData)}
                  disabled={!archData || isParsingDsl}
                  title="Download JSON"
                >
                  <Download className="vscode-icon" />
                  <span>Download</span>
                </button>
              </div>
              {isParsingDsl ? (
                <div className="flex-1 flex items-center justify-center">
                  <div className="text-center">
                    <SrujaLoader size={48} />
                    <p className="mt-4 text-sm" style={{ color: 'var(--vscode-descriptionForeground)' }}>Parsing DSL to JSON...</p>
                  </div>
                </div>
              ) : (
                <div className="flex-1 overflow-auto p-4">
                  <pre className="text-sm font-mono p-4 rounded-lg overflow-auto" style={{
                    margin: 0,
                    whiteSpace: 'pre-wrap',
                    wordBreak: 'break-word',
                    color: 'var(--vscode-foreground)',
                    background: 'var(--vscode-input-background)'
                  }}>
                    {archData ? JSON.stringify(archData, null, 2) : 'No architecture data available. Parse DSL to generate JSON.'}
                  </pre>
                </div>
              )}
            </div>
          )}

          {activePane === 'preview' && (
            <div className="w-full h-full flex flex-col">
              <div className="p-2 border-b bg-background text-sm flex items-center justify-between">
                <span>HTML Export Preview</span>
                <button
                  className="vscode-toolbar-button"
                  onClick={() => downloadHtml(htmlPreview, archData)}
                  disabled={!htmlPreview || isGeneratingHtml}
                  title="Download HTML"
                >
                  <Download className="vscode-icon" />
                  <span>Download</span>
                </button>
              </div>
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
            </div>
          )}

          {activePane === 'markdown' && (
            <div className="w-full h-full flex flex-col">
              <div className="p-2 border-b bg-background text-sm flex items-center justify-between">
                <span>Markdown Preview</span>
                <button
                  className="vscode-toolbar-button"
                  onClick={() => downloadMarkdown(markdownPreview, archData)}
                  disabled={!markdownPreview}
                  title="Download Markdown"
                >
                  <Download className="vscode-icon" />
                  <span>Download</span>
                </button>
              </div>
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
                            return <MermaidDiagram key={`mermaid-${codeHash}`} code={codeString} onExpand={(svg, code) => {
                              setMermaidZoom(1);
                              setExpandedMermaid({ svg, code });
                            }} />;
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
              {/* Expanded Mermaid Modal */}
              {expandedMermaid && (
                <MermaidExpandedModal
                  expandedMermaid={expandedMermaid}
                  zoom={mermaidZoom}
                  onZoomIn={() => setMermaidZoom(prev => Math.min(prev + 0.25, 3))}
                  onZoomOut={() => setMermaidZoom(prev => Math.max(prev - 0.25, 0.5))}
                  onClose={() => {
                    setExpandedMermaid(null);
                    setMermaidZoom(1);
                  }}
                />
              )}
            </div>
          )}



          {activePane === 'diagram' && (
            <div className="w-full h-full">
              <div ref={viewerContainerRef} className="w-full h-full" />
            </div>
          )}
        </div>
      </div>

      {/* Error Modal */}
      <ErrorModal
        validationStatus={validationStatus}
        onClose={() => setShowErrorModal(false)}
      />
    </ThemeProvider>
  );
}
