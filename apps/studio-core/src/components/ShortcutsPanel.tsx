import React from 'react';

export function ShortcutsPanel() {
  const items = [
    { keys: '⌘/Ctrl + S', desc: 'Save' },
    { keys: '⌘/Ctrl + E', desc: 'Export' },
    { keys: '⌘/Ctrl + F', desc: 'Find' },
    { keys: '⌘/Ctrl + Z', desc: 'Undo' },
    { keys: '⌘/Ctrl + Y / ⌘/Ctrl + Shift + Z', desc: 'Redo' },
    { keys: 'N', desc: 'New node (container)' },
    { keys: 'E', desc: 'Add relation' },
    { keys: 'R', desc: 'Rename selected' },
    { keys: 'Delete', desc: 'Delete selected' },
    { keys: '+ / =', desc: 'Zoom in' },
    { keys: '-', desc: 'Zoom out' },
    { keys: '0', desc: 'Fit to screen' },
    { keys: '⌘/Ctrl + B', desc: 'Toggle Explorer' },
    { keys: '⌘/Ctrl + J', desc: 'Toggle Properties' },
    { keys: '⌘/Ctrl + Shift + D', desc: 'Toggle Docs' },
    { keys: '⌘/Ctrl + ?', desc: 'Show shortcuts modal' },
    { keys: '⌘/Ctrl + K', desc: 'Command palette' },
  ];

  return (
    <div className="flex-1 overflow-y-auto p-4 space-y-2 min-h-0">
      <h4 className="m-0 text-sm font-semibold text-[var(--color-text-primary)]">Keyboard Shortcuts</h4>
      <p className="text-xs text-[var(--color-text-secondary)]">Boost productivity with quick actions.</p>
      <div className="mt-2">
        {items.map((it, idx) => (
          <div key={idx} className="flex items-center justify-between py-1.5 border-b border-[var(--color-border)] last:border-b-0">
            <span className="text-xs font-mono text-[var(--color-text-secondary)]">{it.keys}</span>
            <span className="text-xs text-[var(--color-text-primary)]">{it.desc}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

