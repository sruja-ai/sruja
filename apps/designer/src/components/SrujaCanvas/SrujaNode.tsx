import { memo } from 'react';
import { Handle, Position, type NodeProps, type Node } from '@xyflow/react';
import { Tooltip } from '@mantine/core';
import { Building2, User, Box, Package, Database, MessageSquare, Component } from 'lucide-react';
import { getNodeColors } from '../../utils/colorScheme';
import { useTheme } from '@sruja/ui';
import type { C4Node } from './types';
import '../Nodes/nodes.css';

// SVG Path Helpers derived from LikeC4
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

const NodeIcon = ({ kind }: { kind: string }) => {
    switch (kind) {
        case 'person':
        case 'actor':
            return <User size={20} />;
        case 'system':
            return <Building2 size={20} />;
        case 'container':
            return <Box size={20} />;
        case 'component':
            return <Component size={20} />;
        case 'datastore':
            return <Database size={20} />;
        case 'queue':
            return <MessageSquare size={20} />;
        case 'external':
            return <Package size={20} />;
        default:
            return <Box size={20} />;
    }
};

export const SrujaNode = memo(({ data, selected, width, height }: NodeProps<Node<C4Node>>) => {
    const { kind, title, technology, description, metadata } = data;
    
    // Ensure title is always a string (fallback to id or 'Untitled')
    const displayTitle = title || data.id || 'Untitled';
    const isExternal = metadata?.tags?.includes('external') || kind === 'external' || false;
    const colors = getNodeColors(kind === 'external' ? 'system' : kind, isExternal);
    
    // Use shared UI theme to determine if we're in light mode
    const { mode } = useTheme();
    const isDark = mode === 'dark' || (mode === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);
    const isPerson = kind === 'person' || kind === 'actor';
    const isOutlineMode = colors.bg === 'transparent';

    // Default dimensions if React Flow hasn't measured yet, though typically it does.
    const w = width ?? 200;
    const h = height ?? 150;

    const renderShape = () => {
        // For outline mode (transparent bg), use border color for fills
        // For filled mode, use bg color
        const fillColor = colors.bg === 'transparent' ? colors.border : colors.bg;
        const style = {
            stroke: colors.border,
            fill: fillColor,
            strokeWidth: 2
        };

        switch (kind) {
            case 'person':
            case 'actor': {
                // Center the person icon horizontally, position at bottom
                const iconX = (w - PersonIconSize.width) / 2;
                const iconY = h - PersonIconSize.height;
                return (
                    <svg width={w} height={h} style={{ overflow: 'visible' }}>
                        <rect width={w} height={h} rx={6} fill="transparent" stroke="none" />
                        <svg
                            x={iconX}
                            y={iconY}
                            width={PersonIconSize.width}
                            height={PersonIconSize.height}
                            viewBox={`0 0 ${PersonIconSize.width} ${PersonIconSize.height}`}
                        >
                            <path d={PersonIconPath} fill={style.fill} stroke={style.stroke} strokeWidth={style.strokeWidth} />
                        </svg>
                    </svg>
                );
            }
            case 'queue': {
                const { path, rx, ry } = queueSVGPath(w, h);
                return (
                    <svg width={w} height={h} style={{ overflow: 'visible' }}>
                        <path d={path} fill={style.fill} stroke={style.stroke} strokeWidth={style.strokeWidth} />
                        <ellipse cx={rx} cy={ry} rx={rx} ry={ry - 0.75} fill="none" stroke={style.stroke} strokeWidth={style.strokeWidth} />
                    </svg>
                );
            }
            case 'datastore': {
                const { path, rx, ry } = cylinderSVGPath(w, h);
                return (
                    <svg width={w} height={h} style={{ overflow: 'visible' }}>
                        <path d={path} fill={style.fill} stroke={style.stroke} strokeWidth={style.strokeWidth} />
                        <ellipse cx={rx} cy={ry} rx={rx - 0.75} ry={ry} fill="none" stroke={style.stroke} strokeWidth={style.strokeWidth} />
                    </svg>
                );
            }
            case 'mobile': {
                return (
                    <svg width={w} height={h}>
                        <rect width={w} height={h} rx={6} fill="transparent" stroke={style.stroke} strokeWidth={style.strokeWidth} />
                        <g fill={style.fill}>
                            <circle cx={17} cy={h / 2} r={12} fill={style.stroke} />
                            <rect x={33} y={12} width={w - 44} height={h - 24} rx={5} fill={style.stroke} fillOpacity={0.1} />
                        </g>
                    </svg>
                );
            }
            case 'webapp':
            case 'browser': // assuming browser view
                return (
                    <svg width={w} height={h}>
                        <rect width={w} height={h} rx={6} fill={style.fill} stroke={style.stroke} strokeWidth={style.strokeWidth} />
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
                        className={`c4-node ${kind}-node ${selected ? 'selected' : ''} ${isExternal ? 'external' : ''}`}
                        style={{
                            backgroundColor: colors.bg === 'transparent' ? undefined : colors.bg,
                            borderColor: colors.border,
                            color: colors.text,
                            width: '100%',
                            height: '100%',
                            display: 'flex',
                            flexDirection: 'column',
                            boxSizing: 'border-box'
                        }}
                    >
                        {/* Content rendered below */}
                    </div>
                );
        }
    };

    // For SVG shapes, we overlay content. For div shapes, content is inside.
    const isSvgShape = ['person', 'actor', 'queue', 'datastore', 'mobile', 'browser', 'webapp'].includes(kind);

    return (
        <>
            {/* All target handles (incoming edges) */}
            <Handle type="target" position={Position.Top} id="target-top" className="c4-handle" style={{ opacity: 0 }} />
            <Handle type="target" position={Position.Right} id="target-right" className="c4-handle" style={{ opacity: 0 }} />
            <Handle type="target" position={Position.Bottom} id="target-bottom" className="c4-handle" style={{ opacity: 0 }} />
            <Handle type="target" position={Position.Left} id="target-left" className="c4-handle" style={{ opacity: 0 }} />

            <Tooltip
                label={
                    <div style={{ maxWidth: 300 }}>
                        <div style={{ fontWeight: 600, marginBottom: 4 }}>{title}</div>
                        {technology && <div style={{ fontSize: 12, opacity: 0.8, marginBottom: 4 }}>[{technology}]</div>}
                        <div style={{ fontSize: 12 }}>{description}</div>
                    </div>
                }
                multiline
                withArrow
                withinPortal
                disabled={!description && !technology}
            >
                <div style={{ width: '100%', height: '100%', position: 'relative' }}>
                    {/* SVG Background layer - for person nodes, icon is rendered here */}
                    {isSvgShape && (
                        <div style={{ position: 'absolute', inset: 0, zIndex: 0, pointerEvents: 'none' }}>
                            {renderShape()}
                        </div>
                    )}

                    {/* Content wrapper - positioned above SVG */}
                    <div style={{
                        position: isSvgShape ? 'absolute' : 'static',
                        inset: 0,
                        zIndex: 5, // Above SVG background
                        padding: isSvgShape ? '16px' : 0,
                        width: '100%',
                        height: '100%',
                        display: 'flex',
                        flexDirection: 'column',
                        pointerEvents: 'auto', // Ensure content is interactive
                    }}>
                        {/* If not SVG, render the div container which wraps content */}
                        {!isSvgShape ? renderShape() : null}
                    </div>

                    {/* Re-implementing simplified content rendering to fix the nesting mess above */}
                    <div
                        className={`c4-node-content-wrapper ${kind}-node ${selected ? 'selected' : ''} ${isExternal ? 'external' : ''}`}
                        style={{
                            // For rectangles, we use the class styles. For SVGs, we use transparent background so SVG shows through
                            backgroundColor: isSvgShape ? 'transparent' : (colors.bg === 'transparent' ? undefined : colors.bg),
                            borderColor: isSvgShape ? 'transparent' : colors.border,
                            borderWidth: isSvgShape ? 0 : 1,
                            borderStyle: isSvgShape ? 'none' : 'solid',
                            color: colors.text,
                            width: '100%',
                            height: '100%',
                            position: 'absolute',
                            top: 0,
                            left: 0,
                            display: 'flex',
                            flexDirection: 'column',
                            boxSizing: 'border-box',
                            padding: isSvgShape && (kind === 'person' || kind === 'actor') ? '8px 12px' : '12px',
                            zIndex: 10, // Ensure text is above SVG icon
                            overflow: 'visible', // Allow text to be visible (containment handled by child elements)
                            pointerEvents: 'auto', // Ensure text is clickable
                        }}
                    >
                        {/* SVG Background layer - only render if not person (person icon is in outer SVG) */}
                        {isSvgShape && !(kind === 'person' || kind === 'actor') && (
                            <div style={{ position: 'absolute', inset: 0, zIndex: -1, pointerEvents: 'none' }}>
                                {renderShape()}
                            </div>
                        )}

                        <div 
                            className="node-header" 
                            style={{ 
                                flex: 1, 
                                display: 'flex', 
                                flexDirection: 'column', 
                                alignItems: 'center', 
                                justifyContent: kind === 'person' || kind === 'actor' ? 'flex-start' : 'center',
                                minWidth: 0, // Allow flex shrinking
                                width: '100%',
                                overflow: 'visible', // Allow text to be visible
                                paddingTop: kind === 'person' || kind === 'actor' ? '8px' : undefined,
                                // For person nodes, limit height to stay above icon (icon is ~120px tall at bottom)
                                maxHeight: kind === 'person' || kind === 'actor' ? `calc(100% - ${PersonIconSize.height + 16}px)` : '100%',
                            }}
                        >
                            {/* For person nodes, don't show icon since SVG has person icon */}
                            {!(kind === 'person' || kind === 'actor') && (
                                <div className="node-icon" aria-hidden="true" style={{ marginBottom: 8, color: colors.text }}>
                                    <NodeIcon kind={kind} />
                                </div>
                            )}

                            <div 
                                className="node-content" 
                                style={{ 
                                    textAlign: 'center',
                                    width: '100%',
                                    minWidth: 0, // Allow flex shrinking
                                    overflow: (kind === 'person' || kind === 'actor') ? 'visible' : 'hidden', // Allow text to be visible for person nodes
                                    // For person nodes, center text above the icon (no width constraint needed)
                                    maxWidth: '100%',
                                    // For person nodes in outline mode, add background for text visibility
                                    backgroundColor: (kind === 'person' || kind === 'actor') && colors.bg === 'transparent' 
                                        ? 'rgba(255, 255, 255, 1)' 
                                        : 'transparent',
                                    borderRadius: (kind === 'person' || kind === 'actor') && colors.bg === 'transparent' 
                                        ? '6px' 
                                        : undefined,
                                    padding: (kind === 'person' || kind === 'actor') && colors.bg === 'transparent' 
                                        ? '6px 10px' 
                                        : undefined,
                                    boxShadow: (kind === 'person' || kind === 'actor') && colors.bg === 'transparent' 
                                        ? '0 2px 8px rgba(0, 0, 0, 0.15)' 
                                        : undefined,
                                    // Ensure text is visible
                                    position: 'relative',
                                    zIndex: 20, // High z-index to ensure text is above everything
                                }}
                            >
                                <div 
                                    className={kind === 'person' || kind === 'actor' ? undefined : 'node-label'} 
                                    style={{ 
                                        fontWeight: 'bold',
                                        wordBreak: 'break-word',
                                        overflowWrap: 'break-word',
                                        maxWidth: '100%',
                                        fontSize: kind === 'person' || kind === 'actor' ? '0.9em' : undefined,
                                        lineHeight: 1.2,
                                        // For person nodes, use a contrasting color that works on both light background and navy icon
                                        // In light theme + outline mode: black text on white background box
                                        // In dark theme + outline mode: light text with shadow
                                        // In filled mode: use theme-appropriate text color
                                        color: isPerson && isOutlineMode
                                            ? (isDark ? '#e2e8f0' : '#000000') // Light text in dark, black in light
                                            : (isPerson && !isDark && !isOutlineMode
                                                ? '#FFFFFF' // White text on navy blue background in filled mode
                                                : colors.text),
                                        // For person nodes in outline mode, add text shadow for visibility
                                        textShadow: isPerson && isOutlineMode
                                            ? (isDark 
                                                ? '0 0 4px rgba(0, 0, 0, 0.8), 0 1px 3px rgba(0, 0, 0, 0.5)' // Dark shadow in dark theme
                                                : '0 0 4px rgba(255, 255, 255, 1), 0 0 8px rgba(255, 255, 255, 0.8), 0 1px 3px rgba(0, 0, 0, 0.5)') // White shadow in light theme
                                            : undefined,
                                    }}
                                >
                                    {displayTitle}
                                </div>
                                {technology && (
                                    <div 
                                        style={{ 
                                            fontSize: '0.75em', 
                                            opacity: 0.9,
                                            wordBreak: 'break-word',
                                            overflowWrap: 'break-word',
                                            maxWidth: '100%',
                                            marginTop: 2,
                                            lineHeight: 1.2,
                                            // For person nodes, use a contrasting color (same logic as title)
                                            color: isPerson && isOutlineMode
                                                ? (isDark ? '#e2e8f0' : '#000000')
                                                : (isPerson && !isDark && !isOutlineMode
                                                    ? '#FFFFFF'
                                                    : colors.text),
                                            // For person nodes in outline mode, add text shadow for visibility
                                            textShadow: isPerson && isOutlineMode
                                                ? (isDark 
                                                    ? '0 0 4px rgba(0, 0, 0, 0.8), 0 1px 3px rgba(0, 0, 0, 0.5)'
                                                    : '0 0 4px rgba(255, 255, 255, 1), 0 0 8px rgba(255, 255, 255, 0.8), 0 1px 3px rgba(0, 0, 0, 0.5)')
                                                : undefined,
                                        }}
                                    >
                                        [{technology}]
                                    </div>
                                )}
                                {description && (
                                    <div 
                                        className={kind === 'person' || kind === 'actor' ? undefined : 'node-description'} 
                                        style={{ 
                                            fontSize: '0.75em', 
                                            marginTop: 4,
                                            wordBreak: 'break-word',
                                            overflowWrap: 'break-word',
                                            maxWidth: '100%',
                                            display: '-webkit-box',
                                            WebkitLineClamp: kind === 'person' || kind === 'actor' ? 1 : 2,
                                            WebkitBoxOrient: 'vertical',
                                            overflow: 'hidden',
                                            textOverflow: 'ellipsis',
                                            lineHeight: 1.3,
                                            // For person nodes, use a contrasting color (same logic as title)
                                            color: isPerson && isOutlineMode
                                                ? (isDark ? '#e2e8f0' : '#000000')
                                                : (isPerson && !isDark && !isOutlineMode
                                                    ? '#FFFFFF'
                                                    : colors.text),
                                            // For person nodes in outline mode, add text shadow for visibility
                                            textShadow: isPerson && isOutlineMode
                                                ? (isDark 
                                                    ? '0 0 4px rgba(0, 0, 0, 0.8), 0 1px 3px rgba(0, 0, 0, 0.5)'
                                                    : '0 0 4px rgba(255, 255, 255, 1), 0 0 8px rgba(255, 255, 255, 0.8), 0 1px 3px rgba(0, 0, 0, 0.5)')
                                                : undefined,
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
            <Handle type="source" position={Position.Top} id="source-top" className="c4-handle" style={{ opacity: 0 }} />
            <Handle type="source" position={Position.Right} id="source-right" className="c4-handle" style={{ opacity: 0 }} />
            <Handle type="source" position={Position.Bottom} id="source-bottom" className="c4-handle" style={{ opacity: 0 }} />
            <Handle type="source" position={Position.Left} id="source-left" className="c4-handle" style={{ opacity: 0 }} />
        </>
    );
});

SrujaNode.displayName = 'SrujaNode';
