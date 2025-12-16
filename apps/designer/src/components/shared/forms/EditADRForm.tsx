// apps/playground/src/components/shared/forms/EditADRForm.tsx
import { useState, useEffect, useRef } from "react";
import { useArchitectureStore } from "../../../stores";
import type { ADRJSON } from "../../../types";
import { SidePanel } from "../SidePanel";
import { Button, Input, Listbox, Textarea } from "@sruja/ui";
import type { ListOption } from "@sruja/ui";
import { ADR_STATUSES } from "./constants";
import "../EditForms.css";

interface EditADRFormProps {
  isOpen: boolean;
  onClose: () => void;
  adr?: ADRJSON;
}

export function EditADRForm({ isOpen, onClose, adr }: EditADRFormProps) {
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const formRef = useRef<HTMLFormElement>(null);
  const [id, setId] = useState(adr?.id || "");
  const [title, setTitle] = useState(adr?.title || "");
  const [status, setStatus] = useState<ListOption | null>(
    ADR_STATUSES.find((s) => s.id === (adr?.status || "proposed")) || ADR_STATUSES[0]
  );
  const [context, setContext] = useState(adr?.context || "");
  const [decision, setDecision] = useState(adr?.decision || "");
  const [consequences, setConsequences] = useState(adr?.consequences || "");
  const [errors, setErrors] = useState<Record<string, string>>({});

  useEffect(() => {
    if (isOpen) {
      setId(adr?.id || "");
      setTitle(adr?.title || "");
      setStatus(ADR_STATUSES.find((s) => s.id === (adr?.status || "proposed")) || ADR_STATUSES[0]);
      setContext(adr?.context || "");
      setDecision(adr?.decision || "");
      setConsequences(adr?.consequences || "");
      setErrors({});
    }
  }, [isOpen, adr]);

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
    if (!title.trim()) {
      newErrors.title = "Title is required";
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
        const adrs = [...(arch.architecture.adrs || [])];
        const newADR: ADRJSON = {
          id: id.trim(),
          title: title.trim(),
          status: status?.id as ADRJSON["status"],
          context: context.trim() || undefined,
          decision: decision.trim() || undefined,
          consequences: consequences.trim() || undefined,
        };

        if (adr) {
          const index = adrs.findIndex((a) => a.id === adr.id);
          if (index >= 0) {
            adrs[index] = newADR;
          }
        } else {
          adrs.push(newADR);
        }

        return {
          ...arch,
          architecture: {
            ...arch.architecture,
            adrs,
          },
        };
      });
      onClose();
    } catch (err) {
      console.error("Failed to update ADR:", err);
      setErrors({ submit: "Failed to save ADR. Please try again." });
    }
  };

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
          <Button
            variant="primary"
            onClick={(e) => {
              e.preventDefault();
              handleSubmit(e as any);
            }}
          >
            {adr ? "Update" : "Add"}
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
          placeholder="ADR-001"
          error={errors.id}
        />
        <Input
          label="Title *"
          value={title}
          onChange={(e) => {
            setTitle(e.target.value);
            if (errors.title) setErrors({ ...errors, title: "" });
          }}
          required
          placeholder="ADR title"
          error={errors.title}
        />
        <Listbox label="Status" options={ADR_STATUSES} value={status} onChange={setStatus} />
        <Textarea
          label="Context"
          value={context}
          onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => setContext(e.target.value)}
          rows={4}
          placeholder="The context and forces that led to this decision"
        />
        <Textarea
          label="Decision"
          value={decision}
          onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => setDecision(e.target.value)}
          rows={4}
          placeholder="The decision that was made"
        />
        <Textarea
          label="Consequences"
          value={consequences}
          onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => setConsequences(e.target.value)}
          rows={4}
          placeholder="The consequences of this decision"
        />
        {errors.submit && (
          <div className="text-sm text-[var(--color-error-500)] mt-2">{errors.submit}</div>
        )}
      </form>
    </SidePanel>
  );
}
