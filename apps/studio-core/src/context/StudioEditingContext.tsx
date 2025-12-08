// apps/studio-core/src/context/StudioEditingContext.tsx
import React, { createContext, useContext, useState, useCallback, useRef, useEffect } from 'react';
import type { ReactNode } from 'react';
import type { ArchitectureJSON, ViewerInstance } from '@sruja/viewer';
import type { WasmApi } from '@sruja/shared';
import { updateViewer, syncDiagramToDsl } from '../utils/viewerUtils';
import LZString from 'lz-string';

export interface EditingState {
  dsl: string;
  archData: ArchitectureJSON | null;
  selectedNodeId: string | null;
  validationStatus: {
    isValid: boolean;
    errors: number;
    warnings: number;
    lastError?: string;
    diagnostics?: Array<{
      code: string;
      severity: string;
      message: string;
      location: { file: string; line: number; column: number };
    }>;
  };
}

interface StudioEditingContextType {
  // State
  editing: EditingState;
  setEditing: (state: EditingState | ((prev: EditingState) => EditingState)) => void;

  // Refs
  viewerRef: React.RefObject<ViewerInstance | null>;
  wasmApiRef: React.RefObject<WasmApi | null>;

  // Actions
  updateDsl: (newDsl: string) => Promise<void>;
  syncDiagramToDslState: () => Promise<void>;
  updateDiagramFromDsl: (dsl: string) => Promise<void>;
  setSelectedNodeId: (id: string | null) => void;
  setArchData: (data: ArchitectureJSON | null) => void;
  setValidationStatus: (status: EditingState['validationStatus']) => void;
  undo: () => Promise<void>;
  redo: () => Promise<void>;
}

const StudioEditingContext = createContext<StudioEditingContextType | undefined>(undefined);

export function useStudioEditing() {
  const context = useContext(StudioEditingContext);
  if (!context) {
    throw new Error('useStudioEditing must be used within StudioEditingProvider');
  }
  return context;
}

interface StudioEditingProviderProps {
  children: ReactNode;
  initialDsl?: string;
  viewerRef: React.RefObject<ViewerInstance | null>;
  wasmApiRef: React.RefObject<WasmApi | null>;
  onToast: (toast: { message: string; type: 'success' | 'error' | 'info' } | null) => void;
}

export function StudioEditingProvider({
  children,
  initialDsl,
  viewerRef,
  wasmApiRef,
  onToast,
}: StudioEditingProviderProps) {
  // Load initial DSL from hash/URL or use provided initialDsl
  const getInitialDsl = () => {
    if (typeof window !== 'undefined') {
      // Support compressed share via hash: #code=<base64>
      const hash = window.location.hash || '';
      if (hash.startsWith('#code=')) {
        const b64 = hash.substring('#code='.length);
        try {
          const decompressed = LZString.decompressFromBase64(decodeURIComponent(b64));
          if (decompressed) return decompressed;
        } catch (_) {
          // fallback
        }
      }
      // Support query parameter
      const params = new URLSearchParams(window.location.search);
      const urlCode = params.get('code');
      if (urlCode) {
        return decodeURIComponent(urlCode);
      }
    }
    return initialDsl || '';
  };

  const [editing, setEditing] = useState<EditingState>({
    dsl: getInitialDsl(),
    archData: null,
    selectedNodeId: null,
    validationStatus: {
      isValid: true,
      errors: 0,
      warnings: 0,
    },
  });

  const dirtyRef = useRef<boolean>(false);

  // History stacks for undo/redo
  const undoStackRef = useRef<Array<{ dsl: string; archData: ArchitectureJSON | null }>>([]);
  const redoStackRef = useRef<Array<{ dsl: string; archData: ArchitectureJSON | null }>>([]);

  // Update DSL and sync to diagram
  const updateDsl = useCallback(async (newDsl: string) => {
    dirtyRef.current = true;
    setEditing((prev) => {
      // Push previous snapshot to undo stack
      undoStackRef.current.push({ dsl: prev.dsl, archData: prev.archData });
      // Clear redo stack on new change
      redoStackRef.current = [];
      return { ...prev, dsl: newDsl };
    });
    const api = wasmApiRef.current;
    const viewer = viewerRef.current;
    if (!api) return;

    try {
      const parsedJson = await api.parseDslToJson(newDsl);
      const parsed = JSON.parse(parsedJson);
      console.log('Parsed Architecture JSON:', parsed); // DEBUG
      setEditing((prev) => ({ ...prev, archData: parsed }));

      if (viewer) {
        await updateViewer(
          newDsl,
          api,
          viewer,
          (archData) => setEditing((prev) => ({ ...prev, archData })),
          (validationStatus) => setEditing((prev) => ({ ...prev, validationStatus })),
          onToast
        );
      } else {
        // No viewer yet; mark validation as OK and let later hooks load the viewer
        setEditing((prev) => ({
          ...prev,
          validationStatus: { isValid: true, errors: 0, warnings: 0 },
        }));
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      onToast({ message: msg.length > 150 ? msg.slice(0, 147) + '...' : msg, type: 'error' });
      setEditing((prev) => ({
        ...prev,
        validationStatus: { isValid: false, errors: 1, warnings: 0, lastError: msg },
      }));
    }
  }, [wasmApiRef, viewerRef, onToast]);

  useEffect(() => {
    const beforeUnload = (e: BeforeUnloadEvent) => {
      try { localStorage.setItem('sruja-studio-dsl', editing.dsl); } catch { }
      if (dirtyRef.current) {
        e.preventDefault();
        e.returnValue = '';
      }
    };
    const onVisibility = () => {
      if (document.visibilityState === 'hidden') {
        try { localStorage.setItem('sruja-studio-dsl', editing.dsl); } catch { }
      }
    };
    window.addEventListener('beforeunload', beforeUnload);
    document.addEventListener('visibilitychange', onVisibility);
    return () => {
      window.removeEventListener('beforeunload', beforeUnload);
      document.removeEventListener('visibilitychange', onVisibility);
    };
  }, [editing.dsl]);

  // Sync diagram changes back to DSL
  const syncDiagramToDslState = useCallback(async () => {
    if (viewerRef.current && wasmApiRef.current) {
      // Push previous snapshot before syncing
      setEditing((prev) => {
        undoStackRef.current.push({ dsl: prev.dsl, archData: prev.archData });
        redoStackRef.current = [];
        return prev;
      });
      await syncDiagramToDsl(
        viewerRef,
        wasmApiRef,
        (newDsl) => setEditing((prev) => ({ ...prev, dsl: newDsl })),
        onToast,
        (newArchData) => setEditing((prev) => ({ ...prev, archData: newArchData }))
      );
    }
  }, [viewerRef, wasmApiRef, onToast]);

  // Update diagram from DSL (one-way: DSL -> Diagram)
  const updateDiagramFromDsl = useCallback(async (dsl: string) => {
    if (wasmApiRef.current && viewerRef.current) {
      // Treat programmatic DSL set as a change for history
      setEditing((prev) => {
        undoStackRef.current.push({ dsl: prev.dsl, archData: prev.archData });
        redoStackRef.current = [];
        return { ...prev, dsl };
      });
      await updateViewer(
        dsl,
        wasmApiRef.current,
        viewerRef.current,
        (archData) => setEditing((prev) => ({ ...prev, archData })),
        (validationStatus) => setEditing((prev) => ({ ...prev, validationStatus })),
        onToast
      );
    }
  }, [wasmApiRef, viewerRef, onToast]);

  const setSelectedNodeId = useCallback((id: string | null) => {
    setEditing((prev) => ({ ...prev, selectedNodeId: id }));
  }, []);

  const setArchData = useCallback((data: ArchitectureJSON | null) => {
    setEditing((prev) => ({ ...prev, archData: data }));
  }, []);

  const setValidationStatus = useCallback((status: EditingState['validationStatus']) => {
    setEditing((prev) => ({ ...prev, validationStatus: status }));
  }, []);

  const value: StudioEditingContextType = {
    editing,
    setEditing,
    viewerRef,
    wasmApiRef,
    updateDsl,
    syncDiagramToDslState,
    updateDiagramFromDsl,
    setSelectedNodeId,
    setArchData,
    setValidationStatus,
    undo: async () => {
      if (undoStackRef.current.length === 0) return;
      const prev = undoStackRef.current.pop()!;
      // Push current to redo
      redoStackRef.current.push({ dsl: editing.dsl, archData: editing.archData });
      setEditing((cur) => ({ ...cur, dsl: prev.dsl, archData: prev.archData }));
      if (wasmApiRef.current && viewerRef.current && prev.archData) {
        await updateViewer(
          prev.dsl,
          wasmApiRef.current,
          viewerRef.current,
          (archData) => setEditing((p) => ({ ...p, archData })),
          (validationStatus) => setEditing((p) => ({ ...p, validationStatus })),
          onToast
        );
      }
    },
    redo: async () => {
      if (redoStackRef.current.length === 0) return;
      const next = redoStackRef.current.pop()!;
      // Push current to undo
      undoStackRef.current.push({ dsl: editing.dsl, archData: editing.archData });
      setEditing((cur) => ({ ...cur, dsl: next.dsl, archData: next.archData }));
      if (wasmApiRef.current && viewerRef.current && next.archData) {
        await updateViewer(
          next.dsl,
          wasmApiRef.current,
          viewerRef.current,
          (archData) => setEditing((p) => ({ ...p, archData })),
          (validationStatus) => setEditing((p) => ({ ...p, validationStatus })),
          onToast
        );
      }
    },
  };

  return (
    <StudioEditingContext.Provider value={value}>
      {children}
    </StudioEditingContext.Provider>
  );
}
