// packages/shared/src/utils/cssVars.test.ts
// eslint-disable-next-line @typescript-eslint/no-unused-vars
import { describe, it, expect, beforeEach, vi } from "vitest";
import { getCssVar, Colors } from "./cssVars";

describe("cssVars", () => {
  beforeEach(() => {
    // Reset document
    if (typeof document !== "undefined") {
      document.documentElement.style.removeProperty("--color-primary");
      document.documentElement.style.removeProperty("--color-primary-50");
      document.documentElement.style.removeProperty("--color-primary-hover");
      document.documentElement.style.removeProperty("--color-border");
      document.documentElement.style.removeProperty("--color-text-primary");
      document.documentElement.style.removeProperty("--color-text-secondary");
      document.documentElement.style.removeProperty("--color-text-tertiary");
      document.documentElement.style.removeProperty("--color-background");
      document.documentElement.style.removeProperty("--color-surface");
      document.documentElement.style.removeProperty("--color-success-500");
      document.documentElement.style.removeProperty("--color-error-500");
      document.documentElement.style.removeProperty("--color-info-500");
      document.documentElement.style.removeProperty("--color-neutral-500");
    }
  });

  describe("getCssVar", () => {
    it("should return empty string when document is undefined", () => {
      const originalDocument = global.document;
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      delete (global as any).document;

      const result = getCssVar("--test-var");
      expect(result).toBe("");

      global.document = originalDocument;
    });

    it("should return CSS variable value", () => {
      document.documentElement.style.setProperty("--test-var", "red");

      const result = getCssVar("--test-var");
      expect(result).toBe("red");
    });

    it("should trim whitespace from CSS variable value", () => {
      document.documentElement.style.setProperty("--test-var", "  blue  ");

      const result = getCssVar("--test-var");
      expect(result).toBe("blue");
    });

    it("should return empty string for non-existent variable", () => {
      const result = getCssVar("--non-existent-var");
      expect(result).toBe("");
    });
  });

  describe("Colors", () => {
    it("should get primary color", () => {
      document.documentElement.style.setProperty("--color-primary", "#0000ff");

      const result = Colors.primary();
      expect(result).toBe("#0000ff");
    });

    it("should get primary50 color", () => {
      document.documentElement.style.setProperty("--color-primary-50", "#e6f2ff");

      const result = Colors.primary50();
      expect(result).toBe("#e6f2ff");
    });

    it("should get primaryHover color", () => {
      document.documentElement.style.setProperty("--color-primary-hover", "#0000cc");

      const result = Colors.primaryHover();
      expect(result).toBe("#0000cc");
    });

    it("should get border color", () => {
      document.documentElement.style.setProperty("--color-border", "#cccccc");

      const result = Colors.border();
      expect(result).toBe("#cccccc");
    });

    it("should get textPrimary color", () => {
      document.documentElement.style.setProperty("--color-text-primary", "#000000");

      const result = Colors.textPrimary();
      expect(result).toBe("#000000");
    });

    it("should get textSecondary color", () => {
      document.documentElement.style.setProperty("--color-text-secondary", "#666666");

      const result = Colors.textSecondary();
      expect(result).toBe("#666666");
    });

    it("should get textTertiary color", () => {
      document.documentElement.style.setProperty("--color-text-tertiary", "#999999");

      const result = Colors.textTertiary();
      expect(result).toBe("#999999");
    });

    it("should get background color", () => {
      document.documentElement.style.setProperty("--color-background", "#ffffff");

      const result = Colors.background();
      expect(result).toBe("#ffffff");
    });

    it("should get surface color", () => {
      document.documentElement.style.setProperty("--color-surface", "#f5f5f5");

      const result = Colors.surface();
      expect(result).toBe("#f5f5f5");
    });

    it("should get success color", () => {
      document.documentElement.style.setProperty("--color-success-500", "#00ff00");

      const result = Colors.success();
      expect(result).toBe("#00ff00");
    });

    it("should get error color", () => {
      document.documentElement.style.setProperty("--color-error-500", "#ff0000");

      const result = Colors.error();
      expect(result).toBe("#ff0000");
    });

    it("should get info color", () => {
      document.documentElement.style.setProperty("--color-info-500", "#0000ff");

      const result = Colors.info();
      expect(result).toBe("#0000ff");
    });

    it("should get neutral500 color", () => {
      document.documentElement.style.setProperty("--color-neutral-500", "#808080");

      const result = Colors.neutral500();
      expect(result).toBe("#808080");
    });

    it("should return empty string when CSS variable is not set", () => {
      const result = Colors.primary();
      expect(result).toBe("");
    });
  });
});
