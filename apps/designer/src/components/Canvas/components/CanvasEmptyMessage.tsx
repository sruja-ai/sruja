import type { ReactNode } from "react";

interface CanvasEmptyMessageProps {
  title: string;
  children: ReactNode;
  "data-testid"?: string;
  className?: string;
}

export const CanvasEmptyMessage = ({
  title,
  children,
  "data-testid": dataTestId,
  className,
}: CanvasEmptyMessageProps) => {
  return (
    <div className={`canvas-empty ${className ?? ""}`} data-testid={dataTestId}>
      <div className="canvas-empty-content">
        <p>{title}</p>
        <p className="canvas-empty-message">{children}</p>
      </div>
    </div>
  );
};
