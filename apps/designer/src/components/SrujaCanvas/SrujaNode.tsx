import { memo } from "react";
import { Handle, Position, type NodeProps, type Node } from "@xyflow/react";
import { Tooltip } from "@mantine/core";
import {
  Building2,
  User,
  Box,
  Package,
  Database,
  MessageSquare,
  Component,
  AlertTriangle,
  XCircle,
} from "lucide-react";
import { getNodeColors } from "../../utils/colorScheme";
import { useTheme } from "@sruja/ui";
import type { C4Node } from "./types";
import "../Nodes/nodes.css";

// SVG Path Helpers
function cylinderSVGPath(diameter: number, height: number, tilt = 0.065) {
  const radius = Math.round(diameter / 2);
  const rx = radius;
  const ry = Math.round(tilt * radius);
  const tiltAdjustedHeight = height - 2 * ry;

  const path = `M ${diameter},${ry} a ${rx},${ry} 0,0,0 ${-diameter} 0 l 0,${tiltAdjustedHeight} a ${rx},${ry} 0,0,0 ${diameter} 0 l 0,${-tiltAdjustedHeight} z`;
  return { path, rx, ry };
}

function queueSVGPath(width: number, height: number, tilt = 0.185) {
  const diameter = height;
  const radius = Math.round(diameter / 2);
  const ry = radius;
  const rx = Math.round((diameter / 2) * tilt);
  const tiltAdjustedWidth = width - 2 * rx;

  const path = `M ${rx},0 a ${rx},${ry} 0,0,0 0 ${diameter} l ${tiltAdjustedWidth},0 a ${rx},${ry} 0,0,0 0 ${-diameter} z`;
  return { path, rx, ry };
}

const PersonIconPath = `M57.9197 0C10.9124 0 33.5766 54.75 33.5766 54.75C38.6131 62.25 45.3285 60.75 45.3285 66C45.3285 70.5 39.4526 72 33.5766 72.75C24.3431 72.75 15.9489 71.25 7.55474 84.75C2.51825 93 0 120 0 120H115C115 120 112.482 93 108.285 84.75C99.8905 70.5 91.4963 72.75 82.2628 72C76.3869 71.25 70.5109 69.75 70.5109 65.25C70.5109 60.75 77.2263 62.25 82.2628 54C82.2628 54.75 104.927 0 57.9197 0V0Z`;
const PersonIconSize = { width: 115, height: 120 };

/**
 * Convert hex color to rgba with opacity
 */
function hexToRgba(hex: string, opacity: number): string {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  return `rgba(${r}, ${g}, ${b}, ${opacity})`;
}

const NodeIcon = ({ kind }: { kind: string }) => {
  switch (kind) {
    case "person":
    case "actor":
      return <User size={20} />;
    case "system":
      return <Building2 size={20} />;
    case "container":
      return <Box size={20} />;
    case "component":
      return <Component size={20} />;
    case "datastore":
      return <Database size={20} />;
    case "queue":
      return <MessageSquare size={20} />;
    case "external":
      return <Package size={20} />;
    default:
      return <Box size={20} />;
  }
};

export const SrujaNode = memo(({ data, selected, width, height }: NodeProps<Node<C4Node>>) => {
  const { kind, title, technology, description, metadata } = data;

  // Ensure title is always a string (fallback to id or 'Untitled')
  const displayTitle = title || data.id || "Untitled";
  const isExternal = metadata?.tags?.includes("external") || kind === "external" || false;
  const colors = getNodeColors(kind === "external" ? "system" : kind, isExternal);

  // Use shared UI theme to determine if we're in light mode and get theme colors
  const { mode, theme: uiTheme } = useTheme();
  const isDark =
    mode === "dark" ||
    (mode === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);
  const isPerson = kind === "person" || kind === "actor";
  const isOutlineMode = colors.bg === "transparent";

  // Default dimensions if React Flow hasn't measured yet, though typically it does.
  const w = width ?? 200;
  const h = height ?? 150;

  const renderShape = () => {
    // For outline mode (transparent bg), use border color for fills
    // For filled mode, use bg color
    const fillColor = colors.bg === "transparent" ? colors.border : colors.bg;
    const style = {
      stroke: colors.border,
      fill: fillColor,
      strokeWidth: 2,
    };

    switch (kind) {
      case "person":
      case "actor": {
        // Center the person icon horizontally, position at top
        const iconX = (w - PersonIconSize.width) / 2;
        const iconY = 0;
        return (
          <svg width={w} height={h} style={{ overflow: "visible" }}>
            <rect width={w} height={h} rx={6} fill="transparent" stroke="none" />
            <svg
              x={iconX}
              y={iconY}
              width={PersonIconSize.width}
              height={PersonIconSize.height}
              viewBox={`0 0 ${PersonIconSize.width} ${PersonIconSize.height}`}
            >
              <path
                d={PersonIconPath}
                fill={style.fill}
                stroke={style.stroke}
                strokeWidth={style.strokeWidth}
              />
            </svg>
          </svg>
        );
      }
      case "queue": {
        const { path, rx, ry } = queueSVGPath(w, h);
        return (
          <svg width={w} height={h} style={{ overflow: "visible" }}>
            <path
              d={path}
              fill={style.fill}
              stroke={style.stroke}
              strokeWidth={style.strokeWidth}
            />
            <ellipse
              cx={rx}
              cy={ry}
              rx={rx}
              ry={ry - 0.75}
              fill="none"
              stroke={style.stroke}
              strokeWidth={style.strokeWidth}
            />
          </svg>
        );
      }
      case "datastore": {
        const { path, rx, ry } = cylinderSVGPath(w, h);
        return (
          <svg width={w} height={h} style={{ overflow: "visible" }}>
            <path
              d={path}
              fill={style.fill}
              stroke={style.stroke}
              strokeWidth={style.strokeWidth}
            />
            <ellipse
              cx={rx}
              cy={ry}
              rx={rx - 0.75}
              ry={ry}
              fill="none"
              stroke={style.stroke}
              strokeWidth={style.strokeWidth}
            />
          </svg>
        );
      }
      case "mobile": {
        return (
          <svg width={w} height={h}>
            <rect
              width={w}
              height={h}
              rx={6}
              fill="transparent"
              stroke={style.stroke}
              strokeWidth={style.strokeWidth}
            />
            <g fill={style.fill}>
              <circle cx={17} cy={h / 2} r={12} fill={style.stroke} />
              <rect
                x={33}
                y={12}
                width={w - 44}
                height={h - 24}
                rx={5}
                fill={style.stroke}
                fillOpacity={0.1}
              />
            </g>
          </svg>
        );
      }
      case "webapp":
      case "browser": // assuming browser view
        return (
          <svg width={w} height={h}>
            <rect
              width={w}
              height={h}
              rx={6}
              fill={style.fill}
              stroke={style.stroke}
              strokeWidth={style.strokeWidth}
            />
            <g fill={style.stroke}>
              <circle cx={16} cy={17} r={4} />
              <circle cx={32} cy={17} r={4} />
              <circle cx={48} cy={17} r={4} />
              <rect x={60} y={12} width={w - 70} height={10} rx={2} />
              <line x1={0} y1={30} x2={w} y2={30} stroke={style.stroke} strokeWidth={1} />
            </g>
          </svg>
        );
      default:
        // Rectangle shape for system, container, component
        return (
          <div
            className={`c4-node ${kind}-node ${selected ? "selected" : ""} ${isExternal ? "external" : ""}`}
            style={{
              backgroundColor: colors.bg === "transparent" ? undefined : colors.bg,
              borderColor: colors.border,
              color: colors.text,
              width: "100%",
              height: "100%",
              display: "flex",
              flexDirection: "column",
              boxSizing: "border-box",
            }}
          >
            {/* Content rendered below */}
          </div>
        );
    }
  };

  // For SVG shapes, we overlay content. For div shapes, content is inside.
  const isSvgShape = [
    "person",
    "actor",
    "queue",
    "datastore",
    "mobile",
    "browser",
    "webapp",
  ].includes(kind);

  // Compute capacity badge styles if needed
  const capacity = data._capacity as any;
  const hasCapacityBadge = capacity && capacity.replicas;
  const isHighLoad = hasCapacityBadge && capacity.load > 150;
  const capacityBadgeStyle = hasCapacityBadge
    ? {
        position: "absolute" as const,
        bottom: -10,
        right: 10,
        zIndex: 100,
        backgroundColor: isHighLoad ? "#fee2e2" : "#eff6ff",
        borderRadius: "12px",
        padding: "2px 8px",
        border: `1px solid ${isHighLoad ? "#ef4444" : "#3b82f6"}`,
        boxShadow: "0 2px 4px rgba(0,0,0,0.1)",
        fontSize: 10,
        fontWeight: 700,
        color: isHighLoad ? "#b91c1c" : "#1d4ed8",
        display: "flex",
        alignItems: "center",
        gap: "4px",
      }
    : null;

  return (
    <>
      {/* All target handles (incoming edges) */}
      <Handle
        type="target"
        position={Position.Top}
        id="target-top"
        className="c4-handle"
        style={{ opacity: 0 }}
      />
      <Handle
        type="target"
        position={Position.Right}
        id="target-right"
        className="c4-handle"
        style={{ opacity: 0 }}
      />
      <Handle
        type="target"
        position={Position.Bottom}
        id="target-bottom"
        className="c4-handle"
        style={{ opacity: 0 }}
      />
      <Handle
        type="target"
        position={Position.Left}
        id="target-left"
        className="c4-handle"
        style={{ opacity: 0 }}
      />

      <Tooltip
        label={
          <div style={{ maxWidth: 300 }}>
            <div style={{ fontWeight: 600, marginBottom: 4 }}>{title}</div>
            {technology && (
              <div style={{ fontSize: 12, opacity: 0.8, marginBottom: 4 }}>[{technology}]</div>
            )}
            <div style={{ fontSize: 12 }}>{description}</div>
          </div>
        }
        multiline
        withArrow
        withinPortal
        disabled={!description && !technology}
      >
        <div style={{ width: "100%", height: "100%", position: "relative" }}>
          {/* SVG Background layer - for person nodes, icon is rendered here */}
          {isSvgShape && (
            <div style={{ position: "absolute", inset: 0, zIndex: 0, pointerEvents: "none" }}>
              {renderShape()}
            </div>
          )}

          {/* Content wrapper - positioned above SVG */}
          <div
            style={{
              position: isSvgShape ? "absolute" : "static",
              inset: 0,
              zIndex: 5, // Above SVG background
              padding: isSvgShape ? "16px" : 0,
              width: "100%",
              height: "100%",
              display: "flex",
              flexDirection: "column",
              pointerEvents: "auto", // Ensure content is interactive
            }}
          >
            {/* If not SVG, render the div container which wraps content */}
            {!isSvgShape ? renderShape() : null}
          </div>

          {/* Chaos Mode Badges */}
          {data._chaos && (data._chaos as any).isFailed && (
            <div
              style={{
                position: "absolute",
                top: -10,
                right: -10,
                zIndex: 100,
                backgroundColor: "#fee2e2",
                borderRadius: "50%",
                padding: 4,
                border: "2px solid #ef4444",
              }}
            >
              <XCircle size={20} color="#ef4444" fill="#fee2e2" />
            </div>
          )}
          {data._chaos && (data._chaos as any).isImpacted && (
            <div
              style={{
                position: "absolute",
                top: -10,
                right: -10,
                zIndex: 100,
                backgroundColor: "#fef3c7",
                borderRadius: "50%",
                padding: 4,
                border: "2px solid #f59e0b",
              }}
            >
              <AlertTriangle size={20} color="#d97706" fill="#fef3c7" />
            </div>
          )}

          {/* Capacity Mode Badges */}
          {hasCapacityBadge && capacityBadgeStyle && (
            <div style={capacityBadgeStyle}>
              <Box size={10} />
              {capacity.replicas} pods
            </div>
          )}

          {/* Re-implementing simplified content rendering to fix the nesting mess above */}
          <div
            className={`c4-node-content-wrapper ${kind}-node ${selected ? "selected" : ""} ${isExternal ? "external" : ""}`}
            style={{
              // For rectangles, we use the class styles. For SVGs, we use transparent background so SVG shows through
              backgroundColor: isSvgShape
                ? "transparent"
                : colors.bg === "transparent"
                  ? undefined
                  : colors.bg,
              borderColor: isSvgShape ? "transparent" : colors.border,
              borderWidth: isSvgShape ? 0 : 1,
              borderStyle: isSvgShape ? "none" : "solid",
              color: colors.text,
              width: "100%",
              height: "100%",
              position: "absolute",
              top: 0,
              left: 0,
              display: "flex",
              flexDirection: "column",
              boxSizing: "border-box",
              padding: isSvgShape && (kind === "person" || kind === "actor") ? "8px 12px" : "12px",
              zIndex: 10, // Ensure text is above SVG icon
              overflow: "hidden", // Constrain content to node boundaries to prevent text overflow
              pointerEvents: "auto", // Ensure text is clickable
            }}
          >
            {/* SVG Background layer - only render if not person (person icon is in outer SVG) */}
            {isSvgShape && !(kind === "person" || kind === "actor") && (
              <div style={{ position: "absolute", inset: 0, zIndex: -1, pointerEvents: "none" }}>
                {renderShape()}
              </div>
            )}

            <div
              className="node-header"
              style={{
                flex: 1,
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                justifyContent: kind === "person" || kind === "actor" ? "flex-end" : "center",
                minWidth: 0, // Allow flex shrinking
                width: "100%",
                overflow: "hidden", // Constrain content to prevent overflow
                // For person nodes, add padding at top to account for icon, and position label below
                paddingTop:
                  kind === "person" || kind === "actor"
                    ? `${PersonIconSize.height + 12}px`
                    : undefined,
                paddingBottom: kind === "person" || kind === "actor" ? "8px" : undefined,
              }}
            >
              {/* For person nodes, don't show icon since SVG has person icon */}
              {!(kind === "person" || kind === "actor") && (
                <div
                  className="node-icon"
                  aria-hidden="true"
                  style={{ marginBottom: 8, color: colors.text }}
                >
                  <NodeIcon kind={kind} />
                </div>
              )}

              <div
                className="node-content"
                style={{
                  textAlign: "center",
                  width: "100%",
                  minWidth: 0, // Allow flex shrinking
                  overflow: kind === "person" || kind === "actor" ? "visible" : "hidden", // Allow text to be visible for person nodes
                  // Constrain text width to be roughly the same as icon (slightly bigger)
                  // Person icon is 115px wide, so allow ~130px for text (slightly bigger)
                  // System nodes: allow wider text to match more compact node width
                  // Other nodes have icons ~20-24px, but we want text to fill the node width
                  maxWidth:
                    kind === "person" || kind === "actor"
                      ? `${PersonIconSize.width + 15}px` // Slightly bigger than person icon (115px + 15 = 130px)
                      : "calc(100% - 24px)", // Use available width minus padding (12px each side)
                  margin: "0 auto", // Center the constrained width
                  // For person nodes: always add background for text visibility
                  // In outline mode: white background for dark text
                  // In filled mode: navy blue background for white text (match the icon)
                  backgroundColor:
                    kind === "person" || kind === "actor"
                      ? colors.bg === "transparent"
                        ? uiTheme.background // Outline mode: theme background for dark text
                        : colors.bg // Filled mode: navy blue background for white text
                      : "transparent",
                  borderRadius: kind === "person" || kind === "actor" ? "6px" : undefined,
                  padding: kind === "person" || kind === "actor" ? "6px 10px" : undefined,
                  boxShadow:
                    (kind === "person" || kind === "actor") && colors.bg === "transparent"
                      ? "0 2px 8px rgba(0, 0, 0, 0.15)"
                      : undefined,
                  // Ensure text is visible
                  position: "relative",
                  zIndex: 20, // High z-index to ensure text is above everything
                }}
              >
                <div
                  className={kind === "person" || kind === "actor" ? undefined : "node-label"}
                  style={{
                    fontWeight: "bold",
                    wordBreak: "break-word",
                    overflowWrap: "break-word",
                    maxWidth: "100%",
                    fontSize: kind === "person" || kind === "actor" ? "0.9em" : undefined,
                    lineHeight: 1.2,
                    // For person nodes, use a contrasting color that works on both light background and navy icon
                    // In light theme + outline mode: primary text on theme background
                    // In dark theme + outline mode: primary text with shadow
                    // In filled mode: white text on navy blue background
                    color:
                      isPerson && isOutlineMode
                        ? uiTheme.text.primary // Theme-aware text color (dark in light theme, light in dark theme)
                        : isPerson && !isOutlineMode
                          ? "#FFFFFF" // White text on navy blue background in filled mode (both themes)
                          : colors.text,
                    // For person nodes in outline mode, add text shadow for visibility
                    textShadow:
                      isPerson && isOutlineMode
                        ? isDark
                          ? `0 0 4px ${hexToRgba(uiTheme.neutral[900], 0.8)}, 0 1px 3px ${hexToRgba(uiTheme.neutral[900], 0.5)}` // Dark shadow in dark theme using theme colors
                          : `0 1px 2px ${hexToRgba(uiTheme.neutral[900], 0.1)}` // Subtle shadow using theme neutral color
                        : undefined,
                  }}
                >
                  {displayTitle}
                </div>
                {technology && (
                  <div
                    style={{
                      fontSize: "0.75em",
                      opacity: 0.9,
                      wordBreak: "break-word",
                      overflowWrap: "break-word",
                      maxWidth: "100%",
                      marginTop: 2,
                      lineHeight: 1.2,
                      // For person nodes, use a contrasting color (same logic as title)
                      color:
                        isPerson && isOutlineMode
                          ? uiTheme.text.primary // Theme-aware text color
                          : isPerson && !isOutlineMode
                            ? "#FFFFFF" // White text in filled mode (both themes)
                            : colors.text,
                      // For person nodes in outline mode, add text shadow for visibility
                      textShadow:
                        isPerson && isOutlineMode
                          ? isDark
                            ? `0 0 4px ${hexToRgba(uiTheme.neutral[900], 0.8)}, 0 1px 3px ${hexToRgba(uiTheme.neutral[900], 0.5)}`
                            : `0 1px 2px ${hexToRgba(uiTheme.neutral[900], 0.1)}` // Subtle shadow using theme neutral color
                          : undefined,
                    }}
                  >
                    [{technology}]
                  </div>
                )}
                {/* Show descriptions for system nodes directly on diagram, hide for others (show in details panel) */}
                {description && kind === "system" && (
                  <div
                    className="node-description"
                    style={{
                      fontSize: "0.75em",
                      marginTop: 6,
                      wordBreak: "break-word",
                      overflowWrap: "break-word",
                      maxWidth: "100%",
                      // Show full description for systems (no truncation)
                      lineHeight: 1.4,
                      color: colors.text,
                      opacity: 0.85,
                    }}
                  >
                    {description}
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      </Tooltip>

      {/* All source handles (outgoing edges) */}
      <Handle
        type="source"
        position={Position.Top}
        id="source-top"
        className="c4-handle"
        style={{ opacity: 0 }}
      />
      <Handle
        type="source"
        position={Position.Right}
        id="source-right"
        className="c4-handle"
        style={{ opacity: 0 }}
      />
      <Handle
        type="source"
        position={Position.Bottom}
        id="source-bottom"
        className="c4-handle"
        style={{ opacity: 0 }}
      />
      <Handle
        type="source"
        position={Position.Left}
        id="source-left"
        className="c4-handle"
        style={{ opacity: 0 }}
      />
    </>
  );
});

SrujaNode.displayName = "SrujaNode";
