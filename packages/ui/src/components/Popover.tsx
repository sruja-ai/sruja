import type { ReactNode } from "react";
import { Popover as MantinePopover } from "@mantine/core";

export interface PopoverProps {
  /** Button/content that triggers popover */
  trigger: ReactNode;
  /** Popover content */
  content: ReactNode;
  /** Popover placement */
  placement?: "left" | "right" | "top" | "bottom";
  /** Whether popover is controlled */
  isOpen?: boolean;
  /** Callback when popover state changes */
  onOpenChange?: (open: boolean) => void;
}

const placementMap: Record<
  NonNullable<PopoverProps["placement"]>,
  React.ComponentProps<typeof MantinePopover>["position"]
> = {
  bottom: "bottom",
  top: "top",
  right: "right",
  left: "left",
};

export function Popover({
  trigger,
  content,
  placement = "bottom",
  isOpen,
  onOpenChange,
}: PopoverProps) {
  return (
    <MantinePopover
      position={placementMap[placement]}
      opened={isOpen}
      onChange={onOpenChange}
      classNames={{
        dropdown:
          "bg-[var(--color-background)] border border-[var(--color-border)] rounded-md shadow-lg p-4 min-w-[200px] max-w-[400px]",
      }}
    >
      <MantinePopover.Target>{trigger}</MantinePopover.Target>
      <MantinePopover.Dropdown>{content}</MantinePopover.Dropdown>
    </MantinePopover>
  );
}
