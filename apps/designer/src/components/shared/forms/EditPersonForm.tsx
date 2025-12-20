// apps/designer/src/components/shared/forms/EditPersonForm.tsx
// Refactored to use Mantine form components

import { useEffect, useRef } from "react";
import { useArchitectureStore } from "../../../stores";
import type { ElementDump } from "@sruja/shared";
import { SidePanel } from "../SidePanel";
import { Button } from "@sruja/ui";
import { FormField, useFormState, type FormErrors } from "./";
import { collectIds, generateUniqueId } from "./utils";
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
  const data = useArchitectureStore((s) => s.likec4Model);
  const formRef = useRef<HTMLFormElement>(null);


  // Initialize form state
  const form = useFormState<FormValues>({
    initialValues: {
      name: (person as any)?.title || initialName || "",
      description: typeof (person as any)?.description === 'string' ? (person as any).description : ((person as any)?.description?.txt || ""),
      customId: false,
      idInput: (person as any)?.id || "",
      isExternal: (person as any)?.tags?.includes("external") || false,
    },
    validate: (values) => {
      const errors: FormErrors = {};
      if (!values.name.trim()) errors.name = "Name is required";
      if (values.customId && !values.idInput.trim()) errors.idInput = "ID is required for custom ID";
      if (values.customId && values.idInput.trim() && !person) {
        const ids = collectIds(data);
        if (ids.has(values.idInput.trim())) errors.idInput = "ID already exists";
      }
      return errors;
    },
    onSubmit: async (values) => {
      await updateArchitecture((model) => {
        const newElements = { ...model.elements };

        let targetId = (person as any)?.id;

        if (!person) {
          targetId = (values.customId ? values.idInput : "") || generateUniqueId(values.name, data, "person");
        }

        if (!targetId) return model;

        const tags = values.isExternal ? ["external"] : [];

        newElements[targetId] = {
          id: targetId as any,
          kind: "person",
          title: values.name,
          description: (values.description || undefined) as any,
          tags: tags.length > 0 ? tags : undefined,
          links: (person as any)?.links
        };

        if (person && model.elements && model.elements[(person as any).id]) {
          newElements[targetId] = {
            ...(model.elements[(person as any).id] as any),
            title: values.name,
            description: (values.description || undefined) as any,
            tags: tags.length > 0 ? tags : undefined
          };
        }

        return { ...model, elements: newElements };
      });
      onClose();
    },
  });

  // Reset form when opening/switching contexts
  useEffect(() => {
    if (isOpen) {
      form.setValues({
        name: (person as any)?.title || initialName || "",
        description: typeof (person as any)?.description === "string" ? (person as any).description : ((person as any)?.description?.txt || ""),
        idInput: (person as any)?.id || "",
        customId: false,
        isExternal: (person as any)?.tags?.includes("external") || false,
      });
      form.clearErrors();
    }
  }, [isOpen, person, initialName]); // eslint-disable-line react-hooks/exhaustive-deps

  return (
    <SidePanel
      isOpen={isOpen}
      onClose={onClose}
      title={person ? "Edit Person" : "Add Person"}
      size="lg"
      footer={
        <>
          <Button variant="secondary" onClick={onClose} type="button">Cancel</Button>
          <Button variant="primary" type="submit" form="edit-person-form" isLoading={form.isSubmitting}>
            {person ? "Update" : "Create"}
          </Button>
        </>
      }
    >
      <form ref={formRef} id="edit-person-form" onSubmit={form.handleSubmit} className="edit-form">
        <FormField
          label="Name"
          name="name"
          value={form.values.name}
          onChange={(value) => form.setValue("name", value)}
          required
          error={form.errors.name}
        />

        <FormField
          label="Description"
          name="description"
          value={form.values.description}
          onChange={(value) => form.setValue("description", value)}
          type="textarea"
          rows={3}
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
          <>
            <div className="form-group checkbox-row">
              <input
                id="person-custom-id"
                type="checkbox"
                checked={form.values.customId}
                onChange={e => form.setValue("customId", e.target.checked)}
              />
              <label htmlFor="person-custom-id">Set custom ID</label>
            </div>
            {form.values.customId && (
              <FormField
                label="ID"
                name="idInput"
                value={form.values.idInput}
                onChange={(value) => form.setValue("idInput", value)}
                required
                error={form.errors.idInput}
              />
            )}
          </>
        )}

        {form.errors.submit && <div className="text-red-500 text-sm mt-2">{form.errors.submit}</div>}
      </form>
    </SidePanel>
  );
}
