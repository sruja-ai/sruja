import { useState, useEffect } from 'react';
import { loadExamplesManifest, type Example } from '@sruja/shared';

export function useExamples() {
  const [examples, setExamples] = useState<Example[]>([]);

  useEffect(() => {
    if (typeof window === 'undefined') return;
    
    loadExamplesManifest()
      .then(manifest => {
        // Filter out examples that should be skipped in playground
        const availableExamples = manifest.examples
          .filter((ex: Example) => !ex.skipPlayground)
          .sort((a: Example, b: Example) => a.order - b.order);
        setExamples(availableExamples);
      })
      .catch(err => {
        console.error('Failed to load examples manifest:', err);
      });
  }, []);

  return examples;
}


