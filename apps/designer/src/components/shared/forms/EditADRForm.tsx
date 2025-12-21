// apps/designer/src/components/shared/forms/EditADRForm.tsx
// Refactored to use Mantine form components

import { useEffect, useRef } from "react";
import { useArchitectureStore } from "../../../stores";
import type { ADRDump } from "@sruja/shared";
import { SidePanel } from "../SidePanel";
import { Button, Listbox } from "@sruja/ui";
import type { ListOption } from "@sruja/ui";
import { FormField, useFormState, type FormErrors } from "./";
import { ADR_STATUSES } from "./constants";
import "../EditForms.css";

interface EditADRFormProps {
  isOpen: boolean;
  onClose: () => void;
  adr?: ADRDump;
}

interface FormValues {
  id: string;
  title: string;
  status: ListOption | null;
  context: string;
  decision: string;
  consequences: string;
}

export function EditADRForm({ isOpen, onClose, adr }: EditADRFormProps) {
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const formRef = useRef<HTMLFormElement>(null);

  // Initialize form state
  const form = useFormState<FormValues>({
    initialValues: {
      id: adr?.id || "",
      title: adr?.title || "",
      status: ADR_STATUSES.find((s) => s.id === (adr?.status || "proposed")) || ADR_STATUSES[0],
      context: adr?.context || "",
      decision: adr?.decision || "",
      consequences: adr?.consequences || "",
    },
    validate: (values) => {
      const errors: FormErrors = {};
      if (!values.id.trim()) {
        errors.id = "ID is required";
      } else if (!/^[A-Za-z0-9_-]+$/.test(values.id.trim())) {
        errors.id = "ID can only contain letters, numbers, hyphens, and underscores";
      }
      if (!values.title.trim()) {
        errors.title = "Title is required";
      }
      return errors;
    },
    onSubmit: async (values) => {
      await updateArchitecture((model) => {
        const sruja = (model as any).sruja || {};
        const adrs = [...(sruja.adrs || [])];

        const newADR: ADRDump = {
          id: values.id.trim(),
          title: values.title.trim(),
          status: values.status?.id,
          context: values.context.trim() || undefined,
          decision: values.decision.trim() || undefined,
          consequences: values.consequences.trim() || undefined,
        };

        if (adr) {
          const index = adrs.findIndex((a: any) => a.id === adr.id);
          if (index >= 0) {
            adrs[index] = newADR;
          }
        } else {
          adrs.push(newADR);
        }

        return {
          ...model,
          sruja: {
            ...sruja,
            adrs,
          },
        };
      });
      onClose();
    },
  });

  // Reset form when opening/switching contexts
  useEffect(() => {
    if (isOpen) {
      form.setValues({
        id: adr?.id || "",
        title: adr?.title || "",
        status: ADR_STATUSES.find((s) => s.id === (adr?.status || "proposed")) || ADR_STATUSES[0],
        context: adr?.context || "",
        decision: adr?.decision || "",
        consequences: adr?.consequences || "",
      });
      form.clearErrors();
    }
  }, [isOpen, adr]); // eslint-disable-line react-hooks/exhaustive-deps

  // Handle Escape key
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && isOpen) {
        onClose();
      }
    };
    if (isOpen) {
      document.addEventListener("keydown", handleKeyDown);
      return () => document.removeEventListener("keydown", handleKeyDown);
    }
  }, [isOpen, onClose]);

  return (
    <SidePanel
      isOpen={isOpen}
      onClose={onClose}
      title={adr ? "Edit ADR" : "Add ADR"}
      size="2xl"
      footer={
        <>
          <Button variant="secondary" onClick={onClose} type="button">
            Cancel
          </Button>
          <Button variant="primary" type="submit" form="edit-adr-form" isLoading={form.isSubmitting}>
            {adr ? "Update" : "Add"}
          </Button>
        </>
      }
    >
      <form ref={formRef} id="edit-adr-form" onSubmit={form.handleSubmit} className="edit-form">
        <FormField
          label="ID"
          name="id"
          value={form.values.id}
          onChange={(value) => form.setValue("id", value)}
          required
          placeholder="ADR-001"
          error={form.errors.id}
        />
        <FormField
          label="Title"
          name="title"
          value={form.values.title}
          onChange={(value) => form.setValue("title", value)}
          required
          placeholder="ADR title"
          error={form.errors.title}
        />
        <Listbox
          label="Status"
          options={ADR_STATUSES}
          value={form.values.status}
          onChange={(value) => form.setValue("status", value)}
        />
        <FormField
          label="Context"
          name="context"
          value={form.values.context}
          onChange={(value) => form.setValue("context", value)}
          type="textarea"
          rows={4}
          placeholder="The context and forces that led to this decision"
        />
        <FormField
          label="Decision"
          name="decision"
          value={form.values.decision}
          onChange={(value) => form.setValue("decision", value)}
          type="textarea"
          rows={4}
          placeholder="The decision that was made"
        />
        <FormField
          label="Consequences"
          name="consequences"
          value={form.values.consequences}
          onChange={(value) => form.setValue("consequences", value)}
          type="textarea"
          rows={4}
          placeholder="The consequences of this decision"
        />
        {form.errors.submit && (
          <div className="text-sm text-[var(--color-error-500)] mt-2">{form.errors.submit}</div>
        )}
      </form>
    </SidePanel>
  );
}
