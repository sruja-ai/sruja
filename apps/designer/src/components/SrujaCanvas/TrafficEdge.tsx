import { BaseEdge, type EdgeProps, getBezierPath } from "@xyflow/react";
import "./TrafficEdge.css";

interface TrafficEdgeProps extends EdgeProps {
  className?: string;
}

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
  className = "",
}: TrafficEdgeProps) {
  const [edgePath] = getBezierPath({
    sourceX,
    sourceY,
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  });

  const isAnimating =
    className?.includes("animation-edge-active") ||
    className?.includes("animation-edge-highlighted");

  return (
    <>
      {/* SVG Definitions for gradients */}
      <defs>
        {/* Global traffic gradient for non-animated edges */}
        <linearGradient id="traffic-gradient" x1="0%" y1="0%" x2="100%" y2="0%">
          <stop offset="0%" stopColor="#94a3b8" stopOpacity="0.6" />
          <stop offset="50%" stopColor="#64748b" stopOpacity="0.8" />
          <stop offset="100%" stopColor="#475569" stopOpacity="0.6" />
        </linearGradient>
        {/* Animated edge gradient - bright blue */}
        <linearGradient id={`edge-gradient-blue-${id}`} x1="0%" y1="0%" x2="100%" y2="0%">
          <stop offset="0%" stopColor="#60a5fa" stopOpacity="0.8" />
          <stop offset="50%" stopColor="#3b82f6" stopOpacity="1" />
          <stop offset="100%" stopColor="#2563eb" stopOpacity="0.8" />
        </linearGradient>
        {/* Dark mode animated edge gradient */}
        <linearGradient id={`edge-gradient-blue-dark-${id}`} x1="0%" y1="0%" x2="100%" y2="0%">
          <stop offset="0%" stopColor="#93c5fd" stopOpacity="0.9" />
          <stop offset="50%" stopColor="#60a5fa" stopOpacity="1" />
          <stop offset="100%" stopColor="#3b82f6" stopOpacity="0.9" />
        </linearGradient>
      </defs>

      {/* Base Edge (Subtle background) */}
      <BaseEdge
        path={edgePath}
        markerEnd={markerEnd}
        style={{ ...style, strokeOpacity: 0.15, stroke: "var(--mantine-color-gray-5)" }}
      />

      {/* Highlight Line with gradient */}
      <path
        id={`${id}-path`}
        d={edgePath}
        fill="none"
        stroke={isAnimating ? `url(#edge-gradient-blue-${id})` : "url(#traffic-gradient)"}
        strokeWidth={isAnimating ? 3 : 2}
        className={`traffic-edge-glow ${className}`}
        style={{
          filter: isAnimating ? "drop-shadow(0 0 8px rgba(59, 130, 246, 0.6))" : undefined,
        }}
      />

      {/* Beautiful particle animations - only when active */}
      {isAnimating && (
        <>
          {/* Primary particle - larger, brighter */}
          <circle r="5" fill="#3b82f6" style={{ filter: "drop-shadow(0 0 6px #60a5fa)" }}>
            <animateMotion
              dur="1.2s"
              repeatCount="indefinite"
              calcMode="spline"
              keyPoints="0;1"
              keyTimes="0;1"
            >
              <mpath href={`#${id}-path`} />
            </animateMotion>
            <animate
              attributeName="opacity"
              values="0.8;1;0.8"
              dur="1.2s"
              repeatCount="indefinite"
            />
          </circle>

          {/* Secondary particle - offset for flow effect */}
          <circle
            r="4"
            fill="#60a5fa"
            style={{ opacity: 0.8, filter: "drop-shadow(0 0 4px #93c5fd)" }}
          >
            <animateMotion
              dur="1.2s"
              begin="0.4s"
              repeatCount="indefinite"
              calcMode="spline"
              keyPoints="0;1"
              keyTimes="0;1"
            >
              <mpath href={`#${id}-path`} />
            </animateMotion>
            <animate
              attributeName="opacity"
              values="0.6;0.9;0.6"
              dur="1.2s"
              begin="0.4s"
              repeatCount="indefinite"
            />
          </circle>

          {/* Tertiary particle - trailing effect */}
          <circle r="3" fill="#93c5fd" style={{ opacity: 0.7 }}>
            <animateMotion
              dur="1.2s"
              begin="0.8s"
              repeatCount="indefinite"
              calcMode="spline"
              keyPoints="0;1"
              keyTimes="0;1"
            >
              <mpath href={`#${id}-path`} />
            </animateMotion>
            <animate
              attributeName="opacity"
              values="0.4;0.7;0.4"
              dur="1.2s"
              begin="0.8s"
              repeatCount="indefinite"
            />
          </circle>
        </>
      )}

      {/* Standard particles for non-active edges */}
      {!isAnimating && (
        <>
          <circle r="3" fill="#94a3b8" style={{ opacity: 0.5 }}>
            <animateMotion dur="2.5s" repeatCount="indefinite">
              <mpath href={`#${id}-path`} />
            </animateMotion>
          </circle>
          <circle r="2.5" fill="#cbd5e1" style={{ opacity: 0.4 }}>
            <animateMotion dur="2.5s" begin="1.25s" repeatCount="indefinite">
              <mpath href={`#${id}-path`} />
            </animateMotion>
          </circle>
        </>
      )}
    </>
  );
}
