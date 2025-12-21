// packages/ui/src/components/Dialog.tsx
import { Fragment } from 'react';
import type { ReactNode } from 'react';
import { Dialog as HeadlessDialog, Transition } from '@headlessui/react';
import { X } from 'lucide-react';
import { cn } from '../utils/cn';

export interface DialogProps {
  /** Whether the dialog is open */
  isOpen: boolean;
  /** Callback when dialog should close */
  onClose: () => void;
  /** Dialog title */
  title?: string;
  /** Dialog content */
  children: ReactNode;
  /** Custom footer content */
  footer?: ReactNode;
  /** Dialog size */
  size?: 'sm' | 'md' | 'lg' | 'xl' | 'full';
  /** Show close button */
  showCloseButton?: boolean;
  /** Additional CSS classes */
  className?: string;
}

const sizeClasses = {
  sm: 'max-w-[400px] w-[90vw]',
  md: 'max-w-[600px] w-[90vw]',
  lg: 'max-w-[800px] w-[90vw]',
  xl: 'max-w-[1024px] w-[90vw]',
  full: 'max-w-screen w-screen h-screen rounded-none',
};

export function Dialog({
  isOpen,
  onClose,
  title,
  children,
  footer,
  size = 'md',
  showCloseButton = true,
  className = '',
}: DialogProps) {
  return (
    <Transition appear show={isOpen} as={Fragment}>
      <HeadlessDialog as="div" className="relative z-50" onClose={onClose}>
        <Transition.Child
          as={Fragment}
          enter="ease-out duration-300"
          enterFrom="opacity-0"
          enterTo="opacity-100"
          leave="ease-in duration-200"
          leaveFrom="opacity-100"
          leaveTo="opacity-0"
        >
          <div className="fixed inset-0 bg-black/50" aria-hidden="true" />
        </Transition.Child>

        <div className="fixed inset-0 overflow-y-auto">
          <div className="flex min-h-full items-center justify-center p-4">
            <Transition.Child
              as={Fragment}
              enter="ease-out duration-300"
              enterFrom="opacity-0 scale-95"
              enterTo="opacity-100 scale-100"
              leave="ease-in duration-200"
              leaveFrom="opacity-100 scale-100"
              leaveTo="opacity-0 scale-95"
            >
              <HeadlessDialog.Panel
                className={cn(
                  'transform overflow-hidden transition-all',
                  'bg-[var(--color-background)] rounded-lg shadow-xl',
                  sizeClasses[size],
                  className
                )}
              >
                {(title || showCloseButton) && (
                  <div className="px-6 py-4 border-b border-[var(--color-border)] flex justify-between items-center">
                    {title && (
                      <HeadlessDialog.Title
                        as="h3"
                        className="m-0 text-xl font-semibold text-[var(--color-text-primary)]"
                      >
                        {title}
                      </HeadlessDialog.Title>
                    )}
                    {showCloseButton && (
                      <button
                        onClick={onClose}
                        className="p-1 text-[var(--color-text-tertiary)] rounded hover:bg-[var(--color-surface)] hover:text-[var(--color-text-primary)] transition-all focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-[var(--color-primary)]"
                        aria-label="Close dialog"
                      >
                        <X size={20} />
                      </button>
                    )}
                  </div>
                )}

                <div className="p-6 text-[var(--color-text-primary)]">
                  {children}
                </div>

                {footer && (
                  <div className="px-6 py-4 border-t border-[var(--color-border)] flex justify-end gap-3">
                    {footer}
                  </div>
                )}
              </HeadlessDialog.Panel>
            </Transition.Child>
          </div>
        </div>
      </HeadlessDialog>
    </Transition>
  );
}
