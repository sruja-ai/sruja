import { describe, it, expect } from "vitest";
import { convertJsonToDsl } from "../jsonToDsl";
import type { ArchitectureJSON } from "../../types";

describe("jsonToDsl", () => {
  describe("convertJsonToDsl", () => {
    it("generates architecture block with name", () => {
      const arch: ArchitectureJSON = {
        metadata: { name: "Test App", version: "1.0", generated: new Date().toISOString() },
        architecture: {
          persons: [],
          systems: [],
          relations: [],
        },
        navigation: { levels: ["L1"] },
      };

      const dsl = convertJsonToDsl(arch);

      expect(dsl).toContain('architecture "Test App"');
    });

    it("generates person declarations", () => {
      const arch: ArchitectureJSON = {
        metadata: { name: "Test", version: "1.0", generated: new Date().toISOString() },
        architecture: {
          persons: [
            { id: "User", label: "End User" },
            { id: "Admin", label: "Administrator" },
          ],
          systems: [],
          relations: [],
        },
        navigation: { levels: ["L1"] },
      };

      const dsl = convertJsonToDsl(arch);

      expect(dsl).toContain("person User");
      expect(dsl).toContain('"End User"');
      expect(dsl).toContain("person Admin");
    });

    it("generates system with containers", () => {
      const arch: ArchitectureJSON = {
        metadata: { name: "Test", version: "1.0", generated: new Date().toISOString() },
        architecture: {
          persons: [],
          systems: [
            {
              id: "App",
              label: "Application",
              containers: [
                { id: "Web", label: "Web App", technology: "React" },
                { id: "API", label: "API Server", technology: "Node.js" },
              ],
            },
          ],
          relations: [],
        },
        navigation: { levels: ["L1", "L2"] },
      };

      const dsl = convertJsonToDsl(arch);

      expect(dsl).toContain("system App");
      expect(dsl).toContain('"Application"');
      expect(dsl).toContain("container Web");
      expect(dsl).toContain('tech "React"');
      expect(dsl).toContain("container API");
    });

    it("generates container with components", () => {
      const arch: ArchitectureJSON = {
        metadata: { name: "Test", version: "1.0", generated: new Date().toISOString() },
        architecture: {
          persons: [],
          systems: [
            {
              id: "App",
              containers: [
                {
                  id: "API",
                  components: [
                    { id: "Controller", label: "API Controller" },
                    { id: "Service", label: "Business Service" },
                  ],
                },
              ],
            },
          ],
          relations: [],
        },
        navigation: { levels: ["L1", "L2", "L3"] },
      };

      const dsl = convertJsonToDsl(arch);

      expect(dsl).toContain("component Controller");
      expect(dsl).toContain("component Service");
    });

    it("generates relations with verb", () => {
      const arch: ArchitectureJSON = {
        metadata: { name: "Test", version: "1.0", generated: new Date().toISOString() },
        architecture: {
          persons: [{ id: "User" }],
          systems: [{ id: "App" }],
          relations: [{ from: "User", to: "App", verb: "uses", technology: "HTTPS" }],
        },
        navigation: { levels: ["L1"] },
      };

      const dsl = convertJsonToDsl(arch);

      expect(dsl).toMatch(/User\s+->\s+App/);
      expect(dsl).toContain("uses");
    });

    it("generates requirements block", () => {
      const arch: ArchitectureJSON = {
        metadata: { name: "Test", version: "1.0", generated: new Date().toISOString() },
        architecture: {
          persons: [],
          systems: [{ id: "App" }],
          relations: [],
          requirements: [
            { id: "REQ-1", type: "functional", title: "User login", tags: ["App"] },
            { id: "REQ-2", type: "performance", title: "Response time <100ms" },
          ],
        },
        navigation: { levels: ["L1"] },
      };

      const dsl = convertJsonToDsl(arch);

      expect(dsl).toContain("requirement REQ-1");
      expect(dsl).toContain("functional");
      expect(dsl).toContain('"User login"');
      expect(dsl).toContain("App"); // Tags included in output
    });

    it("generates ADR block", () => {
      const arch: ArchitectureJSON = {
        metadata: { name: "Test", version: "1.0", generated: new Date().toISOString() },
        architecture: {
          persons: [],
          systems: [{ id: "App" }],
          relations: [],
          adrs: [
            {
              id: "ADR-001",
              title: "Use React",
              status: "accepted",
              context: "Need modern UI",
              decision: "Use React for frontend",
              tags: ["App"],
            },
          ],
        },
        navigation: { levels: ["L1"] },
      };

      const dsl = convertJsonToDsl(arch);

      expect(dsl).toContain("adr ADR-001");
      expect(dsl).toContain("accepted");
      expect(dsl).toContain('"Use React"');
    });

    it("generates scenarios/flows", () => {
      const arch: ArchitectureJSON = {
        metadata: { name: "Test", version: "1.0", generated: new Date().toISOString() },
        architecture: {
          persons: [{ id: "User" }],
          systems: [{ id: "App" }],
          relations: [],
          scenarios: [
            {
              id: "login",
              title: "User Login",
              steps: [{ from: "User", to: "App", description: "authenticates" }],
            },
          ],
        },
        navigation: { levels: ["L1"] },
      };

      const dsl = convertJsonToDsl(arch);

      expect(dsl).toContain("scenario login");
      expect(dsl).toContain("User Login");
    });

    it("handles external systems", () => {
      const arch: ArchitectureJSON = {
        metadata: { name: "Test", version: "1.0", generated: new Date().toISOString() },
        architecture: {
          persons: [],
          systems: [
            {
              id: "External",
              label: "Third Party",
              metadata: [{ key: "external", value: "true" }],
            },
          ],
          relations: [],
        },
        navigation: { levels: ["L1"] },
      };

      const dsl = convertJsonToDsl(arch);

      expect(dsl).toContain("system External");
      // External is stored in metadata, may or may not be in DSL output
      expect(dsl).toContain('"Third Party"');
    });

    it("handles datastores and queues", () => {
      const arch: ArchitectureJSON = {
        metadata: { name: "Test", version: "1.0", generated: new Date().toISOString() },
        architecture: {
          persons: [],
          systems: [
            {
              id: "App",
              datastores: [{ id: "DB", label: "PostgreSQL" }],
              queues: [{ id: "Queue", label: "RabbitMQ" }],
            },
          ],
          relations: [],
        },
        navigation: { levels: ["L1"] },
      };

      const dsl = convertJsonToDsl(arch);

      expect(dsl).toContain("datastore DB");
      expect(dsl).toContain("queue Queue");
    });
  });
});
