// apps/designer/src/components/shared/forms/EditQueueForm.tsx
// Refactored to use Mantine form components

import { useEffect, useRef, useMemo } from "react";
import { useArchitectureStore } from "../../../stores";
import type { ElementDump } from "@sruja/shared";
import { SidePanel } from "../SidePanel";
import { Button, Select, Checkbox } from "@sruja/ui";
import { FormField, useFormState, type FormErrors } from "./";
import { slugify } from "./utils";
import "../EditForms.css";

interface EditQueueFormProps {
  isOpen: boolean;
  onClose: () => void;
  queue?: ElementDump;
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

export function EditQueueForm({
  isOpen,
  onClose,
  queue,
  parentSystemId,
  initialName,
}: EditQueueFormProps) {
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
      name: queue?.title || initialName || "",
      technology: queue?.technology || "",
      description:
        typeof queue?.description === "string"
          ? queue.description
          : (queue?.description as unknown as { txt: string })?.txt || "",
      customId: false,
      idInput: queue?.id || "",
      selectedSystemId: parentSystemId || (queue?.id ? queue.id.split(".")[0] : "") || "",
    },
    validate: (values) => {
      const errors: FormErrors = {};
      if (!values.name.trim()) errors.name = "Name is required";
      if (!queue && !values.selectedSystemId) {
        errors.selectedSystemId = "Parent System is required";
      }
      if (values.customId && !values.idInput.trim()) errors.idInput = "ID is required";

      if (values.customId && values.idInput.trim() && !queue && values.selectedSystemId) {
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

        let targetId = queue?.id;

        if (!queue) {
          const baseId = values.customId ? values.idInput : slugify(values.name) || "queue";
          if (!values.selectedSystemId) return model;
          targetId = `${values.selectedSystemId}.${baseId}`;
          let i = 1;
          const originalId = targetId;
          while (newElements[targetId as string]) {
            targetId = `${originalId}-${i++}`;
          }
        }

        if (!targetId) return model;

        const tags = queue?.tags ? [...queue.tags] : [];
        if (!tags.includes("queue")) tags.push("queue");

        newElements[targetId as string] = {
          id: targetId,
          kind: "container",
          title: values.name,
          description: typeof values.description === "string" ? values.description : undefined,
          technology: values.technology || undefined,
          tags: tags,
          links: queue?.links,
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
        name: queue?.title || initialName || "",
        technology: queue?.technology || "",
        description:
          (typeof queue?.description === "string"
            ? queue.description
            : (queue?.description as unknown as { txt: string })?.txt || "") || "",
        idInput: queue?.id || "",
        customId: false,
        selectedSystemId: parentSystemId || (queue?.id ? queue.id.split(".")[0] : "") || "",
      });
      form.clearErrors();
    }
  }, [isOpen, queue, parentSystemId, initialName]); // eslint-disable-line react-hooks/exhaustive-deps

  return (
    <SidePanel
      isOpen={isOpen}
      onClose={onClose}
      title={queue ? "Edit Queue" : "Add Queue"}
      size="lg"
      footer={
        <>
          <Button variant="secondary" onClick={onClose} type="button">
            Cancel
          </Button>
          <Button
            variant="primary"
            type="submit"
            form="edit-queue-form"
            isLoading={form.isSubmitting}
          >
            {queue ? "Update" : "Create"}
          </Button>
        </>
      }
    >
      <form ref={formRef} id="edit-queue-form" onSubmit={form.handleSubmit} className="edit-form">
        {!queue && (
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
          placeholder="e.g. RabbitMQ, Kafka"
        />

        <FormField
          label="Description"
          name="description"
          value={form.values.description}
          onChange={(value) => form.setValue("description", value)}
          type="textarea"
          rows={3}
        />

        {!queue && (
          <>
            <Checkbox
              id="queue-custom-id"
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
