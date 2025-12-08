// apps/studio-core/src/handlers/modalHandlers.ts
import type React from 'react';
import type { ViewerInstance } from '@sruja/viewer';
import type { AdrData } from '../components/AdrModal';
import { validateNodeLabel, validateRelationLabel, validateAdrData } from '../utils/inputValidation';

interface ModalHandlersOptions {
  viewerRef: React.RefObject<ViewerInstance | null>;
  modalConfig: any;
  setModalConfig: any;
  setAdrModalOpen: (open: boolean) => void;
  setSourceNode: (node: string | null) => void;
  setIsAddingRelation: (adding: boolean) => void;
  syncDiagramToDslState: () => Promise<void>;
  setToast: any;
}

/**
 * Handle modal confirmation (for node, relation, rename)
 */
export function createHandleModalConfirm({
  viewerRef,
  modalConfig,
  setModalConfig,
  setSourceNode,
  setIsAddingRelation,
  syncDiagramToDslState,
  setToast,
}: ModalHandlersOptions) {
  return async (value: string) => {
    // Validate and sanitize input based on modal type
    if (modalConfig.type === 'node') {
      const validation = validateNodeLabel(value);
      if (!validation.isValid) {
        setToast({ message: validation.error || 'Invalid input', type: 'error' });
        return;
      }
      
      const { nodeType, parentId } = modalConfig.data;
      const sanitizedValue = validation.sanitized || value;
      viewerRef.current?.addNode(nodeType, sanitizedValue, parentId);
      await syncDiagramToDslState();
      setToast({ message: `Added ${nodeType}: ${sanitizedValue}`, type: 'success' });
    } else if (modalConfig.type === 'relation') {
      const validation = validateRelationLabel(value);
      if (!validation.isValid) {
        setToast({ message: validation.error || 'Invalid input', type: 'error' });
        return;
      }
      
      const { source, target } = modalConfig.data;
      const sanitizedValue = validation.sanitized || value;
      viewerRef.current?.addEdge(source, target, sanitizedValue);
      await syncDiagramToDslState();
      setToast({ message: 'Relation added', type: 'success' });

      // Reset relation state
      if (viewerRef.current?.cy) {
        viewerRef.current.cy.getElementById(source).removeStyle();
      }
      setSourceNode(null);
      setIsAddingRelation(false);
    }
    setModalConfig({ ...modalConfig, isOpen: false });
  };
}

/**
 * Handle ADR confirmation
 */
export function createHandleAdrConfirm({
  viewerRef,
  setAdrModalOpen,
  syncDiagramToDslState,
  setToast,
}: Pick<ModalHandlersOptions, 'viewerRef' | 'setAdrModalOpen' | 'syncDiagramToDslState' | 'setToast'>) {
  return async (data: AdrData) => {
    // Validate ADR data
    const validation = validateAdrData({
      title: data.title,
      status: data.status,
      context: data.context,
      decision: data.decision,
      consequences: data.consequences,
    });

    if (!validation.isValid) {
      setToast({ message: validation.error || 'Invalid ADR data', type: 'error' });
      return;
    }

    // Parse sanitized data
    const sanitizedData = JSON.parse(validation.sanitized || '{}');

    viewerRef.current?.addNode(
      'adr',
      sanitizedData.title,
      undefined,
      {
        status: sanitizedData.status,
        context: sanitizedData.context,
        decision: sanitizedData.decision,
        consequences: sanitizedData.consequences,
      }
    );
    await syncDiagramToDslState();
    setToast({ message: `Added ADR: ${sanitizedData.title}`, type: 'success' });
    setAdrModalOpen(false);
  };
}
