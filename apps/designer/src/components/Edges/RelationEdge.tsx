import type { EdgeProps } from "@xyflow/react";
import {
  BaseEdge,
  EdgeLabelRenderer,
  getSmoothStepPath,
  getBezierPath,
  getStraightPath,
} from "@xyflow/react";
import { EDGE_STYLES } from "../../utils/colorScheme";

type RouteType = "direct" | "orthogonal" | "curved" | "splines";

export function RelationEdge({
  id,
  sourceX,
  sourceY,
  targetX,
  targetY,
  sourcePosition,
  targetPosition,
  label,
  selected,
  style,
  data,
}: EdgeProps) {
  // Get preferred route from data or default to orthogonal (smooth step)
  const preferredRoute = (data?.preferredRoute as RouteType) || "orthogonal";

  // Calculate path based on preferred route type
  let edgePath: string;
  let labelX: number;
  let labelY: number;

  if (preferredRoute === "direct") {
    // Straight line from source to target
    [edgePath, labelX, labelY] = getStraightPath({
      sourceX,
      sourceY,
      targetX,
      targetY,
    });
  } else if (preferredRoute === "curved" || preferredRoute === "splines") {
    // Bezier curve for smooth curved paths
    [edgePath, labelX, labelY] = getBezierPath({
      sourceX,
      sourceY,
      sourcePosition,
      targetX,
      targetY,
      targetPosition,
      curvature: 0.25, // Gentle curve
    });
  } else {
    // Default: orthogonal with smooth corners (smooth step)
    [edgePath, labelX, labelY] = getSmoothStepPath({
      sourceX,
      sourceY,
      sourcePosition,
      targetX,
      targetY,
      targetPosition,
      borderRadius: 8, // Smooth corners for orthogonal segments
    });
  }

  const isActive = data?.isActive === true;
  const isDimmed = data?.isDimmed === true;
  const edgeLabel = label || (data?.label as string);
  const technology = data?.technology as string | undefined;
  const interaction = data?.interaction as "sync" | "async" | "event" | undefined;
  const isBidirectional = data?.bidirectional === true;
  const tags = (data?.tags as string[] | undefined) || [];
  const tagSet = new Set(tags.map((t) => t.toLowerCase()));

  // Determine interaction type from explicit field or tags
  const interactionType =
    interaction || (tagSet.has("async") ? "async" : tagSet.has("event") ? "event" : "sync");

  // Base Style with interaction-based styling
  let strokeDasharray: string | undefined;
  let strokeColor = style?.stroke || "#64748b";

  if (interactionType === "async") {
    strokeDasharray = "8,4"; // Dashed for async
  } else if (interactionType === "event") {
    strokeDasharray = "2,2"; // Dotted for event
    strokeColor = "#8B5CF6"; // Purple for events
  } else {
    strokeDasharray = undefined; // Solid for sync
  }

  const edgeStyle = {
    ...(selected || isActive ? EDGE_STYLES.selected : EDGE_STYLES.default),
    ...style,
    stroke: isActive ? "#10b981" : isDimmed ? "#e5e7eb" : strokeColor,
    strokeWidth: isActive ? 3 : isDimmed ? 1 : 2,
    opacity: isDimmed ? 0.3 : 1,
    transition: "all 0.3s ease",
    strokeDasharray: strokeDasharray || style?.strokeDasharray,
  };

  // Smart label offset: shift label perpendicular to edge to avoid overlap
  const edgeAngle = Math.atan2(targetY - sourceY, targetX - sourceX);
  const labelOffset = 8; // Pixels offset perpendicular to edge
  const adjustedLabelX = labelX + Math.sin(edgeAngle) * labelOffset;
  const adjustedLabelY = labelY - Math.cos(edgeAngle) * labelOffset;

  // Determine markers for bidirectional support
  const markerEnd = isActive
    ? "url(#arrow-active)"
    : isDimmed
      ? "url(#arrow-dimmed)"
      : "url(#arrow)";
  const markerStart = isBidirectional
    ? isActive
      ? "url(#arrow-active-start)"
      : isDimmed
        ? "url(#arrow-dimmed-start)"
        : "url(#arrow-start)"
    : undefined;

  return (
    <>
      <BaseEdge
        id={id}
        path={edgePath}
        style={edgeStyle}
        markerEnd={markerEnd}
        markerStart={markerStart}
      />

      {/* Cinematic Particle Animation */}
      {isActive && (
        <circle r="4" fill="#10b981">
          <animateMotion dur="1.5s" repeatCount="indefinite" path={edgePath}>
            <mpath href={`#${id}`} />
          </animateMotion>
        </circle>
      )}

      {(edgeLabel || technology || interactionType !== "sync") && (
        <EdgeLabelRenderer>
          <div
            style={{
              position: "absolute",
              transform: `translate(-50%, -50%) translate(${adjustedLabelX}px,${adjustedLabelY}px)`,
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              gap: 2,
              pointerEvents: "all",
              opacity: isDimmed ? 0.3 : 1,
              transition: "all 0.3s ease",
              zIndex: 1000,
              userSelect: "none",
            }}
            className="nodrag nopan"
          >
            {edgeLabel && (
              <div
                style={{
                  fontSize: 11,
                  fontWeight: 500,
                  background: isActive ? "#ecfdf5" : "rgba(255, 255, 255, 0.95)",
                  color: isActive ? "#059669" : "#374151",
                  padding: "3px 8px",
                  borderRadius: 4,
                  border: isActive ? "1px solid #10b981" : "1px solid #e5e7eb",
                  boxShadow: isActive
                    ? "0 2px 8px rgba(16, 185, 129, 0.2)"
                    : "0 1px 3px rgba(0, 0, 0, 0.08)",
                  whiteSpace: "nowrap",
                }}
              >
                {isBidirectional && <span style={{ marginRight: 4 }}>⇄</span>}
                {edgeLabel}
              </div>
            )}
            {technology && (
              <div
                style={{
                  fontSize: 9,
                  fontWeight: 400,
                  fontStyle: "italic",
                  background: isActive ? "#ecfdf5" : "rgba(255, 255, 255, 0.9)",
                  color: isActive ? "#059669" : "#6B7280",
                  padding: "2px 6px",
                  borderRadius: 3,
                  border: isActive ? "1px solid #10b981" : "1px solid #D1D5DB",
                  whiteSpace: "nowrap",
                }}
              >
                {technology}
              </div>
            )}
            {/* Interaction type badge for non-sync edges */}
            {interactionType !== "sync" && !technology && (
              <div
                style={{
                  fontSize: 8,
                  fontWeight: 500,
                  textTransform: "uppercase",
                  letterSpacing: "0.5px",
                  background: interactionType === "async" ? "#FEF3C7" : "#EDE9FE",
                  color: interactionType === "async" ? "#92400E" : "#5B21B6",
                  padding: "1px 5px",
                  borderRadius: 2,
                  border: interactionType === "async" ? "1px solid #FCD34D" : "1px solid #C4B5FD",
                }}
              >
                {interactionType}
              </div>
            )}
          </div>
        </EdgeLabelRenderer>
      )}
    </>
  );
}
