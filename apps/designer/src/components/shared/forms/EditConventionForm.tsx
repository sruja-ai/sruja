// apps/playground/src/components/shared/forms/EditConventionForm.tsx
import { useState, useEffect, useRef } from "react";
import { useArchitectureStore } from "../../../stores";
import type { ConventionJSON } from "../../../types";
import { Button, Input } from "@sruja/ui";
import { SidePanel } from "../SidePanel";
import "../EditForms.css";

interface EditConventionFormProps {
  isOpen: boolean;
  onClose: () => void;
  convention?: ConventionJSON;
  conventionIndex?: number;
}

export function EditConventionForm({
  isOpen,
  onClose,
  convention,
  conventionIndex,
}: EditConventionFormProps) {
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const formRef = useRef<HTMLFormElement>(null);
  const [key, setKey] = useState(convention?.key || "");
  const [value, setValue] = useState(convention?.value || "");
  const [errors, setErrors] = useState<Record<string, string>>({});

  useEffect(() => {
    if (isOpen) {
      setKey(convention?.key || "");
      setValue(convention?.value || "");
      setErrors({});
    }
  }, [isOpen, convention]);

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

  const validate = (): boolean => {
    const newErrors: Record<string, string> = {};
    if (!key.trim()) {
      newErrors.key = "Key is required";
    }
    if (!value.trim()) {
      newErrors.value = "Value is required";
    }
    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!validate()) return;

    try {
      await updateArchitecture((arch) => {
        if (!arch.architecture) return arch;
        const conventions = [...(arch.architecture.conventions || [])];
        const newConvention: ConventionJSON = {
          key: key.trim(),
          value: value.trim(),
        };

        if (convention && conventionIndex !== undefined) {
          conventions[conventionIndex] = newConvention;
        } else {
          conventions.push(newConvention);
        }

        return {
          ...arch,
          architecture: {
            ...arch.architecture,
            conventions,
          },
        };
      });
      onClose();
    } catch (err) {
      console.error("Failed to update convention:", err);
      setErrors({ submit: "Failed to save convention. Please try again." });
    }
  };

  return (
    <SidePanel
      isOpen={isOpen}
      onClose={onClose}
      title={convention ? "Edit Convention" : "Add Convention"}
      size="md"
      footer={
        <>
          <Button variant="secondary" onClick={onClose} type="button">
            Cancel
          </Button>
          <Button
            variant="primary"
            onClick={(e) => {
              e.preventDefault();
              handleSubmit(e as any);
            }}
          >
            {convention ? "Update" : "Add"}
          </Button>
        </>
      }
    >
      <form ref={formRef} onSubmit={handleSubmit} className="edit-form">
        <Input
          label="Key *"
          value={key}
          onChange={(e) => {
            setKey(e.target.value);
            if (errors.key) setErrors({ ...errors, key: "" });
          }}
          required
          placeholder="e.g., naming, versioning, documentation"
          error={errors.key}
        />
        <Input
          label="Value *"
          value={value}
          onChange={(e) => {
            setValue(e.target.value);
            if (errors.value) setErrors({ ...errors, value: "" });
          }}
          required
          placeholder="Convention value"
          error={errors.value}
        />
      </form>
    </SidePanel>
  );
}
