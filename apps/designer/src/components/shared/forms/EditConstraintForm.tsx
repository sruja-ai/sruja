// apps/playground/src/components/shared/forms/EditConstraintForm.tsx
import { useState, useEffect, useRef } from "react";
import { useArchitectureStore } from "../../../stores";
import type { ConstraintJSON } from "../../../types";
import { Button, Input } from "@sruja/ui";
import { SidePanel } from "../SidePanel";
import "../EditForms.css";

interface EditConstraintFormProps {
  isOpen: boolean;
  onClose: () => void;
  constraint?: ConstraintJSON;
  constraintIndex?: number;
}

export function EditConstraintForm({
  isOpen,
  onClose,
  constraint,
  constraintIndex,
}: EditConstraintFormProps) {
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const formRef = useRef<HTMLFormElement>(null);
  const [key, setKey] = useState(constraint?.key || "");
  const [value, setValue] = useState(constraint?.value || "");
  const [errors, setErrors] = useState<Record<string, string>>({});

  useEffect(() => {
    if (isOpen) {
      setKey(constraint?.key || "");
      setValue(constraint?.value || "");
      setErrors({});
    }
  }, [isOpen, constraint]);

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
        const constraints = [...(arch.architecture.constraints || [])];
        const newConstraint: ConstraintJSON = {
          key: key.trim(),
          value: value.trim(),
        };

        if (constraint && constraintIndex !== undefined) {
          constraints[constraintIndex] = newConstraint;
        } else {
          constraints.push(newConstraint);
        }

        return {
          ...arch,
          architecture: {
            ...arch.architecture,
            constraints,
          },
        };
      });
      onClose();
    } catch (err) {
      console.error("Failed to update constraint:", err);
      setErrors({ submit: "Failed to save constraint. Please try again." });
    }
  };

  return (
    <SidePanel
      isOpen={isOpen}
      onClose={onClose}
      title={constraint ? "Edit Constraint" : "Add Constraint"}
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
            {constraint ? "Update" : "Add"}
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
          placeholder="e.g., maxLatency, minThroughput"
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
          placeholder="Constraint value"
          error={errors.value}
        />
      </form>
    </SidePanel>
  );
}
