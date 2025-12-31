import { BaseEdge, type EdgeProps, getBezierPath } from "@xyflow/react";
import "./TrafficEdge.css";

export default function TrafficEdge({
  id,
  sourceX,
  sourceY,
  targetX,
  targetY,
  sourcePosition,
  targetPosition,
  style = {},
  markerEnd,
}: EdgeProps) {
  const [edgePath] = getBezierPath({
    sourceX,
    sourceY,
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  });

  return (
    <>
      {/* Base Edge (Subtle) */}
      <BaseEdge
        path={edgePath}
        markerEnd={markerEnd}
        style={{ ...style, strokeOpacity: 0.2, stroke: "var(--mantine-color-gray-5)" }}
      />

      {/* Highlight Line (Static but glowing) */}
      <path
        d={edgePath}
        fill="none"
        stroke="url(#traffic-gradient)"
        strokeWidth={2}
        className="traffic-edge-glow"
      />

      {/* Particle Animation using animateMotion */}
      <circle r="4" fill="#3b82f6" style={{ filter: "drop-shadow(0 0 4px #60a5fa)" }}>
        <animateMotion dur="2s" repeatCount="indefinite">
          <mpath href={`#${id}-path`} />
        </animateMotion>
      </circle>
      {/* 
          Duplicate packet for higher density
       */}
      <circle r="3" fill="#60a5fa" style={{ opacity: 0.7 }}>
        <animateMotion dur="2s" begin="1s" repeatCount="indefinite" path={edgePath} />
      </circle>

      {/* Hidden path for mpath reference if needed, ensuring unique ID */}
      <path id={`${id}-path`} d={edgePath} style={{ display: "none" }} />
    </>
  );
}
