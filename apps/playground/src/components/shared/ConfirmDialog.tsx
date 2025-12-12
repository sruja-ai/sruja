// apps/playground/src/components/shared/ConfirmDialog.tsx
import { Dialog, Button } from '@sruja/ui';
import { AlertTriangle } from 'lucide-react';

interface ConfirmDialogProps {
    isOpen: boolean;
    onClose: () => void;
    onConfirm: () => void;
    title: string;
    message: string;
    confirmLabel?: string;
    cancelLabel?: string;
    variant?: 'danger' | 'warning';
}

export function ConfirmDialog({
    isOpen,
    onClose,
    onConfirm,
    title,
    message,
    confirmLabel = 'Confirm',
    cancelLabel = 'Cancel',
    variant = 'danger',
}: ConfirmDialogProps) {
    const handleConfirm = () => {
        onConfirm();
        onClose();
    };

    return (
        <Dialog
            isOpen={isOpen}
            onClose={onClose}
            title={title}
            size="sm"
            footer={
                <>
                    <Button variant="secondary" onClick={onClose} type="button">
                        {cancelLabel}
                    </Button>
                    <Button
                        variant={variant === 'danger' ? 'danger' : 'primary'}
                        onClick={handleConfirm}
                    >
                        {confirmLabel}
                    </Button>
                </>
            }
        >
            <div className="flex gap-4">
                <div className="flex-shrink-0">
                    <AlertTriangle
                        size={24}
                        className={variant === 'danger' ? 'text-[var(--color-error-500)]' : 'text-[var(--color-warning-500)]'}
                    />
                </div>
                <div className="flex-1">
                    <p className="text-[var(--color-text-primary)]">{message}</p>
                </div>
            </div>
        </Dialog>
    );
}