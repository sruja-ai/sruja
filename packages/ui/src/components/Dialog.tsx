import type { ReactNode } from "react";
import { Modal } from "@mantine/core";
import { cn } from "../utils/cn";

export interface DialogProps {
  /** Whether dialog is open */
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
  size?: "sm" | "md" | "lg" | "xl" | "full";
  /** Show close button */
  showCloseButton?: boolean;
  /** Additional CSS classes */
  className?: string;
}

const sizeMap: Record<
  NonNullable<DialogProps["size"]>,
  React.ComponentProps<typeof Modal>["size"]
> = {
  sm: "xs",
  md: "sm",
  lg: "md",
  xl: "lg",
  full: "100%",
};

export function Dialog({
  isOpen,
  onClose,
  title,
  children,
  footer,
  size = "md",
  showCloseButton = true,
  className = "",
}: DialogProps) {
  return (
    <Modal
      opened={isOpen}
      onClose={onClose}
      size={sizeMap[size]}
      title={title}
      classNames={{
        content: cn("bg-[var(--color-background)]", className),
        header: "border-b border-[var(--color-border)]",
        title: "text-[var(--color-text-primary)]",
      }}
      closeButtonProps={{
        "aria-label": "Close dialog",
      }}
      withCloseButton={showCloseButton}
    >
      <div className="text-[var(--color-text-primary)]">{children}</div>

      {footer && (
        <div className="mt-4 pt-4 border-t border-[var(--color-border)] flex justify-end gap-3">
          {footer}
        </div>
      )}
    </Modal>
  );
}
