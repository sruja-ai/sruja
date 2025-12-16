// apps/playground/src/components/shared/forms/EditFlowForm.tsx
import { useState, useEffect, useRef } from "react";
import { useArchitectureStore } from "../../../stores";
import type { FlowJSON, ScenarioStepJSON } from "../../../types";
import { SidePanel } from "../SidePanel";
import { ArrowRight, Trash2, Plus } from "lucide-react";
import { Button, Input, Textarea } from "@sruja/ui";
import "../EditForms.css";

interface EditFlowFormProps {
  isOpen: boolean;
  onClose: () => void;
  flow?: FlowJSON;
}

export function EditFlowForm({ isOpen, onClose, flow }: EditFlowFormProps) {
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const data = useArchitectureStore((s) => s.data);
  const formRef = useRef<HTMLFormElement>(null);
  const [id, setId] = useState(flow?.id || "");
  const [title, setTitle] = useState(flow?.title || flow?.label || "");
  const [description, setDescription] = useState(flow?.description || "");
  const [steps, setSteps] = useState<ScenarioStepJSON[]>(flow?.steps || []);
  const [errors, setErrors] = useState<Record<string, string>>({});

  useEffect(() => {
    if (isOpen) {
      setId(flow?.id || "");
      setTitle(flow?.title || flow?.label || "");
      setDescription(flow?.description || "");
      setSteps(flow?.steps || []);
      setErrors({});
    }
  }, [isOpen, flow]);

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
    setSteps([...steps, { from: "", to: "", description: "" }]);
  };

  const updateStep = (index: number, field: keyof ScenarioStepJSON, value: string) => {
    const newSteps = [...steps];
    newSteps[index] = { ...newSteps[index], [field]: value };
    setSteps(newSteps);
  };

  const removeStep = (index: number) => {
    setSteps(steps.filter((_, i) => i !== index));
  };

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
        const flows = [...(arch.architecture.flows || [])];

        const newFlow: FlowJSON = {
          id: id.trim(),
          title: title.trim(),
          description: description.trim() || undefined,
          steps: steps
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
          ...arch,
          architecture: {
            ...arch.architecture,
            flows,
          },
        };
      });
      onClose();
    } catch (err) {
      console.error("Failed to update flow:", err);
      setErrors({ submit: "Failed to save flow. Please try again." });
    }
  };

  // Get all available node IDs for autocomplete
  const nodeIds: string[] = [];
  if (data?.architecture) {
    data.architecture.systems?.forEach((s) => {
      nodeIds.push(s.id);
      s.containers?.forEach((c) => {
        nodeIds.push(`${s.id}.${c.id}`);
        c.components?.forEach((comp) => {
          nodeIds.push(`${s.id}.${c.id}.${comp.id}`);
        });
      });
    });
    data.architecture.persons?.forEach((p) => nodeIds.push(p.id));
  }

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
            onClick={(e) => {
              e.preventDefault();
              handleSubmit(e as any);
            }}
          >
            {flow ? "Update" : "Add"}
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
          placeholder="F1"
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
          placeholder="Flow title"
          error={errors.title}
        />
        <Textarea
          label="Description"
          value={description}
          onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => setDescription(e.target.value)}
          rows={3}
          placeholder="Flow description"
        />

        <div className="form-group border-t border-[var(--color-border)] pt-4 mt-2">
          <div className="flex items-center justify-between mb-2">
            <label className="text-sm font-medium text-[var(--color-text-secondary)]">
              Steps ({steps.length})
            </label>
            <Button type="button" size="sm" variant="outline" onClick={addStep}>
              <Plus size={14} className="mr-1" /> Add Step
            </Button>
          </div>

          <div className="steps-list space-y-3">
            {steps.length === 0 && (
              <div className="text-center py-8 text-[var(--color-text-tertiary)] italic bg-[var(--color-surface)] rounded-lg border border-dashed border-[var(--color-border)]">
                No steps defined yet. Click "Add Step" to begin.
              </div>
            )}
            {steps.map((step, index) => (
              <div
                key={index}
                className="bg-[var(--color-surface)] rounded-lg p-3 border border-[var(--color-border)] relative group"
              >
                <div className="absolute right-2 top-2 opacity-0 group-hover:opacity-100 transition-opacity">
                  <button
                    type="button"
                    onClick={() => removeStep(index)}
                    className="text-[var(--color-error-500)] hover:opacity-80 p-1 rounded hover:bg-[var(--color-background)]"
                    title="Remove step"
                  >
                    <Trash2 size={14} />
                  </button>
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
