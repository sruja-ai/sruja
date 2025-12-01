// Playground React Component
import { useState, useEffect, useRef } from 'react';
import { createRoot } from 'react-dom/client';
import { compileSrujaCode } from '../utils/wasm';
import { sanitizeSvg } from '../utils/sanitize';
import { processCompileOutput } from '../utils/svg-processing';
import { useWasmReady } from '../hooks/useWasmReady';
import { validateCodeSize, formatBytes, MAX_CODE_SIZE } from '../utils/validation';
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
  const wasmReady = useWasmReady();
  const [error, setError] = useState('');
  const [sizeWarning, setSizeWarning] = useState('');
  const [visualOutput, setVisualOutput] = useState<string>('');
  const [showToolbar, setShowToolbar] = useState(false);
  const [exportFormat, setExportFormat] = useState<'d2' | 'svg'>('svg');
  const outputRef = useRef<HTMLDivElement>(null);
  const zoomContainerRef = useRef<HTMLDivElement>(null);
  const hasAutoRun = useRef(false);

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

  // Handle code input with validation
  const handleCodeChange = (newCode: string) => {
    // Validate size
    const validation = validateCodeSize(newCode);
    if (!validation.valid) {
      setSizeWarning(validation.error || '');
      return; // Don't update code if it exceeds limit
    }
    setSizeWarning('');
    setCode(newCode);
  };

  // codacy-ignore: complexity - UI handler with multiple validation/error paths is expected
  const handleRun = () => {
    if (!wasmReady) {
      setError('WASM not ready');
      return;
    }

    // Validate code size before compilation
    const validation = validateCodeSize(code);
    if (!validation.valid) {
      setError(validation.error || 'Code size exceeds limit');
      return;
    }

    setError('');
    setVisualOutput('');

    try {
      const filename = 'playground.sruja';
      const result = compileSrujaCode(code, filename, exportFormat);

      if (!result) {
        setError('Compilation failed');
        return;
      }

      if (result.error) {
        setError(result.error);
      } else {
        const output = processCompileOutput(result);
        if (output) {
          setVisualOutput(output);
        } else {
          setError('No output');
        }
      }
    } catch (e) {
      setError('Internal Error: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  // Auto-run once when WASM is ready and we have code but no output
  useEffect(() => {
    if (wasmReady && !hasAutoRun.current && !visualOutput && code?.trim().length > 0) {
      hasAutoRun.current = true;
      const timer = setTimeout(() => {
        handleRun();
      }, 50);
      return () => clearTimeout(timer);
    }
  }, [wasmReady, code, visualOutput]); // Include dependencies

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
      const svgString = cloneSvg.outerHTML;
      return sanitizeSvg(svgString);
    }
    const img = outputRef.current?.querySelector('img');
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
  };

  return (
    <div>
      <div className="flex flex-col sm:flex-row items-start sm:items-center gap-3 sm:gap-4 mb-4 flex-wrap">
        <div className="flex items-center gap-3 flex-1 min-w-0">
          <label htmlFor="example-select" className="font-semibold whitespace-nowrap">Examples</label>
          <Select value={selectedExample} onValueChange={handleExampleChange}>
            <SelectTrigger id="example-select" className="w-full sm:w-[180px]">
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
        </div>
        <div className="flex items-center gap-3 flex-wrap">
          <span id="status" className="text-slate-500 dark:text-slate-400 text-sm whitespace-nowrap">
            {wasmReady ? 'Ready' : 'Loading WASM...'}
          </span>
          <Select value={exportFormat} onValueChange={(value: 'd2' | 'svg') => setExportFormat(value)}>
            <SelectTrigger className="w-[120px] min-h-[44px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="svg">Sruja Format</SelectItem>
              <SelectItem value="d2">D2 Format</SelectItem>
            </SelectContent>
          </Select>
          <Button
            id="run-btn"
            variant="default"
            size="sm"
            aria-label="Run"
            onClick={handleRun}
            disabled={!wasmReady}
            className="px-4 min-h-[44px]"
          >
            <span className="inline-flex items-center gap-2"><span className="leading-none">▶</span><span>Run</span></span>
          </Button>
        </div>
        {error && (
          <div id="error-output" className="text-red-600 dark:text-red-400 whitespace-pre-wrap text-sm w-full">
            {error}
          </div>
        )}
      </div>
      <div className="flex flex-col md:flex-row gap-4 border border-slate-300 dark:border-slate-600 rounded-lg p-3 sm:p-4 bg-white dark:bg-slate-800">
        <div className="flex flex-col flex-1 md:h-[500px] min-h-[300px]">
          <textarea
            id="sruja-input"
            spellCheck={false}
            value={code}
            onChange={(e) => handleCodeChange(e.target.value)}
            maxLength={MAX_CODE_SIZE * 2} // Allow typing, but validate before compilation
            className="w-full flex-1 font-mono p-2 sm:p-3 border rounded text-sm sm:text-base resize-none"
            style={{
              backgroundColor: 'var(--sruja-editor-bg)',
              color: 'var(--sruja-editor-fg)',
              borderColor: 'var(--sruja-editor-border)'
            }}
            aria-label="Sruja code input"
          />
          {sizeWarning && (
            <div className="text-xs text-yellow-600 dark:text-yellow-400 mt-1">
              {sizeWarning}
            </div>
          )}
        </div>
        <div className="flex flex-col flex-1 md:h-[500px] min-h-[300px]">
          <div
            id="d2-output"
            ref={outputRef}
            className="w-full flex-1 min-h-0 overflow-auto border border-slate-300 dark:border-slate-600 relative bg-white dark:bg-slate-800 rounded"
          >
            {showToolbar && (
              <div className="preview-toolbar absolute top-2 right-2 flex gap-2 bg-slate-900/60 dark:bg-slate-900/60 border border-slate-700 dark:border-slate-600 rounded-lg p-1.5 backdrop-blur-sm z-10">
                
                <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
                  <DialogTitle className="sr-only">Diagram Preview</DialogTitle>
                  <DialogDescription className="sr-only">Expanded view of the generated diagram</DialogDescription>
                  <Button title="Expand" variant="secondary" size="icon" className="h-8 w-8 sm:h-10 sm:w-10 p-0 min-h-[44px] min-w-[44px]" onClick={() => setIsDialogOpen(true)}>
                    <span className="flex items-center justify-center w-full h-full leading-none text-center text-lg">⛶</span>
                  </Button>
                  <DialogContent className="max-w-[95vw] sm:max-w-[90vw] max-h-[95vh] sm:max-h-[90vh] p-0">
                    <div className="p-3 sm:p-6">
                      <div dangerouslySetInnerHTML={{ __html: sanitizeSvg(getSvgForDialog()) }} />
                    </div>
                  </DialogContent>
                </Dialog>
              </div>
            )}
            {visualOutput && (
              <div ref={zoomContainerRef} className="zoom-container block w-full h-full">
                <div dangerouslySetInnerHTML={{ __html: sanitizeSvg(visualOutput) }} />
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
