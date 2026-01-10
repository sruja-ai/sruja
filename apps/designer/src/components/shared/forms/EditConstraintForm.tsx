// apps/designer/src/components/shared/forms/EditConstraintForm.tsx
// Refactored to use Mantine form components

import { useEffect, useRef } from "react";
import { useArchitectureStore } from "../../../stores";
import type { Constraint, ConstraintJSON } from "@sruja/shared";
import { Button } from "@sruja/ui";
import { SidePanel } from "../SidePanel";
import { FormField, useFormState, type FormErrors } from "./";
import "../EditForms.css";

interface EditConstraintFormProps {
  isOpen: boolean;
  onClose: () => void;
  constraint?: ConstraintJSON;
  constraintIndex?: number;
}

interface FormValues {
  key: string;
  value: string;
}

export function EditConstraintForm({
  isOpen,
  onClose,
  constraint,
  constraintIndex,
}: EditConstraintFormProps) {
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const formRef = useRef<HTMLFormElement>(null);

  // Initialize form state
  const form = useFormState<FormValues>({
    initialValues: {
      key: constraint?.key || "",
      value: constraint?.value || "",
    },
    validate: (values) => {
      const errors: FormErrors = {};
      if (!values.key.trim()) errors.key = "Key is required";
      if (!values.value.trim()) errors.value = "Value is required";
      return errors;
    },
    onSubmit: async (values) => {
      await updateArchitecture((model) => {
        const sruja = model.sruja || {};
        const constraints = [...(sruja.constraints || [])] as Constraint[];

        const newConstraint: Constraint = {
          id: values.key.trim(),
          description: values.value.trim(),
        };

        if (constraint && constraintIndex !== undefined) {
          constraints[constraintIndex] = newConstraint;
        } else {
          constraints.push(newConstraint);
        }

        return {
          ...model,
          sruja: {
            ...sruja,
            constraints,
          },
        };
      });
      onClose();
    },
  });

  // Reset form when opening/switching contexts
  // Note: form.setValues and form.clearErrors are stable (wrapped in useCallback with empty deps)
  // so they don't need to be in the dependency array
  useEffect(() => {
    if (isOpen) {
      form.setValues({
        key: constraint?.key || "",
        value: constraint?.value || "",
      });
      form.clearErrors();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
    // form.setValues and form.clearErrors are stable callbacks from useFormState
  }, [isOpen, constraint]);

  // Handle Escape key
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && isOpen) {
        onClose();
      }
    };
    if (isOpen) {
      document.addEventListener("keydown", handleKeyDown);
      return () => document.removeEventListener("keydown", handleKeyDown);
    }
  }, [isOpen, onClose]);

  return (
    <SidePanel
      isOpen={isOpen}
      onClose={onClose}
      title={constraint ? "Edit Constraint" : "Add Constraint"}
      size="md"
      footer={
        <>
          <Button variant="secondary" onClick={onClose} type="button">
            Cancel
          </Button>
          <Button
            variant="primary"
            type="submit"
            form="edit-constraint-form"
            isLoading={form.isSubmitting}
          >
            {constraint ? "Update" : "Add"}
          </Button>
        </>
      }
    >
      <form
        ref={formRef}
        id="edit-constraint-form"
        onSubmit={form.handleSubmit}
        className="edit-form"
      >
        <FormField
          label="Key"
          name="key"
          value={form.values.key}
          onChange={(value) => form.setValue("key", value)}
          required
          placeholder="e.g., maxLatency, minThroughput"
          error={form.errors.key}
        />
        <FormField
          label="Value"
          name="value"
          value={form.values.value}
          onChange={(value) => form.setValue("value", value)}
          required
          placeholder="Constraint value"
          error={form.errors.value}
        />
        {form.errors.submit && (
          <div className="text-sm text-[var(--color-error-500)] mt-2">{form.errors.submit}</div>
        )}
      </form>
    </SidePanel>
  );
}
