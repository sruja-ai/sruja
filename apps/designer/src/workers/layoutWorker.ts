import type { Node, Edge } from "@xyflow/react";
import type { C4NodeData } from "../types";
import { applySrujaLayout, type ITextMeasurer } from "../utils/layoutEngine";
import { applyC4LevelLayout } from "../utils/c4LevelLayout";

class OffscreenTextMeasurer implements ITextMeasurer {
  private ctx: OffscreenCanvasRenderingContext2D;
  constructor() {
    const canvas = new OffscreenCanvas(1, 1);
    this.ctx = canvas.getContext("2d") as OffscreenCanvasRenderingContext2D;
  }
  private setFont(isBold: boolean, fontSize: number) {
    this.ctx.font = `${isBold ? "600" : "400"} ${fontSize}px Inter, system-ui, sans-serif`;
  }
  measure(text: string): { width: number; height: number } {
    this.setFont(true, 14);
    const m = this.ctx.measureText(text);
    return { width: m.width + 10, height: 20 };
  }
  measureMultiline(
    text: string,
    _kind: any,
    _level: any,
    maxWidth: number
  ): { width: number; height: number; lines: string[] } {
    const isTitle = text.length < 50 && !text.includes("\n");
    const fontSize = isTitle ? 14 : 12;
    const isBold = isTitle;
    const lineHeight = isTitle ? 20 : 16;
    this.setFont(isBold, fontSize);
    const words = text.split(/\s+/);
    const lines: string[] = [];
    let current = words[0] || "";
    for (let i = 1; i < words.length; i++) {
      const w = words[i];
      const width = this.ctx.measureText(current + " " + w).width;
      if (width < maxWidth) current += " " + w;
      else {
        lines.push(current);
        current = w;
      }
    }
    if (current) lines.push(current);
    let maxLineWidth = 0;
    for (const line of lines) {
      maxLineWidth = Math.max(maxLineWidth, this.ctx.measureText(line).width);
    }
    return { width: maxLineWidth + 10, height: lines.length * lineHeight, lines };
  }
  getLineHeight(): number {
    return 22;
  }
  getDescent(): number {
    return 6;
  }
}

type Message = {
  engine: "sruja" | "c4level";
  nodes: Node<C4NodeData>[];
  edges: Edge[];
  options: any;
};

self.onmessage = async (evt: MessageEvent<Message>) => {
  const { engine, nodes, edges, options } = evt.data;
  try {
    if (engine === "c4level") {
      const res = await applyC4LevelLayout(nodes, edges, options);
      (self as any).postMessage({ ok: true, result: res });
    } else {
      const measurer = new OffscreenTextMeasurer();
      const res = await applySrujaLayout(nodes, edges, { ...options, measurer });
      (self as any).postMessage({ ok: true, result: res });
    }
  } catch (e) {
    (self as any).postMessage({ ok: false, error: String(e) });
  }
};
