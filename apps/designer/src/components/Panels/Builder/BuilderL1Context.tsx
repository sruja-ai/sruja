import { useState } from "react";
import { Button, Input, Textarea } from "@sruja/ui";
import { CheckCircle, AlertCircle, Info } from "lucide-react";
import { useArchitectureStore } from "../../../stores/architectureStore";
import { useViewStore } from "../../../stores/viewStore";
import { useFeatureFlagsStore } from "../../../stores/featureFlagsStore";
import { BestPracticeTip } from "../../shared";
import { useBuilderProgress } from "../../../hooks/useBuilderProgress";
import type { SrujaModelDump } from "@sruja/shared";

const slugify = (text: string) =>
    text
        .toLowerCase()
        .trim()
        .replace(/[^a-z0-9_-]+/g, "-")
        .replace(/^-+|-+$/g, "");

export function BuilderL1Context() {
    const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
    const isEditMode = useFeatureFlagsStore((s) => s.isEditMode);
    const drillDown = useViewStore((s) => s.drillDown);

    const {
        systems,
        persons,
        relations,
        requirements,
        scenarios,
        flows,
        hasL1Relations,
        l1Complete,
        systemsWithDescriptions,
        l1ReqTagged,
    } = useBuilderProgress();

    // Local Form State
    const [systemName, setSystemName] = useState("");
    const [systemDescription, setSystemDescription] = useState("");
    const [useCustomSystemId, setUseCustomSystemId] = useState(false);
    const [systemIdInput, setSystemIdInput] = useState("");
    const [personName, setPersonName] = useState("");

    const submitAddSystem = async () => {
        const id = (useCustomSystemId ? systemIdInput || "" : "") || slugify(systemName) || "system";
        if (!id) return;

        await updateArchitecture((model: SrujaModelDump) => {
            const elements = { ...(model.elements || {}) };
            if (!elements[id]) {
                elements[id] = {
                    id: id as any,
                    kind: "system",
                    title: systemName,
                    description: (systemDescription || undefined) as any,
                    technology: "",
                    tags: [],
                    links: [],
                    style: {} as any,
                };
            }
            return { ...model, elements };
        });
        setSystemName("");
        setSystemDescription("");
        setUseCustomSystemId(false);
        setSystemIdInput("");
    };

    const submitAddPerson = async () => {
        const id = slugify(personName) || "person";
        if (!id) return;

        await updateArchitecture((model: SrujaModelDump) => {
            const elements = { ...(model.elements || {}) };
            if (!elements[id]) {
                elements[id] = {
                    id: id as any,
                    kind: "person",
                    style: {} as any,
                    title: personName,
                    description: undefined,
                    technology: "",
                    tags: [],
                    links: [],
                };
            }
            return { ...model, elements };
        });
        setPersonName("");
    };

    return (
        <>
            <div className="guided-description">
                <p>
                    Define the high-level system context: actors (users, external systems) and your
                    system. Establish relationships between actors and systems.
                </p>
            </div>

            {/* L1 Best Practice Tips */}
            <BestPracticeTip variant="tip" show={systems.length === 0}>
                Start by identifying your main system and who interacts with it. Think: "What's the
                core product and who are its users?"
            </BestPracticeTip>

            <BestPracticeTip variant="tip" show={systems.length > 0 && !hasL1Relations}>
                Connect your actors to systems! Every actor should have at least one relationship
                showing how they interact with the system.
            </BestPracticeTip>

            <BestPracticeTip variant="warning" show={systems.length > 3}>
                You have {systems.length} systems at L1. If a system is internal to your main system,
                consider moving it to L2 as a container instead.
            </BestPracticeTip>

            <BestPracticeTip variant="success" show={l1Complete}>
                L1 is complete! You've defined your system context. Continue to L2 to break down your
                system into containers.
            </BestPracticeTip>
            <div className="guided-stats">
                <span>
                    {systems.length} system{systems.length !== 1 ? "s" : ""}
                </span>
                <span>
                    {persons.length} actor{persons.length !== 1 ? "s" : ""}
                </span>
                <span>
                    {relations.length} relation{relations.length !== 1 ? "s" : ""}
                </span>
                {requirements.length > 0 && (
                    <span>
                        {requirements.length} requirement{requirements.length !== 1 ? "s" : ""}
                    </span>
                )}
                {(scenarios.length > 0 || flows.length > 0) && (
                    <span>{scenarios.length + flows.length} scenario/flow</span>
                )}
            </div>
            {isEditMode() && (
                <div className="guided-actions">
                    <Input
                        label="System name"
                        value={systemName}
                        onChange={(e) => setSystemName(e.target.value)}
                        placeholder="e.g., WebApp"
                    />
                    <Textarea
                        label="Description"
                        value={systemDescription}
                        onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) =>
                            setSystemDescription(e.target.value)
                        }
                        rows={3}
                        placeholder="Responsibilities, boundaries, integrations"
                    />
                    <div className="form-group checkbox-row">
                        <input
                            id="guided-system-custom-id"
                            type="checkbox"
                            checked={useCustomSystemId}
                            onChange={(e) => setUseCustomSystemId(e.target.checked)}
                        />
                        <label htmlFor="guided-system-custom-id">Set custom ID (optional)</label>
                    </div>
                    {useCustomSystemId && (
                        <Input
                            label="ID"
                            value={systemIdInput}
                            onChange={(e) => setSystemIdInput(e.target.value)}
                            placeholder="If empty, ID is auto-generated from name"
                        />
                    )}
                    <Button variant="primary" onClick={submitAddSystem} type="button">
                        Add System
                    </Button>
                    <Input
                        label="Actor name"
                        value={personName}
                        onChange={(e) => setPersonName(e.target.value)}
                        placeholder="e.g., User"
                    />
                    <Button variant="secondary" onClick={submitAddPerson} type="button">
                        Add Actor
                    </Button>
                </div>
            )}
            <div className="guided-next">
                <Button
                    variant="primary"
                    disabled={systems.length === 0}
                    onClick={() => systems[0] && drillDown(systems[0].id, "system")}
                    type="button"
                >
                    Continue to L2
                </Button>
            </div>
            <div className="guided-checklist">
                <div className="check-item">
                    {persons.length > 0 ? (
                        <CheckCircle size={14} color="#22c55e" />
                    ) : (
                        <AlertCircle size={14} color="#f59e0b" />
                    )}
                    <span>At least one actor</span>
                </div>
                <div className="check-item">
                    {systems.length > 0 ? (
                        <CheckCircle size={14} color="#22c55e" />
                    ) : (
                        <AlertCircle size={14} color="#f59e0b" />
                    )}
                    <span>At least one system</span>
                </div>
                <div className="check-item">
                    {hasL1Relations ? (
                        <CheckCircle size={14} color="#22c55e" />
                    ) : (
                        <AlertCircle size={14} color="#f59e0b" />
                    )}
                    <span>Actor ↔ System relationships defined</span>
                </div>
                <div className="check-item optional">
                    <Info size={14} />
                    <span>
                        {systemsWithDescriptions > 0
                            ? `${systemsWithDescriptions}/${systems.length} systems documented`
                            : "Document system descriptions"}
                    </span>
                </div>
                <div className="check-item optional">
                    <Info size={14} />
                    <span>
                        {l1ReqTagged > 0
                            ? `${l1ReqTagged} requirements tagged to actors/systems`
                            : "Tag requirements to actors/systems"}
                    </span>
                </div>
            </div>
        </>
    );
}
