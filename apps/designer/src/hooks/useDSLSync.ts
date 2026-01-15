import { useState, useEffect, useCallback, useRef } from "react";
import { useArchitectureStore } from "../stores";

/**
 * Hook to sync DSL source code with the architecture model.
 *
 * When DSL is edited, it:
 * 1. Updates the store's dslSource immediately (for UI responsiveness)
 * 2. Attempts to parse and convert DSL to model
 * 3. Updates the model if conversion succeeds
 * 4. Shows error if conversion fails
 */
export function useDSLSync() {
  const storeDslSource = useArchitectureStore((s) => s.dslSource);
  const setDslSource = useArchitectureStore((s) => s.setDslSource);
  const refreshConvertedJson = useArchitectureStore((s) => s.refreshConvertedJson);

  const [dslSource, setLocalDslSource] = useState<string | null>(storeDslSource || null);
  const [error, setError] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);
  const pendingSyncRef = useRef(false);

  // Sync with store when store's dslSource changes externally
  useEffect(() => {
    if (storeDslSource !== dslSource) {
      setLocalDslSource(storeDslSource || null);
      setError(null);
    }
  }, [storeDslSource, dslSource]);

  // Handle DSL changes with debouncing and validation
  const handleDSLChange = useCallback(
    (newDsl: string) => {
      // Immediate UI update
      setLocalDslSource(newDsl);
      setError(null);
      pendingSyncRef.current = true;
      setDslSource(newDsl, null, { syncModel: false });
    },
    [setDslSource]
  );

  // Debounced model sync
  useEffect(() => {
    if (dslSource === null) return;
    if (!pendingSyncRef.current) return;
    pendingSyncRef.current = false;

    const timer = setTimeout(async () => {
      setIsSaving(true);
      const result = await refreshConvertedJson();
      setError(result.error);
      setIsSaving(false);
    }, 250); // debounce for responsive sync

    return () => clearTimeout(timer);
  }, [dslSource, refreshConvertedJson]);

  return {
    dslSource,
    error,
    isSaving,
    handleDSLChange,
  };
}
