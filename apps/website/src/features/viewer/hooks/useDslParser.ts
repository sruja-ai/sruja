import { useCallback } from 'react';
import type { WasmApi } from '@sruja/shared';
import type { ArchitectureJSON, ValidationStatus } from '../types';
import { generateHtmlPreview, generateMarkdownPreview, generatePdfPreview } from '../utils/previews';

export function useDslParser(
  wasmApiRef: React.MutableRefObject<WasmApi | null>,
  setArchData: (data: ArchitectureJSON | null) => void,
  setValidationStatus: (status: ValidationStatus) => void,
  setIsParsingDsl: (loading: boolean) => void,
  setHtmlPreview: (html: string) => void,
  setIsGeneratingHtml: (loading: boolean) => void,
  setMarkdownPreview: (markdown: string) => void,
  setIsGeneratingMarkdown: (loading: boolean) => void,
  setPdfPreviewUrl: (url: string) => void,
  setIsGeneratingPdf: (loading: boolean) => void,
  archData: ArchitectureJSON | null
) {
  const parseDslToJson = useCallback(async (dslText: string) => {
    if (!wasmApiRef.current) {
      // Wait for WASM to be ready
      setTimeout(() => parseDslToJson(dslText), 100);
      return;
    }

    setIsParsingDsl(true);
    try {
      const jsonStr = await wasmApiRef.current.parseDslToJson(dslText);
      const parsed = JSON.parse(jsonStr);
      setArchData(parsed);
      setValidationStatus({
        isValid: true,
        errors: 0,
        warnings: 0,
        lastError: undefined,
      });

      // Generate previews using DSL
      // Use the newly parsed data for PDF generation instead of stale archData
      await generateHtmlPreview(dslText, wasmApiRef.current, setHtmlPreview, setIsGeneratingHtml);
      await generateMarkdownPreview(dslText, wasmApiRef.current, setMarkdownPreview, setIsGeneratingMarkdown);
      await generatePdfPreview(dslText, parsed, wasmApiRef.current, setPdfPreviewUrl, setIsGeneratingPdf);
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Unknown error';
      // Provide more helpful error message for common syntax errors
      let friendlyError = errorMsg;
      if (errorMsg.includes('unexpected token "["')) {
        friendlyError = 'Syntax error: Use quoted strings for relation labels (e.g., `A -> B "Label"`), not square brackets.';
      }
      
      setValidationStatus({
        isValid: false,
        errors: 1,
        warnings: 0,
        lastError: friendlyError,
      });
    } finally {
      setIsParsingDsl(false);
    }
  }, [
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
    // Removed archData from dependencies to prevent infinite loops
  ]);

  return parseDslToJson;
}

