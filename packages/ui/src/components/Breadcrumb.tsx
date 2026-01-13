import type { ReactNode } from "react";
import { Breadcrumbs, Anchor } from "@mantine/core";
import { Home, ChevronRight } from "lucide-react";
import { cn } from "../utils/cn";

export interface BreadcrumbItem {
  /** Unique identifier for breadcrumb item */
  id: string;
  /** Display label */
  label: string;
}

export interface BreadcrumbProps {
  /** Array of breadcrumb items */
  items: BreadcrumbItem[];
  /** Callback fired when a breadcrumb item is clicked */
  onItemClick: (id: string) => void;
  /** Callback fired when home/root is clicked */
  onHomeClick?: () => void;
  /** Custom home icon */
  homeIcon?: ReactNode;
  /** Custom separator between items */
  separator?: ReactNode;
  /** Whether to show home button */
  showHome?: boolean;
  /** Additional CSS classes */
  className?: string;
}

export function Breadcrumb({
  items,
  onItemClick,
  onHomeClick,
  homeIcon,
  separator,
  showHome = true,
  className = "",
}: BreadcrumbProps) {
  const breadcrumbItems = items.map((item, index) => {
    const isLast = index === items.length - 1;
    return (
      <Anchor
        key={item.id}
        onClick={(e) => {
          e.preventDefault();
          onItemClick(item.id);
        }}
        href={isLast ? undefined : "#"}
        className={cn(
          "text-sm transition-colors",
          isLast
            ? "text-[var(--color-text-primary)] font-medium pointer-events-none"
            : "text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]"
        )}
      >
        {item.label}
      </Anchor>
    );
  });

  return (
    <Breadcrumbs className={className} separator={separator || <ChevronRight size={14} />}>
      {showHome && (
        <Anchor
          onClick={(e) => {
            e.preventDefault();
            if (onHomeClick) {
              onHomeClick();
            } else {
              onItemClick("root");
            }
          }}
          href="#"
          className="text-sm text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] transition-colors"
          aria-label="Home"
        >
          {homeIcon || <Home size={16} />}
        </Anchor>
      )}
      {breadcrumbItems}
    </Breadcrumbs>
  );
}
