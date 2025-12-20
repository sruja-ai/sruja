// apps/designer/src/components/shared/forms/EditPolicyForm.tsx
// Refactored to use Mantine form components

import { useEffect, useRef } from "react";
import { useArchitectureStore } from "../../../stores";
import { Button } from "@sruja/ui";
import { SidePanel } from "../SidePanel";
import { FormField, useFormState, type FormErrors } from "./";
import "../EditForms.css";

interface EditPolicyFormProps {
  isOpen: boolean;
  onClose: () => void;
  policy?: any; // PolicyDump
}

interface FormValues {
  id: string;
  label: string;
  description: string;
  category: string;
  enforcement: string;
}

export function EditPolicyForm({ isOpen, onClose, policy }: EditPolicyFormProps) {
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const formRef = useRef<HTMLFormElement>(null);

  // Initialize form state
  const form = useFormState<FormValues>({
    initialValues: {
      id: policy?.id || "",
      label: policy?.title || policy?.label || "",
      description: policy?.description || "",
      category: policy?.category || "",
      enforcement: policy?.enforcement || "",
    },
    validate: (values) => {
      const errors: FormErrors = {};
      if (!values.id.trim()) {
        errors.id = "ID is required";
      } else if (!/^[A-Za-z0-9_-]+$/.test(values.id.trim())) {
        errors.id = "ID can only contain letters, numbers, hyphens, and underscores";
      }
      if (!values.label.trim() && !values.description.trim()) {
        errors.label = "Label or description is required";
      }
      return errors;
    },
    onSubmit: async (values) => {
      await updateArchitecture((model) => {
        const sruja = (model as any).sruja || {};
        const policies = [...(sruja.policies || [])];

        const newPolicy = {
          id: values.id.trim(),
          title: values.label.trim() || undefined,
          description: values.description.trim() || undefined,
          category: values.category.trim() || undefined,
          enforcement: values.enforcement.trim() || undefined,
        };

        if (policy) {
          const index = policies.findIndex((p: any) => p.id === policy.id);
          if (index >= 0) {
            policies[index] = newPolicy;
          }
        } else {
          policies.push(newPolicy);
        }

        return {
          ...model,
          sruja: {
            ...sruja,
            policies,
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
        id: policy?.id || "",
        label: policy?.title || policy?.label || "",
        description: policy?.description || "",
        category: policy?.category || "",
        enforcement: policy?.enforcement || "",
      });
      form.clearErrors();
    }
  }, [isOpen, policy]); // eslint-disable-line react-hooks/exhaustive-deps

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
      title={policy ? "Edit Policy" : "Add Policy"}
      size="md"
      footer={
        <>
          <Button variant="secondary" onClick={onClose} type="button">
            Cancel
          </Button>
          <Button variant="primary" type="submit" form="edit-policy-form" isLoading={form.isSubmitting}>
            {policy ? "Update" : "Add"}
          </Button>
        </>
      }
    >
      <form ref={formRef} id="edit-policy-form" onSubmit={form.handleSubmit} className="edit-form">
        <FormField
          label="ID"
          name="id"
          value={form.values.id}
          onChange={(value) => form.setValue("id", value)}
          required
          placeholder="SecurityPolicy"
          error={form.errors.id}
        />
        <FormField
          label="Label"
          name="label"
          value={form.values.label}
          onChange={(value) => form.setValue("label", value)}
          placeholder="Policy label"
          error={form.errors.label}
        />
        <FormField
          label="Description"
          name="description"
          value={form.values.description}
          onChange={(value) => form.setValue("description", value)}
          type="textarea"
          rows={3}
          placeholder="Policy description"
        />
        <FormField
          label="Category"
          name="category"
          value={form.values.category}
          onChange={(value) => form.setValue("category", value)}
          placeholder="e.g., security, compliance, performance"
        />
        <FormField
          label="Enforcement"
          name="enforcement"
          value={form.values.enforcement}
          onChange={(value) => form.setValue("enforcement", value)}
          placeholder="e.g., required, recommended, optional"
        />
        {form.errors.submit && (
          <div className="text-sm text-[var(--color-error-500)] mt-2">{form.errors.submit}</div>
        )}
      </form>
    </SidePanel>
  );
}
