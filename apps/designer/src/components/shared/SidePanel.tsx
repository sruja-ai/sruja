import type { ReactNode } from "react";
import { Drawer } from "@mantine/core";
import { X } from "lucide-react";
import "./SidePanel.css";

interface SidePanelProps {
  isOpen: boolean;
  onClose: () => void;
  title: ReactNode;
  children: ReactNode;
  footer?: ReactNode;
  size?: "md" | "lg" | "xl" | "2xl" | "full";
}

const sizeMap: Record<
  NonNullable<SidePanelProps["size"]>,
  React.ComponentProps<typeof Drawer>["size"]
> = {
  md: "sm",
  lg: "md",
  xl: "lg",
  "2xl": "xl",
  full: "100%",
};

export function SidePanel({
  isOpen,
  onClose,
  title,
  children,
  footer,
  size = "lg",
}: SidePanelProps) {
  return (
    <Drawer
      opened={isOpen}
      onClose={onClose}
      size={sizeMap[size]}
      position="right"
      classNames={{
        content: "side-panel-wrapper",
        body: "side-panel-content",
        header: "side-panel-header",
        title: "side-panel-title",
      }}
    >
      <Drawer.Title className="side-panel-title">{title}</Drawer.Title>
      <div className="side-panel-body">{children}</div>
      {footer && <div className="side-panel-footer">{footer}</div>}
      <Drawer.CloseButton className="side-panel-close" icon={<X size={20} />} />
    </Drawer>
  );
}
