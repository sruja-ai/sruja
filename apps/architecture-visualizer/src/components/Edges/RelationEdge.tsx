import type { EdgeProps } from '@xyflow/react';
import { BaseEdge, EdgeLabelRenderer, getBezierPath } from '@xyflow/react';
import { EDGE_STYLES } from '../../utils/colorScheme';

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
}: EdgeProps) {
    const [edgePath, labelX, labelY] = getBezierPath({
        sourceX,
        sourceY,
        sourcePosition,
        targetX,
        targetY,
        targetPosition,
    });

    const edgeStyle = selected ? EDGE_STYLES.selected : EDGE_STYLES.default;

    return (
        <>
            <BaseEdge
                id={id}
                path={edgePath}
                style={{
                    ...edgeStyle,
                    ...style,
                }}
                markerEnd="url(#arrow)"
            />
            {label && (
                <EdgeLabelRenderer>
                    <div
                        style={{
                            position: 'absolute',
                            transform: `translate(-50%, -50%) translate(${labelX}px,${labelY}px)`,
                            fontSize: 11,
                            background: 'white',
                            padding: '2px 6px',
                            borderRadius: 4,
                            border: '1px solid #ddd',
                            pointerEvents: 'all',
                        }}
                        className="nodrag nopan"
                    >
                        {label}
                    </div>
                </EdgeLabelRenderer>
            )}
        </>
    );
}
