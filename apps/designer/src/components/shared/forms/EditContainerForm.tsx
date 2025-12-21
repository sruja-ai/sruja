// apps/designer/src/components/shared/forms/EditContainerForm.tsx
// Refactored to use Mantine form components

import { useEffect, useRef, useMemo } from "react";
import { useArchitectureStore } from "../../../stores";
import type { ElementDump } from "@sruja/shared";
import { SidePanel } from "../SidePanel";
import { Button, Select, Checkbox } from "@sruja/ui";
import { FormField, useFormState, type FormErrors } from "./";
import { slugify } from "./utils";
import "../EditForms.css";

interface EditContainerFormProps {
  isOpen: boolean;
  onClose: () => void;
  container?: ElementDump;
  parentSystemId?: string | null;
  initialName?: string;
}

interface FormValues {
  name: string;
  technology: string;
  description: string;
  customId: boolean;
  idInput: string;
  selectedSystemId: string;
}

export function EditContainerForm({ isOpen, onClose, container, parentSystemId, initialName }: EditContainerFormProps) {
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const data = useArchitectureStore((s) => s.likec4Model);
  const formRef = useRef<HTMLFormElement>(null);

  const allElements = useMemo(() => Object.values(data?.elements || {}) as any[], [data?.elements]);
  const systems = useMemo(() => allElements.filter((e: any) => e.kind === "system"), [allElements]);


  // Initialize form state
  const form = useFormState<FormValues>({
    initialValues: {
      name: (container as any)?.title || initialName || "",
      technology: (container as any)?.technology || "",
      description: typeof (container as any)?.description === 'string' ? (container as any).description : ((container as any)?.description?.txt || ""),
      customId: false,
      idInput: (container as any)?.id || "",
      selectedSystemId: parentSystemId || ((container as any)?.id?.includes(".") ? (container as any).id.split(".")[0] : "") || "",
    },
    validate: (values) => {
      const errors: FormErrors = {};
      if (!values.name.trim()) errors.name = "Name is required";
      if (!container && !values.selectedSystemId) errors.selectedSystemId = "Parent System is required";
      if (values.customId && !values.idInput.trim()) errors.idInput = "ID is required";

      if (values.customId && values.idInput.trim() && !container && values.selectedSystemId) {
        // Check collision
        const fullId = `${values.selectedSystemId}.${values.idInput.trim()}`;
        if (data?.elements?.[fullId]) {
          errors.idInput = "ID already exists in this system";
        }
      }
      return errors;
    },
    onSubmit: async (values) => {
      await updateArchitecture((model) => {
        const newElements = { ...model.elements };

        let targetId = (container as any)?.id;

        if (!container) {
          const baseId = values.customId ? values.idInput : (slugify(values.name) || "container");
          if (!values.selectedSystemId) return model;
          targetId = `${values.selectedSystemId}.${baseId}` as any;
          // Ensure unique?
          let i = 1;
          const originalId = targetId;
          while (newElements[targetId as string]) {
            targetId = `${originalId}-${i++}` as any;
          }
        }

        if (!targetId) return model;

        newElements[targetId as string] = {
          id: targetId as any,
          kind: "container",
          title: values.name,
          description: (typeof values.description === "string" ? values.description : ((values.description as any) && typeof (values.description as any) === "object" && "txt" in (values.description as any) ? (values.description as any).txt : undefined)) as any,
          technology: values.technology || undefined,
          tags: (container as any)?.tags,
          links: (container as any)?.links,
          style: {} as any,
        };

        return { ...model, elements: newElements };
      });
      onClose();
    },
  });

  // Reset form when opening/switching contexts
  useEffect(() => {
    if (isOpen) {
      form.setValues({
        name: (container as any)?.title || initialName || "",
        technology: (container as any)?.technology || "",
        description: typeof (container as any)?.description === "string" ? (container as any).description : ((container as any)?.description && typeof (container as any).description === "object" && "txt" in (container as any).description ? (container as any).description.txt : "") || "",
        idInput: (container as any)?.id || "",
        customId: false,
        selectedSystemId: parentSystemId || ((container as any)?.id?.includes(".") ? (container as any).id.split(".")[0] : "") || "",
      });
      form.clearErrors();
    }
  }, [isOpen, container, parentSystemId, initialName]); // eslint-disable-line react-hooks/exhaustive-deps

  return (
    <SidePanel
      isOpen={isOpen}
      onClose={onClose}
      title={container ? "Edit Container" : "Add Container"}
      size="lg"
      footer={
        <>
          <Button variant="secondary" onClick={onClose} type="button">Cancel</Button>
          <Button variant="primary" type="submit" form="edit-container-form" isLoading={form.isSubmitting}>
            {container ? "Update" : "Create"}
          </Button>
        </>
      }
    >
      <form ref={formRef} id="edit-container-form" onSubmit={form.handleSubmit} className="edit-form">
        {!container && (
          <Select
            label="Parent System *"
            value={form.values.selectedSystemId}
            onChange={(value) => form.setValue("selectedSystemId", value || "")}
            disabled={!!parentSystemId}
            placeholder="Select System"
            error={form.errors.selectedSystemId}
            data={systems.map(s => ({ value: s.id, label: s.title || s.id }))}
          />
        )}

        <FormField
          label="Name"
          name="name"
          value={form.values.name}
          onChange={(value) => form.setValue("name", value)}
          required
          error={form.errors.name}
        />

        <FormField
          label="Technology"
          name="technology"
          value={form.values.technology}
          onChange={(value) => form.setValue("technology", value)}
          placeholder="e.g. Docker, Go, React"
        />

        <FormField
          label="Description"
          name="description"
          value={form.values.description}
          onChange={(value) => form.setValue("description", value)}
          type="textarea"
          rows={3}
        />

        {!container && (
          <>
            <Checkbox
              id="container-custom-id"
              label="Set custom ID"
              checked={form.values.customId}
              onChange={(e) => form.setValue("customId", e.currentTarget.checked)}
            />
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
