// apps/designer/src/hooks/useProjectSync.ts
import { useEffect, useState, useCallback } from "react";
import LZString from "lz-string";
import { useArchitectureStore, useToastStore, useUIStore } from "../stores";
import { firebaseShareService } from "../utils/firebaseShareService";
import { getFirebaseConfig } from "../config/firebase";
// import { convertJsonToDsl } from "../utils/jsonToDsl"; // Legacy, remove if not needed or update to use new types
import { convertDslToModel } from "../wasm";
import { getAllExamples, fetchExampleDsl } from "../examples";
import { safeAsync, handleError, getUserFriendlyMessage, ErrorType } from "../utils/errorHandling";
import type { SrujaModelDump } from "@sruja/shared";

/**
 * Hook for project synchronization and initialization.
 *
 * Handles:
 * - Firebase initialization and configuration
 * - Real-time project synchronization from URL
 * - Autosave to Firebase (debounced, 2 seconds)
 * - Loading projects from URL parameters (project, code, dsl)
 * - Loading demo examples on first visit
 *
 * Automatically initializes on mount and handles URL-based project loading.
 * Supports both new project URLs and legacy share/code/dsl parameters.
 *
 * @returns Object containing loading state and demo loader
 *
 * @example
 * ```tsx
 * const { isLoadingFile, loadDemo } = useProjectSync();
 *
 * if (isLoadingFile) {
 *   return <LoadingSpinner />;
 * }
 * ```
 */
export function useProjectSync() {
  const storeDslSource = useArchitectureStore((s) => s.dslSource);
  const loadFromDSL = useArchitectureStore((s) => s.loadFromDSL);
  const setActiveTab = useUIStore((s) => s.setActiveTab);
  const showToast = useToastStore((s) => s.showToast);

  const [isLoadingFile, setIsLoadingFile] = useState(false);

  // Initialize Firebase on mount
  useEffect(() => {
    const config = getFirebaseConfig();
    if (config) {
      firebaseShareService.initialize(config).catch(() => {
        // Firebase initialization failure is non-critical, silently handle
        // The service will gracefully degrade if Firebase is unavailable
      });
    }
  }, []);

  // Autosave: Save to Firebase when DSL changes (debounced)
  useEffect(() => {
    const { projectId } = firebaseShareService.parseUrl(window.location.href);
    const currentProjectId = firebaseShareService.getCurrentProjectId();

    // Only autosave if we're on a project URL and have data
    // For new model, we rely on dslSource being present.
    if (!projectId || projectId !== currentProjectId || !storeDslSource) {
      return;
    }

    // Debounce saves (wait 2 seconds after last change)
    const timeoutId = setTimeout(async () => {
      const { error } = await safeAsync(
        async () => {
          // We simply save the current DSL source.
          // We don't convert JSON->DSL here anymore as the store maintains DSL source.
          await firebaseShareService.saveProject(storeDslSource);
        },
        "Autosave failed",
        ErrorType.NETWORK
      );
      if (error) {
        handleError(error, "useProjectSync.autosave");
        // Don't show toast for autosave failures to avoid spam
      }
    }, 2000);

    return () => clearTimeout(timeoutId);
  }, [storeDslSource]);

  const loadDemo = useCallback(async () => {
    // Don't load demo if URL has project, DSL, or legacy share params
    // But allow loading if example param is present (even if other params exist, example takes precedence)
    const { projectId } = firebaseShareService.parseUrl(window.location.href);
    const params = new URLSearchParams(window.location.search);
    const exampleParam = params.get("example");

    // If example param is explicitly provided, always try to load it
    // Otherwise, skip if project/share/dsl/code params exist
    if (
      !exampleParam &&
      (projectId || params.get("share") || params.get("dsl") || params.get("code"))
    ) {
      return;
    }

    try {
      setIsLoadingFile(true);
      const examples = await getAllExamples();
      const exampleParam = params.get("example");
      let targetExample = examples[0];

      if (exampleParam) {
        const found = examples.find((ex) => ex.file === exampleParam);
        if (found) targetExample = found;
      }

      if (targetExample) {
        const content = await fetchExampleDsl(targetExample.file);
        // Assume all examples are now DSL based or can be parsed
        const model = await convertDslToModel(content);
        if (model) {
          loadFromDSL(model as SrujaModelDump, content, targetExample.file);
          // Only set tab if not already specified in URL to avoid flickering
          const tabParam = params.get("tab");
          if (!tabParam) {
            setActiveTab("diagram");
          }
          setIsLoadingFile(false);
          return;
        }
      }
    } catch {}

    // Fallback
    const fallbackDsl = `person = kind "Person"
system = kind "System"

user = person "User"
web = system "WebApp"
user -> web "uses"
`;
    const { error: fallbackError, data: fallbackModel } = await safeAsync(
      () => convertDslToModel(fallbackDsl),
      "Failed to load fallback demo",
      ErrorType.UNKNOWN
    );

    if (fallbackError) {
      handleError(fallbackError, "useProjectSync.loadDemo.fallback");
      const errorMessage = getUserFriendlyMessage(fallbackError);
      showToast(`Failed to load demo: ${errorMessage}`, "error");
    } else if (fallbackModel) {
      loadFromDSL(fallbackModel as SrujaModelDump, fallbackDsl, "fallback");
      setActiveTab("diagram");
    }
    setIsLoadingFile(false);
  }, [loadFromDSL, setActiveTab, showToast]);

  // Unified initialization logic (Load from URL or Demo)
  useEffect(() => {
    const init = async () => {
      const { projectId, keyBase64 } = firebaseShareService.parseUrl(window.location.href);
      const params = new URLSearchParams(window.location.search);
      const codeParam = params.get("code");
      const dslParam = params.get("dsl");
      const shareParam = params.get("share");

      // 1. Handle Project URL (Real-time sync)
      if (projectId && keyBase64) {
        setIsLoadingFile(true);
        let hasInitialLoad = false;
        let lastReceivedDsl: string | null = null;

        const unsubscribe = firebaseShareService.loadProjectRealtime(
          projectId,
          keyBase64,
          async (dsl: string) => {
            try {
              if (lastReceivedDsl === dsl) return;
              lastReceivedDsl = dsl;

              const model = await convertDslToModel(dsl);
              if (model) {
                loadFromDSL(model as SrujaModelDump, dsl, undefined);
                if (!hasInitialLoad) {
                  // Only set tab if not already specified in URL to avoid flickering
                  const tabParam = new URLSearchParams(window.location.search).get("tab");
                  if (!tabParam) {
                    setActiveTab("diagram");
                  }
                  setIsLoadingFile(false);
                  hasInitialLoad = true;
                }
              }
            } catch (err) {
              if (!hasInitialLoad) {
                // Error already handled by safeAsync wrapper, no need to log again
                const errorMessage = err instanceof Error ? err.message : "Unknown error";
                if (errorMessage.includes("Cannot decrypt")) {
                  showToast("Cannot decrypt project. Invalid key or corrupted data.", "error");
                } else if (errorMessage.includes("not found")) {
                  showToast(
                    "Project not found. It may have been deleted or the link is invalid.",
                    "error"
                  );
                } else {
                  showToast(`Failed to load project: ${errorMessage}`, "error");
                }
                setIsLoadingFile(false);
              }
            }
          }
        );
        return () => unsubscribe();
      }

      // 2. Handle Legacy "code" param
      if (codeParam) {
        setIsLoadingFile(true);
        const { error, data: res } = await safeAsync(
          async () => {
            const decompressed = LZString.decompressFromBase64(decodeURIComponent(codeParam));
            if (!decompressed) {
              throw new Error("Failed to decompress code parameter");
            }
            const converted = await convertDslToModel(decompressed);
            if (!converted) {
              throw new Error("Failed to convert decompressed DSL to Model");
            }
            return { model: converted, dsl: decompressed };
          },
          "Failed to load from code parameter",
          ErrorType.VALIDATION
        );

        if (error) {
          handleError(error, "useProjectSync.loadFromCode");
          const errorMessage = getUserFriendlyMessage(error);
          showToast(`Failed to load project: ${errorMessage}`, "error");
        } else if (res) {
          loadFromDSL(res.model as SrujaModelDump, res.dsl, undefined);
          // Only set tab if not already specified in URL to avoid flickering
          const tabParam = params.get("tab");
          if (!tabParam) {
            setActiveTab("diagram");
          }
        }
        setIsLoadingFile(false);
        return;
      }

      // 3. Handle Legacy "dsl" param
      if (dslParam) {
        setIsLoadingFile(true);
        const { error, data: res } = await safeAsync(
          async () => {
            const decompressed = LZString.decompressFromBase64(decodeURIComponent(dslParam));
            if (!decompressed) {
              throw new Error("Failed to decompress DSL parameter");
            }
            const converted = await convertDslToModel(decompressed);
            if (!converted) {
              throw new Error("Failed to convert decompressed DSL to Model");
            }
            return { model: converted, dsl: decompressed };
          },
          "Failed to load from DSL parameter",
          ErrorType.VALIDATION
        );

        if (error) {
          handleError(error, "useProjectSync.loadFromDsl");
          const errorMessage = getUserFriendlyMessage(error);
          showToast(`Failed to load project: ${errorMessage}`, "error");
        } else if (res) {
          loadFromDSL(res.model as SrujaModelDump, res.dsl, undefined);
          // Only set tab if not already specified in URL to avoid flickering
          const tabParam = params.get("tab");
          if (!tabParam) {
            setActiveTab("diagram");
          }
        }
        setIsLoadingFile(false);
        return;
      }

      // 4. Load Demo (if no project/share params)
      if (!shareParam) {
        const exampleParam = params.get("example");
        const { model, currentExampleFile: currentExample } = useArchitectureStore.getState();

        // Load demo if:
        // 1. No model exists, OR
        // 2. Example param is present and differs from current example
        // This ensures examples load correctly in incognito mode (no model) or when switching examples
        const shouldLoadDemo = !model || (exampleParam && exampleParam !== currentExample);
        if (shouldLoadDemo) {
          loadDemo();
        }
      }
    };

    init();
  }, [loadDemo, loadFromDSL, setActiveTab, showToast]);

  return { isLoadingFile, setIsLoadingFile, loadDemo };
}
