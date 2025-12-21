import { useCallback } from "react";
import { ContextMenu } from "./ContextMenu";
import { useNodeContextMenu } from "./useNodeContextMenu";
import { useClipboardStore } from "../../stores/clipboardStore";
import { findNodeInArchitecture, getAllNodeIds, generateUniqueId } from "../../utils/nodeUtils";
import { deleteNodeFromArchitecture } from "../../utils/nodeDeletion";
import type { SrujaModelDump, ElementDump } from "../../types";

interface ContextMenuWithActionsProps {
  x: number;
  y: number;
  nodeId: string | null;
  onClose: () => void;
  updateArchitecture: (updater: (arch: SrujaModelDump) => SrujaModelDump) => Promise<void>;
  data: SrujaModelDump | null;
  selectedNodeId: string | null;
  selectNode: (id: string | null) => void;
}

// Logic extracted for reusability (duplicate action) and testing
function getElementsToCopy(data: SrujaModelDump, rootNodeId: string): ElementDump[] {
  const rootNode = findNodeInArchitecture(data, rootNodeId);
  if (!rootNode) return [];

  // BFS to gather all descendants
  const elementsToCopy: ElementDump[] = [rootNode];
  const visited = new Set<string>([rootNode.id]);

  // Create an efficient lookup for children
  const childrenMap = new Map<string, string[]>();
  if (data.elements) {
    for (const el of Object.values(data.elements) as ElementDump[]) {
      // Type guard or safe access for parent
      const parent = el.parent;
      if (parent) {
        if (!childrenMap.has(parent)) childrenMap.set(parent, []);
        childrenMap.get(parent)!.push(el.id);
      }
    }
  }

  const queue = [rootNode.id];
  while (queue.length > 0) {
    const currentId = queue.shift()!;
    const childrenIds = childrenMap.get(currentId) || [];
    for (const childId of childrenIds) {
      if (!visited.has(childId)) {
        visited.add(childId);
        const childEl = data.elements?.[childId] as ElementDump | undefined;
        if (childEl) {
          elementsToCopy.push(childEl);
          queue.push(childId);
        }
      }
    }
  }

  return elementsToCopy;
}

// Pure function to calculate the new architecture state after paste
function calculatePasteState(
  arch: SrujaModelDump,
  clipboardData: { rootId: string; elements: ElementDump[] },
  targetNodeId: string | null
): SrujaModelDump {
  const newArch = { ...arch, elements: { ...arch.elements } };
  const existingIds = getAllNodeIds(arch);

  const { rootId, elements } = clipboardData;
  const rootElement = elements.find((e) => e.id === rootId);
  if (!rootElement) return arch;

  // Determine parent for the pasted root
  // For now, always paste INSIDE if a target node is provided
  const newParentId = targetNodeId ?? null;

  // Generate mapping from Old ID -> New ID
  const idMap = new Map<string, string>();

  // Assign new IDs
  elements.forEach((el) => {
    const baseId = el.id.split('.').pop() || el.id;
    const newId = generateUniqueId(baseId, existingIds);
    existingIds.add(newId);
    idMap.set(el.id, newId);
  });

  // Insert new elements with updated IDs and parent references
  elements.forEach((el) => {
    const newId = idMap.get(el.id)!;
    let parent = el.parent;

    if (el.id === rootId) {
      // Root element gets the target parent
      parent = newParentId ?? undefined;
    } else {
      // Descendants remap their parent ID if it was part of the copied set
      if (parent && idMap.has(parent)) {
        parent = idMap.get(parent)!;
      }
    }

    // Assign to new architecture
    if (newArch.elements) {
      newArch.elements[newId] = {
        ...el,
        id: newId,
        parent: parent ?? undefined,
      };
    }
  });

  return newArch;
}

export function ContextMenuWithActions({
  x,
  y,
  nodeId,
  onClose,
  updateArchitecture,
  data,
  selectedNodeId,
  selectNode,
}: ContextMenuWithActionsProps) {
  const copyNode = useClipboardStore((s) => s.copyNode);
  const clipboard = useClipboardStore((s) => s.clipboard);
  const hasClipboard = useClipboardStore((s) => s.hasClipboard());

  // Copy handler
  const handleCopy = useCallback(() => {
    if (!nodeId || !data) return;
    const elementsToCopy = getElementsToCopy(data, nodeId);
    if (elementsToCopy.length > 0) {
      copyNode(nodeId, elementsToCopy);
    }
  }, [nodeId, data, copyNode]);

  // Paste handler
  const handlePaste = useCallback(async () => {
    // If no clipboard data from store, do nothing
    if (!clipboard || !data) return;

    await updateArchitecture((arch) => {
      return calculatePasteState(arch, clipboard, nodeId);
    });
  }, [clipboard, data, updateArchitecture, nodeId]);

  // Duplicate handler
  const handleDuplicate = useCallback(async () => {
    if (!nodeId || !data) return;

    // 1. Get elements to copy directly (synchronous)
    const elementsToCopy = getElementsToCopy(data, nodeId);
    if (elementsToCopy.length === 0) return;

    const tempClipboard = {
      rootId: nodeId,
      elements: elementsToCopy,
    };

    // 2. Paste immediately using the derived data
    // Duplicate usually means creating a sibling, so we use the *current node's parent* as the target
    // IF we want "Duplicate" behavior (sibling).
    // The previous implementation used `handlePaste` which pastes INSIDE `nodeId`.
    // Let's stick to previous behavior (paste inside) for consistency, OR fix it.
    // If nodeId is the thing we clicked "Duplicate" on, we expect a copy of IT, next to IT.
    // So targetParent should be nodeId's parent.
    const currentNode = data.elements?.[nodeId];
    const targetParentId = currentNode?.parent ?? null;

    await updateArchitecture((arch) => {
      return calculatePasteState(arch, tempClipboard, targetParentId);
    });
  }, [nodeId, data, updateArchitecture]);

  // Delete handler
  const handleDelete = useCallback(
    async (targetNodeId: string) => {
      if (!data || !targetNodeId) return;

      await updateArchitecture((arch) => {
        return deleteNodeFromArchitecture(arch, targetNodeId);
      });

      if (selectedNodeId === targetNodeId) {
        selectNode(null);
      }
    },
    [data, selectedNodeId, selectNode, updateArchitecture]
  );

  const actions = useNodeContextMenu({
    nodeId,
    onCopy: handleCopy,
    onPaste: handlePaste,
    onDuplicate: handleDuplicate,
    onDelete: handleDelete,
    hasClipboard,
  });

  return <ContextMenu x={x} y={y} actions={actions} onClose={onClose} />;
}

