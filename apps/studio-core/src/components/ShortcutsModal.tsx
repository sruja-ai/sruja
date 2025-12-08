import React from 'react';
import { X, Keyboard, MousePointerClick, ZoomIn, ZoomOut, Maximize, Sidebar, Settings, Search, Download, Save, ArrowRight, Trash2, User, Server, Box, Database, Layers } from 'lucide-react';

type ShortcutItem = {
  label: string;
  keys: string;
};

type ShortcutGroup = {
  title: string;
  icon?: React.ReactNode;
  items: ShortcutItem[];
};

export function ShortcutsModal({ isOpen, onClose }: { isOpen: boolean; onClose: () => void }) {
  if (!isOpen) return null;

  const isMac = navigator.platform.toUpperCase().includes('MAC');
  const Mod = isMac ? '⌘' : 'Ctrl';

  const groups: ShortcutGroup[] = [
    {
      title: 'General',
      icon: <Keyboard size={18} />,
      items: [
        { label: 'Command Palette', keys: `${Mod} + K` },
        { label: 'Help / Shortcuts', keys: `${Mod} + ?` },
        { label: 'Search', keys: `${Mod} + F` },
      ],
    },
    {
      title: 'File',
      icon: <Save size={18} />,
      items: [
        { label: 'Save', keys: `${Mod} + S` },
        { label: 'Export', keys: `${Mod} + Shift + E` },
        { label: 'Copy Share Link', keys: `${Mod} + Shift + L` },
      ],
    },
    {
      title: 'Edit',
      icon: <MousePointerClick size={18} />,
      items: [
        { label: 'Copy Node', keys: `${Mod} + C` },
        { label: 'Paste Node', keys: `${Mod} + V` },
        { label: 'Delete Selected', keys: 'Del' },
        { label: 'Add Relation', keys: `${Mod} + R` },
        { label: 'Add Container', keys: `${Mod} + N` },
      ],
    },
    {
      title: 'Panels',
      icon: <Sidebar size={18} />,
      items: [
        { label: 'Toggle Sidebar', keys: `${Mod} + B` },
        { label: 'Toggle Properties', keys: `${Mod} + J` },
        { label: 'Toggle Documentation', keys: `${Mod} + Shift + D` },
      ],
    },
    {
      title: 'View',
      icon: <Maximize size={18} />,
      items: [
        { label: 'Zoom In', keys: `${Mod} + +` },
        { label: 'Zoom Out', keys: `${Mod} + -` },
        { label: 'Fit to Screen', keys: `${Mod} + 0` },
        { label: 'Toggle Explorer', keys: `${Mod} + B` },
        { label: 'Toggle Properties', keys: `${Mod} + .'` },
      ],
    },
    {
      title: 'Add Elements',
      icon: <ArrowRight size={18} />,
      items: [
        { label: 'Add Person', keys: `${Mod} + Shift + P` },
        { label: 'Add System', keys: `${Mod} + Shift + S` },
        { label: 'Add Container', keys: `${Mod} + Shift + C` },
        { label: 'Add Database', keys: `${Mod} + Shift + D` },
        { label: 'Add Queue', keys: `${Mod} + Shift + Q` },
      ],
    },
  ];

  return (
    <div className="fixed inset-0 z-[1000] flex items-center justify-center bg-black/50">
      <div className="w-[800px] max-w-[92vw] bg-[var(--color-background)] rounded-lg shadow-xl border border-[var(--color-border)] overflow-hidden">
        <div className="px-5 py-3 border-b border-[var(--color-border)] bg-[var(--color-surface)] flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Keyboard size={18} className="text-[var(--color-text-secondary)]" />
            <h3 className="m-0 text-base font-semibold text-[var(--color-text-primary)]">Keyboard Shortcuts</h3>
          </div>
          <button
            onClick={onClose}
            className="p-2 rounded hover:bg-[var(--color-surface)] text-[var(--color-text-secondary)]"
            title="Close"
          >
            <X size={18} />
          </button>
        </div>

        <div className="p-4 grid grid-cols-1 md:grid-cols-2 gap-4">
          {groups.map((group) => (
            <div key={group.title} className="border border-[var(--color-border)] rounded-md">
              <div className="px-3 py-2 border-b border-[var(--color-border)] bg-[var(--color-surface)] flex items-center gap-2">
                {group.icon}
                <span className="text-xs font-semibold uppercase tracking-wider text-[var(--color-text-secondary)]">{group.title}</span>
              </div>
              <ul className="p-3 m-0">
                {group.items.map((item) => (
                  <li key={item.label} className="flex items-center justify-between py-1.5">
                    <span className="text-sm text-[var(--color-text-primary)]">{item.label}</span>
                    <code className="text-xs bg-[var(--color-surface)] text-[var(--color-text-primary)] px-2 py-1 rounded border border-[var(--color-border)]">
                      {item.keys}
                    </code>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

