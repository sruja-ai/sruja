// apps/designer/src/components/shared/forms/EditPersonForm.tsx
// Refactored to use Mantine form components

import { useRef } from "react";
import { slugify } from "../../../utils/slugify";
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
import "../EditForms.css";

interface EditPersonFormProps {
  isOpen: boolean;
  onClose: () => void;
  person?: ElementDump;
  initialName?: string;
}

interface FormValues {
  name: string;
  description: string;
  customId: boolean;
  idInput: string;
  isExternal: boolean;
}

export function EditPersonForm({ isOpen, onClose, person, initialName }: EditPersonFormProps) {
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const data = useArchitectureStore((s) => s.model);
  const formRef = useRef<HTMLFormElement>(null);

  // Initialize form state
  const form = useFormState<FormValues>({
    initialValues: {
      name: person?.title || initialName || "",
      description: extractDescription(person),
      customId: false,
      idInput: person?.id || "",
      isExternal: person?.tags?.includes("external") || false,
    },
    validate: (values) => {
      const errors: FormErrors = {};
      if (!values.name.trim()) errors.name = "Name is required";
      if (values.customId && !values.idInput.trim())
        errors.idInput = "ID is required for custom ID";
      if (values.customId && values.idInput.trim() && !person) {
        if (data?.elements?.[values.idInput.trim()]) errors.idInput = "ID already exists";
      }
      return errors;
    },
    onSubmit: async (values) => {
      await updateArchitecture((model) => {
        const newElements = { ...model.elements };

        let targetId = person?.id;

        if (!person) {
          targetId = values.customId ? values.idInput.trim() : slugify(values.name) || "person";
          let i = 1;
          const originalId = targetId;
          while (newElements[targetId as string]) {
            targetId = `${originalId}-${i++}`;
          }
        }

        if (!targetId) return model;

        const tags = values.isExternal ? ["external"] : [];

        newElements[targetId] = {
          id: targetId,
          kind: "person",
          title: values.name,
          description: values.description || undefined,
          tags: tags.length > 0 ? tags : undefined,
          links: person?.links,
          style: {},
        };

        if (person && model.elements && model.elements[person.id]) {
          newElements[targetId] = {
            ...(model.elements[person.id] as ElementDump),
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
      name: person?.title || initialName || "",
      description: extractDescription(person),
      idInput: person?.id || "",
      customId: false,
      isExternal: person?.tags?.includes("external") || false,
    },
    [person, initialName]
  );

  return (
    <SidePanel
      isOpen={isOpen}
      onClose={onClose}
      title={person ? "Edit Person" : "Add Person"}
      size="lg"
      footer={
        <>
          <Button variant="secondary" onClick={onClose} type="button">
            Cancel
          </Button>
          <Button
            variant="primary"
            type="submit"
            form="edit-person-form"
            isLoading={form.isSubmitting}
          >
            {person ? "Update" : "Create"}
          </Button>
        </>
      }
    >
      <form ref={formRef} id="edit-person-form" onSubmit={form.handleSubmit} className="edit-form">
        <NameField
          value={form.values.name}
          onChange={(value) => form.setValue("name", value)}
          error={form.errors.name}
        />

        <DescriptionField
          value={form.values.description}
          onChange={(value) => form.setValue("description", value)}
        />

        <div className="form-group checkbox-row">
          <input
            id="person-external"
            type="checkbox"
            checked={form.values.isExternal}
            onChange={(e) => form.setValue("isExternal", e.target.checked)}
          />
          <label htmlFor="person-external">External Actor</label>
        </div>

        {!person && (
          <CustomIdField
            useCustomId={form.values.customId}
            onUseCustomIdChange={(checked) => form.setValue("customId", checked)}
            idValue={form.values.idInput}
            onIdChange={(value) => form.setValue("idInput", value)}
            error={form.errors.idInput}
            options={{
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
