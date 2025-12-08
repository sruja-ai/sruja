import LZString from 'lz-string';
import { URL_UPDATE_DEBOUNCE_MS } from '../constants';
import type { PaneType, PreviewFormat } from '../types';

export const updateUrlWithCode = (
  dslText: string,
  urlUpdateTimeout: NodeJS.Timeout | null,
  setUrlUpdateTimeout: (timeout: NodeJS.Timeout | null) => void
) => {
  if (typeof window === 'undefined') return;

  // Clear any existing timeout
  if (urlUpdateTimeout) {
    clearTimeout(urlUpdateTimeout);
  }

  // Debounce URL updates to avoid too frequent history changes
  const timeout = setTimeout(() => {
    try {
      if (dslText && dslText.trim()) {
        const compressed = LZString.compressToBase64(dslText);
        const newUrl = `${window.location.pathname}#code=${encodeURIComponent(compressed)}`;
        // Use replaceState to update URL without page reload
        window.history.replaceState(null, '', newUrl);
      } else {
        // If DSL is empty, remove code from URL
        const newUrl = window.location.pathname;
        window.history.replaceState(null, '', newUrl);
      }
    } catch (e) {
      console.warn('Failed to update URL:', e);
    }
  }, URL_UPDATE_DEBOUNCE_MS);

  setUrlUpdateTimeout(timeout);
};

export const copyShareUrl = async (dsl: string): Promise<boolean> => {
  if (typeof window === 'undefined') return false;

  try {
    let shareUrl = window.location.origin + window.location.pathname;

    if (dsl && dsl.trim()) {
      const compressed = LZString.compressToBase64(dsl);
      shareUrl = `${shareUrl}#code=${encodeURIComponent(compressed)}`;
    }

    await navigator.clipboard.writeText(shareUrl);
    return true;
  } catch (e) {
    console.error('Failed to copy URL:', e);
    // Fallback for older browsers
    const textArea = document.createElement('textarea');
    textArea.value = window.location.href;
    textArea.style.position = 'fixed';
    textArea.style.opacity = '0';
    document.body.appendChild(textArea);
    textArea.select();
    try {
      document.execCommand('copy');
      return true;
    } catch (err) {
      console.error('Fallback copy failed:', err);
      return false;
    } finally {
      document.body.removeChild(textArea);
    }
  }
};

/**
 * Parse URL query params for view control
 * Supported params:
 * - view: 'code' | 'preview' | 'split' (default: split)
 * - preview: 'diagram' | 'json' | 'html' | 'markdown' (default: diagram)
 * 
 * Examples:
 * - ?view=preview&preview=html  - Show HTML preview only
 * - ?view=code                  - Show editor only
 * - ?view=split&preview=json    - Split view with JSON preview
 */
export interface ViewParams {
  pane?: PaneType;
  preview?: PreviewFormat;
}

export const parseViewParams = (): ViewParams => {
  if (typeof window === 'undefined') return {};

  const params = new URLSearchParams(window.location.search);
  const result: ViewParams = {};

  // Parse view param (code/preview/split)
  const viewParam = params.get('view');
  if (viewParam) {
    const viewMap: Record<string, PaneType> = {
      'code': 'editor',
      'editor': 'editor',
      'preview': 'preview', // Will be updated based on preview param
      'split': 'split',
      'diagram': 'diagram',
      'json': 'json',
      'html': 'preview',
      'markdown': 'markdown',
    };
    if (viewParam in viewMap) {
      result.pane = viewMap[viewParam];
    }
  }

  // Parse preview param (diagram/json/html/markdown)
  const previewParam = params.get('preview');
  if (previewParam) {
    const previewMap: Record<string, PreviewFormat> = {
      'diagram': 'diagram',
      'json': 'json',
      'html': 'preview',
      'preview': 'preview',
      'markdown': 'markdown',
    };
    if (previewParam in previewMap) {
      result.preview = previewMap[previewParam];
      // If view=preview and preview is specified, adjust pane accordingly
      if (result.pane === 'preview') {
        // Map preview format to correct pane type
        const previewToPaneMap: Record<PreviewFormat, PaneType> = {
          'diagram': 'diagram',
          'json': 'json',
          'preview': 'preview',
          'markdown': 'markdown',
        };
        result.pane = previewToPaneMap[result.preview];
      }
    }
  }

  return result;
};
