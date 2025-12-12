import { BaseEdge, EdgeLabelRenderer, getSmoothStepPath, type EdgeProps } from '@xyflow/react';

export default function PolylineEdge({
    sourceX,
    sourceY,
    targetX,
    targetY,
    sourcePosition,
    label,
    style = {},
    markerEnd,
    data,
}: EdgeProps) {
    let path = '';
    let usePoints = false;
    let labelX = (sourceX + targetX) / 2;
    let labelY = (sourceY + targetY) / 2;

    // Use layout-provided label position if available
    if (data?.labelPosition) {
        labelX = (data.labelPosition as { x: number; y: number }).x;
        labelY = (data.labelPosition as { x: number; y: number }).y;
    }

    if (data && data.points && Array.isArray(data.points) && data.points.length > 0) {
        const points = data.points as { x: number; y: number }[];
        const start = points[0];
        const end = points[points.length - 1];

        // Check if current source/target match the layout points
        // If user dragged the node, these will diverge
        const distS = Math.sqrt(Math.pow(start.x - sourceX, 2) + Math.pow(start.y - sourceY, 2));
        const distT = Math.sqrt(Math.pow(end.x - targetX, 2) + Math.pow(end.y - targetY, 2));

        if (distS < 20 && distT < 20) { // Tolerance of 20px
            usePoints = true;
            path = `M ${points[0].x} ${points[0].y}`;
            for (let i = 1; i < points.length; i++) {
                path += ` L ${points[i].x} ${points[i].y}`;
            }

            // Only calculate label position if not provided by layout
            if (!data?.labelPosition) {
                // Calculate label position from midpoint of the path
                const midIndex = Math.floor(points.length / 2);
                if (points.length > 1) {
                    if (midIndex > 0 && midIndex < points.length) {
                        labelX = points[midIndex].x;
                        labelY = points[midIndex].y;
                    } else {
                        // Use midpoint between first and last
                        labelX = (points[0].x + points[points.length - 1].x) / 2;
                        labelY = (points[0].y + points[points.length - 1].y) / 2;
                    }
                }
            }
        }
    }

    if (!usePoints) {
        const [smoothPath] = getSmoothStepPath({
            sourceX,
            sourceY,
            sourcePosition,
            targetX,
            targetY,
            borderRadius: 10
        });
        path = smoothPath;
        // Label position is already set to midpoint above
    }

    const isActive = data?.isActive === true;
    const isDimmed = data?.isDimmed === true;
    const edgeLabel = String(label || data?.label || '');
    const technology = data?.technology as string | undefined;
    const interaction = data?.interaction as 'sync' | 'async' | 'event' | undefined;
    const tags = (data?.tags as string[] | undefined) || [];
    const tagSet = new Set(tags.map((t) => t.toLowerCase()));

    // Determine interaction type from explicit field or tags
    const interactionType = interaction ||
        (tagSet.has('async') ? 'async' :
            tagSet.has('event') ? 'event' :
                'sync');

    // Apply interaction-based styling to edge
    let edgeStyle = { ...style };
    if (interactionType === 'async') {
        edgeStyle.strokeDasharray = '8,4';
    } else if (interactionType === 'event') {
        edgeStyle.strokeDasharray = '2,2';
        edgeStyle.stroke = edgeStyle.stroke || '#8B5CF6';
    }

    return (
        <>
            <BaseEdge path={path} markerEnd={markerEnd} style={edgeStyle} />
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
                            pointerEvents: 'all',
                            opacity: isDimmed ? 0.3 : 1,
                            transition: 'all 0.3s ease',
                            zIndex: 1000,
                            userSelect: 'none',
                        }}
                        className="nodrag nopan"
                    >
                        {edgeLabel && (
                            <div
                                style={{
                                    fontSize: 11,
                                    background: isActive ? '#ecfdf5' : 'white',
                                    color: isActive ? '#059669' : '#374151',
                                    padding: '2px 6px',
                                    borderRadius: 4,
                                    border: isActive ? '1px solid #10b981' : '1px solid #ddd',
                                    boxShadow: isActive ? '0 2px 8px rgba(16, 185, 129, 0.2)' : '0 1px 2px rgba(0, 0, 0, 0.1)',
                                    whiteSpace: 'nowrap',
                                }}
                            >
                                {String(edgeLabel)}
                            </div>
                        )}
                        {technology && (
                            <div
                                style={{
                                    fontSize: 9,
                                    fontWeight: 400,
                                    fontStyle: 'italic',
                                    background: isActive ? '#ecfdf5' : 'rgba(255, 255, 255, 0.9)',
                                    color: isActive ? '#059669' : '#6B7280',
                                    padding: '2px 6px',
                                    borderRadius: 3,
                                    border: isActive ? '1px solid #10b981' : '1px solid #D1D5DB',
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
