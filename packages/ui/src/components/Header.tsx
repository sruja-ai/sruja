// packages/ui/src/components/Header.tsx
import { ReactNode } from 'react';
import { Logo } from './Logo';
import { cn } from '../utils/cn';

export interface NavLink {
  label: string;
  href: string;
  external?: boolean;
}

export interface HeaderProps {
  /** App title */
  title: string;
  /** App subtitle or description */
  subtitle?: string;
  /** Version string */
  version?: string;
  /** Whether logo is loading (will show rotation) */
  logoLoading?: boolean;
  /** Logo size in pixels */
  logoSize?: number;
  /** Navigation links (shown between logo and right content) */
  navLinks?: NavLink[];
  /** Left side content (before logo) */
  leftContent?: ReactNode;
  /** Right side content (after nav links) */
  rightContent?: ReactNode;
  /** Additional CSS classes */
  className?: string;
}

export function Header({
  title,
  subtitle,
  logoLoading = false,
  logoSize = 32,
  navLinks,
  leftContent,
  rightContent,
  className = '',
}: HeaderProps) {
  return (
    <header
      className={cn(
        'px-4 border-b border-[var(--color-border)]',
        'flex items-center gap-4',
        'bg-[var(--color-background)] shadow-sm',
        className
      )}
      style={{
        minHeight: '56px',
      }}
    >
      {/* Left content */}
      {leftContent && (
        <div className="flex items-center gap-3 flex-shrink-0">
          {leftContent}
        </div>
      )}

      {/* Logo and Title */}
      <div className="flex items-center gap-3 flex-shrink-0">
        <Logo size={logoSize} isLoading={logoLoading} />
        <div className="flex flex-col min-w-0">
          <h1 className="m-0 text-base font-semibold text-[var(--color-text-primary)] leading-tight">
            {title}
          </h1>
          {subtitle && (
            <p className="m-0 text-xs text-[var(--color-text-secondary)] leading-tight">
              {subtitle}
            </p>
          )}
        </div>
      </div>

      {/* Navigation Links */}
      {navLinks && navLinks.length > 0 && (
        <nav className="flex items-center gap-1 flex-1 px-4">
          {navLinks.map((link, idx) => (
            <a
              key={idx}
              href={link.href}
              target={link.external ? '_blank' : undefined}
              rel={link.external ? 'noopener noreferrer' : undefined}
              className="px-3 py-1.5 text-sm font-medium text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] hover:bg-[var(--color-surface)] rounded-md transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-[var(--color-primary)]"
            >
              {link.label}
            </a>
          ))}
        </nav>
      )}

      {/* Right content */}
      {rightContent && (
        <div className="flex items-center gap-2 flex-shrink-0">
          {rightContent}
        </div>
      )}
    </header>
  );
}
