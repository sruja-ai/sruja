// packages/shared/src/utils/__tests__/validation.property.test.ts
// Property-based tests for validation utilities using fast-check

import { describe, it, expect } from "vitest";
import * as fc from "fast-check";
import {
  isNonEmptyString,
  validateNonEmptyString,
  isValidElementId,
  validateElementId,
  isValidPercentage,
  validatePercentage,
  validatePositiveInteger,
} from "../validation";
import { ValidationError } from "../errors";
import { PERCENTAGE } from "../constants";

describe("Property-based validation tests", () => {
  describe("isNonEmptyString", () => {
    it("should return true for all non-empty strings", () => {
      fc.assert(
        fc.property(fc.string({ minLength: 1 }), (str) => {
          expect(isNonEmptyString(str)).toBe(true);
        })
      );
    });

    it("should return false for empty string", () => {
      expect(isNonEmptyString("")).toBe(false);
    });

    it("should return false for all non-string values", () => {
      fc.assert(
        fc.property(
          fc.oneof(
            fc.integer(),
            fc.float(),
            fc.boolean(),
            fc.object(),
            fc.array(fc.anything()),
            fc.constant(null),
            fc.constant(undefined)
          ),
          (value) => {
            expect(isNonEmptyString(value)).toBe(false);
          }
        )
      );
    });
  });

  describe("isValidElementId", () => {
    it("should accept all valid element IDs", () => {
      fc.assert(
        fc.property(
          fc.stringOf(
            fc.constantFrom(
              ..."abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789:_-".split("")
            ),
            { minLength: 1 }
          ),
          (id) => {
            expect(isValidElementId(id)).toBe(true);
          }
        )
      );
    });

    it("should reject IDs with invalid characters", () => {
      fc.assert(
        fc.property(
          fc.string({ minLength: 1 }).filter((str) => /[^a-zA-Z0-9:_-]/.test(str)),
          (id) => {
            expect(isValidElementId(id)).toBe(false);
          }
        )
      );
    });
  });

  describe("isValidPercentage", () => {
    it("should accept all valid percentages (0-100)", () => {
      fc.assert(
        fc.property(
          fc.float({ min: PERCENTAGE.MIN, max: PERCENTAGE.MAX, noNaN: true }),
          (value) => {
            expect(isValidPercentage(value)).toBe(true);
          }
        )
      );
    });

    it("should reject values outside 0-100 range", () => {
      fc.assert(
        fc.property(
          fc.oneof(
            fc.float({ max: Math.fround(-0.0001), noNaN: true }),
            fc.float({ min: Math.fround(PERCENTAGE.MAX + 0.0001), noNaN: true })
          ),
          (value) => {
            expect(isValidPercentage(value)).toBe(false);
          }
        )
      );
    });

    it("should reject NaN", () => {
      expect(isValidPercentage(NaN)).toBe(false);
    });

    it("should reject Infinity", () => {
      expect(isValidPercentage(Infinity)).toBe(false);
      expect(isValidPercentage(-Infinity)).toBe(false);
    });
  });

  describe("validatePercentage", () => {
    it("should return ok for all valid percentages", () => {
      fc.assert(
        fc.property(
          fc.float({ min: PERCENTAGE.MIN, max: PERCENTAGE.MAX, noNaN: true }),
          (value) => {
            const result = validatePercentage(value);
            expect(result.ok).toBe(true);
            if (result.ok) {
              expect(result.value).toBe(value);
            }
          }
        )
      );
    });

    it("should return error for values outside range", () => {
      fc.assert(
        fc.property(
          fc.oneof(
            fc.float({ max: Math.fround(-0.0001), noNaN: true }),
            fc.float({ min: Math.fround(PERCENTAGE.MAX + 0.0001), noNaN: true })
          ),
          (value) => {
            const result = validatePercentage(value);
            expect(result.ok).toBe(false);
            if (!result.ok) {
              expect(result.error).toBeInstanceOf(ValidationError);
            }
          }
        )
      );
    });
  });

  describe("validatePositiveInteger", () => {
    it("should accept all positive integers", () => {
      fc.assert(
        fc.property(fc.integer({ min: 1, max: Number.MAX_SAFE_INTEGER }), (value) => {
          const result = validatePositiveInteger(value);
          expect(result.ok).toBe(true);
          if (result.ok) {
            expect(result.value).toBe(value);
          }
        })
      );
    });

    it("should reject zero and negative integers", () => {
      fc.assert(
        fc.property(fc.integer({ max: 0 }), (value) => {
          const result = validatePositiveInteger(value);
          expect(result.ok).toBe(false);
          if (!result.ok) {
            expect(result.error).toBeInstanceOf(ValidationError);
            expect(result.error.message).toContain("positive integer");
          }
        })
      );
    });

    it("should reject non-integers", () => {
      fc.assert(
        fc.property(
          fc.float().filter((n) => !Number.isInteger(n) && n > 0 && Number.isFinite(n)),
          (value) => {
            const result = validatePositiveInteger(value);
            expect(result.ok).toBe(false);
            if (!result.ok) {
              expect(result.error.message).toContain("integer");
            }
          }
        )
      );
    });
  });

  describe("validateNonEmptyString", () => {
    it("should return ok for all non-empty strings", () => {
      fc.assert(
        fc.property(fc.string({ minLength: 1 }), (str) => {
          const result = validateNonEmptyString(str);
          expect(result.ok).toBe(true);
          if (result.ok) {
            expect(result.value).toBe(str);
          }
        })
      );
    });

    it("should return error for empty string", () => {
      const result = validateNonEmptyString("");
      expect(result.ok).toBe(false);
      if (!result.ok) {
        expect(result.error).toBeInstanceOf(ValidationError);
      }
    });
  });

  describe("validateElementId", () => {
    it("should return ok for all valid element IDs", () => {
      fc.assert(
        fc.property(
          fc.stringOf(
            fc.constantFrom(
              ..."abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789:_-".split("")
            ),
            { minLength: 1 }
          ),
          (id) => {
            const result = validateElementId(id);
            expect(result.ok).toBe(true);
            if (result.ok) {
              expect(result.value).toBe(id);
            }
          }
        )
      );
    });

    it("should return error for IDs with invalid characters", () => {
      fc.assert(
        fc.property(
          fc.string({ minLength: 1 }).filter((str) => /[^a-zA-Z0-9:_-]/.test(str)),
          (id) => {
            const result = validateElementId(id);
            expect(result.ok).toBe(false);
            if (!result.ok) {
              expect(result.error).toBeInstanceOf(ValidationError);
            }
          }
        )
      );
    });
  });
});
