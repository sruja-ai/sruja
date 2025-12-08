// apps/studio-core/src/components/ContextMenu.tsx
import React from 'react';
import { Edit2, Trash2, Copy, ArrowRight, Plus, Layers, ClipboardCopy } from 'lucide-react';

interface ContextMenuAction {
    id: string;
    label: string;
    icon: React.ElementType;
    onClick: () => void;
    variant?: 'default' | 'danger';
    divider?: boolean;
}

interface ContextMenuProps {
    x: number;
    y: number;
    actions: ContextMenuAction[];
    onClose: () => void;
}

export const ContextMenu: React.FC<ContextMenuProps> = ({ x, y, actions, onClose }) => {
    React.useEffect(() => {
        const handleClickOutside = () => onClose();
        const handleEscape = (e: KeyboardEvent) => {
            if (e.key === 'Escape') onClose();
        };

        // Close on outside click
        setTimeout(() => {
            document.addEventListener('click', handleClickOutside);
            document.addEventListener('contextmenu', handleClickOutside);
        }, 0);

        window.addEventListener('keydown', handleEscape);

        return () => {
            document.removeEventListener('click', handleClickOutside);
            document.removeEventListener('contextmenu', handleClickOutside);
            window.removeEventListener('keydown', handleEscape);
        };
    }, [onClose]);

    return (
        <div
            className="fixed z-50 bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-xl py-1 min-w-[180px]"
            style={{ left: `${x}px`, top: `${y}px` }}
            onClick={(e) => e.stopPropagation()}
            onContextMenu={(e) => e.preventDefault()}
        >
            {actions.map((action, index) => {
                const Icon = action.icon;
                const isDanger = action.variant === 'danger';
                
                // Handle divider
                if (action.divider) {
                    return <div key={action.id} className="my-1 border-t border-gray-700" />;
                }
                
                return (
                    <button
                        key={action.id}
                        onClick={() => {
                            action.onClick();
                            onClose();
                        }}
                        className={`w-full flex items-center gap-2.5 px-3 py-2 text-sm transition-colors ${
                            isDanger
                                ? 'text-[var(--color-error-500)] hover:bg-[var(--color-surface)]'
                                : 'text-[var(--color-text-secondary)] hover:bg-[var(--color-surface)] hover:text-[var(--color-text-primary)]'
                        }`}
                    >
                        <Icon size={14} />
                        <span className="truncate">{action.label}</span>
                    </button>
                );
            })}
        </div>
    );
};

