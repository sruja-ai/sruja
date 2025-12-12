import { Fragment } from 'react';
import type { ReactNode } from 'react';
import { Dialog, Transition } from '@headlessui/react';
import { X } from 'lucide-react';
import { cn } from '@sruja/ui';
import './SidePanel.css';

interface SidePanelProps {
    isOpen: boolean;
    onClose: () => void;
    title: ReactNode;
    children: ReactNode;
    footer?: ReactNode;
    size?: 'md' | 'lg' | 'xl' | '2xl' | 'full';
}


export function SidePanel({
    isOpen,
    onClose,
    title,
    children,
    footer,
    size = 'lg',
}: SidePanelProps) {
    return (
        <Transition.Root show={isOpen} as={Fragment}>
            <Dialog as="div" className="side-panel-root" onClose={onClose}>
                <div className="side-panel-overlay" aria-hidden="true" />

                <div className="side-panel-container">
                    <Transition.Child
                        as={Fragment}
                        enter="transform transition ease-in-out duration-300"
                        enterFrom="translate-x-full"
                        enterTo="translate-x-0"
                        leave="transform transition ease-in-out duration-300"
                        leaveFrom="translate-x-0"
                        leaveTo="translate-x-full"
                    >
                        <Dialog.Panel className={cn("side-panel-wrapper", size === '2xl' ? 'side-panel-2xl' : size)}>
                            <div className="side-panel-content">
                                <div className="side-panel-header">
                                    <Dialog.Title className="side-panel-title">
                                        {title}
                                    </Dialog.Title>
                                    <button
                                        type="button"
                                        className="side-panel-close"
                                        onClick={onClose}
                                    >
                                        <span className="sr-only">Close panel</span>
                                        <X size={20} aria-hidden="true" />
                                    </button>
                                </div>
                                <div className="side-panel-body">
                                    {children}
                                </div>
                                {footer && (
                                    <div className="side-panel-footer">
                                        {footer}
                                    </div>
                                )}
                            </div>
                        </Dialog.Panel>
                    </Transition.Child>
                </div>
            </Dialog>
        </Transition.Root>
    );
}
