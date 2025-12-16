// apps/playground/src/components/shared/forms/EditScenarioForm.tsx
import { useState, useEffect, useRef } from "react";
import { useArchitectureStore } from "../../../stores";
import type { ScenarioJSON, ScenarioStepJSON } from "../../../types";
import { Button, Input, Textarea } from "@sruja/ui";
import { SidePanel } from "../SidePanel";
import "../EditForms.css";

interface EditScenarioFormProps {
  isOpen: boolean;
  onClose: () => void;
  scenario?: ScenarioJSON;
}

export function EditScenarioForm({ isOpen, onClose, scenario }: EditScenarioFormProps) {
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const data = useArchitectureStore((s) => s.data);
  const formRef = useRef<HTMLFormElement>(null);
  const [id, setId] = useState(scenario?.id || "");
  const [title, setTitle] = useState(scenario?.title || scenario?.label || "");
  const [description, setDescription] = useState(scenario?.description || "");
  const [steps, setSteps] = useState<ScenarioStepJSON[]>(scenario?.steps || []);
  const [errors, setErrors] = useState<Record<string, string>>({});

  useEffect(() => {
    if (isOpen) {
      setId(scenario?.id || "");
      setTitle(scenario?.title || scenario?.label || "");
      setDescription(scenario?.description || "");
      setSteps(scenario?.steps || []);
      setErrors({});
    }
  }, [isOpen, scenario]);

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
        const scenarios = [...(arch.architecture.scenarios || [])];
        const newScenario: ScenarioJSON = {
          id: id.trim(),
          title: title.trim(),
          description: description.trim() || undefined,
          steps:
            steps.filter((s) => s.from && s.to).length > 0
              ? steps.filter((s) => s.from && s.to)
              : undefined,
        };

        if (scenario) {
          const index = scenarios.findIndex((s) => s.id === scenario.id);
          if (index >= 0) {
            scenarios[index] = newScenario;
          }
        } else {
          scenarios.push(newScenario);
        }

        return {
          ...arch,
          architecture: {
            ...arch.architecture,
            scenarios,
          },
        };
      });
      onClose();
    } catch (err) {
      console.error("Failed to update scenario:", err);
      setErrors({ submit: "Failed to save scenario. Please try again." });
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
      title={scenario ? "Edit Scenario" : "Add Scenario"}
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
            {scenario ? "Update" : "Add"}
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
          placeholder="S1"
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
          placeholder="Scenario title"
          error={errors.title}
        />
        <Textarea
          label="Description"
          value={description}
          onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => setDescription(e.target.value)}
          rows={3}
          placeholder="Scenario description"
        />
        <div className="form-group">
          <label className="block mb-1.5 text-sm font-medium text-[var(--color-text-secondary)]">
            Steps
          </label>
          <div className="steps-list">
            {steps.map((step, index) => (
              <div key={index} className="step-item">
                <div>
                  <Input
                    value={step.from}
                    onChange={(e) => updateStep(index, "from", e.target.value)}
                    placeholder="From (e.g., User, System.Container)"
                    list={`from-options-${index}`}
                  />
                  <datalist id={`from-options-${index}`}>
                    {nodeIds.map((nid) => (
                      <option key={nid} value={nid} />
                    ))}
                  </datalist>
                </div>
                <span className="text-[var(--color-text-tertiary)] font-semibold">→</span>
                <div>
                  <Input
                    value={step.to}
                    onChange={(e) => updateStep(index, "to", e.target.value)}
                    placeholder="To (e.g., API, System.Container)"
                    list={`to-options-${index}`}
                  />
                  <datalist id={`to-options-${index}`}>
                    {nodeIds.map((nid) => (
                      <option key={nid} value={nid} />
                    ))}
                  </datalist>
                </div>
                <div>
                  <Input
                    value={step.description || ""}
                    onChange={(e) => updateStep(index, "description", e.target.value)}
                    placeholder="Description (optional)"
                  />
                </div>
                <Button type="button" variant="danger" size="sm" onClick={() => removeStep(index)}>
                  Remove
                </Button>
              </div>
            ))}
            <Button type="button" variant="outline" onClick={addStep}>
              + Add Step
            </Button>
          </div>
        </div>
      </form>
    </SidePanel>
  );
}
