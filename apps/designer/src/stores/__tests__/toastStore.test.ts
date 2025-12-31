// apps/designer/src/stores/__tests__/toastStore.test.ts
import { describe, it, expect, beforeEach, vi } from "vitest";
import { useToastStore } from "../toastStore";
import type { ToastType } from "@sruja/ui";

describe("toastStore", () => {
  beforeEach(() => {
    // Clear all toasts before each test
    useToastStore.getState().clearToasts();
  });

  it("should initialize with empty toasts array", () => {
    const state = useToastStore.getState();
    expect(state.toasts).toEqual([]);
  });

  it("should add a toast with default type and duration", () => {
    useToastStore.getState().showToast("Test message");

    const state = useToastStore.getState();
    expect(state.toasts).toHaveLength(1);
    expect(state.toasts[0].message).toBe("Test message");
    expect(state.toasts[0].type).toBe("info");
    expect(state.toasts[0].duration).toBe(5000);
    expect(state.toasts[0].id).toBeDefined();
  });

  it("should add a toast with custom type", () => {
    const types: ToastType[] = ["info", "success", "warning", "error"];

    types.forEach((type) => {
      useToastStore.getState().clearToasts();
      useToastStore.getState().showToast("Test message", type);

      const state = useToastStore.getState();
      expect(state.toasts[0].type).toBe(type);
    });
  });

  it("should add a toast with custom duration", () => {
    useToastStore.getState().showToast("Test message", "info", 10000);

    const state = useToastStore.getState();
    expect(state.toasts[0].duration).toBe(10000);
  });

  it("should generate unique IDs for each toast", () => {
    useToastStore.getState().showToast("Message 1");
    useToastStore.getState().showToast("Message 2");
    useToastStore.getState().showToast("Message 3");

    const state = useToastStore.getState();
    const ids = state.toasts.map((t) => t.id);
    const uniqueIds = new Set(ids);

    expect(uniqueIds.size).toBe(3);
    expect(ids.length).toBe(3);
  });

  it("should remove a toast by ID", () => {
    useToastStore.getState().showToast("Message 1");
    useToastStore.getState().showToast("Message 2");
    useToastStore.getState().showToast("Message 3");

    const state = useToastStore.getState();
    const toastToRemove = state.toasts[1];
    const toastId = toastToRemove.id;

    useToastStore.getState().removeToast(toastId);

    const newState = useToastStore.getState();
    expect(newState.toasts).toHaveLength(2);
    expect(newState.toasts.find((t) => t.id === toastId)).toBeUndefined();
  });

  it("should handle removing non-existent toast gracefully", () => {
    useToastStore.getState().showToast("Message 1");

    useToastStore.getState().removeToast("non-existent-id");

    const state = useToastStore.getState();
    expect(state.toasts).toHaveLength(1);
  });

  it("should clear all toasts", () => {
    useToastStore.getState().showToast("Message 1");
    useToastStore.getState().showToast("Message 2");
    useToastStore.getState().showToast("Message 3");

    expect(useToastStore.getState().toasts).toHaveLength(3);

    useToastStore.getState().clearToasts();

    expect(useToastStore.getState().toasts).toHaveLength(0);
  });

  it("should maintain toast order", () => {
    useToastStore.getState().showToast("First");
    useToastStore.getState().showToast("Second");
    useToastStore.getState().showToast("Third");

    const state = useToastStore.getState();
    expect(state.toasts[0].message).toBe("First");
    expect(state.toasts[1].message).toBe("Second");
    expect(state.toasts[2].message).toBe("Third");
  });

  it("should handle multiple toasts with same message", () => {
    useToastStore.getState().showToast("Same message");
    useToastStore.getState().showToast("Same message");
    useToastStore.getState().showToast("Same message");

    const state = useToastStore.getState();
    expect(state.toasts).toHaveLength(3);
    state.toasts.forEach((toast) => {
      expect(toast.message).toBe("Same message");
    });
  });

  it("should handle empty message", () => {
    useToastStore.getState().showToast("");

    const state = useToastStore.getState();
    expect(state.toasts).toHaveLength(1);
    expect(state.toasts[0].message).toBe("");
  });

  it("should handle very long messages", () => {
    const longMessage = "a".repeat(10000);
    useToastStore.getState().showToast(longMessage);

    const state = useToastStore.getState();
    expect(state.toasts[0].message).toBe(longMessage);
  });
});
