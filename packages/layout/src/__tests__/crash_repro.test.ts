import { describe, it, expect } from "vitest";
import { applySrujaLayout } from "../index";
import { createC4LayoutOptions } from "../c4-options";
import type { ArchitectureJSON } from "../bridge";

describe("Layout Crash Reproduction", () => {
    it("should not crash when adding a new empty system", async () => {
        // 1. Setup minimal architecture with one system added dynamically (simulated)
        const arch: ArchitectureJSON = {
            architecture: {
                systems: [
                    {
                        id: "mytestsystem",
                        label: "MyTestSystem",
                        description: undefined,
                        containers: [],
                    },
                ],
                persons: [],
                relations: [],
            },
        };

        // 2. Run layout
        const options = createC4LayoutOptions("L1");
        // We expect this NOT to throw
        try {
            // applySrujaLayout expects: nodes, edges, architectureData, options
            const result = await applySrujaLayout([], [], arch, options);
            expect(result).toBeDefined();
            console.log("Layout result generated successfully");
        } catch (e) {
            console.error("Layout crashed:", e);
            throw e;
        }
    });

    it("should not crash with missing optional fields", async () => {
        const arch: ArchitectureJSON = {
            architecture: {
                systems: [
                    {
                        id: "system2",
                        label: "System 2",
                        // Description missing
                        containers: [],
                    },
                ],
                persons: [
                    {
                        id: "person1",
                        label: "Person 1"
                    }
                ],
                relations: [],
            },
        };

        const options = createC4LayoutOptions("L1");
        // applySrujaLayout expects: nodes, edges, architectureData, options
        await applySrujaLayout([], [], arch, options);
    });
});
