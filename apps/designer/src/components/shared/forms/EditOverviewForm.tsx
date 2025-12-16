// apps/playground/src/components/shared/forms/EditOverviewForm.tsx
import { useState, useEffect, useRef } from "react";
import { useArchitectureStore } from "../../../stores";
import type { OverviewJSON } from "../../../types";
import { Button, Input, Textarea } from "@sruja/ui";
import { SidePanel } from "../SidePanel";
import { X } from "lucide-react";
import "../EditForms.css";

interface EditOverviewFormProps {
  isOpen: boolean;
  onClose: () => void;
}

export function EditOverviewForm({ isOpen, onClose }: EditOverviewFormProps) {
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const data = useArchitectureStore((s) => s.data);
  const formRef = useRef<HTMLFormElement>(null);
  const overview = data?.architecture?.overview;
  const [architectureDescription, setArchitectureDescription] = useState<string>(
    data?.architecture?.description || ""
  );

  const [summary, setSummary] = useState(overview?.summary || "");
  const [audience, setAudience] = useState(overview?.audience || "");
  const [scope, setScope] = useState(overview?.scope || "");
  const [goals, setGoals] = useState<string[]>(overview?.goals || []);
  const [nonGoals, setNonGoals] = useState<string[]>(overview?.nonGoals || []);
  const [risks, setRisks] = useState<string[]>(overview?.risks || []);

  useEffect(() => {
    if (isOpen) {
      setSummary(overview?.summary || "");
      setAudience(overview?.audience || "");
      setScope(overview?.scope || "");
      setGoals(overview?.goals || []);
      setNonGoals(overview?.nonGoals || []);
      setRisks(overview?.risks || []);
      setArchitectureDescription(data?.architecture?.description || "");
    }
  }, [isOpen, overview, data]);

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

  const addItem = (list: string[], setter: (items: string[]) => void) => {
    setter([...list, ""]);
  };

  const updateItem = (
    list: string[],
    index: number,
    value: string,
    setter: (items: string[]) => void
  ) => {
    const newList = [...list];
    newList[index] = value;
    setter(newList);
  };

  const removeItem = (list: string[], index: number, setter: (items: string[]) => void) => {
    setter(list.filter((_, i) => i !== index));
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    try {
      await updateArchitecture((arch) => {
        if (!arch.architecture) return arch;
        const newOverview: OverviewJSON = {
          summary: summary.trim() || undefined,
          audience: audience.trim() || undefined,
          scope: scope.trim() || undefined,
          goals:
            goals.filter((g) => g.trim()).length > 0 ? goals.filter((g) => g.trim()) : undefined,
          nonGoals:
            nonGoals.filter((ng) => ng.trim()).length > 0
              ? nonGoals.filter((ng) => ng.trim())
              : undefined,
          risks:
            risks.filter((r) => r.trim()).length > 0 ? risks.filter((r) => r.trim()) : undefined,
        };

        return {
          ...arch,
          architecture: {
            ...arch.architecture,
            description: architectureDescription.trim() || undefined,
            overview: Object.keys(newOverview).some((k) => newOverview[k as keyof OverviewJSON])
              ? newOverview
              : undefined,
          },
        };
      });
      onClose();
    } catch (err) {
      console.error("Failed to update overview:", err);
    }
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
            onClick={(e) => {
              e.preventDefault();
              handleSubmit(e as any);
            }}
          >
            Save
          </Button>
        </>
      }
    >
      <form ref={formRef} onSubmit={handleSubmit} className="edit-form">
        <Textarea
          label="Architecture Description"
          value={architectureDescription}
          onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) =>
            setArchitectureDescription(e.target.value)
          }
          rows={3}
          placeholder="Purpose, scope, and high-level context of the architecture"
        />
        <Textarea
          label="Summary"
          value={summary}
          onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => setSummary(e.target.value)}
          rows={3}
          placeholder="High-level architecture summary"
        />
        <Input
          label="Audience"
          value={audience}
          onChange={(e) => setAudience(e.target.value)}
          placeholder="Target audience for this architecture"
        />
        <Input
          label="Scope"
          value={scope}
          onChange={(e) => setScope(e.target.value)}
          placeholder="Architecture scope"
        />
        <div className="form-group">
          <label className="block mb-1.5 text-sm font-medium text-[var(--color-text-secondary)]">
            Goals
          </label>
          <div className="list-items">
            {goals.map((goal, index) => (
              <div key={index} className="list-item">
                <div>
                  <Input
                    value={goal}
                    onChange={(e) => updateItem(goals, index, e.target.value, setGoals)}
                    placeholder="Enter a goal"
                  />
                </div>
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  onClick={() => removeItem(goals, index, setGoals)}
                >
                  <X size={16} />
                </Button>
              </div>
            ))}
            <Button type="button" variant="outline" onClick={() => addItem(goals, setGoals)}>
              + Add Goal
            </Button>
          </div>
        </div>
        <div className="form-group">
          <label className="block mb-1.5 text-sm font-medium text-[var(--color-text-secondary)]">
            Non-Goals
          </label>
          <div className="list-items">
            {nonGoals.map((ng, index) => (
              <div key={index} className="list-item">
                <div>
                  <Input
                    value={ng}
                    onChange={(e) => updateItem(nonGoals, index, e.target.value, setNonGoals)}
                    placeholder="Enter a non-goal"
                  />
                </div>
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  onClick={() => removeItem(nonGoals, index, setNonGoals)}
                >
                  <X size={16} />
                </Button>
              </div>
            ))}
            <Button type="button" variant="outline" onClick={() => addItem(nonGoals, setNonGoals)}>
              + Add Non-Goal
            </Button>
          </div>
        </div>
        <div className="form-group">
          <label className="block mb-1.5 text-sm font-medium text-[var(--color-text-secondary)]">
            Risks & Concerns
          </label>
          <div className="list-items">
            {risks.map((risk, index) => (
              <div key={index} className="list-item">
                <div>
                  <Input
                    value={risk}
                    onChange={(e) => updateItem(risks, index, e.target.value, setRisks)}
                    placeholder="Enter a risk or concern"
                  />
                </div>
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  onClick={() => removeItem(risks, index, setRisks)}
                >
                  <X size={16} />
                </Button>
              </div>
            ))}
            <Button type="button" variant="outline" onClick={() => addItem(risks, setRisks)}>
              + Add Risk
            </Button>
          </div>
        </div>
      </form>
    </SidePanel>
  );
}
