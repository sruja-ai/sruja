import type { ReactNode } from "react";

interface CanvasEmptyMessageProps {
    title: string;
    children: ReactNode;
    'data-testid'?: string;
    className?: string;
}

export const CanvasEmptyMessage = ({
    title,
    children,
    'data-testid': dataTestId,
    className
}: CanvasEmptyMessageProps) => {
    return (
        <div className={`likec4-canvas-empty ${className ?? ''}`} data-testid={dataTestId}>
            <div className="likec4-canvas-empty-content">
                <p>{title}</p>
                <p className="likec4-canvas-empty-message">{children}</p>
            </div>
        </div>
    );
};
