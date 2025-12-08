// apps/studio-core/src/hooks/useModalState.ts
import { useState } from 'react';

export interface ModalConfig {
  isOpen: boolean;
  title: string;
  placeholder?: string;
  type: 'node' | 'relation' | 'rename' | null;
  data?: unknown;
}

export function useModalState() {
  const [modalConfig, setModalConfig] = useState<ModalConfig>({
    isOpen: false,
    title: '',
    type: null,
  });
  const [adrModalOpen, setAdrModalOpen] = useState(false);
  const [searchDialogOpen, setSearchDialogOpen] = useState(false);
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false);
  const [shortcutsOpen, setShortcutsOpen] = useState(false);

  return {
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
  };
}



