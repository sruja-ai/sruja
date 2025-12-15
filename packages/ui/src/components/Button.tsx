// packages/ui/src/components/Button.tsx
import { forwardRef, ButtonHTMLAttributes, ReactNode } from "react";
import { vx } from "../utils/variants";

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  /** Button content */
  children: ReactNode;
  /** Button variant */
  variant?: "primary" | "secondary" | "outline" | "ghost" | "danger";
  /** Button size */
  size?: "sm" | "md" | "lg";
  /** Whether the button is in loading state */
  isLoading?: boolean;
  /** Enable or disable auto tracking */
  track?: boolean;
  /** Optional tracking name */
  trackName?: string;
}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(function Button(
  {
    children,
    variant = "primary",
    size = "md",
    isLoading = false,
    track = true,
    trackName,
    disabled,
    className = "",
    ...props
  },
  ref
) {
  const baseClasses =
    "inline-flex items-center justify-center font-medium rounded-md transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:pointer-events-none";

  const sizeClasses = {
    sm: "px-3.5 py-2 text-sm gap-2",
    md: "px-5 py-2.5 text-lg gap-2.5",
    lg: "px-7 py-3.5 text-xl gap-3",
  };

  const variantClasses = {
    primary:
      "bg-[var(--color-primary)] text-[var(--color-background)] hover:opacity-90 focus:ring-[var(--color-primary)] disabled:opacity-60",
    secondary:
      "bg-[var(--color-background)] border border-[var(--color-border)] text-[var(--color-text-primary)] hover:bg-[var(--color-surface)] focus:ring-[var(--color-border)] disabled:opacity-60",
    outline:
      "bg-transparent border border-[var(--color-primary)] text-[var(--color-primary)] hover:bg-[var(--color-surface)] focus:ring-[var(--color-primary)] disabled:opacity-60",
    ghost:
      "bg-transparent text-[var(--color-text-secondary)] hover:bg-[var(--color-surface)] focus:ring-[var(--color-surface)] disabled:opacity-60",
    danger:
      "bg-[var(--color-error-500)] text-[var(--color-background)] hover:opacity-90 focus:ring-[var(--color-error-500)] disabled:opacity-60",
  };

  return (
    <button
      ref={ref}
      type="button"
      disabled={disabled || isLoading}
      className={vx(baseClasses, sizeClasses[size], variantClasses[variant], className)}
      data-track={track ? "click" : undefined}
      data-component="button"
      data-name={trackName}
      {...props}
    >
      {isLoading && (
        <span className="inline-block w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin" />
      )}
      {children}
    </button>
  );
});
