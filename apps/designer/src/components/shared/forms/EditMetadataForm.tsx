// apps/designer/src/components/shared/forms/EditMetadataForm.tsx
// Refactored to use Mantine form components

import { useEffect, useRef } from "react";
import { useArchitectureStore } from "../../../stores";
import { Button, Input } from "@sruja/ui";
import { SidePanel } from "../SidePanel";
import { FormField, useFormState, type FormErrors } from "./";
import { X } from "lucide-react";
import "../EditForms.css";

interface EditMetadataFormProps {
  isOpen: boolean;
  onClose: () => void;
  metadata?: any; // MetadataDump
  metadataIndex?: number;
}

interface FormValues {
  key: string;
  value: string;
  isArray: boolean;
  arrayValues: string[];
}

export function EditMetadataForm({
  isOpen,
  onClose,
  metadata,
  metadataIndex,
}: EditMetadataFormProps) {
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const formRef = useRef<HTMLFormElement>(null);

  // Initialize form state
  const form = useFormState<FormValues>({
    initialValues: {
      key: metadata?.key || "",
      value: metadata?.value || "",
      isArray: !!metadata?.array,
      arrayValues: metadata?.array || [""],
    },
    validate: (values) => {
      const errors: FormErrors = {};
      if (!values.key.trim()) errors.key = "Key is required";
      if (!values.isArray && !values.value.trim()) errors.value = "Value is required";
      if (values.isArray && values.arrayValues.filter((v) => v.trim()).length === 0) {
        errors.arrayValues = "At least one array value is required";
      }
      return errors;
    },
    onSubmit: async (values) => {
      await updateArchitecture((model) => {
        const sruja = (model as any).sruja || {};
        const metadataList = [...(sruja.metadata || [])];

        const newMetadata = {
          key: values.key.trim(),
          value: values.isArray ? undefined : values.value.trim() || undefined,
          array: values.isArray ? values.arrayValues.filter((v) => v.trim()) : undefined,
        };

        if (metadata && metadataIndex !== undefined) {
          metadataList[metadataIndex] = newMetadata;
        } else {
          metadataList.push(newMetadata);
        }

        return {
          ...model,
          sruja: {
            ...sruja,
            metadata: metadataList,
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
        key: metadata?.key || "",
        value: metadata?.value || "",
        isArray: !!metadata?.array,
        arrayValues: metadata?.array || [""],
      });
      form.clearErrors();
    }
  }, [isOpen, metadata]); // eslint-disable-line react-hooks/exhaustive-deps

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

  const addArrayItem = () => {
    form.setValue("arrayValues", [...form.values.arrayValues, ""]);
  };

  const updateArrayItem = (index: number, val: string) => {
    const newArray = [...form.values.arrayValues];
    newArray[index] = val;
    form.setValue("arrayValues", newArray);
  };

  const removeArrayItem = (index: number) => {
    form.setValue("arrayValues", form.values.arrayValues.filter((_, i) => i !== index));
  };

  return (
    <SidePanel
      isOpen={isOpen}
      onClose={onClose}
      title={metadata ? "Edit Metadata" : "Add Metadata"}
      size="md"
      footer={
        <>
          <Button variant="secondary" onClick={onClose} type="button">
            Cancel
          </Button>
          <Button variant="primary" type="submit" form="edit-metadata-form" isLoading={form.isSubmitting}>
            {metadata ? "Update" : "Add"}
          </Button>
        </>
      }
    >
      <form ref={formRef} id="edit-metadata-form" onSubmit={form.handleSubmit} className="edit-form">
        <FormField
          label="Key"
          name="key"
          value={form.values.key}
          onChange={(value) => form.setValue("key", value)}
          required
          placeholder="e.g., team, owner, version"
          error={form.errors.key}
        />
        <div className="form-group">
          <label className="flex items-center gap-2 mb-1.5 text-sm font-medium text-[var(--color-text-secondary)]">
            <input
              type="checkbox"
              checked={form.values.isArray}
              onChange={(e) => form.setValue("isArray", e.target.checked)}
            />
            <span>Array value (multiple items)</span>
          </label>
        </div>
        {!form.values.isArray ? (
          <FormField
            label="Value"
            name="value"
            value={form.values.value}
            onChange={(value) => form.setValue("value", value)}
            required
            placeholder="Metadata value"
            error={form.errors.value}
          />
        ) : (
          <div className="form-group">
            <label className="block mb-1.5 text-sm font-medium text-[var(--color-text-secondary)]">
              Values *
            </label>
            <div className="list-items">
              {form.values.arrayValues.map((val, index) => (
                <div key={index} className="list-item">
                  <div>
                    <Input
                      value={val}
                      onChange={(e) => updateArrayItem(index, e.target.value)}
                      placeholder="Enter a value"
                    />
                  </div>
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    onClick={() => removeArrayItem(index)}
                  >
                    <X size={16} />
                  </Button>
                </div>
              ))}
              <Button type="button" variant="outline" onClick={addArrayItem}>
                + Add Value
              </Button>
            </div>
            {form.errors.arrayValues && (
              <div className="mt-1 text-xs text-[var(--color-error-500)]">{form.errors.arrayValues}</div>
            )}
          </div>
        )}
        {form.errors.submit && (
          <div className="text-sm text-[var(--color-error-500)] mt-2">{form.errors.submit}</div>
        )}
      </form>
    </SidePanel>
  );
}
