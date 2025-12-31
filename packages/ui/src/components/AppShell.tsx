import React from "react";

export type AppShellProps = {
  header?: React.ReactNode;
  sidebar?: React.ReactNode;
  children: React.ReactNode;
  footer?: React.ReactNode;
};

export function AppShell({ header, sidebar, children, footer }: AppShellProps) {
  return (
    <div className="min-h-screen bg-[var(--color-surface)] text-[var(--color-text-primary)]">
      {header && (
        <div className="border-b border-[var(--color-border)] bg-[var(--color-background)]">
          {header}
        </div>
      )}
      <div className="flex">
        {sidebar && (
          <aside className="w-64 border-r border-[var(--color-border)] bg-[var(--color-background)]">
            {sidebar}
          </aside>
        )}
        <main className="flex-1 p-6">{children}</main>
      </div>
      {footer && (
        <div className="border-t border-[var(--color-border)] bg-[var(--color-background)]">
          {footer}
        </div>
      )}
    </div>
  );
}
