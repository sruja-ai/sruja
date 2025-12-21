// apps/designer/src/components/shared/forms/__tests__/useFormState.test.ts
import { describe, it, expect, vi } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useFormState } from "../useFormState";

describe("useFormState", () => {
  describe("initialization", () => {
    it("should initialize with provided values", () => {
      const initialValues = { name: "Test", email: "test@example.com" };
      const { result } = renderHook(() =>
        useFormState({
          initialValues,
          onSubmit: vi.fn(),
        })
      );

      expect(result.current.values).toEqual(initialValues);
      expect(result.current.isSubmitting).toBe(false);
    });

    it("should initialize with empty object if no initial values", () => {
      const { result } = renderHook(() =>
        useFormState({
          initialValues: {},
          onSubmit: vi.fn(),
        })
      );

      expect(result.current.values).toEqual({});
    });
  });

  describe("value updates", () => {
    it("should update single value", () => {
      const { result } = renderHook(() =>
        useFormState({
          initialValues: { name: "" },
          onSubmit: vi.fn(),
        })
      );

      act(() => {
        result.current.setValue("name", "New Name");
      });

      expect(result.current.values.name).toBe("New Name");
    });

    it("should update multiple values", () => {
      const { result } = renderHook(() =>
        useFormState({
          initialValues: { name: "", email: "" },
          onSubmit: vi.fn(),
        })
      );

      act(() => {
        result.current.setValues({ name: "Test", email: "test@example.com" });
      });

      expect(result.current.values.name).toBe("Test");
      expect(result.current.values.email).toBe("test@example.com");
    });

    it("should reset to initial values", () => {
      const initialValues = { name: "Initial" };
      const { result } = renderHook(() =>
        useFormState({
          initialValues,
          onSubmit: vi.fn(),
        })
      );

      act(() => {
        result.current.setValue("name", "Changed");
      });

      expect(result.current.values.name).toBe("Changed");

      act(() => {
        result.current.reset();
      });

      expect(result.current.values.name).toBe("Initial");
    });
  });

  describe("validation", () => {
    it("should validate on submit", async () => {
      const validate = vi.fn((values) => {
        const errors: Record<string, string> = {};
        if (!values.name) errors.name = "Name is required";
        return errors;
      });

      const { result } = renderHook(() =>
        useFormState({
          initialValues: { name: "" },
          validate,
          onSubmit: vi.fn(),
        })
      );

      const mockEvent = {
        preventDefault: vi.fn(),
      } as unknown as React.FormEvent<HTMLFormElement>;

      await act(async () => {
        await result.current.handleSubmit(mockEvent);
      });

      expect(validate).toHaveBeenCalled();
      expect(result.current.errors.name).toBe("Name is required");
    });
  });

  describe("form submission", () => {
    it("should call onSubmit with form values", async () => {
      const onSubmit = vi.fn().mockResolvedValue(undefined);
      const { result } = renderHook(() =>
        useFormState({
          initialValues: { name: "Test" },
          onSubmit,
        })
      );

      const mockEvent = {
        preventDefault: vi.fn(),
      } as unknown as React.FormEvent<HTMLFormElement>;

      await act(async () => {
        await result.current.handleSubmit(mockEvent);
      });

      expect(onSubmit).toHaveBeenCalledWith({ name: "Test" });
    });

    it("should not submit if validation fails", async () => {
      const onSubmit = vi.fn();
      const validate = vi.fn(() => ({ name: "Name is required" }));

      const { result } = renderHook(() =>
        useFormState({
          initialValues: { name: "" },
          validate,
          onSubmit,
        })
      );

      const mockEvent = {
        preventDefault: vi.fn(),
      } as unknown as React.FormEvent<HTMLFormElement>;

      await act(async () => {
        await result.current.handleSubmit(mockEvent);
      });

      expect(validate).toHaveBeenCalled();
      expect(onSubmit).not.toHaveBeenCalled();
      expect(result.current.errors.name).toBe("Name is required");
    });

    it("should set isSubmitting during async submission", async () => {
      let resolveSubmit: (() => void) | undefined;
      const onSubmit = vi.fn(
        () =>
          new Promise<void>((resolve) => {
            resolveSubmit = resolve;
          })
      );

      const { result } = renderHook(() =>
        useFormState({
          initialValues: { name: "Test" },
          onSubmit,
        })
      );

      const mockEvent = {
        preventDefault: vi.fn(),
      } as unknown as React.FormEvent<HTMLFormElement>;

      // Start submission (don't await yet)
      const submitPromise = result.current.handleSubmit(mockEvent);

      // Wait a tiny bit to ensure state has updated
      await act(async () => {
        await new Promise((resolve) => setTimeout(resolve, 10));
      });

      // Check isSubmitting is true during submission
      expect(result.current.isSubmitting).toBe(true);

      // Resolve the promise
      if (resolveSubmit) {
        resolveSubmit();
      }
      await act(async () => {
        await submitPromise;
      });

      // Check isSubmitting is false after submission
      expect(result.current.isSubmitting).toBe(false);
    });

    it("should handle submission errors", async () => {
      const onSubmit = vi.fn().mockRejectedValue(new Error("Submission failed"));
      const consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(() => {});
      
      const { result } = renderHook(() =>
        useFormState({
          initialValues: { name: "Test" },
          onSubmit,
        })
      );

      const mockEvent = {
        preventDefault: vi.fn(),
      } as unknown as React.FormEvent<HTMLFormElement>;

      await act(async () => {
        try {
          await result.current.handleSubmit(mockEvent);
        } catch {
          // Expected to throw/reject
        }
      });

      // Wait for state updates
      await waitFor(() => {
        expect(result.current.isSubmitting).toBe(false);
      });

      expect(consoleErrorSpy).toHaveBeenCalled();
      expect(result.current.errors.submit).toBe("Failed to submit. Please try again.");
      
      consoleErrorSpy.mockRestore();
    });

    it("should clear submit error on value change", async () => {
      const onSubmit = vi.fn().mockRejectedValue(new Error("Submission failed"));
      const consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(() => {});
      
      const { result } = renderHook(() =>
        useFormState({
          initialValues: { name: "Test" },
          onSubmit,
        })
      );

      const mockEvent = {
        preventDefault: vi.fn(),
      } as unknown as React.FormEvent<HTMLFormElement>;

      await act(async () => {
        try {
          await result.current.handleSubmit(mockEvent);
        } catch {
          // Expected to throw/reject
        }
      });

      // Wait for error to be set
      await waitFor(() => {
        expect(result.current.errors.submit).toBe("Failed to submit. Please try again.");
      });

      act(() => {
        result.current.setValue("name", "New Test");
      });

      // Error should be cleared when value changes (submit error is cleared in setValue)
      await waitFor(() => {
        expect(result.current.errors.submit).toBeUndefined();
      });
      
      consoleErrorSpy.mockRestore();
    });
  });
});
