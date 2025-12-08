// apps/studio-core/src/hooks/useKeyboardShortcuts.ts
import { useEffect } from 'react';

interface KeyboardShortcuts {
  onUndo?: () => void;
  onRedo?: () => void;
  onDelete?: () => void;
  onRename?: () => void;
  onAddRelation?: () => void;
  onZoomIn?: () => void;
  onZoomOut?: () => void;
  onFitToScreen?: () => void;
  onEscape?: () => void;
  onCopy?: () => void;
  onPaste?: () => void;
}

export const useKeyboardShortcuts = (shortcuts: KeyboardShortcuts) => {
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Don't trigger shortcuts when typing in inputs
      const target = e.target as HTMLElement;
      if (
        target.tagName === 'INPUT' ||
        target.tagName === 'TEXTAREA' ||
        target.isContentEditable
      ) {
        return;
      }

      // Check for modifier keys
      const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
      const ctrlOrCmd = isMac ? e.metaKey : e.ctrlKey;
      const shift = e.shiftKey;

      // Undo: Cmd/Ctrl + Z
      if (ctrlOrCmd && !shift && e.key === 'z' && shortcuts.onUndo) {
        e.preventDefault();
        shortcuts.onUndo();
        return;
      }

      // Redo: Cmd/Ctrl + Shift + Z or Cmd/Ctrl + Y
      if (
        ctrlOrCmd &&
        ((shift && e.key === 'z') || e.key === 'y') &&
        shortcuts.onRedo
      ) {
        e.preventDefault();
        shortcuts.onRedo();
        return;
      }

      // Delete: Delete or Backspace
      if ((e.key === 'Delete' || e.key === 'Backspace') && shortcuts.onDelete) {
        e.preventDefault();
        shortcuts.onDelete();
        return;
      }

      // Rename: F2
      if (e.key === 'F2' && shortcuts.onRename) {
        e.preventDefault();
        shortcuts.onRename();
        return;
      }

      // Add Relation: R
      if (e.key === 'r' && !ctrlOrCmd && !shift && shortcuts.onAddRelation) {
        e.preventDefault();
        shortcuts.onAddRelation();
        return;
      }

      // Zoom In: Cmd/Ctrl + =
      if (ctrlOrCmd && (e.key === '=' || e.key === '+') && shortcuts.onZoomIn) {
        e.preventDefault();
        shortcuts.onZoomIn();
        return;
      }

      // Zoom Out: Cmd/Ctrl + -
      if (ctrlOrCmd && e.key === '-' && shortcuts.onZoomOut) {
        e.preventDefault();
        shortcuts.onZoomOut();
        return;
      }

      // Fit to Screen: Cmd/Ctrl + 0
      if (ctrlOrCmd && e.key === '0' && shortcuts.onFitToScreen) {
        e.preventDefault();
        shortcuts.onFitToScreen();
        return;
      }

      // Escape: Cancel current action
      if (e.key === 'Escape' && shortcuts.onEscape) {
        e.preventDefault();
        shortcuts.onEscape();
        return;
      }

      // Copy: Cmd/Ctrl + C
      if (ctrlOrCmd && e.key === 'c' && shortcuts.onCopy) {
        e.preventDefault();
        shortcuts.onCopy();
        return;
      }

      // Paste: Cmd/Ctrl + V
      if (ctrlOrCmd && e.key === 'v' && shortcuts.onPaste) {
        e.preventDefault();
        shortcuts.onPaste();
        return;
      }
    };

    window.addEventListener('keydown', handleKeyDown);

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [shortcuts]);
};
