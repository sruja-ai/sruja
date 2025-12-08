// packages/ui/src/components/Footer.tsx
import { ReactNode } from 'react';
import { cn } from '../utils/cn';

export interface FooterProps {
  /** Footer content on the left */
  leftContent?: ReactNode;
  /** Footer content in the center */
  centerContent?: ReactNode;
  /** Footer content on the right */
  rightContent?: ReactNode;
  /** Additional CSS classes */
  className?: string;
}

export function Footer({
  leftContent,
  centerContent,
  rightContent,
  className = '',
}: FooterProps) {
  return (
    <footer
      className={cn(
        'py-2.5 px-6 border-t border-[var(--color-border)]',
        'flex justify-between items-center',
        'bg-[var(--color-background)] text-xs text-[var(--color-text-secondary)]',
        className
      )}
      style={{
        minHeight: '48px',
      }}
    >
      {leftContent && (
        <div className="flex items-center gap-4 flex-1 min-w-0">
          {leftContent}
        </div>
      )}
      {centerContent && (
        <div className="flex items-center gap-4 flex-shrink-0 mx-4">
          {centerContent}
        </div>
      )}
      {rightContent && (
        <div className="flex items-center gap-4 flex-1 justify-end min-w-0">
          {rightContent}
        </div>
      )}
    </footer>
  );
}
