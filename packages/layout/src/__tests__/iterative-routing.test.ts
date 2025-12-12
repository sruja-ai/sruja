import { describe, it, expect } from "vitest";
import { calculateBestPort } from "../algorithms/edge-router";

describe("Iterative Edge Routing Optimization", () => {
  it("distributes ports on the same side based on target position", () => {
    // Source: Tall node (100x400) at 0,0
    // T1: (200, 50)  -> Should connect near top of East side
    // T2: (200, 350) -> Should connect near bottom of East side

    const source = { x: 0, y: 0, width: 100, height: 400 };
    const t1 = { x: 200, y: 50, width: 50, height: 50 };
    const t2 = { x: 200, y: 350, width: 50, height: 50 };

    const p1 = calculateBestPort(source, t1);
    const p2 = calculateBestPort(source, t2);

    console.log("P1:", p1);
    console.log("P2:", p2);

    // Both should be East
    expect(p1.side).toBe("east");
    expect(p2.side).toBe("east");

    // P1.y should be less than P2.y (Top vs Bottom)
    // Currently they will be equal (Center: 200)
    expect(p1.position.y).toBeLessThan(p2.position.y);

    // Specifically, p1.y should be close to t1.center.y (75)
    // p2.y should be close to t2.center.y (375)
    // We allow some clamping/margin, but they definitely shouldn't be equal.
    expect(Math.abs(p1.position.y - 75)).toBeLessThan(50);
    expect(Math.abs(p2.position.y - 375)).toBeLessThan(50);
  });

  it("uses all 4 sides for a central node connected to surrounding nodes", () => {
    // Center: C
    // Surround: N (North), S (South), E (East), W (West)
    // We expect edges to connect to N on North, S on South, etc.

    // Manually define rects to test calculateBestPort explicitly first
    const center = { x: 100, y: 100, width: 100, height: 100 };
    const north = { x: 100, y: 0, width: 100, height: 50 }; // Above
    const south = { x: 100, y: 250, width: 100, height: 50 }; // Below
    const east = { x: 250, y: 100, width: 50, height: 100 }; // Right
    const west = { x: 0, y: 100, width: 50, height: 100 }; // Left

    // Test Center -> North
    const cn = calculateBestPort(center, north);
    // EXPECTATION: Should leave from North side of Center
    expect(cn.side).toBe("north");

    // Test Center -> South
    const cs = calculateBestPort(center, south);
    // EXPECTATION: Should leave from South side of Center
    expect(cs.side).toBe("south");

    // Test Center -> East
    const ce = calculateBestPort(center, east);
    // EXPECTATION: Should leave from East side of Center
    expect(ce.side).toBe("east");

    // Test Center -> West
    const cw = calculateBestPort(center, west);
    // EXPECTATION: Should leave from West side of Center
    expect(cw.side).toBe("west");
  });

  it("optimizes connection when nodes are diagonal", () => {
    // Center at 100,100 size 100x100.
    // Target at 300,300 size 100x100.
    // dx = 200, dy = 200.
    // Current logic (if abs(dx) > abs(dy)) might pick one arbitrarily or strictly.
    // Ideally it should pick the side that is "closest" or "most natural".

    const c1 = { x: 100, y: 100, width: 100, height: 100 };
    const c2 = { x: 300, y: 300, width: 100, height: 100 };

    // dx=200, dy=200. Math.abs(dx) > Math.abs(dy) is false (200 > 200 is false).
    // so it goes to else -> checks dy. dy > 0 -> South.
    // So it will exit South.
    // If we move it slightly more horizontal: 310, 300. dx=210. 210 > 200 -> True.
    // dx > 0 -> East.

    // We want to ensure it behaves consistently.
    const res = calculateBestPort(c1, c2);
    expect(["south", "east"]).toContain(res.side);
  });

  it("chooses correct side for wide nodes (Scenario: L1 expanded)", () => {
    // Parent/Main: Very wide, 800x600.
    // External System: To the right, but vertically centered.

    const main = { x: 0, y: 0, width: 800, height: 600 };
    const ext = { x: 900, y: 250, width: 200, height: 100 }; // vertically centered relative to 600 (300 center)

    const res = calculateBestPort(main, ext);
    expect(res.side).toBe("east"); // Should definitely be east
  });

  it("chooses correct side for tall nodes", () => {
    const main = { x: 0, y: 0, width: 200, height: 800 };
    const ext = { x: 50, y: 900, width: 100, height: 100 };

    const res = calculateBestPort(main, ext);
    expect(res.side).toBe("south");
  });
});
