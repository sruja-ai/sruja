import { describe, it, expect } from "vitest";
import { sortByDate, sortByWeight } from "./sorting";

interface TestEntry {
  slug: string;
  data: {
    title: string;
    pubDate?: string;
    weight?: number;
  };
}

function makeEntry(slug: string, data: TestEntry["data"]): TestEntry {
  return { slug, data };
}

describe("content sorting", () => {
  it("sortByDate sorts newest first and handles missing dates", () => {
    const entries: TestEntry[] = [
      makeEntry("old", { title: "Old", pubDate: "2020-01-01" }),
      makeEntry("new", { title: "New", pubDate: "2024-05-01" }),
      makeEntry("none", { title: "None" }),
    ];
    const sorted = sortByDate(entries);
    expect(sorted.map((e) => e.slug)).toEqual(["new", "old", "none"]);
  });

  it("sortByWeight sorts lower weight first and default high weight when missing", () => {
    const entries: TestEntry[] = [
      makeEntry("w3", { title: "W3", weight: 3 }),
      makeEntry("w1", { title: "W1", weight: 1 }),
      makeEntry("none", { title: "None" }),
    ];
    const sorted = sortByWeight(entries);
    expect(sorted.map((e) => e.slug)).toEqual(["w1", "w3", "none"]);
  });
});
