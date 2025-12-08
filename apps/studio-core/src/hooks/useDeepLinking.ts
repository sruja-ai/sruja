// apps/studio-core/src/hooks/useDeepLinking.ts
import { useEffect } from 'react';

interface UseDeepLinkingOptions {
  activeStep: string | null;
  focusPath: string[];
  setStep: (step: any) => void;
  setFocusPath: (path: string[]) => void;
  setActiveView?: (view: 'editor' | 'split' | 'viewer') => void;
}

/**
 * Hook for deep linking - sync state to/from URL
 */
export function useDeepLinking({
  activeStep,
  focusPath,
  setStep,
  setFocusPath,
  setActiveView,
}: UseDeepLinkingOptions): void {
  // Deep Linking: Sync state to URL
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);

    if (activeStep) params.set('step', activeStep);
    if (focusPath.length > 0) params.set('focus', focusPath.join('.'));

    const newUrl = `${window.location.pathname}?${params.toString()}`;
    window.history.replaceState(null, '', newUrl);
  }, [activeStep, focusPath]);

  // Deep Linking: Hydrate state from URL (Run once)
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const step = params.get('step');
    const focus = params.get('focus');

    // Legacy mode param removed - builder and normal mode are now merged

    if (step) {
      setStep(step);
    }

    if (focus) {
      const pathParts = focus.split('.').filter(Boolean);
      if (pathParts.length > 0) {
        setFocusPath(pathParts);
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []); // Only run once on mount
}
