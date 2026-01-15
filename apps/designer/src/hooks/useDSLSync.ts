import { useState, useEffect, useCallback } from "react";
import { useArchitectureStore } from "../stores";
import { convertDslToModel } from "../wasm";
import { logger, type SrujaModelDump } from "@sruja/shared";

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
  const loadFromDSL = useArchitectureStore((s) => s.loadFromDSL);

  const [dslSource, setLocalDslSource] = useState<string | null>(storeDslSource || null);
  const [error, setError] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);

  // Sync with store when store's dslSource changes externally
  useEffect(() => {
    if (storeDslSource !== dslSource) {
      setLocalDslSource(storeDslSource || null);
      setError(null);
    }
  }, [storeDslSource]);

  // Handle DSL changes with debouncing and validation
  const handleDSLChange = useCallback(
    (newDsl: string) => {
      // Immediate UI update
      setLocalDslSource(newDsl);
      setError(null);
      setDslSource(newDsl);
    },
    [setDslSource]
  );

  // Debounced model sync
  useEffect(() => {
    if (dslSource === null) return;

    const timer = setTimeout(async () => {
      setIsSaving(true);
      try {
        // Attempt to parse and convert DSL to model
        const model = await convertDslToModel(dslSource);
        if (model && typeof model === "object" && "elements" in model) {
          // Load the model into the store
          await loadFromDSL(model as SrujaModelDump, dslSource);
          setError(null);
        } else {
          setError("Failed to parse DSL. Please check the syntax.");
        }
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        logger.error("DSL sync error", {
          component: "useDSLSync",
          error: errorMessage,
        });
        setError(errorMessage);
        // Don't update the model on error, but keep the DSL source
      } finally {
        setIsSaving(false);
      }
    }, 1000); // 1s debounce

    return () => clearTimeout(timer);
  }, [dslSource, loadFromDSL]);

  return {
    dslSource,
    error,
    isSaving,
    handleDSLChange,
  };
}
