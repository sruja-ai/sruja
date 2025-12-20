import type { C4Graph, LayoutOptions, LayoutResult } from "../core/types";

export interface LayoutRequest {
    id: string;
    graph: C4Graph;
    view: any;
    options: LayoutOptions;
}

export interface LayoutResponse {
    id: string;
    result?: LayoutResult;
    error?: string;
}

export type WorkerMessage =
    | { type: "LAYOUT_REQUEST"; payload: LayoutRequest }
    | { type: "LAYOUT_RESPONSE"; payload: LayoutResponse };
