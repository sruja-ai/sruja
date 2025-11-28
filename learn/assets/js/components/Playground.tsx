// Playground React Component
import { useState, useEffect, useRef } from 'react';
import { createRoot } from 'react-dom/client';
// import types if needed
import { initSrujaWasm, compileSrujaCode } from '../utils/wasm';
import { Dialog, DialogContent, DialogTitle, DialogDescription } from './ui/dialog';
import { Button } from './ui/button';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from './ui/select';
import { EXAMPLES } from '../examples.generated';

interface PlaygroundProps {
  initialCode?: string;
}

export function Playground({ initialCode = '' }: PlaygroundProps) {
  const [code, setCode] = useState(initialCode || EXAMPLES[0].code);
  const [selectedExample, setSelectedExample] = useState(initialCode ? '' : EXAMPLES[0].name);
  const [status, setStatus] = useState('Loading WASM...');
  const [error, setError] = useState('');
  const [visualOutput, setVisualOutput] = useState<string>('');
  const [showToolbar, setShowToolbar] = useState(false);
  const outputRef = useRef<HTMLDivElement>(null);
  const zoomContainerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    initSrujaWasm();
    const checkWasm = setInterval(() => {
      if (window.srujaWasmReady) {
        setStatus('Ready');
        // Auto-run once when ready and we have code but no output
        setTimeout(() => {
          if (!visualOutput && (code?.trim() || '').length > 0) {
            try {
              const filename = 'playground.sruja';
              const result = compileSrujaCode(code, filename);
              if (result?.svg || result?.html || result?.image || result?.png || result?.jpg || result?.jpeg) {
                // Reuse handleRun normalization logic by setting state and letting effect adjust sizing
                const parser = result?.svg
                  ? new DOMParser().parseFromString(result.svg!, 'image/svg+xml')
                  : result?.html
                  ? new DOMParser().parseFromString(result.html!, 'text/html')
                  : null;
                const svg = parser ? parser.querySelector('svg') : null;
                if (svg) {
                  svg.removeAttribute('width');
                  svg.removeAttribute('height');
                  svg.setAttribute('preserveAspectRatio', 'xMidYMid meet');
                  svg.setAttribute('style', 'width:100%;height:auto;max-height:100%;display:block');
                  setVisualOutput(svg.outerHTML);
                } else {
                  const src = result.image || result.png || result.jpg || result.jpeg || '';
                  if (src) setVisualOutput(`<img src="${src}" alt="Diagram" style="width:100%;height:auto;max-height:100%;display:block"/>`);
                  else if (result.html) setVisualOutput(result.html);
                  else if (result.svg) setVisualOutput(result.svg);
                }
              }
            } catch {}
          }
        }, 50);
        clearInterval(checkWasm);
      }
    }, 100);
    return () => clearInterval(checkWasm);
  }, []);

// Removed zoom transform; preview fills container

  useEffect(() => {
    if (visualOutput && outputRef.current) {
      setShowToolbar(true);
      // Show toolbar element
      const toolbar = outputRef.current.querySelector('.preview-toolbar') as HTMLElement;
      if (toolbar) {
        toolbar.classList.remove('hidden');
      }
      const svg = outputRef.current.querySelector('svg');
      if (svg) {
        svg.removeAttribute('width');
        svg.removeAttribute('height');
        svg.setAttribute('preserveAspectRatio', 'xMidYMid meet');
        svg.classList.add('canvas-svg');
      }
    } else {
      setShowToolbar(false);
    }
  }, [visualOutput]);

  const handleExampleChange = (value: string) => {
    const example = EXAMPLES.find(e => e.name === value);
    if (example) {
      setCode(example.code);
      setSelectedExample(value);
    }
  };

  const handleRun = () => {
    if (!window.srujaWasmReady) {
      setError('WASM not ready');
      return;
    }

    setError('');
    setVisualOutput('');

    const filename = 'playground.sruja';
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
        setVisualOutput(svg.outerHTML);
      } else {
        setVisualOutput(result.svg);
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
        setVisualOutput(svg.outerHTML);
      } else {
        setVisualOutput(result.html);
      }
    } else if (result?.image || result?.png || result?.jpg || result?.jpeg) {
      const src = result.image || result.png || result.jpg || result.jpeg || '';
      setVisualOutput(src ? `<img src="${src}" alt="Diagram" style="width:100%;height:auto;max-height:100%;display:block"/>` : '');
    } else {
      setError('No output');
    }
  };

  // Zoom controls removed; use Expand for full-size view

  const [isDialogOpen, setIsDialogOpen] = useState(false);

  const getSvgForDialog = (): string => {
    const svg = outputRef.current?.querySelector('svg');
    if (svg) {
      const cloneSvg = svg.cloneNode(true) as SVGSVGElement;
      cloneSvg.removeAttribute('width');
      cloneSvg.removeAttribute('height');
      cloneSvg.setAttribute('preserveAspectRatio', 'xMidYMid meet');
      cloneSvg.classList.add('modal-svg');
      return cloneSvg.outerHTML;
    }
    const img = outputRef.current?.querySelector('img');
    if (img) {
      const cloneImg = img.cloneNode(true) as HTMLImageElement;
      cloneImg.classList.add('modal-svg');
      cloneImg.style.maxWidth = '100%';
      cloneImg.style.height = 'auto';
      return cloneImg.outerHTML;
    }
    return '';
  };

  return (
    <div>
      <div className="flex items-center gap-4 mb-4 flex-wrap">
        <label htmlFor="example-select" className="font-semibold">Examples</label>
        <Select value={selectedExample} onValueChange={handleExampleChange}>
          <SelectTrigger id="example-select" className="w-[180px]">
            <SelectValue placeholder="Select example..." />
          </SelectTrigger>
          <SelectContent>
            {EXAMPLES.map(ex => (
              <SelectItem key={ex.name} value={ex.name}>
                {ex.name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <span id="status" className="text-slate-500 dark:text-slate-400 text-sm">{status}</span>
        <Button
          id="run-btn"
          variant="default"
          size="sm"
          aria-label="Run"
          onClick={handleRun}
          className="px-4"
        >
          <span className="inline-flex items-center gap-2"><span className="leading-none">▶</span><span>Run</span></span>
        </Button>
        {error && (
          <div id="error-output" className="text-red-600 dark:text-red-400 whitespace-pre-wrap ml-3">
            {error}
          </div>
        )}
      </div>
      <div className="flex flex-col md:flex-row gap-4 border border-slate-300 dark:border-slate-600 rounded-lg p-4 bg-white dark:bg-slate-800">
        <div className="flex flex-col flex-1 md:h-[500px]">
          <textarea
            id="sruja-input"
            spellCheck={false}
            value={code}
            onChange={(e) => setCode(e.target.value)}
            className="w-full flex-1 font-mono p-2 border rounded"
            style={{
              backgroundColor: 'var(--sruja-editor-bg)',
              color: 'var(--sruja-editor-fg)',
              borderColor: 'var(--sruja-editor-border)'
            }}
          />
        </div>
        <div className="flex flex-col flex-1 md:h-[500px]">
          <div
            id="d2-output"
            ref={outputRef}
            className="w-full flex-1 min-h-0 overflow-auto border border-slate-300 dark:border-slate-600 relative bg-white dark:bg-slate-800"
          >
            {showToolbar && (
              <div className="preview-toolbar absolute top-2 right-2 flex gap-2 bg-slate-900/60 dark:bg-slate-900/60 border border-slate-700 dark:border-slate-600 rounded-lg p-1.5 backdrop-blur-sm">
                
                <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
                  <DialogTitle className="sr-only">Diagram Preview</DialogTitle>
                  <DialogDescription className="sr-only">Expanded view of the generated diagram</DialogDescription>
                  <Button title="Expand" variant="secondary" size="icon" className="h-8 w-8 p-0" onClick={() => setIsDialogOpen(true)}>
                    <span className="flex items-center justify-center w-full h-full leading-none text-center">⛶</span>
                  </Button>
                  <DialogContent className="max-w-[90vw] max-h-[90vh] p-0">
                    <div className="p-6">
                      <div dangerouslySetInnerHTML={{ __html: getSvgForDialog() }} />
                    </div>
                  </DialogContent>
                </Dialog>
              </div>
            )}
            {visualOutput && (
              <div ref={zoomContainerRef} className="zoom-container block w-full h-full">
                <div dangerouslySetInnerHTML={{ __html: visualOutput }} />
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

// Initialize playground when DOM is ready
export function initPlayground(containerId: string, initialCode?: string): void {
  const container = document.getElementById(containerId);
  if (!container) {
    console.error(`Container #${containerId} not found`);
    return;
  }
  // Clear existing content and create React root
  container.innerHTML = '';
  const root = createRoot(container);
  root.render(<Playground initialCode={initialCode} />);
}
