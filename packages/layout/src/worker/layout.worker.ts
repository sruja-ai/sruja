import { createLayoutEngine } from "../core/engine";
import type { WorkerMessage } from "./types";

const ctx: Worker = self as any;

ctx.onmessage = async (event: MessageEvent<WorkerMessage>) => {
    const { type, payload } = event.data;

    if (type === "LAYOUT_REQUEST") {
        // Cast payload as it is guaranteed to be LayoutRequest by the type guard
        const request = payload as any; // Using any to avoid strict type narrowing issues with union

        try {
            const engine = createLayoutEngine(request.options);

            // Reconstruct Maps from serialized arrays
            const graph = {
                ...request.graph,
                nodes: new Map(request.graph.nodes),
            };

            // Reconstruct Sets from serialized arrays
            const view = {
                ...request.view,
                expandedNodes: new Set(request.view.expandedNodes),
                hiddenNodes: new Set(request.view.hiddenNodes),
            };

            // We don't support progress callbacks across worker boundary yet
            const result = await engine.layout(graph as any, view);

            const response: WorkerMessage = {
                type: "LAYOUT_RESPONSE",
                payload: {
                    id: request.id,
                    result,
                },
            };

            ctx.postMessage(response);
        } catch (error) {
            console.error("Worker layout failed:", error);

            const response: WorkerMessage = {
                type: "LAYOUT_RESPONSE",
                payload: {
                    id: request.id,
                    error: error instanceof Error ? error.message : String(error),
                },
            };

            ctx.postMessage(response);
        }
    }
};
