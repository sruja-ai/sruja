// apps/designer/src/components/shared/forms/EditRequirementForm.tsx
// Refactored to use Mantine form components

import { useEffect, useMemo } from "react";
import { useArchitectureStore } from "../../../stores";
import type { RequirementDump, SrujaModelDump, ElementDump } from "@sruja/shared";
import { Button, Listbox, Select } from "@sruja/ui";
import type { ListOption } from "@sruja/ui";
import { SidePanel } from "../SidePanel";
import { FormField, useFormState, type FormErrors } from "./";
import { REQUIREMENT_TYPES } from "./constants";
import "../EditForms.css";

interface EditRequirementFormProps {
  isOpen: boolean;
  onClose: () => void;
  requirement?: RequirementDump;
  /** Optional: Pre-select element to tag this requirement to */
  preSelectedElementId?: string;
}

interface FormValues {
  id: string;
  type: ListOption | null;
  title: string;
  description: string;
  tags: string[];
}

export function EditRequirementForm({
  isOpen,
  onClose,
  requirement,
  preSelectedElementId,
}: EditRequirementFormProps) {
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const data = useArchitectureStore((s) => s.model);

  // Get all elements for tagging
  const elementOptions = useMemo(() => {
    const elements = data?.elements ? (Object.values(data.elements) as ElementDump[]) : [];
    return elements.map((el) => ({
      value: el.id,
      label: `${el.id}${el.title ? ` (${el.title})` : ""}`,
    }));
  }, [data?.elements]);

  // Initialize form state
  const initialTags = useMemo(() => {
    if (preSelectedElementId) {
      return [preSelectedElementId];
    }
    return (requirement as { tags?: string[] })?.tags || [];
  }, [requirement, preSelectedElementId]);

  const form = useFormState<FormValues>({
    initialValues: {
      id: requirement?.id || "",
      type:
        REQUIREMENT_TYPES.find((t) => t.id === (requirement?.type || "functional")) ||
        REQUIREMENT_TYPES[0],
      title: requirement?.title || "",
      description: requirement?.description || "",
      tags: initialTags,
    },
    validate: (values) => {
      const errors: FormErrors = {};
      if (!values.id.trim()) {
        errors.id = "ID is required";
      } else if (!/^[A-Za-z0-9_-]+$/.test(values.id.trim())) {
        errors.id = "ID can only contain letters, numbers, hyphens, and underscores";
      }
      if (!values.title.trim() && !values.description.trim()) {
        errors.title = "Title or description is required";
      }
      return errors;
    },
    onSubmit: async (values) => {
      await updateArchitecture((model) => {
        const sruja = (model as SrujaModelDump).sruja || {};
        const requirements = [...(sruja.requirements || [])] as RequirementDump[];

        const newRequirement: RequirementDump = {
          id: values.id.trim(),
          type: values.type?.id as RequirementDump["type"],
          title: values.title.trim(),
          description: values.description.trim() || undefined,
          tags: values.tags.length > 0 ? values.tags : undefined,
        };

        if (requirement) {
          const index = requirements.findIndex((r) => r.id === requirement.id);
          if (index >= 0) {
            requirements[index] = newRequirement;
          }
        } else {
          requirements.push(newRequirement);
        }

        return {
          ...model,
          sruja: {
            ...sruja,
            requirements,
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
        id: requirement?.id || "",
        type:
          REQUIREMENT_TYPES.find((t) => t.id === (requirement?.type || "functional")) ||
          REQUIREMENT_TYPES[0],
        title: requirement?.title || "",
        description: requirement?.description || "",
        tags: initialTags,
      });
      form.clearErrors();
    }
  }, [isOpen, requirement]); // eslint-disable-line react-hooks/exhaustive-deps

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
      title={requirement ? "Edit Requirement" : "Add Requirement"}
      size="md"
      footer={
        <>
          <Button variant="secondary" onClick={onClose} type="button">
            Cancel
          </Button>
          <Button
            variant="primary"
            type="submit"
            form="edit-requirement-form"
            isLoading={form.isSubmitting}
          >
            {requirement ? "Update" : "Add"}
          </Button>
        </>
      }
    >
      <form id="edit-requirement-form" onSubmit={form.handleSubmit} className="edit-form">
        <FormField
          name="id"
          label="ID"
          value={form.values.id}
          onChange={(value) => form.setValue("id", value)}
          required
          placeholder="R1"
          error={form.errors.id}
        />
        <Listbox
          label="Type"
          options={REQUIREMENT_TYPES}
          value={form.values.type}
          onChange={(value) => form.setValue("type", value)}
        />
        <FormField
          name="title"
          label="Title"
          value={form.values.title}
          onChange={(value) => form.setValue("title", value)}
          placeholder="Requirement title"
          error={form.errors.title}
        />
        <FormField
          name="description"
          label="Description"
          value={form.values.description}
          onChange={(value) => form.setValue("description", value)}
          type="textarea"
          rows={4}
          placeholder="Requirement description"
        />
        {elementOptions.length > 0 && (
          <div className="form-field">
            <label className="form-label">
              Tag to Architecture Elements <span className="text-muted">(optional)</span>
            </label>
            <p className="form-hint text-sm text-muted mb-2">
              Link this requirement to specific architecture elements (systems, containers,
              components) for traceability
            </p>
            <Select
              label=""
              value={form.values.tags.length > 0 ? form.values.tags[0] : ""}
              onChange={(value) => {
                form.setValue("tags", value ? [value] : []);
              }}
              data={elementOptions.map((opt) => ({ value: opt.value, label: opt.label }))}
              placeholder="Select element to tag..."
            />
            {form.values.tags.length > 0 && (
              <div className="mt-2 text-sm text-muted">
                Tagged to: <code>{form.values.tags[0]}</code>
              </div>
            )}
          </div>
        )}
        {form.errors.submit && (
          <div className="text-sm text-[var(--color-error-500)] mt-2">{form.errors.submit}</div>
        )}
      </form>
    </SidePanel>
  );
}
