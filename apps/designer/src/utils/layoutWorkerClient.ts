import type { Node, Edge } from "@xyflow/react";
import type { C4NodeData } from "../types";

type Req = {
  engine: "sruja" | "c4level";
  nodes: Node<C4NodeData>[];
  edges: Edge[];
  options: any;
};

type Res =
  | { ok: true; result: { nodes: Node<C4NodeData>[]; edges: Edge[] } }
  | { ok: false; error: string };

export class LayoutWorkerClient {
  private worker: Worker | null = null;
  private ready = false;
  init() {
    if (this.worker) return;
    this.worker = new Worker(new URL("../workers/layoutWorker.ts", import.meta.url), {
      type: "module",
    });
    this.ready = true;
  }
  async run(
    engine: "sruja" | "c4level",
    nodes: Node<C4NodeData>[],
    edges: Edge[],
    options: any
  ): Promise<{ nodes: Node<C4NodeData>[]; edges: Edge[] }> {
    if (!this.ready) this.init();
    return new Promise((resolve, reject) => {
      const w = this.worker!;
      const onMessage = (e: MessageEvent<Res>) => {
        w.removeEventListener("message", onMessage);
        const data = e.data;
        if (data.ok) resolve(data.result);
        else reject(new Error(data.error));
      };
      w.addEventListener("message", onMessage);
      const payload: Req = { engine, nodes, edges, options };
      w.postMessage(payload);
    });
  }
}

export const layoutWorkerClient = new LayoutWorkerClient();
