// apps/designer/src/components/shared/forms/EditDataStoreForm.tsx
// Refactored to use Mantine form components

import { useEffect, useRef, useMemo } from "react";
import { useArchitectureStore } from "../../../stores";
import type { ElementDump } from "@sruja/shared";
import { SidePanel } from "../SidePanel";
import { Button, Select, Checkbox } from "@sruja/ui";
import { FormField, useFormState, type FormErrors } from "./";
import { slugify } from "../../../utils/slugify";
import "../EditForms.css";

interface EditDataStoreFormProps {
  isOpen: boolean;
  onClose: () => void;
  dataStore?: ElementDump;
  parentSystemId?: string | null;
  initialName?: string;
}

interface FormValues {
  name: string;
  description: string;
  technology: string;
  customId: boolean;
  idInput: string;
  selectedSystemId: string;
}

export function EditDataStoreForm({
  isOpen,
  onClose,
  dataStore,
  parentSystemId,
  initialName,
}: EditDataStoreFormProps) {
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const data = useArchitectureStore((s) => s.model);
  const formRef = useRef<HTMLFormElement>(null);

  const allElements = useMemo(
    () => Object.values(data?.elements || {}) as ElementDump[],
    [data?.elements]
  );
  const systems = useMemo(() => allElements.filter((e) => e.kind === "system"), [allElements]);

  // Initialize form state
  const form = useFormState<FormValues>({
    initialValues: {
      name: dataStore?.title || initialName || "",
      technology: dataStore?.technology || "",
      description:
        typeof dataStore?.description === "string"
          ? dataStore.description
          : (dataStore?.description as unknown as { txt: string })?.txt || "",
      customId: false,
      idInput: dataStore?.id || "",
      selectedSystemId: parentSystemId || (dataStore?.id ? dataStore.id.split(".")[0] : "") || "",
    },
    validate: (values) => {
      const errors: FormErrors = {};
      if (!values.name.trim()) errors.name = "Name is required";
      if (!dataStore && !values.selectedSystemId) {
        errors.selectedSystemId = "Parent System is required";
      }
      if (values.customId && !values.idInput.trim()) errors.idInput = "ID is required";

      if (values.customId && values.idInput.trim() && !dataStore && values.selectedSystemId) {
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

        let targetId = dataStore?.id;

        if (!dataStore) {
          const baseId = values.customId ? values.idInput : slugify(values.name) || "db";
          if (!values.selectedSystemId) return model;
          targetId = `${values.selectedSystemId}.${baseId}`;
          let i = 1;
          const originalId = targetId;
          while (newElements[targetId as string]) {
            targetId = `${originalId}-${i++}`;
          }
        }

        if (!targetId) return model;

        const tags = dataStore?.tags ? [...dataStore.tags] : [];
        if (!tags.includes("database")) tags.push("database");

        newElements[targetId as string] = {
          id: targetId,
          kind: "container",
          title: values.name,
          description: typeof values.description === "string" ? values.description : undefined,
          technology: values.technology || undefined,
          tags: tags,
          links: dataStore?.links,
          style: {},
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
        name: dataStore?.title || initialName || "",
        technology: dataStore?.technology || "",
        description:
          (typeof dataStore?.description === "string"
            ? dataStore.description
            : (dataStore?.description as unknown as { txt: string })?.txt || "") || "",
        idInput: dataStore?.id || "",
        customId: false,
        selectedSystemId: parentSystemId || (dataStore?.id ? dataStore.id.split(".")[0] : "") || "",
      });
      form.clearErrors();
    }
  }, [isOpen, dataStore, parentSystemId, initialName]); // eslint-disable-line react-hooks/exhaustive-deps

  return (
    <SidePanel
      isOpen={isOpen}
      onClose={onClose}
      title={dataStore ? "Edit Datastore" : "Add Datastore"}
      size="lg"
      footer={
        <>
          <Button variant="secondary" onClick={onClose} type="button">
            Cancel
          </Button>
          <Button
            variant="primary"
            type="submit"
            form="edit-datastore-form"
            isLoading={form.isSubmitting}
          >
            {dataStore ? "Update" : "Create"}
          </Button>
        </>
      }
    >
      <form
        ref={formRef}
        id="edit-datastore-form"
        onSubmit={form.handleSubmit}
        className="edit-form"
      >
        {!dataStore && (
          <Select
            label="Parent System *"
            value={form.values.selectedSystemId}
            onChange={(value) => form.setValue("selectedSystemId", value || "")}
            disabled={!!parentSystemId}
            placeholder="Select System"
            error={form.errors.selectedSystemId}
            data={systems.map((s) => ({ value: s.id, label: s.title || s.id }))}
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
          placeholder="e.g. PostgreSQL, Redis"
        />

        <FormField
          label="Description"
          name="description"
          value={form.values.description}
          onChange={(value) => form.setValue("description", value)}
          type="textarea"
          rows={3}
        />

        {!dataStore && (
          <>
            <Checkbox
              id="ds-custom-id"
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

        {form.errors.submit && (
          <div className="text-red-500 text-sm mt-2">{form.errors.submit}</div>
        )}
      </form>
    </SidePanel>
  );
}
