import { useMemo, type RefObject } from "react";
// import type { ArchitectureCanvasRef } from "../components/Canvas";
import type { SrujaModelDump } from "@sruja/shared";
import type { ViewTab } from "../types";

export interface ShortcutDefinition {
  key: string;
  ctrlKey?: boolean;
  shiftKey?: boolean;
  altKey?: boolean;
  action: () => void;
  description: string;
  preventDefault?: boolean;
}

interface UseAppShortcutsProps {
  activeTab: ViewTab;
  model: SrujaModelDump | null;
  canvasRef: RefObject<any>;
  handlers: {
    handleExport: () => void;
    handleExportPNG: () => Promise<void>;
    handleImport: () => void;
    handleCopy: () => void;
    handlePaste: () => void;
    handleDuplicate: () => void;
    undo: () => SrujaModelDump | undefined | null;
    redo: () => SrujaModelDump | undefined | null;
    updateArchitecture: (updater: () => SrujaModelDump) => Promise<void>;
  };
  ui: {
    setShowCommandPalette: (show: boolean) => void;
    setShowShortcuts: (show: boolean) => void;
    setShowActions: (show: boolean) => void;
    setShowSettings: (show: boolean) => void;
  };
}

export function useAppShortcuts({
  activeTab,
  model,
  canvasRef,
  handlers,
  ui,
}: UseAppShortcutsProps) {
  return useMemo<ShortcutDefinition[]>(
    () => [
      {
        key: "s",
        ctrlKey: true,
        action: () => {
          if (activeTab === "diagram" && canvasRef.current && model) {
            void handlers.handleExportPNG();
          } else if (model) {
            handlers.handleExport();
          }
        },
        description: "Save/Export",
      },
      {
        key: "o",
        ctrlKey: true,
        action: handlers.handleImport,
        description: "Open/Import",
      },
      {
        key: "z",
        ctrlKey: true,
        shiftKey: false,
        action: () => {
          const previousState = handlers.undo();
          if (previousState && model) {
            void handlers.updateArchitecture(() => previousState);
          }
        },
        description: "Undo",
      },
      {
        key: "z",
        ctrlKey: true,
        shiftKey: true,
        action: () => {
          const nextState = handlers.redo();
          if (nextState && model) {
            void handlers.updateArchitecture(() => nextState);
          }
        },
        description: "Redo",
      },
      {
        key: "y",
        ctrlKey: true,
        action: () => {
          const nextState = handlers.redo();
          if (nextState && model) {
            void handlers.updateArchitecture(() => nextState);
          }
        },
        description: "Redo (Alt)",
      },
      {
        key: "k",
        ctrlKey: true,
        action: () => {
          ui.setShowCommandPalette(true);
        },
        description: "Command Palette",
      },
      {
        key: "c",
        ctrlKey: true,
        action: handlers.handleCopy,
        description: "Copy selected node",
      },
      {
        key: "v",
        ctrlKey: true,
        action: handlers.handlePaste,
        description: "Paste node",
      },
      {
        key: "d",
        ctrlKey: true,
        action: handlers.handleDuplicate,
        description: "Duplicate selected node",
      },
      {
        key: "0",
        ctrlKey: true,
        action: () => {
          if (canvasRef.current) {
            canvasRef.current.fitView();
          }
        },
        description: "Fit to Screen",
      },
      {
        key: "=",
        ctrlKey: true,
        action: () => {
          if (canvasRef.current) {
            canvasRef.current.zoomToSelection();
          }
        },
        description: "Zoom to Selection",
      },
      {
        key: "1",
        ctrlKey: true,
        action: () => {
          if (canvasRef.current) {
            canvasRef.current.zoomToActualSize();
          }
        },
        description: "Actual Size (100%)",
      },
      {
        key: "?",
        action: () => {
          ui.setShowShortcuts(true);
        },
        description: "Show Keyboard Shortcuts",
      },
      {
        key: "Escape",
        action: () => {
          ui.setShowActions(false);
          ui.setShowSettings(false);
        },
        description: "Close dialogs",
        preventDefault: false,
      },
    ],
    [
      activeTab,
      model,
      canvasRef,
      handlers, // handlers object reference should be stable or this will re-calc often
      ui,
    ]
  );
}
