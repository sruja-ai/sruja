import { useState } from "react";
import { ArrowRight, Plus, Trash2 } from "lucide-react";
import { Button, Input } from "@sruja/ui";
import { useArchitectureStore } from "../../stores/architectureStore";
import type { RelationJSON } from "../../types";
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
  filterFn?: (rel: RelationJSON) => boolean;
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
  const data = useArchitectureStore((s) => s.data);
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);

  const allRelations = data?.architecture?.relations ?? [];
  const relations = filterFn ? allRelations.filter(filterFn) : allRelations;

  // Form state
  const [fromId, setFromId] = useState("");
  const [toId, setToId] = useState("");
  const [label, setLabel] = useState("");

  const addRelation = () => {
    if (!fromId || !toId || !data?.architecture) return;

    const newRelation: RelationJSON = {
      from: fromId,
      to: toId,
      label: label.trim() || undefined,
    };

    updateArchitecture((arch) => ({
      ...arch,
      architecture: {
        ...arch.architecture,
        relations: [...(arch.architecture.relations ?? []), newRelation],
      },
    }));

    setFromId("");
    setToId("");
    setLabel("");
  };

  const removeRelation = (from: string, to: string, relLabel?: string) => {
    updateArchitecture((arch) => ({
      ...arch,
      architecture: {
        ...arch.architecture,
        relations: (arch.architecture.relations ?? []).filter(
          (r) => !(r.from === from && r.to === to && r.label === relLabel)
        ),
      },
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
        {relations.map((rel, index) => (
          <div key={`${rel.from}-${rel.to}-${index}`} className="item-card relation-card">
            <span className="relation-element from">{getElementLabel(rel.from)}</span>
            <ArrowRight size={14} className="relation-arrow-small" />
            <span className="relation-element to">{getElementLabel(rel.to)}</span>
            {rel.label && <span className="relation-label-text">"{rel.label}"</span>}
            <button
              className="item-remove"
              onClick={() => removeRelation(rel.from, rel.to, rel.label)}
            >
              <Trash2 size={14} />
            </button>
          </div>
        ))}
      </div>

      <div className="add-form relation-form-inline">
        <div className="form-group">
          <select value={fromId} onChange={(e) => setFromId(e.target.value)}>
            <option value="">From...</option>
            {fromElements.map((el) => (
              <option key={el.id} value={el.id}>
                {el.id}
              </option>
            ))}
          </select>
        </div>

        <ArrowRight size={16} className="form-arrow" />

        <div className="form-group">
          <select value={toId} onChange={(e) => setToId(e.target.value)}>
            <option value="">To...</option>
            {toElements
              .filter((e) => e.id !== fromId)
              .map((el) => (
                <option key={el.id} value={el.id}>
                  {el.id}
                </option>
              ))}
          </select>
        </div>

        <Input
          label=""
          value={label}
          onChange={(e) => setLabel(e.target.value)}
          placeholder="Label (e.g., uses)"
          onKeyDown={(e) => e.key === "Enter" && addRelation()}
        />

        <Button variant="secondary" onClick={addRelation} disabled={!fromId || !toId}>
          <Plus size={16} />
        </Button>
      </div>
    </div>
  );
}
