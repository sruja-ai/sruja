// apps/designer/src/components/shared/forms/EditContainerForm.tsx
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
import { slugify } from "../../../utils/slugify";
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
  type: ContainerType;
}

type ContainerType = "container" | "database" | "queue";

export function EditContainerForm({
  isOpen,
  onClose,
  container,
  parentSystemId,
  initialName,
}: EditContainerFormProps) {
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
      name: container?.title || initialName || "",
      technology: container?.technology || "",
      description: extractDescription(container),
      customId: false,
      idInput: container?.id || "",
      selectedSystemId: parentSystemId || (container?.id ? container.id.split(".")[0] : "") || "",
      type: (container?.tags?.includes("queue")
        ? "queue"
        : container?.tags?.includes("database")
          ? "database"
          : "container") as ContainerType,
    },
    validate: (values) => {
      const errors: FormErrors = {};
      if (!values.name.trim()) errors.name = "Name is required";
      if (!container && !values.selectedSystemId) {
        errors.selectedSystemId = "Parent System is required";
      }
      if (values.customId && !values.idInput.trim()) errors.idInput = "ID is required";

      if (values.customId && values.idInput.trim() && !container && values.selectedSystemId) {
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

        let targetId = container?.id;

        if (!container) {
          const baseId = values.customId ? values.idInput : slugify(values.name) || "container";
          if (!values.selectedSystemId) return model;
          targetId = `${values.selectedSystemId}.${baseId}`;
          let i = 1;
          const originalId = targetId;
          while (newElements[targetId as string]) {
            targetId = `${originalId}-${i++}`;
          }
        }

        if (!targetId) return model;

        const tags = container?.tags ? [...container.tags] : [];
        // Manage type tags
        const typeTags = ["database", "queue"];
        typeTags.forEach((t) => {
          const idx = tags.indexOf(t);
          if (idx > -1) tags.splice(idx, 1);
        });
        if (values.type === "database") tags.push("database");
        if (values.type === "queue") tags.push("queue");

        newElements[targetId as string] = {
          id: targetId,
          kind: "container",
          title: values.name,
          description: values.description || undefined,
          technology: values.technology || undefined,
          tags: tags,
          links: container?.links,
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
      name: container?.title || initialName || "",
      technology: container?.technology || "",
      description: extractDescription(container),
      idInput: container?.id || "",
      customId: false,
      selectedSystemId: parentSystemId || (container?.id ? container.id.split(".")[0] : "") || "",
      type: (container?.tags?.includes("queue")
        ? "queue"
        : container?.tags?.includes("database")
          ? "database"
          : "container") as ContainerType,
    },
    [container, parentSystemId, initialName]
  );

  return (
    <SidePanel
      isOpen={isOpen}
      onClose={onClose}
      title={container ? "Edit Container" : "Add Container"}
      size="lg"
      footer={
        <>
          <Button variant="secondary" onClick={onClose} type="button">
            Cancel
          </Button>
          <Button
            variant="primary"
            type="submit"
            form="edit-container-form"
            isLoading={form.isSubmitting}
          >
            {container ? "Update" : "Create"}
          </Button>
        </>
      }
    >
      <form
        ref={formRef}
        id="edit-container-form"
        onSubmit={form.handleSubmit}
        className="edit-form"
      >
        {!container && (
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
          placeholder="e.g. Docker, Go, React"
        />

        <DescriptionField
          value={form.values.description}
          onChange={(value) => form.setValue("description", value)}
        />

        {!container && (
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
