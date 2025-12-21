import { useState } from "react";
import { Button, Input, Textarea, Checkbox } from "@sruja/ui";
import { CheckCircle, AlertCircle, Info, Plus } from "lucide-react";
import { useArchitectureStore } from "../../../stores/architectureStore";
import { useViewStore } from "../../../stores/viewStore";
import { useFeatureFlagsStore } from "../../../stores/featureFlagsStore";
import { useBuilderProgress } from "../../../hooks/useBuilderProgress";
import type { SrujaModelDump } from "@sruja/shared";

const slugify = (text: string) =>
    text
        .toLowerCase()
        .trim()
        .replace(/[^a-z0-9_-]+/g, "-")
        .replace(/^-+|-+$/g, "");

export function BuilderL2Container() {
    const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
    const isEditMode = useFeatureFlagsStore((s) => s.isEditMode);
    const drillDown = useViewStore((s) => s.drillDown);
    const {
        systems,
        l1Complete,
        containersWithTech,
        allContainers: containers,
    } = useBuilderProgress();

    // Form State
    const [activeSystemId, setActiveSystemId] = useState<string>(systems[0]?.id || "");
    const [containerName, setContainerName] = useState("");
    const [containerTech, setContainerTech] = useState("");
    const [containerDescription, setContainerDescription] = useState("");
    const [useCustomContainerId, setUseCustomContainerId] = useState(false);
    const [containerIdInput, setContainerIdInput] = useState("");

    // Submit Handler
    const submitAddContainer = async () => {
        if (!activeSystemId) return;
        const id = (useCustomContainerId ? containerIdInput || "" : "") || slugify(containerName) || "container";
        if (!id) return;

        // Ensure ID is unique globally (simple check)
        // In real app, we should check existing IDs

        await updateArchitecture((model: SrujaModelDump) => {
            const elements = { ...(model.elements || {}) };
            const fullId = `${activeSystemId}.${id} `; // Enforce hierarchy in ID for now
            if (!elements[fullId]) {
                elements[fullId] = {
                    id: fullId as any,
                    kind: "container",
                    title: containerName,
                    description: (containerDescription || undefined) as any,
                    technology: containerTech || undefined,
                    tags: [],
                    style: {} as any,
                    links: [],
                    // parent: activeSystemId  // If Supported
                };
            }
            return { ...model, elements };
        });

        setContainerName("");
        setContainerTech("");
        setContainerDescription("");
        setUseCustomContainerId(false);
        setContainerIdInput("");
    };

    // Filter containers for active system
    // We assume ID hierarchy "sys.container" for now as per useBuilderProgress assumptions
    const activeSystemContainers = containers.filter(c => {
        // Check if ID starts with systemId + "." and has no more dots (depth check)
        // Or usage of containerParentById if reliable
        return c.id.startsWith(`${activeSystemId}.`) && c.id.split(".").length === 2;
    });


    if (!l1Complete) {
        return (
            <div className="p-8 text-center text-[var(--color-text-secondary)]">
                <AlertCircle size={48} className="mx-auto mb-4 opacity-50" />
                <h3 className="text-lg font-medium mb-2">Complete L1 First</h3>
                <p>You need to define systems and actors before moving to L2.</p>
            </div>
        );
    }

    return (
        <>
            <div className="guided-description">
                <p>
                    Break down your system into containers (applications, data stores, microservices).
                </p>
            </div>

            {/* System Selector */}
            <div className="mb-6">
                <label className="block text-sm font-medium mb-2">Select System to Edit</label>
                <div className="flex gap-2 flex-wrap">
                    {systems.map((sys) => (
                        <Button
                            key={sys.id}
                            variant={activeSystemId === sys.id ? "primary" : "outline"}
                            size="sm"
                            onClick={() => setActiveSystemId(sys.id)}
                        >
                            {sys.title || sys.id}
                        </Button>
                    ))}
                </div>
            </div>

            {isEditMode() && activeSystemId && (
                <div className="guided-actions">
                    <h3 className="text-sm font-medium text-[var(--color-text-secondary)] mb-2">
                        Add Container to {systems.find(s => s.id === activeSystemId)?.title}
                    </h3>
                    <Input
                        label="Container Name"
                        value={containerName}
                        onChange={(e) => setContainerName(e.target.value)}
                        placeholder="e.g., API Server"
                    />
                    <Input
                        label="Technology"
                        value={containerTech}
                        onChange={(e) => setContainerTech(e.target.value)}
                        placeholder="e.g., Node.js, PostgreSQL"
                    />
                    <Textarea
                        label="Description"
                        value={containerDescription}
                        onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => setContainerDescription(e.target.value)}
                        rows={2}
                    />
                    <Checkbox
                        id="guided-container-custom-id"
                        label="Use custom ID"
                        checked={useCustomContainerId}
                        onChange={(e) => setUseCustomContainerId(e.currentTarget.checked)}
                    />
                    {useCustomContainerId && (
                        <Input
                            label="ID Suffix"
                            value={containerIdInput}
                            onChange={(e) => setContainerIdInput(e.target.value)}
                            placeholder="will be appended to system ID"
                        />
                    )}
                    <Button variant="primary" onClick={submitAddContainer} type="button">
                        <Plus size={16} className="mr-2" /> Add Container
                    </Button>
                </div>
            )}

            {/* List Existing Containers */}
            <div className="space-y-4 my-6">
                {activeSystemContainers.length === 0 ? (
                    <div className="text-center py-8 bg-[var(--color-surface-hover)] rounded-lg text-[var(--color-text-tertiary)]">
                        No containers in this system yet.
                    </div>
                ) : (
                    activeSystemContainers.map((c) => (
                        <div key={c.id} className="p-3 bg-[var(--color-surface)] rounded border border-[var(--color-border)] flex justify-between items-center">
                            <div>
                                <div className="font-medium">{c.title || c.id}</div>
                                <div className="text-xs text-[var(--color-text-secondary)]">
                                    {c.technology} {c.kind !== 'container' && `(${c.kind})`}
                                </div>
                            </div>
                            <Button size="sm" variant="outline" onClick={() => drillDown(c.id, "container")}>
                                View
                            </Button>
                        </div>
                    ))
                )}
            </div>

            <div className="guided-checklist">
                <div className="check-item">
                    {activeSystemContainers.length > 0 ? (
                        <CheckCircle size={14} color="#22c55e" />
                    ) : (
                        <AlertCircle size={14} color="#f59e0b" />
                    )}
                    <span>Current system has containers</span>
                </div>
                <div className="check-item optional">
                    <Info size={14} />
                    <span>
                        {containersWithTech > 0 ? `${containersWithTech} containers have technology` : "Document technologies"}
                    </span>
                </div>
            </div>
        </>
    );
}
