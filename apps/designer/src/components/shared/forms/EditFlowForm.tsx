// apps/designer/src/components/shared/forms/EditFlowForm.tsx
import { useEffect, useRef } from "react";
import { useArchitectureStore } from "../../../stores";
import { SidePanel } from "../SidePanel";
import { ArrowRight, Trash2, Plus } from "lucide-react";
import { Button, Input } from "@sruja/ui";
import { FormField, useFormState, type FormErrors } from "./";
import type { FlowDump, SrujaModelDump /* ScenarioStepDump unavailable */ } from "@sruja/shared";
import "../EditForms.css";

interface EditFlowFormProps {
  isOpen: boolean;
  onClose: () => void;
  flow?: FlowDump;
}

interface FormValues {
  id: string;
  title: string;
  description: string;
  steps: any[];
}

export function EditFlowForm({ isOpen, onClose, flow }: EditFlowFormProps) {
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const data = useArchitectureStore((s) => s.model);
  const formRef = useRef<HTMLFormElement>(null);

  // Initialize form state
  const form = useFormState<FormValues>({
    initialValues: {
      id: flow?.id || "",
      title: flow?.title || "",
      description: flow?.description || "",
      steps: [...(flow?.steps || [])] as any[],
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
      await updateArchitecture((model: SrujaModelDump) => {
        const sruja = model.sruja || {};
        const flows = [...(sruja.flows || [])];

        const newFlow: FlowDump = {
          id: values.id.trim(),
          title: values.title.trim(),
          description: values.description.trim() || undefined,
          steps: values.steps
            .filter((s) => s.from && s.to && s.from.trim() !== "" && s.to.trim() !== "")
            .map((s) => ({
              from: s.from,
              to: s.to,
              description: s.description || undefined,
            })),
        };

        if (flow) {
          const index = flows.findIndex((f) => f.id === flow.id);
          if (index >= 0) {
            flows[index] = newFlow;
          }
        } else {
          flows.push(newFlow);
        }

        return {
          ...model,
          sruja: {
            ...sruja,
            flows,
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
        id: flow?.id || "",
        title: flow?.title || "",
        description: flow?.description || "",
        steps: [...(flow?.steps || [])] as any[],
      });
      form.clearErrors();
    }
  }, [isOpen, flow, form]);

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

  const addStep = () => {
    form.setValue("steps", [...form.values.steps, { from: "", to: "", description: "" }]);
  };

  const updateStep = (index: number, field: string, value: string) => {
    const newSteps = [...form.values.steps];
    newSteps[index] = { ...newSteps[index], [field]: value };
    form.setValue("steps", newSteps);
  };

  const removeStep = (index: number) => {
    form.setValue(
      "steps",
      form.values.steps.filter((_, i) => i !== index)
    );
  };

  // Get all available node IDs for autocomplete
  const nodeIds: string[] = Object.keys(data?.elements || {});

  return (
    <SidePanel
      isOpen={isOpen}
      onClose={onClose}
      title={flow ? "Edit Flow" : "Add Flow"}
      size="2xl"
      footer={
        <>
          <Button variant="secondary" onClick={onClose} type="button">
            Cancel
          </Button>
          <Button
            variant="primary"
            type="submit"
            form="edit-flow-form"
            isLoading={form.isSubmitting}
          >
            {flow ? "Update" : "Add"}
          </Button>
        </>
      }
    >
      <form ref={formRef} id="edit-flow-form" onSubmit={form.handleSubmit} className="edit-form">
        <FormField
          label="ID"
          name="id"
          value={form.values.id}
          onChange={(value) => form.setValue("id", value)}
          required
          placeholder="F1"
          error={form.errors.id}
        />
        <FormField
          label="Title"
          name="title"
          value={form.values.title}
          onChange={(value) => form.setValue("title", value)}
          required
          placeholder="Flow title"
          error={form.errors.title}
        />
        <FormField
          label="Description"
          name="description"
          value={form.values.description}
          onChange={(value) => form.setValue("description", value)}
          type="textarea"
          rows={3}
          placeholder="Flow description"
        />

        <div className="form-group border-t border-[var(--color-border)] pt-4 mt-2">
          <div className="flex items-center justify-between mb-2">
            <label className="text-sm font-medium text-[var(--color-text-secondary)]">
              Steps ({form.values.steps.length})
            </label>
            <Button type="button" size="sm" variant="outline" onClick={addStep}>
              <Plus size={14} className="mr-1" /> Add Step
            </Button>
          </div>

          <div className="steps-list space-y-3">
            {form.values.steps.length === 0 && (
              <div className="text-center py-8 text-[var(--color-text-tertiary)] italic bg-[var(--color-surface)] rounded-lg border border-dashed border-[var(--color-border)]">
                No steps defined yet. Click "Add Step" to begin.
              </div>
            )}
            {form.values.steps.map((step, index) => (
              <div
                key={index}
                className="bg-[var(--color-surface)] rounded-lg p-3 border border-[var(--color-border)] relative group"
              >
                <div className="absolute right-2 top-2 opacity-0 group-hover:opacity-100 transition-opacity">
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    onClick={() => removeStep(index)}
                    className="text-[var(--color-error-500)] hover:opacity-80 p-1 rounded hover:bg-[var(--color-background)]"
                    title="Remove step"
                  >
                    <Trash2 size={14} />
                  </Button>
                </div>
                <div className="flex items-center gap-2 mb-2">
                  <span className="text-xs font-mono text-[var(--color-text-tertiary)] bg-[var(--color-background)] px-1.5 py-0.5 rounded border border-[var(--color-border)]">
                    {index + 1}
                  </span>
                  <div className="flex-1 flex items-center gap-2">
                    <div className="flex-1">
                      <Input
                        value={step.from}
                        onChange={(e) => updateStep(index, "from", e.target.value)}
                        placeholder="From Node"
                        list={`flow-from-options-${index}`}
                        className="h-8 text-sm"
                      />
                      <datalist id={`flow-from-options-${index}`}>
                        {nodeIds.map((nid) => (
                          <option key={nid} value={nid} />
                        ))}
                      </datalist>
                    </div>
                    <ArrowRight size={14} className="text-[var(--color-text-tertiary)]" />
                    <div className="flex-1">
                      <Input
                        value={step.to}
                        onChange={(e) => updateStep(index, "to", e.target.value)}
                        placeholder="To Node"
                        list={`flow-to-options-${index}`}
                        className="h-8 text-sm"
                      />
                      <datalist id={`flow-to-options-${index}`}>
                        {nodeIds.map((nid) => (
                          <option key={nid} value={nid} />
                        ))}
                      </datalist>
                    </div>
                  </div>
                  <div className="w-8"></div> {/* Spacer for delete button */}
                </div>
                <Input
                  value={step.description || ""}
                  onChange={(e) => updateStep(index, "description", e.target.value)}
                  placeholder="Step description (what happens here?)"
                  className="text-sm"
                />
              </div>
            ))}
          </div>
        </div>
      </form>
    </SidePanel>
  );
}
