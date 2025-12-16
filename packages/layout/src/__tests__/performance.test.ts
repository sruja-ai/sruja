import { describe, it, expect } from "vitest";
import { createC4Id } from "../brand";
import { createC4Graph } from "../c4-model";
import { layout } from "../c4-layout";
import { createDefaultViewState } from "../c4-view";

function makeGraph(n: number) {
  const nodes = [] as any[];
  const rels = [] as any[];
  for (let i = 0; i < n; i++) {
    nodes.push({
      id: createC4Id(`N${i}`),
      label: `Node ${i}`,
      kind: "SoftwareSystem",
      level: "context",
      tags: new Set<string>(),
    });
    if (i > 0)
      rels.push({ id: `N${i - 1}->N${i}`, from: createC4Id(`N${i - 1}`), to: createC4Id(`N${i}`) });
  }
  return createC4Graph(nodes as any, rels as any);
}

describe("Performance (soft bounds)", () => {
  it("layouts 100 nodes within reasonable time", () => {
    const graph = makeGraph(100);
    const res = layout(graph, createDefaultViewState());
    expect(res.debug?.layoutTimeMs ?? 0).toBeLessThan(2000);
    expect(res.metrics.edgeCrossings).toBeGreaterThanOrEqual(0);
  });
});
