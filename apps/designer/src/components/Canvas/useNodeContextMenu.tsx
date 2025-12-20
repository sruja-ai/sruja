// apps/designer/src/components/Canvas/useNodeContextMenu.tsx
// Force Vite cache refresh
import { useMemo } from "react";
import { Copy, Clipboard, Files, Trash2 } from "lucide-react";
import type { ContextMenuAction } from "./ContextMenu";

interface UseNodeContextMenuOptions {
  nodeId: string | null;
  onCopy: () => void;
  onPaste: () => void;
  onDuplicate: () => void;
  onDelete: (nodeId: string) => void;
  hasClipboard: boolean;
}

export function useNodeContextMenu({
  nodeId,
  onCopy,
  onPaste,
  onDuplicate,
  onDelete,
  hasClipboard,
}: UseNodeContextMenuOptions): ContextMenuAction[] {
  return useMemo(() => {
    const actions: ContextMenuAction[] = [];

    if (nodeId) {
      // Node-specific actions
      actions.push(
        {
          id: "copy",
          label: "Copy",
          icon: <Copy size={16} />,
          action: onCopy,
        },
        {
          id: "duplicate",
          label: "Duplicate",
          icon: <Files size={16} />,
          action: onDuplicate,
        },
        {
          id: "separator-1",
          label: "",
          action: () => {},
          separator: true,
        },
        {
          id: "delete",
          label: "Delete",
          icon: <Trash2 size={16} />,
          action: () => onDelete(nodeId),
        }
      );
    }

    // Paste is always available if clipboard has data
    if (hasClipboard) {
      if (nodeId) {
        // Insert paste before delete
        actions.splice(actions.length - 1, 0, {
          id: "paste",
          label: "Paste",
          icon: <Clipboard size={16} />,
          action: onPaste,
        });
      } else {
        // Pane context menu - just paste
        actions.push({
          id: "paste",
          label: "Paste",
          icon: <Clipboard size={16} />,
          action: onPaste,
        });
      }
    }

    return actions;
  }, [nodeId, hasClipboard, onCopy, onPaste, onDuplicate, onDelete]);
}
