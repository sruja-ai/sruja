// apps/designer/src/components/shared/forms/EditSystemForm.tsx
// Refactored to use Mantine form components and useFormState hook

import { useEffect, useRef } from "react";
import { useArchitectureStore } from "../../../stores";
import type { ElementDump } from "@sruja/shared";
import { SidePanel } from "../SidePanel";
import { Button } from "@sruja/ui";
import { FormField, useFormState, type FormErrors } from "./";
import { collectIds, generateUniqueId } from "./utils";
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
  const data = useArchitectureStore((s) => s.likec4Model);
  const formRef = useRef<HTMLFormElement>(null);


  // Initialize form state with validation
  const form = useFormState<FormValues>({
    initialValues: {
      name: (system as any)?.title || initialName || "",
      description: typeof (system as any)?.description === 'string' ? (system as any).description : ((system as any)?.description?.txt || ""),
      customId: false,
      idInput: (system as any)?.id || "",
      isExternal: (system as any)?.tags?.includes("external") || false,
    },
    validate: (values) => {
      const errors: FormErrors = {};

      if (!values.name.trim()) {
        errors.name = "System name is required";
      }

      if (values.customId && !values.idInput.trim()) {
        errors.idInput = "ID is required when custom ID is checked";
      }

      if (values.customId && values.idInput.trim() && !system) {
        const ids = collectIds(data);
        if (ids.has(values.idInput.trim())) {
          errors.idInput = "ID already exists";
        }
      }

      return errors;
    },
    onSubmit: async (values) => {
      await updateArchitecture((model) => {
        const newElements = { ...model.elements };

        let targetId = (system as any)?.id;

        if (!system) {
          // Create Mode
          targetId = (values.customId ? values.idInput : "") || generateUniqueId(values.name, data, "system");
        }

        if (!targetId) return model; // Should not happen given generateUniqueId fallback

        const tags = values.isExternal ? ["external"] : [];

        newElements[targetId] = {
          id: targetId as any,
          kind: "system",
          title: values.name,
          description: (values.description || undefined) as any,
          tags: tags.length > 0 ? tags : undefined,
          links: (system as any)?.links
          // Preserve other properties if editing?
          // If editing, we should merge.
          // If system exists, we are getting a SystemJSON which is different from ElementDump?
          // Actually system passed to props is SystemJSON from legacy types?
          // Wait, we need to check what SystemJSON is or if we should update Props too.
        };

        // If editing, merge existing props?
        if (system && model.elements && model.elements[(system as any).id]) {
          newElements[targetId] = {
            ...(model.elements[(system as any).id] as any),
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
        name: (system as any)?.title || initialName || "",
        description: typeof (system as any)?.description === "string" ? (system as any).description : ((system as any)?.description?.txt || ""),
        idInput: (system as any)?.id || "",
        customId: false,
        isExternal: (system as any)?.tags?.includes("external") || false,
      });
      form.clearErrors();
    }
  }, [isOpen, system, initialName]); // eslint-disable-line react-hooks/exhaustive-deps




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
      <form
        ref={formRef}
        id="edit-system-form"
        onSubmit={form.handleSubmit}
        className="edit-form"
      >
        <FormField
          label="System Name"
          name="name"
          value={form.values.name}
          onChange={(value) => form.setValue("name", value)}
          required
          placeholder="e.g. Payment Gateway"
          error={form.errors.name}
        />

        <FormField
          label="Description"
          name="description"
          value={form.values.description}
          onChange={(value) => form.setValue("description", value)}
          type="textarea"
          rows={3}
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
          <>
            <div className="form-group checkbox-row">
              <input
                id="system-custom-id"
                type="checkbox"
                checked={form.values.customId}
                onChange={(e) => form.setValue("customId", e.target.checked)}
              />
              <label htmlFor="system-custom-id">Set custom ID</label>
            </div>
            {form.values.customId && (
              <FormField
                label="ID"
                name="idInput"
                value={form.values.idInput}
                onChange={(value) => form.setValue("idInput", value)}
                required
                placeholder="e.g. payment_gateway"
                error={form.errors.idInput}
              />
            )}
          </>
        )}

        {form.errors.submit && (
          <div className="text-red-500 text-sm mt-2">{form.errors.submit}</div>
        )}
      </form>
    </SidePanel>
  );
}
