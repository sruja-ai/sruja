import { useState, useEffect } from "react";
import {
    getLayoutMetrics,
    getHeatmapVisible,
} from "../types/windowGlobals";
import type { DiagramQualityMetrics } from "../utils/diagramQuality";

export function useLayoutMetrics() {
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    const [layoutMetrics, setLayoutMetrics] = useState<DiagramQualityMetrics | null>(null);
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    const [showHeatmap, setShowHeatmap] = useState(false);

    useEffect(() => {
        const checkMetrics = () => {
            const metrics = getLayoutMetrics();
            if (metrics) {
                setLayoutMetrics(metrics);
            }
            const heatmapVisible = getHeatmapVisible();
            if (typeof heatmapVisible === "boolean") {
                setShowHeatmap(heatmapVisible);
            }
        };
        checkMetrics();
        const interval = setInterval(checkMetrics, 500); // Check every 500ms
        return () => clearInterval(interval);
    }, []);

    return {
        layoutMetrics,
        showHeatmap
    };
}
