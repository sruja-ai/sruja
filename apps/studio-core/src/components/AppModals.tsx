// apps/studio-core/src/components/AppModals.tsx
import React from 'react';
import { InputModal } from './InputModal';
import { AdrModal } from './AdrModal';
import { ContextMenu } from './ContextMenu';
import { Toast } from './Toast';
import { SearchDialog } from './SearchDialog';
import { CommandPalette, type Command } from './CommandPalette';
import { ShortcutsModal } from './ShortcutsModal';
import {
  Edit2, ArrowRight, Copy, X, Trash2, User, Server, ClipboardCopy
} from 'lucide-react';
import type { ArchitectureJSON, ViewerInstance } from '@sruja/viewer';
import type { ModalConfig } from '../hooks/useModalState';
import type { ContextMenuState, CopiedNode, ToastState } from '../hooks/useUIState';
import { updateDocumentationForNode } from '../utils/documentationUtils';

interface AppModalsProps {
  // Modal state
  modalConfig: ModalConfig;
  setModalConfig: (config: ModalConfig) => void;
  adrModalOpen: boolean;
  setAdrModalOpen: (open: boolean) => void;
  searchDialogOpen: boolean;
  setSearchDialogOpen: (open: boolean) => void;
  commandPaletteOpen: boolean;
  setCommandPaletteOpen: (open: boolean) => void;
  shortcutsOpen: boolean;
  setShortcutsOpen: (open: boolean) => void;
  contextMenu: ContextMenuState | null;
  setContextMenu: (menu: ContextMenuState | null) => void;
  toast: ToastState | null;
  setToast: (toast: ToastState | null) => void;

  // Data
  archData: ArchitectureJSON | null;
  selectedNodeId: string | null;
  copiedNode: CopiedNode | null;
  viewerRef: React.RefObject<ViewerInstance | null>;
  sidebar: {
    showSidebar: boolean;
    activePanel: 'explorer' | 'documentation' | 'shortcuts' | 'guide';
    width: number;
  };
  setSidebar: (sidebar: { showSidebar: boolean; activePanel: 'explorer' | 'documentation' | 'shortcuts' | 'guide'; width: number } | ((prev: { showSidebar: boolean; activePanel: 'explorer' | 'documentation' | 'shortcuts' | 'guide'; width: number }) => { showSidebar: boolean; activePanel: 'explorer' | 'documentation' | 'shortcuts' | 'guide'; width: number })) => void;
  setDocumentation: (doc: { selectedNodeType: string | null; selectedNodeId: string | undefined; selectedNodeLabel: string | undefined } | ((prev: { selectedNodeType: string | null; selectedNodeId: string | undefined; selectedNodeLabel: string | undefined }) => { selectedNodeType: string | null; selectedNodeId: string | undefined; selectedNodeLabel: string | undefined })) => void;

  // Handlers
  onModalConfirm: (value?: string) => void | Promise<void>;
  onAdrConfirm: (data?: unknown) => void | Promise<void>;
  onToggleAddRelation: () => void;
  onCopy: () => void;
  onDelete: () => void;
  onRename: () => void;
  onPaste: () => void;
  onAddNode: (type: 'person' | 'system' | 'container' | 'component' | 'datastore' | 'queue' | 'requirement' | 'adr' | 'deployment') => void;
  onSearchSelect: (id: string) => void;
  commands: Command[];
  setSelectedNodeId: (id: string | null) => void;
}

/**
 * Consolidated component for all modals, dialogs, and overlays in the app
 * This reduces clutter in App.tsx
 */
export function AppModals({
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
  contextMenu,
  setContextMenu,
  toast,
  setToast,
  archData,
  selectedNodeId,
  copiedNode,
  viewerRef,
  sidebar,
  onModalConfirm,
  onAdrConfirm,
  onToggleAddRelation,
  onCopy,
  onDelete,
  onRename,
  onPaste,
  onAddNode,
  onSearchSelect,
  commands,
  setSidebar,
  setSelectedNodeId,
  setDocumentation,
}: AppModalsProps) {
  return (
    <>
      <InputModal
        isOpen={modalConfig.isOpen && modalConfig.type !== 'node'}
        title={modalConfig.title}
        placeholder={modalConfig.placeholder}
        onConfirm={onModalConfirm}
        onCancel={() => setModalConfig({ ...modalConfig, isOpen: false })}
      />

      <AdrModal
        isOpen={adrModalOpen}
        onConfirm={onAdrConfirm}
        onCancel={() => setAdrModalOpen(false)}
      />

      {/* Context Menu */}
      {contextMenu && (
        <ContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          actions={contextMenu.nodeId ? [
            {
              id: 'rename',
              label: 'Rename',
              icon: Edit2,
              onClick: onRename,
            },
            {
              id: 'add-relation',
              label: 'Add Relation',
              icon: ArrowRight,
              onClick: onToggleAddRelation,
            },
            {
              id: 'copy',
              label: 'Copy',
              icon: Copy,
              onClick: onCopy,
            },
            {
              id: 'divider-1',
              label: '',
              icon: X,
              onClick: () => {},
              divider: true
            },
            {
              id: 'delete',
              label: 'Delete',
              icon: Trash2,
              onClick: () => {
                if (contextMenu.nodeId) {
                  setSelectedNodeId(contextMenu.nodeId);
                  onDelete();
                }
              },
              variant: 'danger'
            }
          ] : [
            {
              id: 'add-person',
              label: 'Add Person',
              icon: User,
              onClick: () => onAddNode('person')
            },
            {
              id: 'add-system',
              label: 'Add System',
              icon: Server,
              onClick: () => onAddNode('system')
            },
            ...(copiedNode ? [{
              id: 'paste',
              label: 'Paste',
              icon: ClipboardCopy,
              onClick: onPaste,
            }] : [])
          ]}
          onClose={() => setContextMenu(null)}
        />
      )}

      {toast && (
        <Toast
          message={toast.message}
          type={toast.type}
          onClose={() => setToast(null)}
        />
      )}

      <SearchDialog
        isOpen={searchDialogOpen}
        archData={archData}
        onSelect={(id) => {
          onSearchSelect(id);
          updateDocumentationForNode(id, archData, setDocumentation);
          // Auto-switch to documentation panel if sidebar is open
          if (sidebar.showSidebar && sidebar.activePanel !== 'documentation') {
            setSidebar((prev) => ({ ...prev, activePanel: 'documentation', width: prev.width }));
          }
        }}
        onClose={() => setSearchDialogOpen(false)}
      />

      <CommandPalette
        isOpen={commandPaletteOpen}
        commands={commands}
        onClose={() => setCommandPaletteOpen(false)}
      />

      <ShortcutsModal isOpen={shortcutsOpen} onClose={() => setShortcutsOpen(false)} />
    </>
  );
}



