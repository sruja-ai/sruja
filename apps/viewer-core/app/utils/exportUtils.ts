// apps/viewer-core/app/utils/exportUtils.ts
import { logger, trackInteraction } from '@sruja/shared';
import { jsPDF } from 'jspdf';
import type { ArchitectureJSON } from '@sruja/viewer';
import type { ViewerInstance } from '@sruja/viewer';
import type { ExportOptions } from '../components/ExportDialog';

/**
 * Convert SVG to PNG using canvas
 * This allows us to use our custom SVG exporter for PDF generation
 */
async function svgToPng(svgString: string, scale: number = 2): Promise<string> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    const svgBlob = new Blob([svgString], { type: 'image/svg+xml;charset=utf-8' });
    const url = URL.createObjectURL(svgBlob);

    img.onload = () => {
      const canvas = document.createElement('canvas');
      canvas.width = img.width * scale;
      canvas.height = img.height * scale;
      const ctx = canvas.getContext('2d');
      
      if (!ctx) {
        reject(new Error('Failed to get canvas context'));
        return;
      }

      // Scale context for high resolution
      ctx.scale(scale, scale);
      ctx.drawImage(img, 0, 0);
      
      const dataUrl = canvas.toDataURL('image/png');
      URL.revokeObjectURL(url);
      resolve(dataUrl);
    };

    img.onerror = () => {
      URL.revokeObjectURL(url);
      reject(new Error('Failed to load SVG image'));
    };

    img.src = url;
  });
}

export async function exportDiagram(
  viewer: ViewerInstance,
  wasmApi: any,
  dsl: string,
  archData: ArchitectureJSON | null,
  options: ExportOptions,
  setToast: (toast: { message: string; type: 'success' | 'error' | 'info' } | null) => void
) {
  let filename = options.filename;
  if (options.includeTimestamp) {
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-').split('T')[0];
    filename = `${filename}_${timestamp}`;
  }
  filename = `${filename}.${options.format}`;

  if (options.format === 'png') {
    // Option 1: Use custom SVG exporter (if available) for consistent output
    // Option 2: Fallback to Cytoscape's built-in PNG export
    if (wasmApi?.dslToSvg && dsl) {
      try {
        const svg = await wasmApi.dslToSvg(dsl);
        if (svg) {
          const pngDataUrl = await svgToPng(svg, options.scale);
          const link = document.createElement('a');
          link.href = pngDataUrl;
          link.download = filename;
          document.body.appendChild(link);
          link.click();
          document.body.removeChild(link);
          setToast({ message: `Exported ${filename}`, type: 'success' });
          return;
        }
      } catch (err) {
        console.warn('Custom SVG→PNG export failed, falling back to Cytoscape:', err);
      }
    }

    // Fallback to Cytoscape's built-in PNG export
    if (typeof viewer.exportPNG === 'function') {
      const exportOptions = {
        scale: options.scale,
        full: true,
      };
      const content = viewer.exportPNG(exportOptions);
      if (content) {
        const link = document.createElement('a');
        link.href = content;
        link.download = filename;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        setToast({ message: `Exported ${filename}`, type: 'success' });
      }
    } else {
      setToast({ message: 'PNG export not available', type: 'error' });
    }
  } else if (options.format === 'svg') {
    try {
      let content: string | null = null;
      
      // Priority 1: Use custom WASM SVG exporter (best quality, consistent with DSL)
      if (wasmApi?.dslToSvg && dsl) {
        try {
          content = await wasmApi.dslToSvg(dsl);
        } catch (wasmError) {
          logger.warn('WASM SVG export failed, falling back to viewer', {
            component: 'viewer',
            action: 'export_svg',
            errorType: wasmError instanceof Error ? wasmError.constructor.name : 'unknown',
            error: wasmError instanceof Error ? wasmError.message : String(wasmError),
          });
        }
      }
      
      // Priority 2: Use viewer's custom SVG export (from Cytoscape elements)
      if (!content && typeof viewer.exportSVG === 'function') {
        content = viewer.exportSVG({ scale: options.scale, full: true });
      }
      
      // Priority 3: Use Cytoscape's built-in SVG export (if available)
      if (!content && viewer.cy) {
        try {
          content = viewer.cy.svg({ full: true });
        } catch (cyError) {
          logger.warn('Cytoscape SVG export failed', {
            component: 'viewer',
            action: 'export_svg',
            errorType: cyError instanceof Error ? cyError.constructor.name : 'unknown',
            error: cyError instanceof Error ? cyError.message : String(cyError),
          });
        }
      }
      
      if (!content) {
        throw new Error('SVG export not available');
      }

      // Add metadata if requested
      if (options.includeMetadata && archData) {
        const metadata = `
  <metadata>
    <title>${archData.metadata?.name || 'Architecture Diagram'}</title>
    <author>Sruja Viewer</author>
    <created>${new Date().toISOString()}</created>
    <version>${archData.metadata?.version || '1.0.0'}</version>
  </metadata>`;
        content = content.replace(/<svg[^>]*>/, (match: string) => `${match}\n${metadata}`);
      }

      if (content) {
        const blob = new Blob([content], { type: 'image/svg+xml;charset=utf-8' });
        const url = URL.createObjectURL(blob);
        const link = document.createElement('a');
        link.href = url;
        link.download = filename;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        URL.revokeObjectURL(url);
        setToast({ message: `Exported ${filename}`, type: 'success' });
      }
    } catch (error) {
      logger.error('SVG Export failed', {
        component: 'viewer',
        action: 'export_svg',
        errorType: error instanceof Error ? error.constructor.name : 'unknown',
        error: error instanceof Error ? error.message : String(error),
      });
      setToast({ message: 'Failed to export SVG', type: 'error' });
    }
  } else if (options.format === 'pdf') {
    try {
      let pngDataUrl: string | null = null;

      // Strategy: Use custom SVG exporter → convert to PNG → PDF
      // This ensures consistent output regardless of viewer state
      if (wasmApi?.dslToSvg && dsl) {
        try {
          const svg = await wasmApi.dslToSvg(dsl);
          if (svg) {
            // Convert SVG to high-resolution PNG for PDF
            pngDataUrl = await svgToPng(svg, Math.max(2, options.scale));
          }
        } catch (svgError) {
          logger.warn('SVG→PNG conversion failed, falling back to Cytoscape PNG', {
            component: 'viewer',
            action: 'export_png',
            errorType: svgError instanceof Error ? svgError.constructor.name : 'unknown',
            error: svgError instanceof Error ? svgError.message : String(svgError),
          });
        }
      }

      // Fallback: Use Cytoscape's built-in PNG export
      if (!pngDataUrl && typeof viewer.exportPNG === 'function') {
        const exportOptions = { scale: Math.max(2, options.scale), full: true };
        pngDataUrl = viewer.exportPNG(exportOptions);
      }

      if (!pngDataUrl) {
        setToast({ message: 'PDF export not available', type: 'error' });
        return;
      }

      const img = new Image();
      const loadPromise = new Promise<HTMLImageElement>((resolve, reject) => {
        img.onload = () => resolve(img);
        img.onerror = reject;
      });
      img.src = pngDataUrl;
      
      const loaded = await loadPromise;
      const imgW = loaded.naturalWidth;
      const imgH = loaded.naturalHeight;

      const orientation = imgW >= imgH ? 'landscape' : 'portrait';
      const doc = new jsPDF({ orientation, unit: 'pt', format: 'a4' });
      const pageSize = doc.internal.pageSize;
      const pageW = pageSize.getWidth();
      const pageH = pageSize.getHeight();
      const margin = 28;
      const usableW = pageW - margin * 2;
      const usableH = pageH - margin * 2;
      const scale = Math.min(usableW / imgW, usableH / imgH);
      const drawW = imgW * scale;
      const drawH = imgH * scale;
      const x = (pageW - drawW) / 2;
      const y = (pageH - drawH) / 2;

      doc.addImage(pngDataUrl, 'PNG', x, y, drawW, drawH);
      
      // Add branding footer
      const footerY = pageH - 15;
      doc.setFontSize(8);
      doc.setTextColor(128, 128, 128);
      doc.text('Powered by Sruja - Architecture as Code', pageW / 2, footerY, { align: 'center' });
      
      // Add metadata if requested
      if (options.includeMetadata && archData) {
        const metadata = {
          title: archData.metadata?.name || 'Architecture Diagram',
          author: 'Sruja Viewer',
          subject: 'Architecture Diagram',
          keywords: 'architecture, diagram, c4 model',
          creator: 'Sruja',
          producer: 'Sruja Viewer'
        };
        doc.setProperties(metadata);
      }

      doc.save(filename);
      setToast({ message: `Exported ${filename}`, type: 'success' });
    } catch (e) {
      logger.error('PDF export error', {
        component: 'viewer',
        action: 'export_pdf',
        errorType: e instanceof Error ? e.constructor.name : 'unknown',
        error: e instanceof Error ? e.message : String(e),
      });
      setToast({ message: 'Failed to export PDF', type: 'error' });
    }
  } else if (options.format === 'json') {
    if (archData) {
      const jsonStr = JSON.stringify(archData, null, 2);
      const blob = new Blob([jsonStr], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = filename;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);
      setToast({ message: `Exported ${filename}`, type: 'success' });
    }
  } else if (options.format === 'markdown') {
    if (!wasmApi) {
      setToast({ message: 'WASM API not available for markdown export', type: 'error' });
      return;
    }
    try {
      let markdown = await wasmApi.dslToMarkdown(dsl);
      if (markdown) {
        // Add branding footer to markdown
        const branding = '\n\n---\n\n*Powered by [Sruja](https://sruja.ai) - Architecture as Code*';
        markdown += branding;
        
        const blob = new Blob([markdown], { type: 'text/markdown' });
        const url = URL.createObjectURL(blob);
        const link = document.createElement('a');
        link.href = url;
        link.download = filename;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        URL.revokeObjectURL(url);
        setToast({ message: `Exported ${filename}`, type: 'success' });
      }
    } catch (error) {
      logger.error('Markdown export error', {
        component: 'viewer',
        action: 'export_markdown',
        errorType: error instanceof Error ? error.constructor.name : 'unknown',
        error: error instanceof Error ? error.message : String(error),
      });
      setToast({ message: 'Failed to export markdown', type: 'error' });
    }
  } else if (options.format === 'html') {
    if (!archData) {
      setToast({ message: 'Architecture data not available', type: 'error' });
      return;
    }
    try {
      const json = JSON.stringify(archData);
      // Escape for safe JSON embedding
      const escapedJson = json.replace(/</g, '\\u003c').replace(/>/g, '\\u003e');
      const html = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Architecture: ${archData.metadata?.name || 'Architecture'}</title>
  <link rel="stylesheet" href="./embed-viewer.css" />
  <style>
    body { margin: 0; padding: 0; font-family: system-ui, sans-serif; }
    #root { width: 100%; height: 100vh; }
    .sruja-footer {
      position: fixed;
      bottom: 0;
      left: 0;
      right: 0;
      background: rgba(255, 255, 255, 0.95);
      border-top: 1px solid #e2e8f0;
      padding: 8px 16px;
      text-align: center;
      font-size: 12px;
      color: #64748b;
      z-index: 1000;
    }
    .sruja-footer a {
      color: #3b82f6;
      text-decoration: none;
      font-weight: 500;
    }
    .sruja-footer a:hover {
      text-decoration: underline;
    }
  </style>
</head>
<body style="margin:0">
  <div id="root"></div>
  <div class="sruja-footer">
    Powered by <a href="https://sruja.ai" target="_blank" rel="noopener noreferrer">Sruja</a> - Architecture as Code
  </div>
  <script>
    window.__SRUJA_ARCH__ = ${escapedJson};
  </script>
  <script src="./embed-viewer.js"></script>
  <script>
    if (typeof SrujaViewer !== 'undefined' && SrujaViewer.mount) {
      SrujaViewer.mount('#root', window.__SRUJA_ARCH__);
    } else {
      logger.error('SrujaViewer not loaded', { component: 'viewer', action: 'export_html' });
      document.getElementById('root').innerHTML = '<div style="padding: 20px; text-align: center;"><p>Failed to load viewer. Please check your connection.</p></div>';
    }
  </script>
</body>
</html>`;
      
      const blob = new Blob([html], { type: 'text/html' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = filename;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);
      setToast({ message: `Exported ${filename}`, type: 'success' });
    } catch (error) {
      logger.error('HTML export error', {
        component: 'viewer',
        action: 'export_html',
        errorType: error instanceof Error ? error.constructor.name : 'unknown',
        error: error instanceof Error ? error.message : String(error),
      });
      setToast({ message: 'Failed to export HTML', type: 'error' });
    }
  }
}
