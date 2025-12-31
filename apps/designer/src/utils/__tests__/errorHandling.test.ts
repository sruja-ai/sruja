// apps/designer/src/utils/__tests__/errorHandling.test.ts
import { describe, it, expect } from "vitest";
import {
  AppError,
  NetworkError,
  ValidationError,
  ErrorType,
  safeAsync,
  isRetryableError,
  getUserFriendlyMessage,
} from "../errorHandling";

describe("errorHandling", () => {
  describe("AppError", () => {
    it("should create error with message", () => {
      const error = new AppError("Test error");
      expect(error.message).toBe("Test error");
      expect(error.name).toBe("AppError");
      expect(error.type).toBe(ErrorType.UNKNOWN);
    });

    it("should create error with type", () => {
      const error = new AppError("Test error", ErrorType.NETWORK);
      expect(error.type).toBe(ErrorType.NETWORK);
    });

    it("should create error with context", () => {
      const context = { url: "/api/data", status: 404 };
      const error = new AppError("Test error", ErrorType.NETWORK, context);
      expect(error.context).toEqual(context);
    });

    it("should be instance of Error", () => {
      const error = new AppError("Test error");
      expect(error).toBeInstanceOf(Error);
      expect(error).toBeInstanceOf(AppError);
    });
  });

  describe("NetworkError", () => {
    it("should create network error with status code", () => {
      const error = new NetworkError("Not found", 404);
      expect(error.message).toBe("Not found");
      expect(error.type).toBe(ErrorType.NETWORK);
      expect(error.statusCode).toBe(404);
      expect(error.name).toBe("NetworkError");
    });

    it("should create network error without status code", () => {
      const error = new NetworkError("Connection failed");
      expect(error.statusCode).toBeUndefined();
    });

    it("should include status code in context", () => {
      const error = new NetworkError("Not found", 404, { endpoint: "/api" });
      expect(error.context?.statusCode).toBe(404);
      expect(error.context?.endpoint).toBe("/api");
    });

    it("should be instance of AppError", () => {
      const error = new NetworkError("Test");
      expect(error).toBeInstanceOf(Error);
      expect(error).toBeInstanceOf(AppError);
      expect(error).toBeInstanceOf(NetworkError);
    });
  });

  describe("ValidationError", () => {
    it("should create validation error with field", () => {
      const error = new ValidationError("Email is required", "email");
      expect(error.message).toBe("Email is required");
      expect(error.type).toBe(ErrorType.VALIDATION);
      expect(error.field).toBe("email");
      expect(error.name).toBe("ValidationError");
    });

    it("should create validation error without field", () => {
      const error = new ValidationError("Invalid input");
      expect(error.field).toBeUndefined();
    });

    it("should include field in context", () => {
      const error = new ValidationError("Email is required", "email", { minLength: 5 });
      expect(error.context?.field).toBe("email");
      expect(error.context?.minLength).toBe(5);
    });

    it("should be instance of AppError", () => {
      const error = new ValidationError("Test");
      expect(error).toBeInstanceOf(Error);
      expect(error).toBeInstanceOf(AppError);
      expect(error).toBeInstanceOf(ValidationError);
    });
  });

  describe("safeAsync", () => {
    it("should return data on success", async () => {
      const { data, error } = await safeAsync(async () => "success");
      expect(data).toBe("success");
      expect(error).toBeNull();
    });

    it("should return error on failure", async () => {
      const { data, error } = await safeAsync(
        async () => {
          throw new Error("Operation failed");
        },
        "Custom error message",
        ErrorType.NETWORK
      );

      expect(data).toBeNull();
      expect(error).toBeInstanceOf(AppError);
      expect(error?.message).toBe("Custom error message");
      expect(error?.type).toBe(ErrorType.NETWORK);
    });

    it("should preserve AppError instances", async () => {
      const originalError = new NetworkError("Original error", 404);
      const { data, error } = await safeAsync(async () => {
        throw originalError;
      }, "Should not override");

      expect(data).toBeNull();
      expect(error).toBe(originalError);
      expect(error?.type).toBe(ErrorType.NETWORK);
    });

    it("should handle non-Error exceptions", async () => {
      const { data, error } = await safeAsync(async () => {
        throw "String error";
      }, "Custom error");

      expect(data).toBeNull();
      expect(error).toBeInstanceOf(AppError);
      expect(error?.context?.originalError).toBe("String error");
    });

    it("should use default error message and type", async () => {
      const { error } = await safeAsync(async () => {
        throw new Error("Test");
      });

      expect(error?.message).toBe("Operation failed");
      expect(error?.type).toBe(ErrorType.UNKNOWN);
    });
  });

  describe("isRetryableError", () => {
    it("should return true for network errors with retryable status codes", () => {
      expect(isRetryableError(new NetworkError("Timeout", 408))).toBe(true);
      expect(isRetryableError(new NetworkError("Too many requests", 429))).toBe(true);
      expect(isRetryableError(new NetworkError("Service unavailable", 503))).toBe(true);
      expect(isRetryableError(new NetworkError("Gateway timeout", 504))).toBe(true);
    });

    it("should return false for network errors with non-retryable status codes", () => {
      expect(isRetryableError(new NetworkError("Not found", 404))).toBe(false);
      expect(isRetryableError(new NetworkError("Forbidden", 403))).toBe(false);
      expect(isRetryableError(new NetworkError("Bad request", 400))).toBe(false);
    });

    it("should return false for validation errors", () => {
      expect(isRetryableError(new ValidationError("Invalid"))).toBe(false);
    });

    it("should return false for unknown errors", () => {
      expect(isRetryableError(new AppError("Unknown"))).toBe(false);
      expect(isRetryableError(new Error("Generic"))).toBe(false);
    });
  });

  describe("getUserFriendlyMessage", () => {
    it("should return message from AppError", () => {
      const error = new AppError("User-friendly message");
      expect(getUserFriendlyMessage(error)).toBe("User-friendly message");
    });

    it("should return message from standard Error", () => {
      const error = new Error("Standard error");
      expect(getUserFriendlyMessage(error)).toBe("Standard error");
    });

    it("should return default message for unknown error types", () => {
      expect(getUserFriendlyMessage("String error")).toBe("An unexpected error occurred");
      expect(getUserFriendlyMessage(null)).toBe("An unexpected error occurred");
      expect(getUserFriendlyMessage(undefined)).toBe("An unexpected error occurred");
    });

    it("should return message from error object with message property", () => {
      const error = { message: "Custom error" };
      expect(getUserFriendlyMessage(error)).toBe("Custom error");
    });

    it("should handle network errors with status codes", () => {
      const error = new NetworkError("Not found", 404);
      expect(getUserFriendlyMessage(error)).toBe("Not found");
    });

    it("should handle validation errors with fields", () => {
      const error = new ValidationError("Email is required", "email");
      expect(getUserFriendlyMessage(error)).toBe("Email is required");
    });
  });
});
