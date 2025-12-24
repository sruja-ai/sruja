import { useState } from "react";
import { ArrowRight, Plus, Trash2 } from "lucide-react";
import { Button, Input, Select } from "@sruja/ui";
import { useArchitectureStore } from "../../stores/architectureStore";
import type { RelationDump } from "@sruja/shared";
import "./WizardSteps.css";

interface ElementOption {
  id: string;
  label: string;
  type: string;
}

interface RelationsSectionProps {
  /** Elements available for "from" selection */
  fromElements: ElementOption[];
  /** Elements available for "to" selection */
  toElements: ElementOption[];
  /** Optional filter to show only relevant relations */
  filterFn?: (rel: RelationDump) => boolean;
  /** Title for the section */
  title?: string;
  /** Description for the section */
  description?: string;
}

export function RelationsSection({
  fromElements,
  toElements,
  filterFn,
  title = "Relations",
  description = "Connect elements to show how they communicate",
}: RelationsSectionProps) {
  const data = useArchitectureStore((s) => s.likec4Model);
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);

  const allRelations = data?.relations ?? [];
  const relations = filterFn ? allRelations.filter(filterFn) : allRelations;

  // Form state
  const [fromId, setFromId] = useState("");
  const [toId, setToId] = useState("");
  const [label, setLabel] = useState("");

  const addRelation = () => {
    if (!fromId || !toId || !data) return;

    // We can't easily generate a unique ID without full list, but updateArchitecture helper usually handles or push.
    // For now we assume we are just pushing a structure that will be handled.
    // Actually RelationDump has mandatory ID. We need to generate one.
    // Simplified: Just pass data to updateArchitecture and let it handle logic if possible,
    // but here we are manipulating the model directly.
    // We should probably rely on a 'addRelation' action in store if available, or generate a random ID.
    const newId = `rel_${Math.random().toString(36).substr(2, 9)}`;

    const newRelation: RelationDump = {
      id: newId as any,
      source: { model: fromId } as any,
      target: { model: toId } as any,
      title: label.trim() || undefined,
    };

    updateArchitecture((model) => ({
      ...model,
      relations: [...(model.relations || []), newRelation],
    }));

    setFromId("");
    setToId("");
    setLabel("");
  };

  const removeRelation = (id: string) => {
    updateArchitecture((model) => ({
      ...model,
      relations: (model.relations || []).filter((r) => r.id !== id),
    }));
  };

  const getElementLabel = (id: string) => {
    const el = [...fromElements, ...toElements].find((e) => e.id === id);
    return el?.label || id;
  };

  // Don't render if no elements to connect
  if (fromElements.length === 0 || toElements.length === 0) {
    return null;
  }

  return (
    <div className="step-section relations-section">
      <h3>
        <ArrowRight size={18} />
        {title}
        <span className="count-badge">{relations.length}</span>
      </h3>
      <p className="section-description">{description}</p>

      <div className="items-list">
        {(relations || []).map((rel, index: number) => {
          const srcFqn =
            typeof rel.source === "object" && rel.source?.model
              ? rel.source.model
              : String(rel.source || "");
          const tgtFqn =
            typeof rel.target === "object" && rel.target?.model
              ? rel.target.model
              : String(rel.target || "");
          return (
            <div key={rel.id || `${srcFqn}-${tgtFqn}-${index}`} className="item-card relation-card">
              <span className="relation-element from">{getElementLabel(srcFqn)}</span>
              <ArrowRight size={14} className="relation-arrow-small" />
              <span className="relation-element to">{getElementLabel(tgtFqn)}</span>
              {rel.title && <span className="relation-label-text">"{rel.title}"</span>}
              <Button
                variant="ghost"
                size="sm"
                className="item-remove"
                onClick={() => removeRelation(rel.id)}
              >
                <Trash2 size={14} />
              </Button>
            </div>
          );
        })}
      </div>

      <div className="add-form relation-form-inline">
        <div className="form-group">
          <Select
            size="sm"
            value={fromId}
            onChange={(value) => setFromId(value || "")}
            placeholder="From..."
            data={fromElements.map((el) => ({ value: el.id, label: el.id }))}
          />
        </div>

        <ArrowRight size={16} className="form-arrow" />

        <div className="form-group">
          <Select
            size="sm"
            value={toId}
            onChange={(value) => setToId(value || "")}
            placeholder="To..."
            data={toElements
              .filter((e) => e.id !== fromId)
              .map((el) => ({ value: el.id, label: el.id }))}
          />
        </div>

        <Input
          size="sm"
          label=""
          value={label}
          onChange={(e) => setLabel(e.target.value)}
          placeholder="Label (e.g., uses)"
          onKeyDown={(e) => e.key === "Enter" && addRelation()}
          style={{ flex: 1.5, minWidth: "120px" }}
        />

        <Button variant="secondary" onClick={addRelation} disabled={!fromId || !toId} size="sm">
          <Plus size={16} />
        </Button>
      </div>
    </div>
  );
}
