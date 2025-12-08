// apps/studio-core/src/hooks/useAppHandlers.ts
/**
 * @fileoverview Custom hook for managing all application event handlers.
 * Consolidates all handler functions with proper memoization using useCallback.
 */
import { useCallback, useRef } from 'react';
import type { ArchitectureJSON, ViewerInstance } from '@sruja/viewer';
import type { WasmApi } from '@sruja/shared';
import { logger } from '@sruja/shared';
import type { DocumentationState, SidebarState } from '../context/StudioStateContext';
import LZString from 'lz-string';
import { createHandleDelete, createHandleCopy, createHandlePaste, createHandleAddNode } from '../handlers/nodeHandlers';
import { createHandleModalConfirm, createHandleAdrConfirm } from '../handlers/modalHandlers';
import { createHandleZoomIn, createHandleZoomOut, createHandleFitToScreen, createHandleSetLevel, createHandleShare, createHandleNewDiagram } from '../handlers/viewerHandlers';
import { createHandleApplyTemplate } from '../handlers/templateHandlers';
import { createMonacoSelectionHandler } from '../handlers/monacoHandlers';
import { createExplorerSelectHandler } from '../handlers/explorerHandlers';
import { handlePropertiesUpdate as handlePropertiesUpdateUtil } from '../utils/viewerUtils';
import type { ExampleKey } from '../examples';
import type { ToastState } from './useUIState';
import type { ModalConfig } from './useModalState';
import type { CopiedNode } from './useUIState';

interface UseAppHandlersOptions {
  // Refs
  viewerRef: React.RefObject<ViewerInstance | null>;
  wasmApiRef: React.RefObject<WasmApi | null>;
  monacoEditorRef: React.RefObject<unknown>;
  isUpdatingViewerRef: React.MutableRefObject<boolean>;
  isUpdatingPropertiesRef: React.MutableRefObject<boolean>;

  // State
  dsl: string;
  archData: ArchitectureJSON | null;
  selectedNodeId: string | null;
  copiedNode: CopiedNode | null;
  currentLevel: number | undefined;
  focusSystemId: string | undefined;
  focusContainerId: string | undefined;
  selectedExample: ExampleKey;
  sourceNode: string | null;
  isAddingRelation: boolean;
  modalConfig: ModalConfig;
  sidebar: {
    showSidebar: boolean;
    activePanel: 'explorer' | 'documentation' | 'shortcuts' | 'guide';
  };

  // Setters
  updateDsl: (newDsl: string) => Promise<void>;
  syncDiagramToDslState: () => Promise<void>;
  setSelectedNodeId: (id: string | null) => void;
  setArchData: (data: ArchitectureJSON | null) => void;
  setSelectedExample: (example: ExampleKey) => void;
  setCopiedNode: (node: CopiedNode | null) => void;
  setZoomLevel: (level: number) => void;
  setCurrentLevel: (level: number | undefined) => void;
  setIsAddingRelation: (adding: boolean) => void;
  setSourceNode: (node: string | null) => void;
  setModalConfig: (config: ModalConfig) => void;
  setAdrModalOpen: (open: boolean) => void;
  setToast: (toast: ToastState | null) => void;
  setLastSaved: (date: Date | null) => void;
  setDocumentation: (doc: DocumentationState | ((prev: DocumentationState) => DocumentationState)) => void;
  setSidebar: (sidebar: SidebarState | ((prev: SidebarState) => SidebarState)) => void;
}

/**
 * Custom hook that consolidates all application event handlers.
 * 
 * All handlers are memoized with useCallback to prevent unnecessary re-renders.
 * This hook centralizes handler logic that was previously scattered across App.tsx.
 * 
 * @param options - Configuration object containing refs, state, and setters
 * @returns Object containing all memoized event handlers
 * 
 * @example
 * ```typescript
 * const handlers = useAppHandlers({
 *   viewerRef,
 *   wasmApiRef,
 *   // ... other options
 * });
 * 
 * // Use handlers
 * <button onClick={handlers.handleDelete}>Delete</button>
 * ```
 */
export function useAppHandlers(options: UseAppHandlersOptions) {
  const {
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
  } = options;

  // DSL handlers
  const handleDslTextChange = useCallback(async (newDsl: string) => {
    setSelectedExample('Simple Web App');
    await updateDsl(newDsl);
  }, [updateDsl, setSelectedExample]);

  // Monaco handlers
  const handleMonacoSelectionChange = useCallback(
    createMonacoSelectionHandler(
      viewerRef,
      archData,
      dsl,
      selectedNodeId,
      setSelectedNodeId,
      setDocumentation,
      sidebar,
      setSidebar
    ),
    [viewerRef, archData, dsl, selectedNodeId, setSelectedNodeId, setDocumentation, sidebar, setSidebar]
  );

  // Node handlers
  const handleDelete = useCallback(
    createHandleDelete({
      viewerRef,
      syncDiagramToDslState,
      setToast,
    }),
    [viewerRef, syncDiagramToDslState, setToast]
  );

  const handleToggleCollapse = useCallback(() => {
    if (viewerRef.current) {
      viewerRef.current.toggleCollapse();
    }
  }, [viewerRef]);

  const handleCopy = useCallback(
    createHandleCopy({
      viewerRef,
      selectedNodeId,
      archData,
      setCopiedNode,
      setToast,
    }),
    [viewerRef, selectedNodeId, archData, setCopiedNode, setToast]
  );

  const handlePaste = useCallback(
    createHandlePaste({
      viewerRef,
      copiedNode,
      setSelectedNodeId,
      syncDiagramToDslState,
      setToast,
    }),
    [viewerRef, copiedNode, setSelectedNodeId, syncDiagramToDslState, setToast]
  );

  const handleExplorerSelect = useCallback(
    createExplorerSelectHandler(
      viewerRef,
      archData,
      selectedNodeId,
      setSelectedNodeId,
      setDocumentation,
      setZoomLevel,
      sidebar,
      setSidebar
    ),
    [viewerRef, archData, selectedNodeId, setSelectedNodeId, setDocumentation, setZoomLevel, sidebar, setSidebar]
  );

  const handlePropertiesUpdate = useCallback(async (newData: ArchitectureJSON) => {
    await handlePropertiesUpdateUtil(
      newData,
      wasmApiRef,
      viewerRef,
      selectedNodeId,
      setArchData,
      (newDsl) => updateDsl(newDsl),
      setToast,
      isUpdatingPropertiesRef
    );
  }, [wasmApiRef, viewerRef, selectedNodeId, setArchData, updateDsl, setToast, isUpdatingPropertiesRef]);

  // Template handler
  const handleApplyTemplate = useCallback(
    createHandleApplyTemplate({
      updateDsl,
      setToast,
    }),
    [updateDsl, setToast]
  );

  // Node handlers
  const handleAddNode = useCallback(
    createHandleAddNode({
      viewerRef,
      setSelectedNodeId,
      syncDiagramToDslState,
      setToast,
      setAdrModalOpen,
    }),
    [viewerRef, setSelectedNodeId, syncDiagramToDslState, setToast, setAdrModalOpen]
  );

  // Modal handlers - wrapped to match AppModals interface (optional params)
  const handleModalConfirm = useCallback(
    (value?: string) => {
      if (value !== undefined) {
        const handler = createHandleModalConfirm({
          viewerRef,
          modalConfig,
          setModalConfig,
          setSourceNode,
          setIsAddingRelation,
          syncDiagramToDslState,
          setToast,
        });
        return handler(value);
      }
    },
    [viewerRef, modalConfig, setModalConfig, setSourceNode, setIsAddingRelation, syncDiagramToDslState, setToast]
  );

  const handleAdrConfirm = useCallback(
    (data?: unknown) => {
      if (data !== undefined) {
        const handler = createHandleAdrConfirm({
          viewerRef,
          setAdrModalOpen,
          syncDiagramToDslState,
          setToast,
        });
        return handler(data as any);
      }
    },
    [viewerRef, setAdrModalOpen, syncDiagramToDslState, setToast]
  );

  // Viewer handlers
  const handleSetLevel = useCallback(
    createHandleSetLevel({
      viewerRef,
      setCurrentLevel,
    }),
    [viewerRef, setCurrentLevel]
  );

  const handleShare = useCallback(
    createHandleShare({
      dsl,
      currentLevel,
      focusSystemId,
      focusContainerId,
      setToast,
    }),
    [dsl, currentLevel, focusSystemId, focusContainerId, setToast]
  );

  const handleNewDiagram = useCallback(
    createHandleNewDiagram({
      updateDsl,
      dsl,
      setSelectedExample,
      setSelectedNodeId,
      setToast,
    }),
    [updateDsl, dsl, setSelectedExample, setSelectedNodeId, setToast]
  );

  const toggleAddRelation = useCallback(() => {
    setIsAddingRelation(!isAddingRelation);
    setSourceNode(null);
    // Reset any active styling
    if (sourceNode && viewerRef.current?.cy) {
      viewerRef.current.cy.getElementById(sourceNode).removeStyle();
    }
  }, [isAddingRelation, setIsAddingRelation, setSourceNode, viewerRef, sourceNode]);

  // Command handlers
  const handleSave = useCallback(() => {
    setLastSaved(new Date());
    setToast({ message: 'Saved', type: 'success' });
  }, [setLastSaved, setToast]);

  const handleCopyShareLink = useCallback(() => {
    // Type-safe access to import.meta.env
    const baseUrl = (import.meta.env?.BASE_URL as string | undefined) || '/studio/';
    const base = `${location.origin}${baseUrl}`;
    const b64 = encodeURIComponent(LZString.compressToBase64(dsl));
    const shareUrl = `${base}#code=${b64}`;
    navigator.clipboard.writeText(shareUrl);
    setToast({ message: 'Share link copied', type: 'success' });
  }, [dsl, setToast]);

  const handleZoomIn = useCallback(
    createHandleZoomIn({
      viewerRef,
      setZoomLevel,
    }),
    [viewerRef, setZoomLevel]
  );

  const handleZoomOut = useCallback(
    createHandleZoomOut({
      viewerRef,
      setZoomLevel,
    }),
    [viewerRef, setZoomLevel]
  );

  const handleFitToScreen = useCallback(
    createHandleFitToScreen({
      viewerRef,
      setZoomLevel,
    }),
    [viewerRef, setZoomLevel]
  );

  const handleViewInViewer = useCallback(() => {
    const base = typeof window !== 'undefined' ? window.location.origin : '';
    const viewerPath = '/viewer';
    const compressed = LZString.compressToBase64(dsl);
    const url = `${base}${viewerPath}?code=${encodeURIComponent(compressed)}`;
    window.open(url, '_blank');
  }, [dsl]);

  const handleRename = useCallback(() => {
    if (!selectedNodeId) {
      setToast({ message: 'Select a node to rename', type: 'info' });
      return;
    }
    setModalConfig({ 
      isOpen: true, 
      title: 'Rename Node', 
      placeholder: 'Enter new id...', 
      type: 'rename', 
      data: { oldId: selectedNodeId } 
    });
  }, [selectedNodeId, setModalConfig, setToast]);

  return {
    // DSL handlers
    handleDslTextChange,
    handleMonacoSelectionChange,

    // Node handlers
    handleDelete,
    handleToggleCollapse,
    handleCopy,
    handlePaste,
    handleExplorerSelect,
    handlePropertiesUpdate,
    handleAddNode,

    // Modal handlers
    handleModalConfirm,
    handleAdrConfirm,

    // Viewer handlers
    handleSetLevel,
    handleShare,
    handleNewDiagram,
    toggleAddRelation,

    // Command handlers
    handleSave,
    handleCopyShareLink,
    handleZoomIn,
    handleZoomOut,
    handleFitToScreen,
    handleViewInViewer,
    handleRename,

    // Template handler
    handleApplyTemplate,
  };
}



