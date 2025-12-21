// apps/designer/src/components/shared/Skeleton.tsx
import "./Skeleton.css";

interface SkeletonProps {
    variant?: "text" | "title" | "avatar" | "button" | "badge" | "card";
    width?: string | number;
    height?: string | number;
    className?: string;
    count?: number;
}

export function Skeleton({
    variant = "text",
    width,
    height,
    className = "",
    count = 1,
}: SkeletonProps) {
    const style: React.CSSProperties = {};
    if (width) style.width = typeof width === "number" ? `${width}px` : width;
    if (height) style.height = typeof height === "number" ? `${height}px` : height;

    const baseClass = `skeleton skeleton-${variant}`;

    if (count === 1) {
        return <div className={`${baseClass} ${className}`} style={style} aria-busy="true" aria-label="Loading" />;
    }

    return (
        <>
            {Array.from({ length: count }).map((_, i) => (
                <div
                    key={i}
                    className={`${baseClass} ${className}`}
                    style={style}
                    aria-busy="true"
                    aria-label="Loading"
                />
            ))}
        </>
    );
}

interface SkeletonCardProps {
    lines?: number;
    showAvatar?: boolean;
    showButton?: boolean;
}

export function SkeletonCard({ lines = 3, showAvatar = false, showButton = false }: SkeletonCardProps) {
    return (
        <div className="skeleton-card">
            {showAvatar && (
                <div className="skeleton skeleton-avatar" style={{ marginBottom: "1rem" }} />
            )}
            <div className="skeleton-list">
                {Array.from({ length: lines }).map((_, i) => (
                    <div
                        key={i}
                        className={`skeleton skeleton-text ${i === lines - 1 ? "short" : ""}`}
                    />
                ))}
            </div>
            {showButton && (
                <div className="skeleton skeleton-button" style={{ marginTop: "1rem", width: "120px" }} />
            )}
        </div>
    );
}

interface SkeletonListProps {
    items?: number;
    showAvatar?: boolean;
}

export function SkeletonList({ items = 5, showAvatar = false }: SkeletonListProps) {
    return (
        <div className="skeleton-list">
            {Array.from({ length: items }).map((_, i) => (
                <div key={i} className="skeleton-list-item">
                    {showAvatar && <div className="skeleton skeleton-avatar" />}
                    <div className="skeleton skeleton-text" style={{ flex: 1 }} />
                </div>
            ))}
        </div>
    );
}
