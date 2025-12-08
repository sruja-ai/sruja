import { useState, useCallback } from 'react';
import { loadExampleFile } from '@sruja/shared';
import { saveDslToStorage } from '../utils/storage';
import { updateUrlWithCode } from '../utils/urlState';
import type { ValidationStatus } from '../types';

export function useExampleLoader(
  setDsl: (dsl: string) => void,
  parseDslToJson: (dsl: string) => Promise<void>,
  setValidationStatus: (status: ValidationStatus) => void,
  urlUpdateTimeout: NodeJS.Timeout | null,
  setUrlUpdateTimeout: (timeout: NodeJS.Timeout | null) => void
) {
  const [selectedExample, setSelectedExample] = useState<string>('');
  const [isLoadingExample, setIsLoadingExample] = useState(false);

  const loadExample = useCallback(async (exampleFile: string) => {
    if (!exampleFile) return;
    
    setIsLoadingExample(true);
    try {
      const content = await loadExampleFile(exampleFile);
      if (!content || content.trim().length === 0) {
        throw new Error(`Example file is empty: ${exampleFile}`);
      }
      setDsl(content);
      saveDslToStorage(content);
      updateUrlWithCode(content, urlUpdateTimeout, setUrlUpdateTimeout);
      await parseDslToJson(content);
      // Clear any previous errors on successful load
      setValidationStatus({
        isValid: true,
        errors: 0,
        warnings: 0,
        lastError: undefined,
      });
    } catch (err) {
      console.error('Failed to load example:', err);
      setValidationStatus({
        isValid: false,
        errors: 1,
        warnings: 0,
        lastError: `Failed to load example: ${err instanceof Error ? err.message : 'Unknown error'}`,
      });
    } finally {
      setIsLoadingExample(false);
    }
  }, [setDsl, parseDslToJson, setValidationStatus, urlUpdateTimeout, setUrlUpdateTimeout]);

  return { selectedExample, setSelectedExample, isLoadingExample, loadExample };
}


