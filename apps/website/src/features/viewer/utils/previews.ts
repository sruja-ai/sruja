import { jsPDF } from 'jspdf';
import React from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { createRoot } from 'react-dom/client';
import type { WasmApi } from '@sruja/shared';
import type { ArchitectureJSON } from '@sruja/viewer';

export const generateHtmlPreview = async (
  dslText: string,
  wasmApi: WasmApi | null,
  setHtmlPreview: (html: string) => void,
  setIsGeneratingHtml: (loading: boolean) => void
) => {
  if (!wasmApi) return;

  setIsGeneratingHtml(true);
  try {
    // Check if HTML export is available
    if (!wasmApi.dslToHtml) {
      throw new Error('HTML export is not available. The WASM module needs to be rebuilt with HTML support.');
    }
    // Use Go HTML exporter from WASM - this generates proper standalone HTML
    const html = await wasmApi.dslToHtml(dslText);
    const instrumented = injectPreviewDiagnostics(html);
    setHtmlPreview(instrumented);
  } catch (err) {
    console.error('Failed to generate HTML preview:', err);
    // Fallback: show error message
    const errorHtml = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Preview Error</title>
  <style>
    body { margin: 0; padding: 40px 20px; font-family: system-ui, sans-serif; }
    .preview-message {
      text-align: center;
      color: var(--color-text-secondary);
      max-width: 600px;
      margin: 0 auto;
    }
    .preview-message h3 {
      color: var(--color-error-500);
      margin-bottom: 12px;
      font-size: 18px;
    }
    .preview-message code {
      background: var(--color-surface);
      padding: 2px 6px;
      border-radius: 4px;
      font-size: 14px;
      font-family: 'Monaco', 'Menlo', monospace;
    }
  </style>
</head>
<body>
  <div class="preview-message">
    <h3>HTML Export Error</h3>
    <p>Failed to generate HTML preview: ${err instanceof Error ? err.message : 'Unknown error'}</p>
    <p style="margin-top: 16px; font-size: 14px;">Please check the browser console for details.</p>
  </div>
</body>
</html>`;
    setHtmlPreview(errorHtml);
  } finally {
    setIsGeneratingHtml(false);
  }
};

function injectPreviewDiagnostics(html: string): string {
  const snippet = `\n<script>\n(function(){\n  function safe(v){ try { return typeof v === 'string' ? v : (v && v.stack) || JSON.stringify(v) } catch(_) { return String(v) } }\n  window.addEventListener('error', function(e){\n    try {\n      parent.postMessage({\n        type: 'sruja:html-preview:error',\n        message: e.message, filename: e.filename, lineno: e.lineno, colno: e.colno, stack: e.error ? e.error.stack : null\n      }, '*')\n    } catch(_){}\n  });\n  var origError = console.error.bind(console);\n  console.error = function(){\n    try {\n      parent.postMessage({ type: 'sruja:html-preview:console', level: 'error', args: Array.from(arguments).map(safe) }, '*')\n    } catch(_){}\n    origError.apply(console, arguments);\n  };\n})();\n</script>\n`;
  if (html.includes('</head>')) return html.replace('</head>', snippet + '</head>');
  if (html.includes('<body')) return html.replace('<body>', '<body>' + snippet);
  return snippet + html;
}

export const generateMarkdownPreview = async (
  dslText: string,
  wasmApi: WasmApi | null,
  setMarkdownPreview: (markdown: string) => void,
  setIsGeneratingMarkdown: (loading: boolean) => void
) => {
  if (!wasmApi) return;

  setIsGeneratingMarkdown(true);
  try {
    const markdown = await wasmApi.dslToMarkdown(dslText);
    setMarkdownPreview(markdown);
  } catch (err) {
    console.error('Failed to generate markdown preview:', err);
    setMarkdownPreview(`# Error\n\nFailed to generate markdown preview: ${err instanceof Error ? err.message : 'Unknown error'}`);
  } finally {
    setIsGeneratingMarkdown(false);
  }
};

export const generatePdfPreview = async (
  dslText: string,
  archData: ArchitectureJSON | null,
  wasmApi: WasmApi | null,
  setPdfPreviewUrl: (url: string) => void,
  setIsGeneratingPdf: (loading: boolean) => void
) => {
  if (!wasmApi) return;

  setIsGeneratingPdf(true);
  try {
    // Generate markdown using WASM
    const markdown = await wasmApi.dslToMarkdown(dslText);
    
    // Create a temporary container to render markdown
    const tempDiv = document.createElement('div');
    tempDiv.style.position = 'absolute';
    tempDiv.style.left = '-9999px';
    tempDiv.style.width = '800px';
    tempDiv.style.padding = '20px';
    document.body.appendChild(tempDiv);

    // Render markdown using react-markdown
    const root = createRoot(tempDiv);
    root.render(
      React.createElement(ReactMarkdown, { remarkPlugins: [remarkGfm] }, markdown)
    );

    // Wait for React to render
    await new Promise(resolve => setTimeout(resolve, 100));

    // Extract HTML content
    const htmlContent = tempDiv.innerHTML;

    // Clean up
    root.unmount();
    document.body.removeChild(tempDiv);

    // Wrap in proper HTML structure with styles
    const html = `
      <!DOCTYPE html>
      <html>
      <head>
        <meta charset="UTF-8">
        <style>
          body { 
            font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; 
            padding: 20px; 
            line-height: 1.6; 
            color: var(--color-text-primary);
          }
          h1 { font-size: 24px; margin-top: 20px; margin-bottom: 10px; font-weight: 700; }
          h2 { font-size: 20px; margin-top: 18px; margin-bottom: 8px; font-weight: 600; }
          h3 { font-size: 18px; margin-top: 16px; margin-bottom: 6px; font-weight: 600; }
          h4 { font-size: 16px; margin-top: 14px; margin-bottom: 4px; font-weight: 600; }
          p { margin: 10px 0; }
          ul, ol { margin: 10px 0; padding-left: 30px; }
          li { margin: 5px 0; }
          code { 
            background: var(--color-surface); 
            padding: 2px 6px; 
            border-radius: 3px; 
            font-family: 'Monaco', 'Menlo', 'Courier New', monospace;
            font-size: 0.9em;
          }
          pre { 
            background: var(--color-surface); 
            padding: 10px; 
            border-radius: 5px; 
            overflow-x: auto;
            margin: 10px 0;
          }
          pre code { 
            background: none; 
            padding: 0; 
          }
          table {
            border-collapse: collapse;
            width: 100%;
            margin: 10px 0;
          }
          table th, table td {
            border: 1px solid var(--color-border);
            padding: 8px;
            text-align: left;
          }
          table th {
            background-color: var(--color-surface);
            font-weight: 600;
          }
          blockquote {
            border-left: 4px solid var(--color-border);
            padding-left: 16px;
            margin: 10px 0;
            color: var(--color-text-secondary);
          }
          a {
            color: var(--color-primary);
            text-decoration: none;
          }
        </style>
      </head>
      <body>
        ${htmlContent}
      </body>
      </html>
    `;

    // Create PDF using jsPDF
    const pdf = new jsPDF({
      orientation: 'portrait',
      unit: 'mm',
      format: 'a4'
    });

    // Generate PDF and create blob URL for preview
    await pdf.html(html, {
      callback: (doc) => {
        const pdfBlob = doc.output('blob');
        const url = URL.createObjectURL(pdfBlob);
        setPdfPreviewUrl(url);
      },
      x: 10,
      y: 10,
      width: 190,
      windowWidth: 800
    });
  } catch (err) {
    console.error('Failed to generate PDF preview:', err);
    setPdfPreviewUrl('');
  } finally {
    setIsGeneratingPdf(false);
  }
};
