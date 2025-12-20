/// <reference path="../worker-env.d.ts" />
import type { C4Graph, C4ViewState, LayoutOptions, LayoutResult } from "../core/types";
import type { LayoutResponse, WorkerMessage } from "./types";
import LayoutWorker from "./layout.worker?worker";

export class LayoutWorkerClient {
    private worker: Worker | null = null;
    private pendingRequests = new Map<string, {
        resolve: (result: LayoutResult) => void;
        reject: (error: Error) => void;
        timer: any;
    }>();

    constructor() {
        this.initWorker();
    }

    private initWorker() {
        if (typeof Worker === "undefined") return;

        try {
            this.worker = new LayoutWorker();
            if (this.worker) {
                this.worker.onmessage = this.handleMessage.bind(this);
                this.worker.onerror = this.handleError.bind(this);
            }
        } catch (e) {
            console.warn("Failed to initialize layout worker:", e);
            this.worker = null;
        }
    }

    private handleMessage(event: MessageEvent<WorkerMessage>) {
        const { type, payload } = event.data;

        if (type === "LAYOUT_RESPONSE") {
            const response = payload as LayoutResponse;
            const request = this.pendingRequests.get(response.id);

            if (request) {
                clearTimeout(request.timer);
                this.pendingRequests.delete(response.id);

                if (response.error) {
                    request.reject(new Error(response.error));
                } else if (response.result) {
                    request.resolve(response.result);
                }
            }
        }
    }

    private handleError(error: ErrorEvent) {
        console.error("Layout worker error:", error);
        // Fail all pending requests
        for (const [, req] of this.pendingRequests) {
            clearTimeout(req.timer);
            req.reject(new Error("Worker error: " + error.message));
        }
        this.pendingRequests.clear();

        // Try to restart worker
        this.terminate();
        this.initWorker();
    }

    public terminate() {
        if (this.worker) {
            this.worker.terminate();
            this.worker = null;
        }
    }

    public async layout(
        graph: C4Graph,
        view: C4ViewState,
        options: Partial<LayoutOptions>,
        timeoutMs: number = 30000
    ): Promise<LayoutResult> {
        if (!this.worker) {
            throw new Error("Worker not supported");
        }

        // Clone data to ensure it's serializable and detached
        // Simplified cloning - in prod might need structuredClone
        // Handle Map serialization manually since JSON.stringify/postMessage doesn't handle Map methods
        const safeGraph = {
            ...graph,
            nodes: Array.from(graph.nodes.entries()),
            relationships: graph.relationships,
        };
        const safeOptions = JSON.parse(JSON.stringify(options));

        // Serialize view state (Sets become empty objects in JSON, so we handle it manually if needed)
        // For now, we assume Sets are empty or strictly handle known Sets
        const safeView = {
            ...view,
            expandedNodes: Array.from(view.expandedNodes || []),
            hiddenNodes: Array.from(view.hiddenNodes || []),
        };

        const id = Math.random().toString(36).substring(7);

        return new Promise<LayoutResult>((resolve, reject) => {
            const timer = setTimeout(() => {
                if (this.pendingRequests.has(id)) {
                    this.pendingRequests.delete(id);
                    reject(new Error("Layout timeout"));
                }
            }, timeoutMs);

            this.pendingRequests.set(id, { resolve, reject, timer });

            const message: WorkerMessage = {
                type: "LAYOUT_REQUEST",
                payload: {
                    id,
                    graph: safeGraph as any,
                    view: safeView,
                    options: safeOptions,
                },
            };

            this.worker!.postMessage(message);
        });
    }
}

// Singleton instance
let instance: LayoutWorkerClient | null = null;

export function getLayoutWorker(): LayoutWorkerClient {
    if (!instance) {
        instance = new LayoutWorkerClient();
    }
    return instance;
}
