// apps/designer/src/components/Canvas/GridOverlay.tsx
import { useMemo } from "react";
import "./GridOverlay.css";

interface GridOverlayProps {
  size: number; // Grid size in pixels (8, 16, 32)
  visible: boolean;
}

export function GridOverlay({ size, visible }: GridOverlayProps) {
  const patternId = useMemo(() => `grid-pattern-${size}`, [size]);

  if (!visible) return null;

  return (
    <svg
      className="grid-overlay"
      style={{
        position: "absolute",
        top: 0,
        left: 0,
        width: "100%",
        height: "100%",
        pointerEvents: "none",
        zIndex: 0,
      }}
    >
      <defs>
        <pattern
          id={patternId}
          width={size}
          height={size}
          patternUnits="userSpaceOnUse"
        >
          <path
            d={`M ${size} 0 L 0 0 0 ${size}`}
            fill="none"
            stroke="var(--grid-color, rgba(128, 128, 128, 0.2))"
            strokeWidth="1"
          />
        </pattern>
      </defs>
      <rect width="100%" height="100%" fill={`url(#${patternId})`} />
    </svg>
  );
}
