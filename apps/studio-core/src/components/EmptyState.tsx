import { LucideIcon } from 'lucide-react';
import './EmptyState.css';

interface EmptyStateProps {
    icon: LucideIcon;
    title: string;
    description: string;
    action?: {
        label: string;
        onClick: () => void;
    };
    iconColor?: string;
}

export function EmptyState({
    icon: Icon,
    title,
    description,
    action,
    iconColor = '#94a3b8'
}: EmptyStateProps) {
    return (
        <div className="empty-state">
            <div className="empty-state-icon" style={{ color: iconColor }}>
                <Icon size={64} strokeWidth={1.5} />
            </div>
            <h3 className="empty-state-title">{title}</h3>
            <p className="empty-state-description">{description}</p>
            {action && (
                <button onClick={action.onClick} className="empty-state-action">
                    {action.label}
                </button>
            )}
        </div>
    );
}
