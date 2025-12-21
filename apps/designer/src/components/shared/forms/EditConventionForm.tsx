// apps/designer/src/components/shared/forms/EditConventionForm.tsx
// Refactored to use Mantine form components

import { useEffect, useRef } from "react";
import { useArchitectureStore } from "../../../stores";
import { Button } from "@sruja/ui";
import { SidePanel } from "../SidePanel";
import { FormField, useFormState, type FormErrors } from "./";
import "../EditForms.css";

interface EditConventionFormProps {
  isOpen: boolean;
  onClose: () => void;
  convention?: any; // ConventionDump
  conventionIndex?: number;
}

interface FormValues {
  key: string;
  value: string;
}

export function EditConventionForm({
  isOpen,
  onClose,
  convention,
  conventionIndex,
}: EditConventionFormProps) {
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const formRef = useRef<HTMLFormElement>(null);

  // Initialize form state
  const form = useFormState<FormValues>({
    initialValues: {
      key: convention?.key || "",
      value: convention?.value || "",
    },
    validate: (values) => {
      const errors: FormErrors = {};
      if (!values.key.trim()) errors.key = "Key is required";
      if (!values.value.trim()) errors.value = "Value is required";
      return errors;
    },
    onSubmit: async (values) => {
      await updateArchitecture((model) => {
        const sruja = (model as any).sruja || {};
        const conventions = [...(sruja.conventions || [])];

        const newConvention = {
          key: values.key.trim(),
          value: values.value.trim(),
        };

        if (convention && conventionIndex !== undefined) {
          conventions[conventionIndex] = newConvention;
        } else {
          conventions.push(newConvention);
        }

        return {
          ...model,
          sruja: {
            ...sruja,
            conventions,
          },
        };
      });
      onClose();
    },
  });

  // Reset form when opening/switching contexts
  useEffect(() => {
    if (isOpen) {
      form.setValues({
        key: convention?.key || "",
        value: convention?.value || "",
      });
      form.clearErrors();
    }
  }, [isOpen, convention]); // eslint-disable-line react-hooks/exhaustive-deps

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
      title={convention ? "Edit Convention" : "Add Convention"}
      size="md"
      footer={
        <>
          <Button variant="secondary" onClick={onClose} type="button">
            Cancel
          </Button>
          <Button variant="primary" type="submit" form="edit-convention-form" isLoading={form.isSubmitting}>
            {convention ? "Update" : "Add"}
          </Button>
        </>
      }
    >
      <form ref={formRef} id="edit-convention-form" onSubmit={form.handleSubmit} className="edit-form">
        <FormField
          label="Key"
          name="key"
          value={form.values.key}
          onChange={(value) => form.setValue("key", value)}
          required
          placeholder="e.g., naming, versioning, documentation"
          error={form.errors.key}
        />
        <FormField
          label="Value"
          name="value"
          value={form.values.value}
          onChange={(value) => form.setValue("value", value)}
          required
          placeholder="Convention value"
          error={form.errors.value}
        />
        {form.errors.submit && (
          <div className="text-sm text-[var(--color-error-500)] mt-2">{form.errors.submit}</div>
        )}
      </form>
    </SidePanel>
  );
}
