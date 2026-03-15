import { formatStatusLines, formatReviewLines, type StatusJson, type ReviewJson } from "./cliOutput";

describe("cliOutput", () => {
  describe("formatStatusLines", () => {
    it("formats minimal status", () => {
      const status: StatusJson = {};
      const lines = formatStatusLines(status);
      expect(lines).toContain("Baseline: (none)");
      expect(lines).toContain("Truth: unknown (0 violation(s))");
      expect(lines[lines.length - 1]).toBe("--- Done ---");
    });
  it("includes health score and context when present", () => {
    const status: StatusJson = {
      baseline: "main",
      truth_status: "synced",
      violations_count: 1,
      health_score: 85,
      context_updated_at: "2025-01-01",
    };
    const lines = formatStatusLines(status);
    expect(lines[0]).toBe("Baseline: main");
    expect(lines[1]).toContain("85/100");
    expect(lines[1]).toContain("Context: 2025-01-01");
  });

  it("includes health score when zero", () => {
    const status: StatusJson = { health_score: 0, truth_status: "unknown" };
    const lines = formatStatusLines(status);
    expect(lines[1]).toContain("0/100");
  });
});

  describe("formatReviewLines", () => {
    it("formats minimal review", () => {
      const review: ReviewJson = {
        truth_status: "unknown",
        has_drift: false,
        violations_count: 0,
        new_components: [],
        missing_components: [],
        drifted_dependencies: [],
        open_questions: [],
        suggestions: [],
      };
      const lines = formatReviewLines(review);
      expect(lines).toContain("Has drift: false");
      expect(lines[lines.length - 1]).toBe("--- Done ---");
    });
  it("includes new/missing/drifted sections when present", () => {
    const review: ReviewJson = {
      truth_status: "synced",
      has_drift: true,
      violations_count: 2,
      new_components: ["A"],
      missing_components: ["B"],
      drifted_dependencies: ["C->D"],
      open_questions: [],
      suggestions: [],
    };
    const lines = formatReviewLines(review);
    expect(lines.some((l) => l.includes("+ A"))).toBe(true);
    expect(lines.some((l) => l.includes("- B"))).toBe(true);
    expect(lines.some((l) => l.includes("~ C->D"))).toBe(true);
  });

  it("includes open_questions and suggestions when present", () => {
    const review: ReviewJson = {
      truth_status: "synced",
      has_drift: false,
      violations_count: 0,
      new_components: [],
      missing_components: [],
      drifted_dependencies: [],
      open_questions: ["Q1?"],
      suggestions: ["Do X"],
    };
    const lines = formatReviewLines(review);
    expect(lines.some((l) => l.includes("? Q1?"))).toBe(true);
    expect(lines.some((l) => l.includes("> Do X"))).toBe(true);
  });

  it("includes health_score when zero", () => {
    const review: ReviewJson = {
      truth_status: "synced",
      has_drift: false,
      violations_count: 0,
      health_score: 0,
      new_components: [],
      missing_components: [],
      drifted_dependencies: [],
      open_questions: [],
      suggestions: [],
    };
    const lines = formatReviewLines(review);
    expect(lines[1]).toContain("0/100");
  });

  it("handles missing optional arrays defensively", () => {
    const review = {
      truth_status: "ok",
      has_drift: false,
      violations_count: 0,
    } as unknown as ReviewJson;
    const lines = formatReviewLines(review);
    expect(lines[lines.length - 1]).toBe("--- Done ---");
  });
});
});
