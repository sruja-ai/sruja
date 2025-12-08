// apps/studio-core/src/hooks/useUIState.ts
import { useState } from 'react';
import type { ExampleKey } from '../examples';

export interface ToastState {
  message: string;
  type: 'success' | 'error' | 'info';
}

export interface ContextMenuState {
  x: number;
  y: number;
  nodeId: string | null;
}

export interface CopiedNode {
  id: string;
  data: unknown;
  type: string;
}

export function useUIState() {
  const [selectedExample, setSelectedExample] = useState<ExampleKey>('Simple Web App');
  const [isAddingRelation, setIsAddingRelation] = useState(false);
  const [sourceNode, setSourceNode] = useState<string | null>(null);
  const [activeView, setActiveView] = useState<'editor' | 'split' | 'viewer'>('split');
  const [designEnabled, setDesignEnabled] = useState<boolean>(true);
  const [designStep, setDesignStep] = useState<number>(1);
  const [contextMenu, setContextMenu] = useState<ContextMenuState | null>(null);
  const [copiedNode, setCopiedNode] = useState<CopiedNode | null>(null);
  const [zoomLevel, setZoomLevel] = useState(1);
  const [lastSaved, setLastSaved] = useState<Date | null>(null);
  const [currentLevel, setCurrentLevel] = useState<number | undefined>(undefined);
  const [focusSystemId, setFocusSystemId] = useState<string | undefined>(undefined);
  const [focusContainerId, setFocusContainerId] = useState<string | undefined>(undefined);
  const [toast, setToast] = useState<ToastState | null>(null);
  const [isWasmLoading, setIsWasmLoading] = useState(true);
  const [showWelcome, setShowWelcome] = useState(() => {
    const hideWelcome = localStorage.getItem('studio-hide-welcome');
    return hideWelcome !== 'true';
  });

  return {
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
    designStep,
    setDesignStep,
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
    setFocusSystemId,
    focusContainerId,
    setFocusContainerId,
    toast,
    setToast,
    isWasmLoading,
    setIsWasmLoading,
    showWelcome,
    setShowWelcome,
  };
}



