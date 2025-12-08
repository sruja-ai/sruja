// apps/studio-core/src/handlers/viewerHandlers.ts
import type React from 'react';
import type { ViewerInstance, ArchitectureJSON } from '@sruja/viewer';
import LZString from 'lz-string';

interface ViewerHandlersOptions {
  viewerRef: React.RefObject<ViewerInstance | null>;
  setZoomLevel: (level: number) => void;
  setCurrentLevel: (level: number | undefined) => void;
  currentLevel: number | undefined;
  focusSystemId: string | undefined;
  focusContainerId: string | undefined;
  dsl: string;
  setToast: (toast: { message: string; type: 'success' | 'error' | 'info' } | null) => void;
}

/**
 * Handle zoom in
 */
export function createHandleZoomIn({
  viewerRef,
  setZoomLevel,
}: Pick<ViewerHandlersOptions, 'viewerRef' | 'setZoomLevel'>) {
  return () => {
    if (viewerRef.current?.cy) {
      const currentZoom = viewerRef.current.cy.zoom();
      const newZoom = Math.min(currentZoom * 1.2, 3);
      viewerRef.current.cy.zoom(newZoom);
      setZoomLevel(newZoom);
    }
  };
}

/**
 * Handle zoom out
 */
export function createHandleZoomOut({
  viewerRef,
  setZoomLevel,
}: Pick<ViewerHandlersOptions, 'viewerRef' | 'setZoomLevel'>) {
  return () => {
    if (viewerRef.current?.cy) {
      const currentZoom = viewerRef.current.cy.zoom();
      const newZoom = Math.max(currentZoom / 1.2, 0.1);
      viewerRef.current.cy.zoom(newZoom);
      setZoomLevel(newZoom);
    }
  };
}

/**
 * Handle fit to screen
 */
export function createHandleFitToScreen({
  viewerRef,
  setZoomLevel,
}: Pick<ViewerHandlersOptions, 'viewerRef' | 'setZoomLevel'>) {
  return () => {
    if (viewerRef.current?.cy) {
      viewerRef.current.cy.fit(undefined, 80);
      setZoomLevel(viewerRef.current.cy.zoom());
    }
  };
}

/**
 * Handle set level
 */
export function createHandleSetLevel({
  viewerRef,
  setCurrentLevel,
}: Pick<ViewerHandlersOptions, 'viewerRef' | 'setCurrentLevel'>) {
  return (level: number) => {
    if (viewerRef.current) {
      viewerRef.current.setLevel(level);
      setCurrentLevel(level);
    }
  };
}

/**
 * Handle share (copy share link)
 */
export function createHandleShare({
  dsl,
  currentLevel,
  focusSystemId,
  focusContainerId,
  setToast,
}: Pick<ViewerHandlersOptions, 'dsl' | 'currentLevel' | 'focusSystemId' | 'focusContainerId' | 'setToast'>) {
  return () => {
    try {
      const dslText = dsl || '';
      const b64 = encodeURIComponent(LZString.compressToBase64(dslText));
      const params = new URLSearchParams();
      if (currentLevel) params.set('level', String(currentLevel));
      if (focusContainerId) {
        params.set('focus', focusContainerId);
      } else if (focusSystemId) {
        params.set('focus', focusSystemId);
      }
      const url = `${window.location.origin}/viewer#code=${b64}${params.toString() ? '&' + params.toString() : ''}`;
      navigator.clipboard.writeText(url);
      setToast({ message: 'Share link copied', type: 'success' });
    } catch (e) {
      setToast({ message: 'Failed to copy share link', type: 'error' });
    }
  };
}

/**
 * Handle new diagram
 */
export function createHandleNewDiagram({
  updateDsl,
  dsl,
  setSelectedExample,
  setSelectedNodeId,
  setToast,
}: {
  updateDsl: (dsl: string) => Promise<void>;
  dsl: string;
  setSelectedExample: (example: any) => void;
  setSelectedNodeId: (id: string | null) => void;
  setToast: (toast: { message: string; type: 'success' | 'error' | 'info' } | null) => void;
}) {
  return async () => {
    // Check if there's existing content
    const hasContent =
      dsl &&
      dsl.trim().length > 0 &&
      !dsl.trim().match(/^architecture\s+"[^"]*"\s*\{\s*\}$/);

    if (hasContent) {
      // Show confirmation
      const confirmed = window.confirm(
        'Are you sure you want to start a new diagram? This will clear all current content.'
      );
      if (!confirmed) {
        return;
      }
    }

    // Create empty architecture
    const emptyDsl = 'architecture "New Architecture" {\n}';
    await updateDsl(emptyDsl);
    setSelectedExample('Simple Web App'); // Reset example selector
    setSelectedNodeId(null);
    setToast({ message: 'New diagram created', type: 'success' });
  };
}
