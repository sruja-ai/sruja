import { X, AlertCircle } from 'lucide-react';
import { Button } from '@sruja/ui';
import type { ValidationStatus } from '../types';

interface ErrorModalProps {
  validationStatus: ValidationStatus;
  onClose: () => void;
}

export function ErrorModal({ validationStatus, onClose }: ErrorModalProps) {
  if (!validationStatus.lastError) return null;

  return (
    <div 
      className="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
      onClick={onClose}
      style={{ zIndex: 10000 }}
    >
      <div 
        className="bg-[var(--color-background)] rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-[80vh] flex flex-col"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between p-4 border-b border-[var(--color-border)]">
          <div className="flex items-center gap-2">
            <AlertCircle className="w-5 h-5 text-[var(--color-error-500)]" />
            <h2 className="text-lg font-semibold text-[var(--color-text-primary)]">Validation Error</h2>
          </div>
          <button
            onClick={onClose}
            className="text-[var(--color-text-tertiary)] hover:text-[var(--color-text-primary)] transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-[var(--color-primary)] rounded"
            title="Close"
          >
            <X className="w-5 h-5" />
          </button>
        </div>
        <div className="p-4 overflow-auto flex-1">
          <div className="bg-[var(--color-surface)] border border-[var(--color-error-500)] rounded-lg p-4">
            <pre className="text-sm text-[var(--color-error-500)] whitespace-pre-wrap break-words font-mono">
              {validationStatus.lastError}
            </pre>
          </div>
          {validationStatus.errors > 0 && (
            <p className="mt-4 text-sm text-[var(--color-text-secondary)]">
              {validationStatus.errors} error{validationStatus.errors !== 1 ? 's' : ''} found
            </p>
          )}
        </div>
        <div className="p-4 border-t border-[var(--color-border)] flex justify-end">
          <Button
            variant="primary"
            size="sm"
            onClick={onClose}
          >
            Close
          </Button>
        </div>
      </div>
    </div>
  );
}

