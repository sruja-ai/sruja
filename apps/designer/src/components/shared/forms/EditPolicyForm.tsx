// apps/playground/src/components/shared/forms/EditPolicyForm.tsx
import { useState, useEffect, useRef } from "react";
import { useArchitectureStore } from "../../../stores";
import type { PolicyJSON } from "../../../types";
import { Button, Input, Textarea } from "@sruja/ui";
import { SidePanel } from "../SidePanel";
import "../EditForms.css";

interface EditPolicyFormProps {
  isOpen: boolean;
  onClose: () => void;
  policy?: PolicyJSON;
}

export function EditPolicyForm({ isOpen, onClose, policy }: EditPolicyFormProps) {
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const formRef = useRef<HTMLFormElement>(null);
  const [id, setId] = useState(policy?.id || "");
  const [label, setLabel] = useState(policy?.label || "");
  const [description, setDescription] = useState(policy?.description || "");
  const [category, setCategory] = useState(policy?.category || "");
  const [enforcement, setEnforcement] = useState(policy?.enforcement || "");
  const [errors, setErrors] = useState<Record<string, string>>({});

  useEffect(() => {
    if (isOpen) {
      setId(policy?.id || "");
      setLabel(policy?.label || "");
      setDescription(policy?.description || "");
      setCategory(policy?.category || "");
      setEnforcement(policy?.enforcement || "");
      setErrors({});
    }
  }, [isOpen, policy]);

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
    if (!id.trim()) {
      newErrors.id = "ID is required";
    } else if (!/^[A-Za-z0-9_-]+$/.test(id.trim())) {
      newErrors.id = "ID can only contain letters, numbers, hyphens, and underscores";
    }
    if (!label.trim() && !description.trim()) {
      newErrors.label = "Label or description is required";
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
        const policies = [...(arch.architecture.policies || [])];
        const newPolicy: PolicyJSON = {
          id: id.trim(),
          label: label.trim() || undefined,
          description: description.trim() || undefined,
          category: category.trim() || undefined,
          enforcement: enforcement.trim() || undefined,
        };

        if (policy) {
          const index = policies.findIndex((p) => p.id === policy.id);
          if (index >= 0) {
            policies[index] = newPolicy;
          }
        } else {
          policies.push(newPolicy);
        }

        return {
          ...arch,
          architecture: {
            ...arch.architecture,
            policies,
          },
        };
      });
      onClose();
    } catch (err) {
      console.error("Failed to update policy:", err);
      setErrors({ submit: "Failed to save policy. Please try again." });
    }
  };

  return (
    <SidePanel
      isOpen={isOpen}
      onClose={onClose}
      title={policy ? "Edit Policy" : "Add Policy"}
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
            {policy ? "Update" : "Add"}
          </Button>
        </>
      }
    >
      <form ref={formRef} onSubmit={handleSubmit} className="edit-form">
        <Input
          label="ID *"
          value={id}
          onChange={(e) => {
            setId(e.target.value);
            if (errors.id) setErrors({ ...errors, id: "" });
          }}
          required
          placeholder="SecurityPolicy"
          error={errors.id}
        />
        <Input
          label="Label"
          value={label}
          onChange={(e) => {
            setLabel(e.target.value);
            if (errors.label) setErrors({ ...errors, label: "" });
          }}
          placeholder="Policy label"
          error={errors.label}
        />
        <Textarea
          label="Description"
          value={description}
          onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => setDescription(e.target.value)}
          rows={3}
          placeholder="Policy description"
        />
        <Input
          label="Category"
          value={category}
          onChange={(e) => setCategory(e.target.value)}
          placeholder="e.g., security, compliance, performance"
        />
        <Input
          label="Enforcement"
          value={enforcement}
          onChange={(e) => setEnforcement(e.target.value)}
          placeholder="e.g., required, recommended, optional"
        />
      </form>
    </SidePanel>
  );
}
