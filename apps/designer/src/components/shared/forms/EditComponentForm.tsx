// apps/designer/src/components/shared/forms/EditComponentForm.tsx
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

interface EditComponentFormProps {
  isOpen: boolean;
  onClose: () => void;
  component?: ElementDump;
  parentSystemId?: string | null;
  parentContainerId?: string | null;
  initialName?: string;
}

interface FormValues {
  name: string;
  technology: string;
  description: string;
  customId: boolean;
  idInput: string;
  selectedSystemId: string;
  selectedContainerId: string;
}

export function EditComponentForm({
  isOpen,
  onClose,
  component,
  parentSystemId,
  parentContainerId,
  initialName,
}: EditComponentFormProps) {
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const data = useArchitectureStore((s) => s.model);
  const formRef = useRef<HTMLFormElement>(null);

  const allElements = useMemo(
    () => Object.values(data?.elements || {}) as ElementDump[],
    [data?.elements]
  );
  const systems = useMemo(() => allElements.filter((e) => e.kind === "system"), [allElements]);

  // Derive containers for selected system
  // We can't do this easily inside the component body if selectedSystemId is in form state.
  // Actually we can access form.values.selectedSystemId but it triggers re-renders.

  // Initialize form state
  const form = useFormState<FormValues>({
    initialValues: {
      name: component?.title || initialName || "",
      technology: component?.technology || "",
      description: extractDescription(component),
      customId: false,
      idInput: component?.id || "",
      selectedSystemId: parentSystemId || (component?.id ? component.id.split(".")[0] : "") || "",
      selectedContainerId:
        parentContainerId ||
        (component?.id ? component.id.split(".").slice(0, 2).join(".") : "") ||
        "",
    },
    validate: (values) => {
      const errors: FormErrors = {};
      if (!values.name.trim()) errors.name = "Name is required";
      if (!component) {
        if (!values.selectedSystemId) errors.selectedSystemId = "System is required";
        if (!values.selectedContainerId) errors.selectedContainerId = "Container is required";
      }
      if (values.customId && !values.idInput.trim()) errors.idInput = "ID is required";

      if (values.customId && values.idInput.trim() && !component && values.selectedContainerId) {
        const fullId = `${values.selectedContainerId}.${values.idInput.trim()}`;
        if (data?.elements?.[fullId]) {
          errors.idInput = "ID already exists in this container";
        }
      }

      return errors;
    },
    onSubmit: async (values) => {
      await updateArchitecture((model) => {
        const newElements = { ...model.elements };

        let targetId = component?.id;

        if (!component) {
          const baseId = values.customId ? values.idInput : slugify(values.name) || "component";
          if (!values.selectedContainerId) return model;
          targetId = `${values.selectedContainerId}.${baseId}`;
          let i = 1;
          const originalId = targetId;
          while (newElements[targetId as string]) {
            targetId = `${originalId}-${i++}`;
          }
        }

        if (!targetId) return model;

        newElements[targetId as string] = {
          id: targetId,
          kind: "component",
          title: values.name,
          description: values.description || undefined,
          technology: values.technology || undefined,
          tags: component?.tags,
          links: component?.links,
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
      name: component?.title || initialName || "",
      technology: component?.technology || "",
      description: extractDescription(component),
      idInput: component?.id || "",
      customId: false,
      selectedSystemId: parentSystemId || (component?.id ? component.id.split(".")[0] : "") || "",
      selectedContainerId:
        parentContainerId ||
        (component?.id ? component.id.split(".").slice(0, 2).join(".") : "") ||
        "",
    },
    [component, parentSystemId, parentContainerId, initialName]
  );

  // Update available containers when system changes
  const currentAvailableContainers = useMemo(() => {
    if (!form.values.selectedSystemId) return [];
    return allElements.filter(
      (e) => e.kind === "container" && e.id.startsWith(form.values.selectedSystemId + ".")
    );
  }, [allElements, form.values.selectedSystemId]);

  return (
    <SidePanel
      isOpen={isOpen}
      onClose={onClose}
      title={component ? "Edit Component" : "Add Component"}
      size="lg"
      footer={
        <>
          <Button variant="secondary" onClick={onClose} type="button">
            Cancel
          </Button>
          <Button
            variant="primary"
            type="submit"
            form="edit-component-form"
            isLoading={form.isSubmitting}
          >
            {component ? "Update" : "Create"}
          </Button>
        </>
      }
    >
      <form
        ref={formRef}
        id="edit-component-form"
        onSubmit={form.handleSubmit}
        className="edit-form"
      >
        {!component && (
          <>
            <ParentSelectField
              label="Parent System"
              value={form.values.selectedSystemId}
              onChange={(value) => {
                form.setValue("selectedSystemId", value || "");
                form.setValue("selectedContainerId", ""); // Reset container when system changes
              }}
              options={systems.map((s) => ({ value: s.id, label: s.title || s.id }))}
              error={form.errors.selectedSystemId}
              placeholder="Select System"
              required
              disabled={!!parentSystemId}
            />
            <ParentSelectField
              label="Parent Container"
              value={form.values.selectedContainerId}
              onChange={(value) => form.setValue("selectedContainerId", value || "")}
              options={currentAvailableContainers.map((c) => ({
                value: c.id,
                label: c.title || c.id,
              }))}
              error={form.errors.selectedContainerId}
              placeholder="Select Container"
              required
              disabled={!form.values.selectedSystemId || !!parentContainerId}
            />
          </>
        )}

        <NameField
          value={form.values.name}
          onChange={(value) => form.setValue("name", value)}
          error={form.errors.name}
        />

        <TechnologyField
          value={form.values.technology}
          onChange={(value) => form.setValue("technology", value)}
          placeholder="e.g. Redux, JpaRepository"
        />

        <DescriptionField
          value={form.values.description}
          onChange={(value) => form.setValue("description", value)}
        />

        {!component && (
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
