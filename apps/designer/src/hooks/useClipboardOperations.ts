// apps/designer/src/hooks/useClipboardOperations.ts
import { useCallback } from "react";
// import { useArchitectureStore, useClipboardStore, useSelectionStore, useViewStore, useToastStore } from "../stores";
// import { findNodeInArchitecture, getAllNodeIds, generateUniqueId } from "../utils/nodeUtils";
// import { safeAsync, handleError, getUserFriendlyMessage } from "../utils/errorHandling";
// import type { C4NodeData, C4NodeType, SystemJSON, PersonJSON, ContainerJSON, ComponentJSON } from "../types";
import type { ArchitectureCanvasRef } from "../components/Canvas/LikeC4Canvas";

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
/**
 * Hook for clipboard operations: copy, paste, and duplicate nodes.
 * 
 * TODO: Refactor for SrujaModelDump structure. Currently disabled during migration.
 */
export function useClipboardOperations(_canvasRef: React.RefObject<ArchitectureCanvasRef>) {
    // const data = useArchitectureStore((s) => s.likec4Model);
    // const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);

    // Stubbed handlers
    const handleCopy = useCallback(async () => {
        console.warn("Clipboard copy temporarily disabled during migration.");
    }, []);

    const handlePaste = useCallback(async () => {
        console.warn("Clipboard paste temporarily disabled during migration.");
    }, []);

    const handleDuplicate = useCallback(async () => {
        console.warn("Clipboard duplicate temporarily disabled during migration.");
    }, []);

    return { handleCopy, handlePaste, handleDuplicate };
}
