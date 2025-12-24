import { memo } from "react";
import { BaseEdge, EdgeLabelRenderer, type EdgeProps } from "@xyflow/react";

interface SplineEdgeData extends Record<string, unknown> {
  label?: string;
  points?: Array<[number, number]>;
}

/**
 * Custom edge component that renders bezier splines from Graphviz output.
 * Falls back to straight line if no spline points are provided.
 */
function SplineEdge({
  id,
  sourceX,
  sourceY,
  targetX,
  targetY,
  label,
  labelStyle,
  markerEnd,
  style,
  data,
}: EdgeProps) {
  const edgeData = data as SplineEdgeData | undefined;
  const points = edgeData?.points;

  let edgePath: string;
  let labelX: number;
  let labelY: number;

  if (points && points.length >= 2) {
    // Build SVG path from bezier spline points
    // Graphviz bezier splines are cubic bezier curves: start, cp1, cp2, end, cp1, cp2, end, ...
    // For SVG path: M start C cp1 cp2 end S cp2 end ...
    // NOTE: We use sourceX/sourceY and targetX/targetY as endpoints to ensure edges stay connected
    // when nodes move, using the spline points only for intermediate curve control points
    // Use React Flow's provided source coordinates (which update when nodes move) as the start
    edgePath = `M ${sourceX} ${sourceY}`;

    // Process points in groups of 3 for cubic bezier (after start point)
    // Pattern: [start, cp1, cp2, end, cp1, cp2, end, ...]
    // We use the control points from Graphviz but always end at targetX/targetY

    if (points.length >= 4) {
      // First cubic bezier: use control points but end at target
      edgePath += ` C ${points[1][0]} ${points[1][1]}, ${points[2][0]} ${points[2][1]}, ${targetX} ${targetY}`;

      // For multi-segment curves, we could add more segments here, but for simplicity
      // we'll just use the first cubic bezier ending at target
      // This ensures the edge always connects properly when nodes move
    } else if (points.length === 2) {
      // Just two points - straight line to target
      edgePath += ` L ${targetX} ${targetY}`;
    } else if (points.length === 3) {
      // Quadratic bezier ending at target
      edgePath += ` Q ${points[1][0]} ${points[1][1]}, ${targetX} ${targetY}`;
    }

    // Calculate label position at midpoint between source and target
    labelX = (sourceX + targetX) / 2;
    labelY = (sourceY + targetY) / 2;
  } else {
    // Fallback to straight line between source and target
    edgePath = `M ${sourceX} ${sourceY} L ${targetX} ${targetY}`;
    labelX = (sourceX + targetX) / 2;
    labelY = (sourceY + targetY) / 2;
  }

  const displayLabel = edgeData?.label || (typeof label === "string" ? label : undefined);

  return (
    <>
      <BaseEdge
        id={id}
        path={edgePath}
        markerEnd={markerEnd}
        style={{
          strokeWidth: 2,
          stroke: "#64748b",
          ...style,
        }}
      />
      {displayLabel && (
        <EdgeLabelRenderer>
          <div
            style={{
              position: "absolute",
              transform: `translate(-50%, -50%) translate(${labelX}px, ${labelY}px)`,
              fontSize: 12,
              fontFamily: "Arial, sans-serif",
              fontWeight: 600,
              color: "#FFFFFF", // White text for high contrast
              background: "#2563eb", // Blue background for visibility
              padding: "4px 8px",
              borderRadius: 6,
              border: "1px solid #1d4ed8", // Darker blue border for definition
              boxShadow: "0 2px 4px rgba(0, 0, 0, 0.15)", // Shadow for depth
              textShadow: "0 1px 2px rgba(0, 0, 0, 0.2)", // Text shadow for readability
              pointerEvents: "all",
              // Merge with labelStyle if provided, but keep our high-contrast defaults
              ...(typeof labelStyle === "object" && labelStyle ? labelStyle : {}),
            }}
            className="nodrag nopan"
          >
            {displayLabel}
          </div>
        </EdgeLabelRenderer>
      )}
    </>
  );
}

export default memo(SplineEdge);
