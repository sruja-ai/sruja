// apps/designer/src/components/shared/forms/EditQueueForm.tsx
// Refactored to use Mantine form components

import { useRef, useMemo } from "react";
import { useArchitectureStore } from "../../../stores";
import type { ElementDump } from "@sruja/shared";
import { SidePanel } from "../SidePanel";
import { Button } from "@sruja/ui";
import {
  useFormState,
  type FormErrors,
  NameField,
  DescriptionField,
  TechnologyField,
  CustomIdField,
  ParentSelectField,
  useFormReset,
  extractDescription,
} from "./";
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
      description: extractDescription(queue),
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
          description: values.description || undefined,
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
  useFormReset(
    form,
    isOpen,
    {
      name: queue?.title || initialName || "",
      technology: queue?.technology || "",
      description: extractDescription(queue),
      idInput: queue?.id || "",
      customId: false,
      selectedSystemId: parentSystemId || (queue?.id ? queue.id.split(".")[0] : "") || "",
    },
    [queue, parentSystemId, initialName]
  );

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
          <ParentSelectField
            label="Parent System"
            value={form.values.selectedSystemId}
            onChange={(value) => form.setValue("selectedSystemId", value || "")}
            options={systems.map((s) => ({ value: s.id, label: s.title || s.id }))}
            error={form.errors.selectedSystemId}
            placeholder="Select System"
            required
            disabled={!!parentSystemId}
          />
        )}

        <NameField
          value={form.values.name}
          onChange={(value) => form.setValue("name", value)}
          error={form.errors.name}
        />

        <TechnologyField
          value={form.values.technology}
          onChange={(value) => form.setValue("technology", value)}
          placeholder="e.g. RabbitMQ, Kafka"
        />

        <DescriptionField
          value={form.values.description}
          onChange={(value) => form.setValue("description", value)}
        />

        {!queue && (
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
