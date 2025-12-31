// packages/shared/src/utils/__tests__/pathValidation.test.ts
// Unit tests for path validation utilities

import { describe, it, expect } from "vitest";
import { validateFilePath, sanitizeFilePath, isSafePath } from "../pathValidation";
import { ValidationError } from "../errors";

describe("validateFilePath", () => {
  describe("valid paths", () => {
    it("should accept relative paths", () => {
      const result = validateFilePath("path/to/file.txt");
      expect(result.ok).toBe(true);
      if (result.ok) {
        expect(result.value).toBe("path/to/file.txt");
      }
    });

    it("should accept paths with forward slashes", () => {
      const result = validateFilePath("dir/subdir/file");
      expect(result.ok).toBe(true);
    });

    it("should accept paths with backslashes (normalized)", () => {
      const result = validateFilePath("dir\\subdir\\file");
      expect(result.ok).toBe(true);
    });

    it("should accept absolute paths when allowed", () => {
      const result = validateFilePath("/absolute/path", true);
      expect(result.ok).toBe(true);
    });

    it("should accept Windows absolute paths when allowed", () => {
      const result = validateFilePath("C:\\Windows\\file.txt", true);
      expect(result.ok).toBe(true);
    });
  });

  describe("path traversal prevention", () => {
    it("should reject paths with ..", () => {
      const result = validateFilePath("../etc/passwd");
      expect(result.ok).toBe(false);
      if (!result.ok) {
        expect(result.error).toBeInstanceOf(ValidationError);
        expect(result.error.message).toContain("path traversal");
      }
    });

    it("should reject paths with .. in middle", () => {
      const result = validateFilePath("path/../etc/passwd");
      expect(result.ok).toBe(false);
    });

    it("should reject paths ending with ..", () => {
      const result = validateFilePath("path/..");
      expect(result.ok).toBe(false);
    });
  });

  describe("null byte prevention", () => {
    it("should reject paths with null bytes", () => {
      const result = validateFilePath("path/to/file\0.txt");
      expect(result.ok).toBe(false);
      if (!result.ok) {
        expect(result.error.message).toContain("null bytes");
      }
    });
  });

  describe("control character prevention", () => {
    it("should reject paths with control characters", () => {
      const result = validateFilePath("path/to/file\x00.txt");
      expect(result.ok).toBe(false);
    });

    it("should reject paths with newlines", () => {
      const result = validateFilePath("path/to/file\n.txt");
      expect(result.ok).toBe(false);
    });
  });

  describe("absolute path restrictions", () => {
    it("should reject absolute paths by default", () => {
      const result = validateFilePath("/absolute/path");
      expect(result.ok).toBe(false);
      if (!result.ok) {
        expect(result.error.message).toContain("Absolute paths");
      }
    });

    it("should reject Windows absolute paths by default", () => {
      const result = validateFilePath("C:\\Windows\\file.txt");
      expect(result.ok).toBe(false);
    });
  });

  describe("path length limits", () => {
    it("should reject paths exceeding maximum length", () => {
      const longPath = "a".repeat(5000);
      const result = validateFilePath(longPath);
      expect(result.ok).toBe(false);
      if (!result.ok) {
        expect(result.error.message).toContain("maximum length");
      }
    });
  });

  describe("input validation", () => {
    it("should reject non-string inputs", () => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const result = validateFilePath(null as any);
      expect(result.ok).toBe(false);
    });

    it("should reject empty strings", () => {
      const result = validateFilePath("");
      expect(result.ok).toBe(false);
      if (!result.ok) {
        expect(result.error.message).toContain("cannot be empty");
      }
    });
  });
});

describe("sanitizeFilePath", () => {
  it("should normalize path separators", () => {
    expect(sanitizeFilePath("path\\to\\file")).toBe("path/to/file");
  });

  it("should remove duplicate separators", () => {
    expect(sanitizeFilePath("path//to///file")).toBe("path/to/file");
  });

  it("should remove leading and trailing slashes", () => {
    expect(sanitizeFilePath("/path/to/file/")).toBe("path/to/file");
  });

  it("should trim whitespace", () => {
    expect(sanitizeFilePath("  path/to/file  ")).toBe("path/to/file");
  });

  it("should handle empty strings", () => {
    expect(sanitizeFilePath("")).toBe("");
  });

  it("should handle non-string inputs", () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    expect(sanitizeFilePath(null as any)).toBe("");
  });
});

describe("isSafePath", () => {
  it("should return true for safe paths", () => {
    expect(isSafePath("path/to/file")).toBe(true);
  });

  it("should return false for paths with ..", () => {
    expect(isSafePath("../etc/passwd")).toBe(false);
  });

  it("should return false for paths with null bytes", () => {
    expect(isSafePath("path\0to/file")).toBe(false);
  });

  it("should return false for non-string inputs", () => {
    expect(isSafePath(null)).toBe(false);
    expect(isSafePath(123)).toBe(false);
  });
});
