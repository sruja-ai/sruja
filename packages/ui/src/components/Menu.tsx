// packages/ui/src/components/Menu.tsx
import { Fragment } from 'react';
import type { ReactNode } from 'react';
import { Menu as HeadlessMenu, Transition } from '@headlessui/react';
import { cn } from '../utils/cn';

export interface MenuItem {
  /** Item label */
  label: string;
  /** Item action */
  onClick: () => void;
  /** Whether item is disabled */
  disabled?: boolean;
  /** Item icon (optional) */
  icon?: ReactNode;
  /** Divider before this item */
  divider?: boolean;
  /** Danger styling */
  danger?: boolean;
}

export interface MenuProps {
  /** Button/content that triggers the menu */
  trigger: ReactNode;
  /** Menu items */
  items: MenuItem[];
  /** Menu placement */
  placement?: 'left' | 'right' | 'top' | 'bottom';
}

const placementClasses = {
  bottom: 'top-full mt-2',
  top: 'bottom-full mb-2',
  right: 'left-full ml-2',
  left: 'right-full mr-2',
};

export function Menu({ trigger, items, placement = 'bottom' }: MenuProps) {
  return (
    <HeadlessMenu as="div" className="relative inline-block text-left">
      <HeadlessMenu.Button as={Fragment}>{trigger}</HeadlessMenu.Button>

      <Transition
        as={Fragment}
        enter="transition ease-out duration-100"
        enterFrom="transform opacity-0 scale-95"
        enterTo="transform opacity-100 scale-100"
        leave="transition ease-in duration-75"
        leaveFrom="transform opacity-100 scale-100"
        leaveTo="transform opacity-0 scale-95"
      >
        <HeadlessMenu.Items
          className={cn(
            'absolute z-50',
            placementClasses[placement],
            'min-w-[200px]',
            'bg-[var(--color-background)] border border-[var(--color-border)] rounded-md shadow-lg',
            'py-1'
          )}
        >
          {items.map((item, index) => (
            <Fragment key={index}>
              {item.divider && index > 0 && (
                <div className="h-px bg-[var(--color-border)] my-1" />
              )}
              <HeadlessMenu.Item disabled={item.disabled}>
                {({ active, disabled }) => (
                  <button
                    onClick={item.onClick}
                    disabled={disabled || item.disabled}
                    className={cn(
                      'w-full flex items-center gap-3 px-4 py-2 text-sm',
                      'border-none text-left transition-all',
                      disabled || item.disabled
                        ? 'text-[var(--color-text-tertiary)] opacity-50 cursor-not-allowed'
                        : item.danger
                        ? 'text-[var(--color-error-500)]'
                        : 'text-[var(--color-text-primary)]',
                      active && !disabled && 'bg-[var(--color-surface)]'
                    )}
                  >
                    {item.icon && <span className="flex items-center">{item.icon}</span>}
                    <span>{item.label}</span>
                  </button>
                )}
              </HeadlessMenu.Item>
            </Fragment>
          ))}
        </HeadlessMenu.Items>
      </Transition>
    </HeadlessMenu>
  );
}
