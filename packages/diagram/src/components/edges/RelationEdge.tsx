// packages/diagram/src/components/edges/RelationEdge.tsx
// Relation edge component with technology annotations for C4 diagrams
import type { EdgeProps } from '@xyflow/react';
import { BaseEdge, EdgeLabelRenderer, getSmoothStepPath, getBezierPath, getStraightPath } from '@xyflow/react';

type RouteType = 'direct' | 'orthogonal' | 'curved' | 'splines';

export function RelationEdge({
    id,
    sourceX,
    sourceY,
    targetX,
    targetY,
    sourcePosition,
    targetPosition,
    label,
    style,
    data,
}: EdgeProps) {
    // Get preferred route from data or default to orthogonal (smooth step)
    const preferredRoute = (data?.preferredRoute as RouteType) || 'orthogonal';

    // Calculate path based on preferred route type
    let edgePath: string;
    let labelX: number;
    let labelY: number;

    if (preferredRoute === 'direct') {
        // Straight line from source to target
        [edgePath, labelX, labelY] = getStraightPath({
            sourceX,
            sourceY,
            targetX,
            targetY,
        });
    } else if (preferredRoute === 'curved' || preferredRoute === 'splines') {
        // Bezier curve for smooth curved paths
        [edgePath, labelX, labelY] = getBezierPath({
            sourceX,
            sourceY,
            sourcePosition,
            targetX,
            targetY,
            targetPosition,
            curvature: 0.25,
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
            borderRadius: 8,
        });
    }

    const isActive = data?.isActive === true;
    const isDimmed = data?.isDimmed === true;
    const edgeLabel = label || (data?.label as string);
    const technology = data?.technology as string | undefined;
    const interaction = data?.interaction as 'sync' | 'async' | 'event' | undefined;
    const tags = (data?.tags as string[] | undefined) || [];
    const tagSet = new Set(tags.map((t) => t.toLowerCase()));

    // Determine interaction type from explicit field or tags
    const interactionType = interaction ||
        (tagSet.has('async') ? 'async' :
            tagSet.has('event') ? 'event' :
                'sync');

    // Interaction-based styling
    let strokeDasharray: string | undefined;
    let strokeColor = (style?.stroke as string) || '#64748b';

    if (interactionType === 'async') {
        strokeDasharray = '8,4';
    } else if (interactionType === 'event') {
        strokeDasharray = '2,2';
        strokeColor = '#8B5CF6';
    }

    const edgeStyle = {
        ...style,
        stroke: isActive ? '#10b981' : (isDimmed ? '#e5e7eb' : strokeColor),
        strokeWidth: isActive ? 3 : (isDimmed ? 1 : 2),
        opacity: isDimmed ? 0.3 : 1,
        strokeDasharray: strokeDasharray || (style?.strokeDasharray as string),
    };

    return (
        <>
            <BaseEdge
                id={id}
                path={edgePath}
                style={edgeStyle}
                markerEnd={isDimmed ? 'url(#arrow-dimmed)' : 'url(#arrow)'}
            />

            {(edgeLabel || technology) && (
                <EdgeLabelRenderer>
                    <div
                        style={{
                            position: 'absolute',
                            transform: `translate(-50%, -50%) translate(${labelX}px,${labelY}px)`,
                            display: 'flex',
                            flexDirection: 'column',
                            alignItems: 'center',
                            gap: 2,
                            pointerEvents: 'none',
                            opacity: isDimmed ? 0.3 : 1,
                            zIndex: 1000,
                        }}
                    >
                        {edgeLabel && (
                            <div
                                style={{
                                    fontSize: 11,
                                    fontWeight: 500,
                                    background: isActive ? '#ecfdf5' : 'rgba(255, 255, 255, 0.95)',
                                    color: isActive ? '#059669' : '#374151',
                                    padding: '3px 8px',
                                    borderRadius: 4,
                                    border: isActive ? '1px solid #10b981' : '1px solid #e5e7eb',
                                    boxShadow: '0 1px 3px rgba(0, 0, 0, 0.08)',
                                    whiteSpace: 'nowrap',
                                }}
                            >
                                {edgeLabel}
                            </div>
                        )}
                        {technology && (
                            <div
                                style={{
                                    fontSize: 9,
                                    fontWeight: 400,
                                    fontStyle: 'italic',
                                    background: 'rgba(255, 255, 255, 0.9)',
                                    color: '#6B7280',
                                    padding: '2px 6px',
                                    borderRadius: 3,
                                    border: '1px solid #D1D5DB',
                                    whiteSpace: 'nowrap',
                                }}
                            >
                                {technology}
                            </div>
                        )}
                    </div>
                </EdgeLabelRenderer>
            )}
        </>
    );
}
