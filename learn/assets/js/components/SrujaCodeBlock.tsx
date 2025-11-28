// Sruja Code Block Component - Replaces enhanceSrujaBlocks DOM manipulation
import React, { useState, useEffect, useRef } from 'react';
import { createRoot } from 'react-dom/client';
import { Button } from './ui/button';
import { Dialog, DialogContent, DialogTitle, DialogDescription } from './ui/dialog';
import { Textarea } from './ui/textarea';
import { initSrujaWasm, compileSrujaCode } from '../utils/wasm';
import { sanitizeSvg } from '../utils/sanitize';

// Simple icon components with centered layout
const IconWrap = ({ children }: { children: React.ReactNode }) => (
  <span className="flex items-center justify-center w-full h-full leading-none text-center select-none text-xs">
    {children}
  </span>
);
const CopyIcon = () => <IconWrap>📋</IconWrap>;
const EditIcon = () => <IconWrap>✏️</IconWrap>;
const PlayIcon = () => <IconWrap>▶</IconWrap>;
const CheckIcon = () => <IconWrap>✓</IconWrap>;

interface SrujaCodeBlockProps {
  code: string;
  filename: string;
}

export function SrujaCodeBlock({ code: initialCode, filename }: SrujaCodeBlockProps) {
  const [code, setCode] = useState(initialCode);
  const [isEditing, setIsEditing] = useState(false);
  const [output, setOutput] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [wasmReady, setWasmReady] = useState(false);
  const codeRef = useRef<HTMLElement>(null);
  const preRef = useRef<HTMLPreElement>(null);
  const outputRef = useRef<HTMLDivElement>(null);
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const wrapperRef = useRef<HTMLDivElement>(null);
  const toolbarRef = useRef<HTMLDivElement>(null);
  const toolbarSpacerRef = useRef<HTMLDivElement>(null);

  const downloadCurrentSvg = () => {
    const container = outputRef.current;
    const svg = container?.querySelector('svg');
    if (!svg) return;
    const xml = `<?xml version="1.0" encoding="UTF-8"?>\n` + svg.outerHTML;
    const blob = new Blob([xml], { type: 'image/svg+xml;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = (filename.replace(/\.[^./]+$/, '') || 'diagram') + '.svg';
    document.body.appendChild(a);
    a.click();
    setTimeout(() => {
      URL.revokeObjectURL(url);
      a.remove();
    }, 0);
  };

  useEffect(() => {
    const NAV_HEIGHT = 64;
    const margin = 8;
    const updateToolbar = () => {
      const wrapper = wrapperRef.current;
      const tb = toolbarRef.current;
      const spacer = toolbarSpacerRef.current;
      if (!wrapper || !tb) return;

      const rect = wrapper.getBoundingClientRect();
      const viewportTop = NAV_HEIGHT + margin;
      const viewportBottom = window.innerHeight - margin;

      const isVisible = rect.bottom > viewportTop && rect.top < viewportBottom;
      if (!isVisible) {
        tb.classList.remove('is-fixed');
        if (spacer) {
          spacer.classList.remove('is-visible');
        }
        return;
      }

      const shouldPin = rect.top < viewportTop && rect.bottom > viewportTop + margin * 2;
      
      if (shouldPin) {
        // Measure toolbar size before making it fixed
        const tbWidth = tb.offsetWidth;
        const tbHeight = tb.offsetHeight;
        
        // Set CSS custom properties for positioning and sizing
        tb.style.setProperty('--toolbar-fixed-top', `${viewportTop}px`);
        tb.style.setProperty('--toolbar-width', `${tbWidth}px`);
        tb.style.setProperty('--toolbar-height', `${tbHeight}px`);
        
        const left = Math.max(
          rect.left + margin,
          Math.min(rect.right - tbWidth - margin, window.innerWidth - tbWidth - margin)
        );
        tb.style.setProperty('--toolbar-fixed-left', `${left}px`);
        
        // Update spacer
        if (spacer) {
          spacer.style.setProperty('--toolbar-width', `${tbWidth}px`);
          spacer.style.setProperty('--toolbar-height', `${tbHeight}px`);
          spacer.classList.add('is-visible');
        }
        
        tb.classList.add('is-fixed');
      } else {
        tb.classList.remove('is-fixed');
        if (spacer) {
          spacer.classList.remove('is-visible');
        }
      }
    };

    updateToolbar();

    const ro = new ResizeObserver(updateToolbar);
    if (wrapperRef.current) ro.observe(wrapperRef.current);
    window.addEventListener('scroll', updateToolbar, { passive: true });
    window.addEventListener('resize', updateToolbar);
    return () => {
      ro.disconnect();
      window.removeEventListener('scroll', updateToolbar);
      window.removeEventListener('resize', updateToolbar);
    };
  }, [isEditing]);

  useEffect(() => {
    initSrujaWasm();
    const checkWasm = setInterval(() => {
      if (window.srujaWasmReady) {
        setWasmReady(true);
        clearInterval(checkWasm);
      }
    }, 100);
    return () => clearInterval(checkWasm);
  }, []);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      setTimeout(() => setCopied(false), 1200);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  };

  const handleEdit = () => {
    setIsEditing(!isEditing);
    if (isEditing && codeRef.current) {
      codeRef.current.textContent = code;
    }
  };

  const handleRun = () => {
    if (!wasmReady || typeof window.compileSruja === 'undefined') {
      setError('WASM not ready');
      setOutput(null);
      return;
    }

    setError(null);
    setOutput(null);

    try {
      const result = compileSrujaCode(code, filename);
      if (result?.error) {
        setError(result.error);
      } else if (result?.svg) {
        const parser = new DOMParser();
        const doc = parser.parseFromString(result.svg, 'image/svg+xml');
        const svg = doc.querySelector('svg');
        if (svg) {
          svg.removeAttribute('width');
          svg.removeAttribute('height');
          svg.setAttribute('preserveAspectRatio', 'xMidYMid meet');
          svg.setAttribute('style', 'width:100%;height:auto;max-height:100%;display:block');
          setOutput(svg.outerHTML);
        } else {
          setOutput(result.svg);
        }
      } else if (result?.html) {
        const parser = new DOMParser();
        const doc = parser.parseFromString(result.html, 'text/html');
        const svg = doc.querySelector('svg');
        if (svg) {
          svg.removeAttribute('width');
          svg.removeAttribute('height');
          svg.setAttribute('preserveAspectRatio', 'xMidYMid meet');
          svg.setAttribute('style', 'width:100%;height:auto;max-height:100%;display:block');
          setOutput(svg.outerHTML);
        } else {
          setOutput(result.html);
        }
      } else if (result?.image || result?.png || result?.jpg || result?.jpeg) {
        const src = result.image || result.png || result.jpg || result.jpeg || '';
        setOutput(src ? `<img src="${src}" alt="Diagram" style="width:100%;height:auto;max-height:100%;display:block"/>` : '');
      } else {
        setError('No output');
      }
    } catch (e) {
      setError('Internal Error: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  return (
    <div ref={wrapperRef} className="sruja-code-wrapper relative">
      <pre ref={preRef} style={{ display: isEditing ? 'none' : 'block' }}>
        <code ref={codeRef} className="language-sruja">
          {code}
        </code>
      </pre>
      {isEditing && (
        <Textarea
          className="sruja-editor font-mono min-h-[220px] bg-slate-900 dark:bg-slate-950 text-inherit"
          value={code}
          onChange={(e) => setCode(e.target.value)}
          spellCheck={false}
        />
      )}
      <div ref={toolbarSpacerRef} className="sruja-code-toolbar-spacer" aria-hidden="true" />
      <div ref={toolbarRef} className="sruja-code-toolbar inline-flex items-center gap-1 bg-slate-900/60 dark:bg-slate-900/60 border border-slate-700 dark:border-slate-600 rounded-md px-1 py-[2px] backdrop-blur-sm shadow-md">
        <Button
          variant="ghost"
          size="icon-xs"
          className="h-5 w-5 p-0 text-slate-200 hover:text-slate-50"
          onClick={handleCopy}
          title="Copy"
          aria-label="Copy code"
        >
          {copied ? <CheckIcon /> : <CopyIcon />}
        </Button>
        <Button
          variant="ghost"
          size="icon-xs"
          className={`h-5 w-5 p-0 text-slate-200 hover:text-slate-50 ${isEditing ? 'bg-slate-800' : ''}`}
          onClick={handleEdit}
          title="Edit"
          aria-label={isEditing ? "Save edit" : "Edit code"}
        >
          <EditIcon />
        </Button>
        <Button
          variant="ghost"
          size="icon-xs"
          className="h-5 w-5 p-0 text-slate-200 hover:text-slate-50"
          onClick={handleRun}
          title="Run"
          disabled={!wasmReady}
          aria-label="Run code"
        >
          <PlayIcon />
        </Button>
      </div>
      {(output || error) && (
        <div ref={outputRef} className="sruja-run-output relative mt-2 border border-slate-300 dark:border-slate-600 rounded-lg p-2 bg-white dark:bg-slate-800">
          {!error && (
            <div className="absolute top-2 right-2 flex gap-1">
              <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
                <DialogTitle className="sr-only">Diagram Preview</DialogTitle>
                <DialogDescription className="sr-only">Expanded view of the generated diagram</DialogDescription>
                <Button title="Expand" variant="ghost" size="icon-xs" className="h-5 w-5 p-0" onClick={() => setIsDialogOpen(true)} aria-label="Expand diagram">
                  <IconWrap>⛶</IconWrap>
                </Button>
                <DialogContent className="max-w-[90vw] max-h-[90vh] p-0">
                  <div className="p-6">
                    <div dangerouslySetInnerHTML={{ __html: sanitizeSvg(getOutputForDialog()) }} />
                  </div>
                </DialogContent>
              </Dialog>
              <Button title="Download SVG" variant="ghost" size="icon-xs" className="h-5 w-5 p-0" onClick={downloadCurrentSvg} aria-label="Download SVG">
                <IconWrap>⬇️</IconWrap>
              </Button>
              <Button title="Close" variant="ghost" size="icon-xs" className="h-5 w-5 p-0" onClick={() => { setOutput(null); setError(null); }} aria-label="Close output">
                <IconWrap>✕</IconWrap>
              </Button>
            </div>
          )}
          {error ? (
            <div className="text-red-600 dark:text-red-400 whitespace-pre-wrap">{error}</div>
          ) : (
            <div dangerouslySetInnerHTML={{ __html: output ? sanitizeSvg(output) : '' }} />
          )}
        </div>
      )}
    </div>
  );
}

// Initialize code blocks - replaces enhanceSrujaBlocks
export function initSrujaCodeBlocks(): void {
  function slugify(s: string): string {
    return s.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '') || 'snippet';
  }

  const blocks = document.querySelectorAll('pre > code.language-sruja');
  const pageSlug = slugify(window.location.pathname.replace(/\//g, '-'));

  blocks.forEach((codeEl, idx) => {
    const pre = codeEl.parentElement as HTMLPreElement;
    if (!pre || pre.dataset.enhanced === 'true') return;
    pre.dataset.enhanced = 'true';

    const code = codeEl.textContent || '';
    const filename = `page-${pageSlug}-snippet-${idx}.sruja`;

    // Create wrapper and mount React component
    const wrapper = document.createElement('div');
    pre.parentNode?.insertBefore(wrapper, pre);
    pre.remove(); // Remove original pre, React will render it

    const root = createRoot(wrapper);
    root.render(<SrujaCodeBlock code={code} filename={filename} />);
  });
}

function getOutputForDialog(): string {
  const container = document.querySelector('.sruja-run-output');
  const svg = container?.querySelector('svg');
  if (svg) {
    const cloneSvg = svg.cloneNode(true) as SVGSVGElement;
    cloneSvg.removeAttribute('width');
    cloneSvg.removeAttribute('height');
    cloneSvg.setAttribute('preserveAspectRatio', 'xMidYMid meet');
    cloneSvg.classList.add('modal-svg');
    // Remove any event handlers and scripts for safety
    const svgString = cloneSvg.outerHTML;
    return sanitizeSvg(svgString);
  }
  const img = container?.querySelector('img');
  if (img) {
    const cloneImg = img.cloneNode(true) as HTMLImageElement;
    cloneImg.classList.add('modal-svg');
    cloneImg.style.maxWidth = '100%';
    cloneImg.style.height = 'auto';
    // Remove any event handlers
    Array.from(cloneImg.attributes).forEach(attr => {
      if (attr.name.startsWith('on')) {
        cloneImg.removeAttribute(attr.name);
      }
    });
    return cloneImg.outerHTML;
  }
  return '';
}
