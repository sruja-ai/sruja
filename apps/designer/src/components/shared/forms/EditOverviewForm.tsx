// apps/designer/src/components/shared/forms/EditOverviewForm.tsx
// Refactored to use Mantine form components

import { useEffect, useRef } from "react";
import { useArchitectureStore } from "../../../stores";
import { Button, Input } from "@sruja/ui";
import type { SrujaExtensions } from "@sruja/shared";

import { SidePanel } from "../SidePanel";
import { FormField, useFormState, DescriptionField } from "./";
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
  const { updateArchitecture } = useArchitectureStore();
  const data = useArchitectureStore((s) => s.model);
  const formRef = useRef<HTMLFormElement>(null);

  // Need to be careful mapping from model if it doesn't have overview yet or it's in sruja
  const sruja = data?.sruja as SrujaExtensions | undefined;
  const overview = sruja?.overview;

  // Initialize form state
  const form = useFormState<FormValues>({
    initialValues: {
      architectureDescription: sruja?.description || "",
      summary: overview?.summary || "",
      audience: overview?.audience || "",
      scope: overview?.scope || "",
      goals: overview?.goals ? [...overview.goals] : [""],
      nonGoals: overview?.nonGoals ? [...overview.nonGoals] : [""],
      risks: overview?.risks ? [...overview.risks] : [""],
    },
    // ...
    onSubmit: async (values) => {
      await updateArchitecture((model) => {
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

        // Check if newOverview has any non-undefined properties
        const hasOverviewContent = Object.values(newOverview).some(
          (val) => val !== undefined && (!Array.isArray(val) || val.length > 0)
        );

        return {
          ...model,
          sruja: {
            ...(model?.sruja || {}),
            description: values.architectureDescription.trim() || undefined,
            overview: hasOverviewContent ? newOverview : undefined,
          },
        };
      });
      onClose();
    },
  });

  // Reset form when opening/switching contexts
  // Note: form.setValues and form.clearErrors are stable (wrapped in useCallback with empty deps)
  // so they don't need to be in the dependency array
  useEffect(() => {
    if (isOpen) {
      const currentSruja = data?.sruja as SrujaExtensions | undefined;
      const currentOverview = currentSruja?.overview;

      form.setValues({
        architectureDescription: currentSruja?.description || "",
        summary: currentOverview?.summary || "",
        audience: currentOverview?.audience || "",
        scope: currentOverview?.scope || "",
        goals: currentOverview?.goals ? ([...currentOverview.goals] as string[]) : [],
        nonGoals: currentOverview?.nonGoals ? ([...currentOverview.nonGoals] as string[]) : [],
        risks: currentOverview?.risks ? ([...currentOverview.risks] as string[]) : [],
      });
      form.clearErrors();
    }

    // form.setValues and form.clearErrors are stable callbacks from useFormState
  }, [isOpen, data]);

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
    form.setValue(
      listName,
      form.values[listName].filter((_, i) => i !== index)
    );
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
          <Button
            variant="primary"
            type="submit"
            form="edit-overview-form"
            isLoading={form.isSubmitting}
          >
            Save
          </Button>
        </>
      }
    >
      <form
        ref={formRef}
        id="edit-overview-form"
        onSubmit={form.handleSubmit}
        className="edit-form"
      >
        <DescriptionField
          label="Architecture Description"
          value={form.values.architectureDescription}
          onChange={(value) => form.setValue("architectureDescription", value)}
          rows={3}
          placeholder="Purpose, scope, and high-level context of the architecture"
        />
        <DescriptionField
          label="Summary"
          value={form.values.summary}
          onChange={(value) => form.setValue("summary", value)}
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
