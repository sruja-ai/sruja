// apps/studio-core/src/utils/exportUtils.ts
import { jsPDF } from 'jspdf';
import type { ArchitectureJSON } from '@sruja/viewer';
import type { ViewerInstance } from '@sruja/viewer';
import type { ExportOptions } from '../components/ExportDialog';
import { logger } from '@sruja/shared';
import { trackInteraction } from '@sruja/shared';

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
    if (typeof viewer.exportPNG !== 'function') {
      logger.error('PNG export method not available', { component: 'studio', action: 'export_png' });
      return;
    }
    trackInteraction('export', 'png', { component: 'studio', scale: options.scale });
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
      trackInteraction('export_success', 'png', { component: 'studio', filename });
    }
  } else if (options.format === 'svg') {
    trackInteraction('export', 'svg', { component: 'studio', scale: options.scale });
    try {
      let content: string | null = null;
      
      // Try WASM API first
      if (wasmApi) {
        try {
          content = await wasmApi.dslToSvg(dsl);
        } catch (wasmError) {
          logger.warn('WASM SVG export failed, falling back to viewer', {
            component: 'studio',
            action: 'export_svg',
            errorType: 'wasm_fallback',
            error: wasmError instanceof Error ? wasmError.message : String(wasmError),
          });
        }
      }
      
      // Fallback to viewer.exportSVG if WASM failed or not available
      if (!content && typeof viewer.exportSVG === 'function') {
        content = viewer.exportSVG({ scale: options.scale, full: true });
      }
      
      if (!content) {
        throw new Error('SVG export not available');
      }

      // Add metadata if requested
      if (options.includeMetadata && archData) {
        const metadata = `
  <metadata>
    <title>${archData.metadata?.name || 'Architecture Diagram'}</title>
    <author>Sruja Studio</author>
    <created>${new Date().toISOString()}</created>
    <version>${archData.metadata?.version || '1.0.0'}</version>
  </metadata>`;
        // Insert metadata after the opening svg tag
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
        trackInteraction('export_success', 'svg', { component: 'studio', filename });
      }
    } catch (error) {
      logger.error('SVG Export failed', {
        component: 'studio',
        action: 'export_svg',
        errorType: error instanceof Error ? error.constructor.name : 'unknown',
        error: error instanceof Error ? error.message : String(error),
      });
      setToast({ message: 'Failed to export SVG', type: 'error' });
    }
  } else if (options.format === 'pdf') {
    trackInteraction('export', 'pdf', { component: 'studio', scale: options.scale });
    if (typeof viewer.exportPNG !== 'function') {
      logger.error('PNG export method not available for PDF', { component: 'studio', action: 'export_pdf' });
      return;
    }
    const exportOptions = { scale: Math.max(2, options.scale), full: true };
    const dataUrl = viewer.exportPNG(exportOptions);
    if (!dataUrl) return;

    const img = new Image();
    const loadPromise = new Promise<HTMLImageElement>((resolve, reject) => {
      img.onload = () => resolve(img);
      img.onerror = reject;
    });
    img.src = dataUrl;
    try {
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

      doc.addImage(dataUrl, 'PNG', x, y, drawW, drawH);
      doc.save(filename);
      setToast({ message: `Exported ${filename}`, type: 'success' });
      trackInteraction('export_success', 'pdf', { component: 'studio', filename });
    } catch (e) {
      logger.error('PDF export error', {
        component: 'studio',
        action: 'export_pdf',
        errorType: e instanceof Error ? e.constructor.name : 'unknown',
        error: e instanceof Error ? e.message : String(e),
      });
      setToast({ message: 'Failed to export PDF', type: 'error' });
    }
  } else if (options.format === 'json') {
    trackInteraction('export', 'json', { component: 'studio' });
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
      trackInteraction('export_success', 'json', { component: 'studio', filename });
    }
  }
}






