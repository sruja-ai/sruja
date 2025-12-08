import { useState, useRef } from 'react';
import { logger } from '@sruja/shared';
import type { ViewerInstance } from '@sruja/viewer';
import { StatusBar } from './components/StatusBar';
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts';
import { useClickToConnect } from './hooks/useClickToConnect';
import { useDragToConnect } from './hooks/useDragToConnect';
import { useViewerInteractions } from './hooks/useViewerInteractions';
import { useDeepLinking } from './hooks/useDeepLinking';
import { LoadingScreen } from './components/LoadingScreen';
import { WelcomeScreen } from './components/WelcomeScreen';
import { createCommands } from './utils/commands';
import { EXAMPLES } from './examples';
import { useViewStore } from './stores/ViewStore';
import { useViewer } from './hooks/useViewer';
import { StudioToolbar } from './components/StudioToolbar';
import { StudioStateProvider, useStudioState } from './context/StudioStateContext';
import { StudioEditingProvider, useStudioEditing } from './context/StudioEditingContext';
import { UnifiedLayout } from './components/UnifiedLayout';
import { useModalState } from './hooks/useModalState';
import { useUIState } from './hooks/useUIState';
import { useAppHandlers } from './hooks/useAppHandlers';
import { useAppEffects } from './hooks/useAppEffects';
import { AppModals } from './components/AppModals';
import { SectionErrorBoundary } from './components/SectionErrorBoundary';
import type { editor } from 'monaco-editor';

function AppContent() {
  const { sidebar, setSidebar, properties, setProperties, setDocumentation } = useStudioState();
  const {
    editing,
    updateDsl,
    syncDiagramToDslState,
    setSelectedNodeId,
    setArchData,
    undo,
    redo,
    viewerRef,
    wasmApiRef,
  } = useStudioEditing();

  const { dsl, archData, selectedNodeId, validationStatus } = editing;
  const { activeStep, setStep, focusPath, setFocusPath } = useViewStore();
  
  // Extract UI state into custom hook
  const uiState = useUIState();
  const {
    selectedExample,
    setSelectedExample,
    isAddingRelation,
    setIsAddingRelation,
    sourceNode,
    setSourceNode,
    activeView,
    setActiveView,
    designEnabled,
    setDesignEnabled,
    contextMenu,
    setContextMenu,
    copiedNode,
    setCopiedNode,
    zoomLevel,
    setZoomLevel,
    lastSaved,
    setLastSaved,
    currentLevel,
    setCurrentLevel,
    focusSystemId,
    focusContainerId,
    toast,
    setToast,
    isWasmLoading,
    setIsWasmLoading,
    showWelcome,
    setShowWelcome,
  } = uiState;

  // Extract modal state into custom hook
  const modalState = useModalState();
  const {
    modalConfig,
    setModalConfig,
    adrModalOpen,
    setAdrModalOpen,
    searchDialogOpen,
    setSearchDialogOpen,
    commandPaletteOpen,
    setCommandPaletteOpen,
    shortcutsOpen,
    setShortcutsOpen,
  } = modalState;

  const monacoEditorRef = useRef<editor.IStandaloneCodeEditor | null>(null);

  const containerRef = useRef<HTMLDivElement>(null);
  const isUpdatingViewerRef = useRef(false);
  const isUpdatingPropertiesRef = useRef(false);

  // Use consolidated hooks
  const handlers = useAppHandlers({
    viewerRef,
    wasmApiRef,
    monacoEditorRef,
    isUpdatingViewerRef,
    isUpdatingPropertiesRef,
    dsl,
    archData,
    selectedNodeId,
    copiedNode,
    currentLevel,
    focusSystemId,
    focusContainerId,
    selectedExample,
    sourceNode,
    isAddingRelation,
    modalConfig,
    sidebar,
    updateDsl,
    syncDiagramToDslState,
    setSelectedNodeId,
    setArchData,
    setSelectedExample,
    setCopiedNode,
    setZoomLevel,
    setCurrentLevel,
    setIsAddingRelation,
    setSourceNode,
    setModalConfig,
    setAdrModalOpen,
    setToast,
    setLastSaved,
    setDocumentation,
    setSidebar,
  });

  // Use consolidated effects
  useAppEffects({
    containerRef,
    viewerRef,
    wasmApiRef,
    isUpdatingViewerRef,
    isUpdatingPropertiesRef,
    dsl,
    archData,
    selectedNodeId,
    sidebar,
    updateDsl,
    setSelectedNodeId,
    setArchData,
    setZoomLevel,
    setLastSaved,
    setDocumentation,
    setCommandPaletteOpen,
  });



  // Deep linking
  useDeepLinking({
    activeStep,
    focusPath,
    setStep,
    setFocusPath,
    setActiveView,
  });

  // Click-to-connect relation creation
  useClickToConnect({
    viewerRef,
    isAddingRelation,
    sourceNode,
    setSourceNode,
    setIsAddingRelation,
    setModalConfig,
  });

  // Drag-to-connect relation creation
  useDragToConnect({
    viewerRef,
    setModalConfig,
  });

  // Viewer interactions (double-click, context menu)
  useViewerInteractions({
    viewerRef,
    setSelectedNodeId,
    setModalConfig,
    setContextMenu,
  });

  // Initialize viewer with current DSL
  useViewer({
    containerRef,
    showSidebar: sidebar.showSidebar,
    activeSidebarPanel: sidebar.activePanel,
    updateDsl,
    setSelectedNodeId,
    setArchData,
    setSelectedDocNodeType: () => { }, // No longer needed - handled by state context
    setSelectedDocNodeId: () => { }, // No longer needed - handled by state context
    setSelectedDocNodeLabel: () => { }, // No longer needed - handled by state context
    setZoomLevel,
    setIsWasmLoading,
    initialDsl: editing.dsl,
    viewerRefExternal: viewerRef,
    wasmApiRefExternal: wasmApiRef,
  });

  // Keyboard shortcuts are handled in UnifiedLayout
  useKeyboardShortcuts({
    onUndo: () => { undo(); },
    onRedo: () => { redo(); },
    onDelete: handlers.handleDelete,
    onRename: handlers.handleRename,
    onZoomIn: handlers.handleZoomIn,
    onZoomOut: handlers.handleZoomOut,
    onFitToScreen: handlers.handleFitToScreen,
  });

  // Command palette commands
  const commands = createCommands({
    handleSave: handlers.handleSave,
    handleCopyShareLink: handlers.handleCopyShareLink,
    handleZoomIn: handlers.handleZoomIn,
    handleZoomOut: handlers.handleZoomOut,
    handleFitToScreen: handlers.handleFitToScreen,
    handleAddNode: handlers.handleAddNode,
    toggleAddRelation: handlers.toggleAddRelation,
    setShowSidebar: (show: boolean) => setSidebar((prev) => ({ ...prev, showSidebar: show })),
    setActiveSidebarPanel: (panel: 'explorer' | 'documentation' | 'shortcuts' | 'guide') => setSidebar((prev) => ({ ...prev, activePanel: panel })),
    setShowProperties: (show: boolean) => setProperties((prev) => ({ ...prev, showProperties: show })),
    showSidebar: sidebar.showSidebar,
    activeSidebarPanel: sidebar.activePanel,
    showProperties: properties.showProperties,
    handleRename: handlers.handleRename,
  });




  // Toolbar buttons now use Button component from @sruja/ui

  // Helper to find node in architecture


  return (
    <div className="app h-full w-full flex flex-col overflow-hidden" style={{ width: '100%', height: '100%', margin: 0, padding: 0 }}>

      {/* Professional Loading Screen */}
      <LoadingScreen isLoading={isWasmLoading} />

      {/* Welcome Screen for First-Time Users */}
      {
        showWelcome && !isWasmLoading && (
          <WelcomeScreen
            onClose={() => setShowWelcome(false)}
            onSelectExample={async (key: keyof typeof EXAMPLES) => {
              setSelectedExample(key);
              await updateDsl(EXAMPLES[key]);
              setShowWelcome(false);
            }}
          />
        )
      }

      <StudioToolbar
        selectedExample={selectedExample}
        onExampleChange={async (ex: keyof typeof EXAMPLES) => {
          setSelectedExample(ex);
          await updateDsl(EXAMPLES[ex]);
        }}
        examples={EXAMPLES}
        onShare={handlers.handleShare}
        onViewInViewer={handlers.handleViewInViewer}
        onNewDiagram={handlers.handleNewDiagram}
        isWasmLoading={isWasmLoading}
      />

      <UnifiedLayout
        activeView={activeView}
        onViewChange={setActiveView}
        dsl={dsl}
        onDslChange={handlers.handleDslTextChange}
        monacoEditorRef={monacoEditorRef}
        onMonacoSelectionChange={handlers.handleMonacoSelectionChange}
        containerRef={containerRef}
        viewerRef={viewerRef}
        selectedNodeId={selectedNodeId}
        archData={archData}
        onPropertiesUpdate={handlers.handlePropertiesUpdate}
        onAddNode={handlers.handleAddNode}
        onToggleRelation={handlers.toggleAddRelation}
        isAddingRelation={isAddingRelation}
        sourceNode={sourceNode}
        currentLevel={currentLevel}
        onSetLevel={handlers.handleSetLevel}
        zoomLevel={zoomLevel}
        onZoomIn={handlers.handleZoomIn}
        onZoomOut={handlers.handleZoomOut}
        onFitToScreen={handlers.handleFitToScreen}
        onToggleCollapse={handlers.handleToggleCollapse}
        onDelete={handlers.handleDelete}
        onRename={handlers.handleRename}
        onUndo={() => { undo(); }}
        onRedo={() => { redo(); }}
        onShowHelp={() => setShortcutsOpen(true)}
        onExport={async (format: 'svg' | 'png') => {
          try {
            if (!viewerRef.current?.cy) return;
            const cy = viewerRef.current.cy;
            const png = await cy.png({ output: 'blob', scale: format === 'png' ? 2 : 1 });
            const url = URL.createObjectURL(png);
            const a = document.createElement('a');
            a.href = url;
            a.download = `diagram.${format}`;
            a.click();
            URL.revokeObjectURL(url);
            setToast({ message: `Exported as ${format.toUpperCase()}`, type: 'success' });
          } catch (error) {
            logger.error('Export failed', {
              component: 'studio',
              action: 'export',
              errorType: error instanceof Error ? error.constructor.name : 'unknown',
              error: error instanceof Error ? error.message : String(error),
            });
            setToast({ message: 'Export failed', type: 'error' });
          }
        }}
        onShare={handlers.handleShare}
        onExplorerSelect={handlers.handleExplorerSelect}
      />

      {/* Legacy mode switching code removed - using UnifiedLayout above */}

      {/* Legacy StepGuide - Removed (design mode merged into unified layout) */}
      {false && !designEnabled && (
        <button
          className="fixed bottom-4 right-4 z-[1000] px-3 py-1.5 rounded-md border border-[var(--color-border)] bg-[var(--color-background)]"
          onClick={() => setDesignEnabled(true)}
        >
          Design Mode
        </button>
      )}

      <SectionErrorBoundary sectionName="Modals and Dialogs">
        <AppModals
        modalConfig={modalConfig}
        setModalConfig={setModalConfig}
        adrModalOpen={adrModalOpen}
        setAdrModalOpen={setAdrModalOpen}
        searchDialogOpen={searchDialogOpen}
        setSearchDialogOpen={setSearchDialogOpen}
        commandPaletteOpen={commandPaletteOpen}
        setCommandPaletteOpen={setCommandPaletteOpen}
        shortcutsOpen={shortcutsOpen}
        setShortcutsOpen={setShortcutsOpen}
        contextMenu={contextMenu}
        setContextMenu={setContextMenu}
        toast={toast}
        setToast={setToast}
        archData={archData}
        selectedNodeId={selectedNodeId}
        copiedNode={copiedNode}
        viewerRef={viewerRef}
        sidebar={sidebar}
        onModalConfirm={handlers.handleModalConfirm as (value?: string) => void | Promise<void>}
        onAdrConfirm={handlers.handleAdrConfirm as (data?: unknown) => void | Promise<void>}
        setSidebar={setSidebar}
        setDocumentation={setDocumentation}
        onToggleAddRelation={handlers.toggleAddRelation}
        onCopy={handlers.handleCopy}
        onDelete={handlers.handleDelete}
        onRename={handlers.handleRename}
        onPaste={handlers.handlePaste}
        onAddNode={handlers.handleAddNode}
        onSearchSelect={(id) => {
          if (viewerRef.current) {
            viewerRef.current.selectNode(id);
            setSelectedNodeId(id);
          }
        }}
        commands={commands}
        setSelectedNodeId={setSelectedNodeId}
      />
      </SectionErrorBoundary>

      <SectionErrorBoundary sectionName="Status Bar">
        <StatusBar
        archData={archData}
        zoomLevel={zoomLevel}
        lastSaved={lastSaved}
        validationStatus={validationStatus}
        currentLevel={currentLevel}
        onValidationClick={() => {
          if (validationStatus.lastError) {
            setToast({ message: validationStatus.lastError, type: 'error' });
          }
        }}
      />
      </SectionErrorBoundary>

    </div >
  );
}

function App() {
  const viewerRef = useRef<ViewerInstance | null>(null);
  const wasmApiRef = useRef<import('@sruja/shared').WasmApi | null>(null);
  const [, setToast] = useState<{ message: string; type: 'success' | 'error' | 'info' } | null>(null);

  return (
    <StudioStateProvider>
      <StudioEditingProvider
        viewerRef={viewerRef}
        wasmApiRef={wasmApiRef}
        onToast={setToast}
      >
        <AppContent />
      </StudioEditingProvider>
    </StudioStateProvider>
  );
}

export default App;
