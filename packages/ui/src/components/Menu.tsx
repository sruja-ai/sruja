import type { ReactNode } from "react";
import { Menu as MantineMenu } from "@mantine/core";
import { cn } from "../utils/cn";

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
  /** Button/content that triggers menu */
  trigger: ReactNode;
  /** Menu items */
  items: MenuItem[];
  /** Menu placement */
  placement?: "left" | "right" | "top" | "bottom";
}

const placementMap: Record<
  NonNullable<MenuProps["placement"]>,
  React.ComponentProps<typeof MantineMenu>["position"]
> = {
  bottom: "bottom",
  top: "top",
  right: "right",
  left: "left",
};

export function Menu({ trigger, items, placement = "bottom" }: MenuProps) {
  return (
    <MantineMenu position={placementMap[placement]}>
      <MantineMenu.Target>{trigger}</MantineMenu.Target>

      <MantineMenu.Dropdown
        className={cn(
          "bg-[var(--color-background)] border border-[var(--color-border)]",
          "min-w-[200px]"
        )}
      >
        {items.map((item, index) => (
          <>
            {item.divider && index > 0 && (
              <MantineMenu.Divider className="bg-[var(--color-border)]" />
            )}
            <MantineMenu.Item
              key={index}
              onClick={item.onClick}
              disabled={item.disabled}
              color={item.danger ? "red" : undefined}
              leftSection={item.icon}
              className={cn(
                "text-sm",
                "border-none text-left",
                item.disabled
                  ? "text-[var(--color-text-tertiary)] opacity-50 cursor-not-allowed"
                  : item.danger
                    ? "text-[var(--color-error-500)]"
                    : "text-[var(--color-text-primary)]"
              )}
            >
              {item.label}
            </MantineMenu.Item>
          </>
        ))}
      </MantineMenu.Dropdown>
    </MantineMenu>
  );
}
