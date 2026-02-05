// apps/designer/src/App.tsx
import { useEffect, useRef, useState, useMemo } from "react";
import { Play, Plus, PanelLeft, PanelRight } from "lucide-react";
import { SrujaCanvas } from "./components/SrujaCanvas";
import "./App.css";
import "./components/shared/GlobalFocusStyles.css";
import { NavigationPanel, InspectorPanel, CodePanel, MarkdownPanel } from "./components/Panels";
import { BuilderWizard } from "./components/Wizard";
import { ErrorBoundary, SentryInit } from "./components/shared";
import { SplitLayout } from "./components/Layout/SplitLayout";
import { BestPracticesView } from "./components/Review/BestPracticesView";
import {
  ToastContainer,
  Logo,
  SrujaLoader,
  PosthogProvider,
  CommandPalette,
  ShortcutsModal,
} from "@sruja/ui"; // Assuming these exports exist or adjust imports
import {
  useArchitectureStore,
  useSelectionStore,
  useUIStore,
  useFeatureFlagsStore,
  useHistoryStore,
  useToastStore,
} from "./stores";
import { getArchitectureModel } from "./models/ArchitectureModel";

import { useClipboardOperations, useProjectSync, useFileHandlers } from "./hooks";
import { useUrlState } from "./hooks/useUrlState";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
import { setGlobalCanvasRef } from "./hooks/useTagNavigation";
import type { CanvasHandle } from "./components/SrujaCanvas/types";

// New components and hooks
import { Header } from "./components/Header";
import { useAppCommands } from "./hooks/useAppCommands";
import { useAppShortcuts } from "./hooks/useAppShortcuts";
import { OnboardingTooltip } from "./components/non-core/onboarding/OnboardingTooltip";
import { studioScope } from "./config/studioScope";

export default function App() {
  // Sync URL state (level, expanded nodes) with view store
  useUrlState();

  const model = useArchitectureStore((s) => s.model);
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const selectedNodeId = useSelectionStore((s) => s.selectedNodeId);
  const undo = useHistoryStore((s) => s.undo);
  const redo = useHistoryStore((s) => s.redo);

  // Split View & Layout State
  const activeTab = useUIStore((s) => s.activeTab);
  const setActiveTab = useUIStore((s) => s.setActiveTab);

  const isNavVisible = useUIStore((s) => s.isNavigationVisible);
  const isInspectorVisible = useUIStore((s) => s.isInspectorVisible);
  const toggleNav = useUIStore((s) => s.toggleNavigation);
  const toggleInspector = useUIStore((s) => s.toggleInspector);

  const editMode = useFeatureFlagsStore((s) => s.editMode);
  const setEditMode = useFeatureFlagsStore((s) => s.setEditMode);

  const toasts = useToastStore((s) => s.toasts);
  const removeToast = useToastStore((s) => s.removeToast);

  // Local UI state
  const [isNavOpen, setIsNavOpen] = useState(false); // For mobile drawer?
  const [showActions, setShowActions] = useState(false);
  const [showCommandPalette, setShowCommandPalette] = useState(false);
  const [showShortcuts, setShowShortcuts] = useState(false);

  const canvasRef = useRef<CanvasHandle | null>(null);

  // Hooks for separated logic
  const { isLoadingFile: isSyncLoading, loadDemo } = useProjectSync();
  const {
    handleShare: handleShareHeader,
    handleExport,
    handleExportPNG,
    handleExportSVG,
    handleImport,
    handleFileChange,
    handleCreateNew: handleCreateNewRemote,
    handleCreateLocal,
    reloadFromDsl,
    isImporting,
    isExporting,
    isSharing,
    fileInputRef,
  } = useFileHandlers(canvasRef);

  const { handleCopy, handlePaste, handleDuplicate } = useClipboardOperations(canvasRef);

  const isLoadingFile = isSyncLoading || isImporting || isExporting || isSharing;

  // Set global canvas ref for tag navigation
  useEffect(() => {
    if (canvasRef.current) {
      setGlobalCanvasRef(canvasRef);
    }
    return () => {
      setGlobalCanvasRef({ current: null });
    };
  }, []);

  // Sync ArchitectureModel with architecture store
  useEffect(() => {
    const archModel = getArchitectureModel();
    if (model) {
      archModel.updateModel(model);
    }
  }, [model]);

  // Init Store
  useEffect(() => {
    // Check if we need to load example
    const store = useArchitectureStore.getState();
    if (!store.model) {
      // Fallback or explicit init if store method exists
      // store.init({});
    }
  }, []);

  // Shortcuts & Commands
  const shortcuts = useAppShortcuts({
    activeTab,
    model,
    canvasRef,
    handlers: {
      handleExport,
      handleExportPNG,
      handleImport,
      handleCopy,
      handlePaste,
      handleDuplicate,
      undo,
      redo,
      updateArchitecture,
    },
    ui: {
      setShowCommandPalette,
      setShowShortcuts,
      setShowActions,
    },
  });

  useKeyboardShortcuts(shortcuts); // Global hotkeys

  const commandPaletteCommands = useAppCommands({
    activeTab,
    setActiveTab,
    handleExport,
    handleImport,
    handleExportPNG,
    handleExportSVG,
  });

  const modalShortcuts = useMemo(() => {
    return shortcuts.map((s) => {
      const keys: string[] = [];
      if (s.ctrlKey) keys.push("Ctrl");
      if (s.shiftKey) keys.push("Shift");
      if (s.altKey) keys.push("Alt");
      keys.push(s.key.toUpperCase());

      return {
        keys,
        description: s.description,
        category: "general",
      };
    });
  }, [shortcuts]);

  // Render Helpers
  const renderContent = (type: string) => {
    if (!model && !isLoadingFile) {
      if (type === "diagram" || type === "builder") {
        // Show empty state only in main view if nothing loaded
        return (
          <div className="drop-zone">
            <Logo size={64} />
            <h2>Design, visualize, and govern your architecture</h2>
            <p>
              Start from a demo or template, or import a <code>.sruja</code> file.
            </p>
            <div className="empty-state-actions">
              <button className="demo-btn large" onClick={() => void loadDemo()}>
                <Play size={18} />
                Try a Demo
              </button>
              <button className="upload-btn large" onClick={() => void handleCreateLocal()}>
                <Plus size={18} />
                Create New
              </button>
            </div>
          </div>
        );
      }
      return null;
    }

    if (isLoadingFile) {
      return (
        <div className="loading">
          <SrujaLoader size={64} />
          <p>Loading architecture...</p>
        </div>
      );
    }

    switch (type) {
      case "builder":
        return (
          <ErrorBoundary fallback={<div className="error-state">Builder Error</div>}>
            <BuilderWizard />
          </ErrorBoundary>
        );
      case "code":
        return <CodePanel />;
      case "diagram":
        // Refactored Diagram View wrapper
        return (
          <div
            className="canvas-wrapper-full"
            style={{ width: "100%", height: "100%", display: "flex", flexDirection: "column" }}
          >
            <ErrorBoundary fallback={<div className="error-state">Canvas Error</div>}>
              <SrujaCanvas ref={canvasRef} />
            </ErrorBoundary>
          </div>
        );
      case "docs":
        return <MarkdownPanel />;
      case "review":
        return <BestPracticesView />;
      default:
        // Default to diagram if unsure, or specific pages
        return (
          <div
            className="canvas-wrapper-full"
            style={{ width: "100%", height: "100%", display: "flex", flexDirection: "column" }}
          >
            <ErrorBoundary fallback={<div className="error-state">Canvas Error</div>}>
              <SrujaCanvas ref={canvasRef} />
            </ErrorBoundary>
          </div>
        );
    }
  };

  return (
    <>
      <SentryInit />
      <PosthogProvider
        apiKey={import.meta.env.VITE_POSTHOG_KEY || ""}
        host={import.meta.env.VITE_POSTHOG_HOST}
      >
        <div className={`app-container ${editMode === "edit" ? "edit-mode" : "view-mode"}`}>
          <ToastContainer toasts={toasts} onClose={removeToast} />
          <input
            type="file"
            ref={fileInputRef}
            className="hidden"
            accept=".sruja,.json"
            onChange={handleFileChange}
          />

          <Header
            isNavOpen={isNavOpen}
            setIsNavOpen={setIsNavOpen}
            model={model}
            showActions={showActions}
            setShowActions={setShowActions}
            activeTab={activeTab}
            setActiveTab={setActiveTab}
            editMode={editMode}
            setEditMode={setEditMode}
            selectedNodeId={selectedNodeId}
            isDetailsOpen={false}
            setIsDetailsOpen={() => {}}
            handleImport={handleImport}
            handleExport={handleExport}
            handleExportPNG={handleExportPNG}
            handleExportSVG={handleExportSVG}
            reloadFromDsl={reloadFromDsl}
            handleShareHeader={handleShareHeader}
            handleCreateNewRemote={handleCreateNewRemote}
            onOpenCommandPalette={() => setShowCommandPalette(true)}
          />

          {isNavOpen && <div className="mobile-overlay" onClick={() => setIsNavOpen(false)} />}

          <main className="app-main">
            {/* LEFT SIDEBAR: Navigation */}
            <div className={`sidebar-container left ${isNavVisible ? "open" : "closed"}`}>
              <div className="sidebar-content">
                <NavigationPanel onClose={toggleNav} />
              </div>
              {/* Collapsed State Toggle */}
              {!isNavVisible && (
                <div className="collapsed-bar left" onClick={toggleNav} title="Open Explorer">
                  <PanelLeft size={20} />
                </div>
              )}
            </div>

            {/* CENTER: Main Stage (Split View) */}
            <div className="center-stage">
              <div className="canvas-container">
                {/* Logic for Split vs Single View
                    Left Pane: Controlled by activeEditor ("builder" | "code" | null)
                    Right Pane: Controlled by activeView ("diagram" | "docs" | ...)
                 */}
                <SplitLayout
                  leftContent={renderContent(useUIStore((s) => s.activeEditor) || "none")}
                  rightContent={renderContent(useUIStore((s) => s.activeView) || "diagram")}
                  isLeftVisible={!!useUIStore((s) => s.activeEditor)}
                  onCollapse={() => useUIStore.getState().setActiveEditor(null)}
                  onExpand={() => useUIStore.getState().setActiveEditor("builder")} // Default re-expand to builder? Or last used?
                />
              </div>
            </div>

            {/* RIGHT SIDEBAR: Inspector */}
            <div className={`sidebar-container right ${isInspectorVisible ? "open" : "closed"}`}>
              <div className="sidebar-content">
                <InspectorPanel />
              </div>
              {!isInspectorVisible && (
                <div
                  className="collapsed-bar right"
                  onClick={toggleInspector}
                  title="Open Inspector"
                >
                  <PanelRight size={20} />
                </div>
              )}
            </div>
          </main>

          {studioScope.commandPalette && (
            <CommandPalette
              isOpen={showCommandPalette}
              onClose={() => setShowCommandPalette(false)}
              commands={commandPaletteCommands}
            />
          )}
          {studioScope.shortcutsModal && (
            <ShortcutsModal
              isOpen={showShortcuts}
              onClose={() => setShowShortcuts(false)}
              shortcuts={modalShortcuts}
            />
          )}
          {studioScope.onboarding && model && <OnboardingTooltip />}
        </div>
      </PosthogProvider>
    </>
  );
}
