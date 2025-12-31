// apps/designer/src/App.tsx
import { useState, useEffect, useRef, useMemo } from "react";
import { Play, Plus } from "lucide-react";
// import { type ArchitectureCanvasRef } from "./components/Canvas";
import { SrujaCanvas } from "./components/SrujaCanvas";
import "./App.css";
import "./components/shared/GlobalFocusStyles.css";
import { NavigationPanel, DetailsPanel, CodePanel, GovernancePanel } from "./components/Panels";
import { BuilderWizard } from "./components/Wizard";
import { CommandPalette, ShortcutsModal, ErrorBoundary, SentryInit } from "./components/shared";
import { ToastContainer, Logo, SrujaLoader, PosthogProvider } from "@sruja/ui"; // Consolidated imports
import {
  useArchitectureStore,
  useSelectionStore,
  useUIStore,
  useFeatureFlagsStore,
  useHistoryStore,
  useToastStore,
} from "./stores";
import { getArchitectureModel } from "./models/ArchitectureModel";
import { DynamicPersonaView } from "./components/Personas/DynamicPersonaView";
import type { Persona } from "./components/PersonaSwitcher";

import { useClipboardOperations, useProjectSync, useFileHandlers } from "./hooks";
import { useTabCounts } from "./hooks/useTabCounts";
import { DetailsView } from "./components/Views/DetailsView";
import { ViewTabs } from "./components/ViewTabs";
import { FeatureSettingsDialog } from "./components/Overview/FeatureSettingsDialog";
import { useUrlState } from "./hooks/useUrlState";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
import { setGlobalCanvasRef } from "./hooks/useTagNavigation";
import { trackInteraction } from "@sruja/shared";
import type { ViewTab } from "./types";

// New components and hooks
import { Header } from "./components/Header";
import { useAppCommands } from "./hooks/useAppCommands";
import { useAppShortcuts } from "./hooks/useAppShortcuts";
import { OnboardingTooltip } from "./components/OnboardingTooltip";

const VALID_TABS: ViewTab[] = ["overview", "diagram", "details", "code", "builder", "governance"];

export default function App() {
  // Sync URL state (level, expanded nodes) with view store
  useUrlState();

  const model = useArchitectureStore((s) => s.model);
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const selectedNodeId = useSelectionStore((s) => s.selectedNodeId);
  const undo = useHistoryStore((s) => s.undo);
  const redo = useHistoryStore((s) => s.redo);
  const activeTab = useUIStore((s) => s.activeTab);
  const setActiveTab = useUIStore((s) => s.setActiveTab);
  const selectedPersona = useUIStore((s) => s.selectedPersona);
  const setSelectedPersona = useUIStore((s) => s.setSelectedPersona);
  const editMode = useFeatureFlagsStore((s) => s.editMode);
  const setEditMode = useFeatureFlagsStore((s) => s.setEditMode);

  const toasts = useToastStore((s) => s.toasts);
  const removeToast = useToastStore((s) => s.removeToast);

  const [isNavOpen, setIsNavOpen] = useState(false);
  const [isDetailsOpen, setIsDetailsOpen] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [showActions, setShowActions] = useState(false);

  const [showCommandPalette, setShowCommandPalette] = useState(false);
  const [showShortcuts, setShowShortcuts] = useState(false);

  const canvasRef = useRef<any>(null);

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
  } = useFileHandlers(canvasRef as React.RefObject<any>);

  const { handleCopy, handlePaste, handleDuplicate } = useClipboardOperations(
    canvasRef as React.RefObject<any>
  );

  const isLoadingFile = isSyncLoading || isImporting || isExporting || isSharing;

  // Set global canvas ref for tag navigation
  useEffect(() => {
    if (canvasRef.current) {
      setGlobalCanvasRef(canvasRef as React.RefObject<any>);
    }
    return () => {
      setGlobalCanvasRef({ current: null } as unknown as React.RefObject<any>);
    };
  }, []);

  // Sync ArchitectureModel with architecture store
  useEffect(() => {
    const archModel = getArchitectureModel();
    if (model) {
      archModel.updateModel(model);
    }
  }, [model]);

  // Render persona view based on selected persona
  const renderPersonaView = (persona: Persona) => {
    // Determine props based on persona
    let title = "Architect View";
    let description = "Design and govern system structure";

    switch (persona) {
      case "product":
        title = "Product View";
        description = "Feature library, user stories, requirements coverage";
        break;
      case "devops":
        title = "DevOps View";
        description = "Infrastructure, capacity, cost, deployments";
        break;
      case "security":
        title = "Security View";
        description = "Trust boundaries, compliance, data flows";
        break;
      case "cto":
        title = "CTO View";
        description = "Health scores, risks, technical debt";
        break;
      case "sre":
        title = "SRE View";
        description = "SLOs, error budgets, reliability";
        break;
      case "architect":
      default:
        title = "Architect View";
        description = "Design and govern system structure";
        break;
    }

    return <DynamicPersonaView persona={persona} title={title} description={description} />;
  };

  // Initialize activeTab from URL on mount
  // If tab param is explicitly set, respect it. Otherwise, only set default if not loading code from URL
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const tabParam = params.get("tab");
    const hasCodeParam = params.get("code") || params.get("dsl") || params.get("share");

    // If tab is explicitly set in URL, always respect it
    if (tabParam === "requirements" || tabParam === "adrs") {
      setActiveTab("details");
      return;
    } else if (tabParam === "guided") {
      setActiveTab("builder");
      return;
    } else if (tabParam === "overview") {
      setActiveTab(editMode === "edit" ? "builder" : "diagram");
      return;
    } else if (tabParam && VALID_TABS.includes(tabParam as ViewTab)) {
      setActiveTab(tabParam as ViewTab);
      return;
    }

    // If no explicit tab param, only set default if we're NOT loading code from URL
    // (to avoid flickering - useProjectSync will handle tab setting for code params)
    if (!hasCodeParam) {
      setActiveTab(editMode === "edit" ? "builder" : "diagram");
    }
  }, [setActiveTab, editMode]);

  // Sync URL when activeTab changes (skip during initial load to avoid flickering)
  useEffect(() => {
    // Skip URL sync if we're still loading (to prevent flickering during initialization)
    if (isLoadingFile) {
      return;
    }

    const params = new URLSearchParams(window.location.search);
    const previousTab = params.get("tab");
    if (previousTab !== activeTab) {
      params.set("tab", activeTab);
      const newUrl = `${window.location.pathname}?${params.toString()}`;
      window.history.replaceState({}, "", newUrl);

      // Track tab change (only if tab actually changed, not on initial load)
      if (previousTab && previousTab !== activeTab) {
        trackInteraction("switch", "tab", { from: previousTab, to: activeTab });
      }
    }
  }, [activeTab, isLoadingFile]);

  // Auto-open Details Panel when a node is selected
  useEffect(() => {
    if (selectedNodeId) {
      setIsDetailsOpen(true);
    }
  }, [selectedNodeId]);

  const counts = useTabCounts(model);

  // --- Logic Extracted to Hooks ---
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
      setShowSettings,
    },
  });

  useKeyboardShortcuts(shortcuts);

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

  return (
    <>
      <SentryInit />
      <PosthogProvider
        apiKey={import.meta.env.VITE_POSTHOG_KEY || ""}
        host={import.meta.env.VITE_POSTHOG_HOST}
      >
        <div className="app-container">
          <ToastContainer toasts={toasts} onClose={removeToast} />
          {/* ... existing content ... */}
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
            editMode={editMode}
            setEditMode={setEditMode}
            setShowSettings={setShowSettings}
            selectedNodeId={selectedNodeId}
            isDetailsOpen={isDetailsOpen}
            setIsDetailsOpen={setIsDetailsOpen}
            selectedPersona={selectedPersona}
            onPersonaChange={setSelectedPersona}
            handleImport={handleImport}
            handleExport={handleExport}
            handleExportPNG={handleExportPNG}
            handleExportSVG={handleExportSVG}
            reloadFromDsl={reloadFromDsl}
            handleShareHeader={handleShareHeader}
            handleCreateNewRemote={handleCreateNewRemote}
          />

          {/* Mobile Overlay */}
          {(isNavOpen || isDetailsOpen) && (
            <div
              className="mobile-overlay"
              onClick={() => {
                setIsNavOpen(false);
                setIsDetailsOpen(false);
              }}
            />
          )}

          {/* Main Content */}
          <main className="app-main">
            <div className={`navigation-panel-wrapper ${isNavOpen ? "open" : ""}`}>
              <NavigationPanel onClose={() => setIsNavOpen(false)} />
            </div>

            <div className={`center-panel ${editMode === "edit" ? "edit-mode" : ""}`}>
              {/* View Tabs */}
              {model && (
                <ViewTabs activeTab={activeTab} onTabChange={setActiveTab} counts={counts} />
              )}

              {/* Tab Content */}
              <div className="canvas-container">
                {/* Persona View Overlay - Show when persona is selected and model exists, BUT only on builder/overview tabs */}
                {model &&
                  selectedPersona !== "architect" &&
                  (activeTab === "builder" || activeTab === "overview") && (
                    <div className="persona-view-overlay">{renderPersonaView(selectedPersona)}</div>
                  )}

                {!model && !isLoadingFile && (
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
                )}
                {isLoadingFile && (
                  <div className="loading">
                    <SrujaLoader size={64} />
                    <p>Loading architecture...</p>
                  </div>
                )}

                {model && activeTab === "overview" && (
                  <div role="tabpanel" id="tabpanel-overview" aria-labelledby="tab-overview">
                    {/* Overview merged into Diagram/Builder, redirects handled in effect */}
                  </div>
                )}

                {/* Builder Tab */}
                {model && activeTab === "builder" && (
                  <div role="tabpanel" id="tabpanel-builder" aria-labelledby="tab-builder">
                    <ErrorBoundary
                      fallback={
                        <div
                          className="error-state"
                          style={{ padding: "2rem", textAlign: "center" }}
                        >
                          <h2>Builder Error</h2>
                          <p>Failed to load the builder wizard. Please try refreshing the page.</p>
                        </div>
                      }
                    >
                      <BuilderWizard />
                    </ErrorBoundary>
                  </div>
                )}

                {/* Standard Diagram Tab (Full Screen) */}
                {model && activeTab === "diagram" && (
                  <div role="tabpanel" id="tabpanel-diagram" aria-labelledby="tab-diagram">
                    <ErrorBoundary
                      fallback={
                        <div
                          className="error-state"
                          style={{ padding: "2rem", textAlign: "center" }}
                        >
                          <h2>Canvas Error</h2>
                          <p>
                            Failed to render the architecture canvas. Please try refreshing the
                            page.
                          </p>
                        </div>
                      }
                    >
                      <SrujaCanvas />
                      {/* 
                    <ArchitectureCanvas
                      ref={canvasRef}
                      model={model}
                      dragEnabled={editMode === "edit"}
                      viewId={currentLevel}
                    /> 
                    */}
                    </ErrorBoundary>
                  </div>
                )}

                {model && activeTab === "details" && (
                  <div role="tabpanel" id="tabpanel-details" aria-labelledby="tab-details">
                    <ErrorBoundary
                      fallback={
                        <div
                          className="error-state"
                          style={{ padding: "2rem", textAlign: "center" }}
                        >
                          <h2>Details Error</h2>
                          <p>Failed to load details view. Please try refreshing the page.</p>
                        </div>
                      }
                    >
                      <DetailsView />
                    </ErrorBoundary>
                  </div>
                )}

                {model && activeTab === "code" && (
                  <div role="tabpanel" id="tabpanel-code" aria-labelledby="tab-code">
                    <CodePanel />
                  </div>
                )}

                {model && activeTab === "governance" && (
                  <div role="tabpanel" id="tabpanel-governance" aria-labelledby="tab-governance">
                    <GovernancePanel />
                  </div>
                )}
              </div>
            </div>

            <div className={`details-panel-wrapper ${isDetailsOpen ? "open" : ""}`}>
              <ErrorBoundary
                fallback={
                  <div className="error-state" style={{ padding: "2rem", textAlign: "center" }}>
                    <h2>Details Panel Error</h2>
                    <p>Failed to load the details panel. Please try closing and reopening it.</p>
                  </div>
                }
              >
                <DetailsPanel onClose={() => setIsDetailsOpen(false)} />
              </ErrorBoundary>
            </div>
          </main>

          {/* Modals & Dialogs */}
          <FeatureSettingsDialog isOpen={showSettings} onClose={() => setShowSettings(false)} />

          <CommandPalette
            isOpen={showCommandPalette}
            onClose={() => setShowCommandPalette(false)}
            commands={commandPaletteCommands}
          />

          <ShortcutsModal
            isOpen={showShortcuts}
            onClose={() => setShowShortcuts(false)}
            shortcuts={modalShortcuts}
          />

          {/* Onboarding Tooltip */}
          {model && <OnboardingTooltip />}
        </div>
      </PosthogProvider>
    </>
  );
}
