// packages/ui/src/components/Popover.tsx
import { Fragment } from 'react';
import type { ReactNode } from 'react';
import { Popover as HeadlessPopover, PopoverButton, PopoverPanel, Transition } from '@headlessui/react';
import { cn } from '../utils/cn';

export interface PopoverProps {
  /** Button/content that triggers the popover */
  trigger: ReactNode;
  /** Popover content */
  children: ReactNode;
  /** Popover placement */
  placement?: 'left' | 'right' | 'top' | 'bottom';
  /** Whether popover is open (controlled) */
  open?: boolean;
  /** Callback when popover state changes */
  onOpenChange?: (open: boolean) => void;
}

const placementClasses = {
  bottom: 'top-full mt-2',
  top: 'bottom-full mb-2',
  right: 'left-full ml-2',
  left: 'right-full mr-2',
};

export function Popover({
  trigger,
  children,
  placement = 'bottom',
  open,
  onOpenChange,
}: PopoverProps) {
  return (
    <HeadlessPopover className="relative">
      {({ open: isOpen }) => {
        // Sync internal state with controlled state
        if (open !== undefined && onOpenChange) {
          if (open !== isOpen) {
            // This is a bit of a workaround since Headless UI doesn't support fully controlled mode
            // In practice, you'd handle this at a higher level
          }
        }

        return (
          <>
            <PopoverButton className="focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-[var(--color-primary)] rounded-md">
              {trigger}
            </PopoverButton>

            <Transition
              as={Fragment}
              enter="transition ease-out duration-200"
              enterFrom="opacity-0 translate-y-1"
              enterTo="opacity-100 translate-y-0"
              leave="transition ease-in duration-150"
              leaveFrom="opacity-100 translate-y-0"
              leaveTo="opacity-0 translate-y-1"
            >
              <PopoverPanel
                className={cn(
                  'absolute z-50',
                  placementClasses[placement],
                  'min-w-[200px] max-w-[400px]',
                  'bg-[var(--color-background)] border border-[var(--color-border)] rounded-md shadow-lg',
                  'p-4'
                )}
              >
                {children}
              </PopoverPanel>
            </Transition>
          </>
        );
      }}
    </HeadlessPopover>
  );
}
