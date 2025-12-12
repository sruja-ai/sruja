/**
 * LaneSeparator Component
 * 
 * Renders horizontal lane separators with labels for L3 Component diagrams.
 * These are decorative nodes that show the architectural layer names.
 */

import './LaneSeparator.css';

interface LaneSeparatorProps {
    data: {
        label: string;
        laneName: string;
        laneWidth: number;
        [key: string]: unknown;
    };
}

export function LaneSeparator({ data }: LaneSeparatorProps) {
    return (
        <div className="lane-separator" style={{ width: data.laneWidth }}>
            <div className="lane-label">
                {data.label}
            </div>
            <div className="lane-line" />
        </div>
    );
}

