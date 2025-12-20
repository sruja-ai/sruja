// apps/designer/src/components/shared/forms/EditOverviewForm.tsx
// Refactored to use Mantine form components

import { useEffect, useRef } from "react";
import { useArchitectureStore } from "../../../stores";
import { Button, Input } from "@sruja/ui";
import { SidePanel } from "../SidePanel";
import { FormField, useFormState } from "./";
import { X } from "lucide-react";
import "../EditForms.css";

interface EditOverviewFormProps {
  isOpen: boolean;
  onClose: () => void;
}

interface FormValues {
  architectureDescription: string;
  summary: string;
  audience: string;
  scope: string;
  goals: string[];
  nonGoals: string[];
  risks: string[];
}

export function EditOverviewForm({ isOpen, onClose }: EditOverviewFormProps) {
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const data = useArchitectureStore((s) => s.likec4Model);
  const formRef = useRef<HTMLFormElement>(null);

  // Need to be careful mapping from likec4Model if it doesn't have overview yet or it's in sruja
  const sruja = (data as any)?.sruja || {};
  const overview = sruja?.overview;

  // Initialize form state
  const form = useFormState<FormValues>({
    initialValues: {
      architectureDescription: sruja?.description || "",
      summary: overview?.summary || "",
      audience: overview?.audience || "",
      scope: overview?.scope || "",
      goals: overview?.goals || [],
      nonGoals: overview?.nonGoals || [],
      risks: overview?.risks || [],
    },
    validate: () => ({}), // No validation needed for overview
    onSubmit: async (values) => {
      await updateArchitecture((model) => {
        const sruja = (model as any).sruja || {};

        const newOverview = {
          summary: values.summary.trim() || undefined,
          audience: values.audience.trim() || undefined,
          scope: values.scope.trim() || undefined,
          goals:
            values.goals.filter((g) => g.trim()).length > 0
              ? values.goals.filter((g) => g.trim())
              : undefined,
          nonGoals:
            values.nonGoals.filter((ng) => ng.trim()).length > 0
              ? values.nonGoals.filter((ng) => ng.trim())
              : undefined,
          risks:
            values.risks.filter((r) => r.trim()).length > 0
              ? values.risks.filter((r) => r.trim())
              : undefined,
        };

        return {
          ...model,
          sruja: {
            ...sruja,
            description: values.architectureDescription.trim() || undefined,
            overview: Object.keys(newOverview).some((k) => newOverview[k as keyof typeof newOverview])
              ? newOverview
              : undefined,
          },
        };
      });
      onClose();
    },
  });

  // Reset form when opening/switching contexts
  useEffect(() => {
    if (isOpen) {
      const sruja = (data as any)?.sruja || {};
      const overview = sruja?.overview;

      form.setValues({
        architectureDescription: sruja?.description || "",
        summary: overview?.summary || "",
        audience: overview?.audience || "",
        scope: overview?.scope || "",
        goals: overview?.goals || [],
        nonGoals: overview?.nonGoals || [],
        risks: overview?.risks || [],
      });
      form.clearErrors();
    }
  }, [isOpen, data]); // eslint-disable-line react-hooks/exhaustive-deps

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

  const addItem = (listName: "goals" | "nonGoals" | "risks") => {
    form.setValue(listName, [...form.values[listName], ""]);
  };

  const updateItem = (listName: "goals" | "nonGoals" | "risks", index: number, value: string) => {
    const newList = [...form.values[listName]];
    newList[index] = value;
    form.setValue(listName, newList);
  };

  const removeItem = (listName: "goals" | "nonGoals" | "risks", index: number) => {
    form.setValue(listName, form.values[listName].filter((_, i) => i !== index));
  };

  return (
    <SidePanel
      isOpen={isOpen}
      onClose={onClose}
      title="Edit Overview"
      size="xl"
      footer={
        <>
          <Button variant="secondary" onClick={onClose} type="button">
            Cancel
          </Button>
          <Button variant="primary" type="submit" form="edit-overview-form" isLoading={form.isSubmitting}>
            Save
          </Button>
        </>
      }
    >
      <form ref={formRef} id="edit-overview-form" onSubmit={form.handleSubmit} className="edit-form">
        <FormField
          label="Architecture Description"
          name="architectureDescription"
          value={form.values.architectureDescription}
          onChange={(value) => form.setValue("architectureDescription", value)}
          type="textarea"
          rows={3}
          placeholder="Purpose, scope, and high-level context of the architecture"
        />
        <FormField
          label="Summary"
          name="summary"
          value={form.values.summary}
          onChange={(value) => form.setValue("summary", value)}
          type="textarea"
          rows={3}
          placeholder="High-level architecture summary"
        />
        <FormField
          label="Audience"
          name="audience"
          value={form.values.audience}
          onChange={(value) => form.setValue("audience", value)}
          placeholder="Target audience for this architecture"
        />
        <FormField
          label="Scope"
          name="scope"
          value={form.values.scope}
          onChange={(value) => form.setValue("scope", value)}
          placeholder="Architecture scope"
        />
        <div className="form-group">
          <label className="block mb-1.5 text-sm font-medium text-[var(--color-text-secondary)]">
            Goals
          </label>
          <div className="list-items">
            {form.values.goals.map((goal, index) => (
              <div key={index} className="list-item">
                <div>
                  <Input
                    value={goal}
                    onChange={(e) => updateItem("goals", index, e.target.value)}
                    placeholder="Enter a goal"
                  />
                </div>
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  onClick={() => removeItem("goals", index)}
                >
                  <X size={16} />
                </Button>
              </div>
            ))}
            <Button type="button" variant="outline" onClick={() => addItem("goals")}>
              + Add Goal
            </Button>
          </div>
        </div>
        <div className="form-group">
          <label className="block mb-1.5 text-sm font-medium text-[var(--color-text-secondary)]">
            Non-Goals
          </label>
          <div className="list-items">
            {form.values.nonGoals.map((ng, index) => (
              <div key={index} className="list-item">
                <div>
                  <Input
                    value={ng}
                    onChange={(e) => updateItem("nonGoals", index, e.target.value)}
                    placeholder="Enter a non-goal"
                  />
                </div>
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  onClick={() => removeItem("nonGoals", index)}
                >
                  <X size={16} />
                </Button>
              </div>
            ))}
            <Button type="button" variant="outline" onClick={() => addItem("nonGoals")}>
              + Add Non-Goal
            </Button>
          </div>
        </div>
        <div className="form-group">
          <label className="block mb-1.5 text-sm font-medium text-[var(--color-text-secondary)]">
            Risks & Concerns
          </label>
          <div className="list-items">
            {form.values.risks.map((risk, index) => (
              <div key={index} className="list-item">
                <div>
                  <Input
                    value={risk}
                    onChange={(e) => updateItem("risks", index, e.target.value)}
                    placeholder="Enter a risk or concern"
                  />
                </div>
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  onClick={() => removeItem("risks", index)}
                >
                  <X size={16} />
                </Button>
              </div>
            ))}
            <Button type="button" variant="outline" onClick={() => addItem("risks")}>
              + Add Risk
            </Button>
          </div>
        </div>
      </form>
    </SidePanel>
  );
}
