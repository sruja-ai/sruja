import { useState, useEffect, useCallback, useRef } from "react";
import { useArchitectureStore } from "../stores";
import { convertDslToModel } from "../wasm";
import { logger, type SrujaModelDump } from "@sruja/shared";

/**
 * Hook to sync DSL source code with the architecture model.
 *
 * Flow:
 * - User edits in editor → handleDSLChange → updates store → debounced parse → loadFromDSL
 * - Builder updates model → store dslSource changes → sync to editor (external update)
 *
 * Key: Only sync from store to editor when change came from outside (builder), not from editor itself.
 */
export function useDSLSync() {
  const storeDslSource = useArchitectureStore((s) => s.dslSource);
  const setDslSource = useArchitectureStore((s) => s.setDslSource);
  const loadFromDSL = useArchitectureStore((s) => s.loadFromDSL);

  const [dslSource, setLocalDslSource] = useState<string | null>(storeDslSource || null);
  const [error, setError] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);

  // Track if we're currently processing a user edit to prevent circular sync
  const isProcessingUserEditRef = useRef(false);
  // Track the last DSL we sent to the store to detect external changes
  const lastSentDslRef = useRef<string | null>(storeDslSource || null);
  // Track if this is the initial mount to handle initial sync correctly
  const isInitialMountRef = useRef(true);
  // Track timeout for clearing processing flag
  const processingTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  // Track the current parsing attempt to avoid race conditions
  const currentParseRef = useRef<number>(0);

  // Sync with store when store's dslSource changes externally (from builder, not from editor)
  useEffect(() => {
    // On initial mount, sync if store has DSL
    if (isInitialMountRef.current) {
      isInitialMountRef.current = false;
      if (storeDslSource && storeDslSource !== dslSource) {
        setLocalDslSource(storeDslSource);
        lastSentDslRef.current = storeDslSource;
      } else if (storeDslSource) {
        // Store has DSL and local state matches, just update ref
        lastSentDslRef.current = storeDslSource;
      }
      return;
    }

    // Skip sync if:
    // 1. Values are already the same (no change)
    // 2. We're processing a user edit (change came from editor, don't sync back)
    // 3. The store value matches what we last sent (change came from our own setDslSource)
    if (storeDslSource === dslSource) {
      return;
    }

    if (isProcessingUserEditRef.current) {
      // User is editing, don't sync from store back to editor
      return;
    }

    if (storeDslSource === lastSentDslRef.current) {
      // This change came from our own setDslSource call, don't sync
      return;
    }

    // This is an external change (from builder), sync to editor
    setLocalDslSource(storeDslSource || null);
    setError(null);
    lastSentDslRef.current = storeDslSource || null;
  }, [storeDslSource, dslSource]);

  // Handle DSL changes from user editing
  const handleDSLChange = useCallback(
    (newDsl: string) => {
      // Mark that we're processing a user edit
      isProcessingUserEditRef.current = true;

      // Immediate UI update (optimistic)
      setLocalDslSource(newDsl);
      setError(null);

      // Update store
      setDslSource(newDsl);
      lastSentDslRef.current = newDsl;

      // Clear the flag after a delay to allow store update to complete
      // Increased to 500ms to ensure we don't get "echoes" from store updates
      // caused by our own changes.
      if (processingTimeoutRef.current) {
        clearTimeout(processingTimeoutRef.current);
      }
      processingTimeoutRef.current = setTimeout(() => {
        isProcessingUserEditRef.current = false;
      }, 500);
    },
    [setDslSource]
  );

  // Debounced model sync: Parse DSL and update model
  useEffect(() => {
    if (dslSource === null) return;

    // Adaptive debounce: shorter for typing, longer for structural changes
    const debounceTime = dslSource.length < 100 ? 100 : dslSource.length < 500 ? 300 : 1000;

    const parseAttempt = ++currentParseRef.current;
    const timer = setTimeout(async () => {
      // Skip if this is not the latest parse attempt
      if (parseAttempt !== currentParseRef.current) {
        return;
      }

      setIsSaving(true);
      try {
        // Attempt to parse and convert DSL to model
        const model = await convertDslToModel(dslSource);
        if (model && typeof model === "object" && "elements" in model) {
          // Load the model into the store
          // loadFromDSL will only update dslSource if it's different (see store implementation)
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
    }, debounceTime);

    return () => clearTimeout(timer);
  }, [dslSource, loadFromDSL]);

  // Return sync state and handlers
  return {
    dslSource,
    error,
    isSaving,
    handleDSLChange,
    // Add sync status for visualization
    syncStatus: {
      isProcessing: isProcessingUserEditRef.current,
      isSyncing: isSaving,
      hasError: error !== null,
      lastSync: new Date().toISOString(),
    },
  };
}
