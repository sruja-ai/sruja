// apps/designer/src/hooks/useClipboardOperations.ts
import { useCallback } from "react";
import { logger } from "@sruja/shared";
import {
  useArchitectureStore,
  useClipboardStore,
  useSelectionStore,
  useToastStore,
} from "../stores";
import { findNodeInArchitecture, getAllNodeIds, generateUniqueId } from "../utils/nodeUtils";
import type { ElementDump } from "../types";

/**
 * Hook for clipboard operations: copy, paste, and duplicate nodes.
 *
 * Provides handlers for:
 * - Copying selected nodes to clipboard
 * - Pasting nodes from clipboard (with automatic ID generation)
 * - Duplicating selected nodes (copy + immediate paste)
 *
 * Supports all node types: systems, persons, containers, and components.
 * Automatically generates unique IDs for pasted/duplicated nodes.
 *
 * @param canvasRef - Reference to the ArchitectureCanvas component for node access
 * @returns Object containing clipboard operation handlers
 *
 * @example
 * ```tsx
 * const canvasRef = useRef<ArchitectureCanvasRef>(null);
 * const { handleCopy, handlePaste, handleDuplicate } = useClipboardOperations(canvasRef);
 *
 * <button onClick={handleCopy}>Copy</button>
 * <button onClick={handlePaste}>Paste</button>
 * <button onClick={handleDuplicate}>Duplicate</button>
 * ```
 */
import type { CanvasHandle } from "../components/SrujaCanvas/types";

export function useClipboardOperations(_canvasRef: React.RefObject<CanvasHandle | null>) {
  const model = useArchitectureStore((s) => s.model);
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const selectedNodeId = useSelectionStore((s) => s.selectedNodeId);
  // Use separate selectors to avoid creating a new object on every render
  // This prevents infinite loops in useSyncExternalStore
  const copyNode = useClipboardStore((s) => s.copyNode);
  const hasClipboard = useClipboardStore((s) => s.hasClipboard);
  const clipboard = useClipboardStore((s) => s.clipboard);
  const showToast = useToastStore((s) => s.showToast);

  const handleCopy = useCallback(async () => {
    try {
      if (!model || !selectedNodeId) {
        return;
      }
      const element = findNodeInArchitecture(model, selectedNodeId);
      if (!element) {
        return;
      }
      const elements: ElementDump[] = [element];
      copyNode(selectedNodeId, elements);
      showToast("Copied to clipboard", "success");
    } catch (error) {
      logger.warn("Clipboard copy failed", {
        component: "useClipboardOperations",
        action: "handleCopy",
        error,
      });
    }
  }, [model, selectedNodeId, copyNode, showToast]);

  const handlePaste = useCallback(async () => {
    try {
      const latestClipboard = clipboard;
      if (!model || !hasClipboard() || !latestClipboard) {
        return;
      }
      const existingIds = getAllNodeIds(model);
      const baseId = latestClipboard.rootId || (latestClipboard.elements[0]?.id ?? "element");
      const newId = generateUniqueId(baseId, existingIds);

      await updateArchitecture((arch) => {
        const newArch = {
          ...arch,
          elements: { ...arch.elements },
        };
        const source = latestClipboard.elements[0];
        if (source) {
          const cloned: ElementDump = { ...source, id: newId };
          newArch.elements[newId] = cloned;
        }
        return newArch;
      });
    } catch (error) {
      logger.warn("Clipboard paste failed", {
        component: "useClipboardOperations",
        action: "handlePaste",
        error,
      });
    }
  }, [model, clipboard, hasClipboard, updateArchitecture]);

  const handleDuplicate = useCallback(async () => {
    try {
      if (!model || !selectedNodeId) {
        return;
      }
      const existingIds = getAllNodeIds(model);
      const baseId = selectedNodeId;
      const newId = generateUniqueId(baseId, existingIds);
      const source = findNodeInArchitecture(model, selectedNodeId);

      await updateArchitecture((arch) => {
        const newArch = {
          ...arch,
          elements: { ...arch.elements },
        };
        if (source) {
          newArch.elements[newId] = { ...source, id: newId };
        }
        return newArch;
      });
    } catch (error) {
      logger.warn("Clipboard duplicate failed", {
        component: "useClipboardOperations",
        action: "handleDuplicate",
        error,
      });
    }
  }, [model, selectedNodeId, updateArchitecture]);

  return { handleCopy, handlePaste, handleDuplicate };
}
