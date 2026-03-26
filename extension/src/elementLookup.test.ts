import { SrujaElement } from "./wasm";
import { findElementById, findAllElementsById } from "./elementLookup";

describe("elementLookup", () => {
  const elements: SrujaElement[] = [
    { id: "SystemA.Api", kind: "container", title: "API A", range: { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } } },
    { id: "SystemB.Api", kind: "container", title: "API B", range: { start: { line: 1, character: 0 }, end: { line: 1, character: 0 } } },
    { id: "App", kind: "system", title: "App", range: { start: { line: 2, character: 0 }, end: { line: 2, character: 0 } } },
  ];

  describe("findElementById", () => {
    it("finds exact match", () => {
      expect(findElementById(elements, "SystemA.Api")?.id).toBe("SystemA.Api");
    });

    it("finds unique suffix match", () => {
      expect(findElementById(elements, "App")?.id).toBe("App");
    });

    it("returns undefined for ambiguous suffix match", () => {
      expect(findElementById(elements, "Api")).toBeUndefined();
    });

    it("returns undefined for non-existent ID", () => {
      expect(findElementById(elements, "Other")).toBeUndefined();
    });
  });

  describe("findAllElementsById", () => {
    it("returns exact match only if it exists", () => {
      expect(findAllElementsById(elements, "SystemA.Api")).toHaveLength(1);
      expect(findAllElementsById(elements, "SystemA.Api")[0].id).toBe("SystemA.Api");
    });

    it("returns all suffix matches if no exact match", () => {
      const matches = findAllElementsById(elements, "Api");
      expect(matches).toHaveLength(2);
      expect(matches.map(m => m.id)).toContain("SystemA.Api");
      expect(matches.map(m => m.id)).toContain("SystemB.Api");
    });
  });
});
