// apps/designer/src/components/shared/forms/EditSystemForm.tsx
// Refactored to use Mantine form components and useFormState hook

import { useRef } from "react";
import { useArchitectureStore } from "../../../stores";
import type { ElementDump } from "@sruja/shared";
import { SidePanel } from "../SidePanel";
import { Button } from "@sruja/ui";
import {
  useFormState,
  type FormErrors,
  NameField,
  DescriptionField,
  CustomIdField,
  useFormReset,
  extractDescription,
} from "./";
import { slugify } from "../../../utils/slugify";
import "../EditForms.css";

interface EditSystemFormProps {
  isOpen: boolean;
  onClose: () => void;
  system?: ElementDump;
  initialName?: string;
}

interface FormValues {
  name: string;
  description: string;
  customId: boolean;
  idInput: string;
  isExternal: boolean;
}

export function EditSystemForm({ isOpen, onClose, system, initialName }: EditSystemFormProps) {
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const data = useArchitectureStore((s) => s.model);
  const formRef = useRef<HTMLFormElement>(null);

  // Initialize form state with validation
  const form = useFormState<FormValues>({
    initialValues: {
      name: system?.title || initialName || "",
      description: extractDescription(system),
      customId: false,
      idInput: system?.id || "",
      isExternal: system?.tags?.includes("external") || false,
    },
    validate: (values) => {
      const errors: FormErrors = {};

      if (!values.name.trim()) {
        errors.name = "Name is required";
      }

      if (values.customId && !values.idInput.trim()) {
        errors.idInput = "ID is required";
      }

      if (values.customId && values.idInput.trim() && !system) {
        if (data?.elements?.[values.idInput.trim()]) {
          errors.idInput = "ID already exists";
        }
      }

      return errors;
    },
    onSubmit: async (values) => {
      await updateArchitecture((model) => {
        const newElements = { ...model.elements };

        let targetId = system?.id;

        if (!system) {
          // Create Mode
          targetId = values.customId ? values.idInput.trim() : slugify(values.name) || "system";
          let i = 1;
          const originalId = targetId;
          while (newElements[targetId as string]) {
            targetId = `${originalId}-${i++}`;
          }
        }

        if (!targetId) return model;

        const tags = values.isExternal ? ["external"] : [];

        newElements[targetId as string] = {
          id: targetId,
          kind: "system",
          title: values.name,
          description: values.description || undefined,
          tags: tags.length > 0 ? tags : undefined,
          links: system?.links,
          style: {},
        };

        // If editing, merge existing props?
        if (system && model.elements && model.elements[system.id]) {
          newElements[targetId as string] = {
            ...(model.elements[system.id] as ElementDump),
            title: values.name,
            description: values.description || undefined,
            tags: tags.length > 0 ? tags : undefined,
          };
        }

        return { ...model, elements: newElements };
      });
      onClose();
    },
  });

  // Reset form when opening/switching contexts
  useFormReset(
    form,
    isOpen,
    {
      name: system?.title || initialName || "",
      description: extractDescription(system),
      idInput: system?.id || "",
      customId: false,
      isExternal: system?.tags?.includes("external") || false,
    },
    [system, initialName]
  );

  return (
    <SidePanel
      isOpen={isOpen}
      onClose={onClose}
      title={system ? "Edit System" : "Add System"}
      size="lg"
      footer={
        <>
          <Button variant="secondary" onClick={onClose} type="button">
            Cancel
          </Button>
          <Button
            variant="primary"
            type="submit"
            form="edit-system-form"
            isLoading={form.isSubmitting}
          >
            {system ? "Update" : "Create"}
          </Button>
        </>
      }
    >
      <form ref={formRef} id="edit-system-form" onSubmit={form.handleSubmit} className="edit-form">
        <NameField
          label="System Name"
          value={form.values.name}
          onChange={(value) => form.setValue("name", value)}
          error={form.errors.name}
          placeholder="e.g. Payment Gateway"
        />

        <DescriptionField
          value={form.values.description}
          onChange={(value) => form.setValue("description", value)}
          placeholder="What does this system do?"
        />

        <div className="form-group checkbox-row">
          <input
            id="system-external"
            type="checkbox"
            checked={form.values.isExternal}
            onChange={(e) => form.setValue("isExternal", e.target.checked)}
          />
          <label htmlFor="system-external">External System</label>
        </div>

        {!system && (
          <CustomIdField
            useCustomId={form.values.customId}
            onUseCustomIdChange={(checked) => form.setValue("customId", checked)}
            idValue={form.values.idInput}
            onIdChange={(value) => form.setValue("idInput", value)}
            error={form.errors.idInput}
            options={{
              placeholder: "e.g. payment_gateway",
              checkboxLabel: "Set custom ID",
              inputLabel: "ID",
            }}
          />
        )}

        {form.errors.submit && (
          <div className="text-red-500 text-sm mt-2">{form.errors.submit}</div>
        )}
      </form>
    </SidePanel>
  );
}
