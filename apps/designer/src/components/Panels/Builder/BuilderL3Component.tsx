import { useState } from "react";
import { Button, Input, Select } from "@sruja/ui";
import { Info, Plus } from "lucide-react";
import { useArchitectureStore } from "../../../stores/architectureStore";
import { useFeatureFlagsStore } from "../../../stores/featureFlagsStore";
import { useBuilderProgress } from "../../../hooks/useBuilderProgress";
import type { SrujaModelDump } from "@sruja/shared";

const slugify = (text: string) =>
    text
        .toLowerCase()
        .trim()
        .replace(/[^a-z0-9_-]+/g, "-")
        .replace(/^-+|-+$/g, "");

export function BuilderL3Component() {
    const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
    const isEditMode = useFeatureFlagsStore((s) => s.isEditMode);
    const {
        systems,
        allContainers,
        componentsAll,
        componentsWithTech
    } = useBuilderProgress();

    // Form State
    const [activeSystemId, setActiveSystemId] = useState<string>(systems[0]?.id || "");
    // Derived containers for active system
    const activeSystemContainers = allContainers.filter(c => c.id.startsWith(`${activeSystemId}.`) && c.id.split(".").length === 2);

    const [activeContainerId, setActiveContainerId] = useState<string>("");

    // Auto-select first container if changed system
    if (activeSystemId && activeSystemContainers.length > 0 && !activeSystemContainers.find(c => c.id === activeContainerId)) {
        // This is a render side-effect, ideally use useEffect, but for simple wizard it might pass if handled carefully.
        // Better:
    }
    // Actually let's just default in render or use effect.

    const [componentName, setComponentName] = useState("");
    const [componentTech, setComponentTech] = useState("");
    const [componentDescription, setComponentDescription] = useState("");

    const submitAddComponent = async () => {
        if (!activeContainerId) return;
        const id = slugify(componentName) || "component";
        if (!id) return;

        await updateArchitecture((model: SrujaModelDump) => {
            const elements = { ...(model.elements || {}) };
            const fullId = `${activeContainerId}.${id}`;
            if (!elements[fullId]) {
                elements[fullId] = {
                    id: fullId as any,
                    kind: "component",
                    title: componentName,
                    description: (componentDescription || undefined) as any,
                    technology: componentTech || undefined,
                    tags: [],
                    style: {} as any,
                    links: [],
                };
            }
            return { ...model, elements };
        });

        setComponentName("");
        setComponentTech("");
        setComponentDescription("");
    };

    const activeContainerComponents = componentsAll.filter(c => c.id.startsWith(`${activeContainerId}.`) && c.id.split(".").length === 3);

    return (
        <>
            <div className="guided-description">
                <p>
                    Define components within your containers.
                </p>
            </div>

            <div className="grid grid-cols-2 gap-4 mb-6">
                <Select
                    label="System"
                    value={activeSystemId}
                    onChange={(value) => { setActiveSystemId(value || ""); setActiveContainerId(""); }}
                    data={systems.map(s => ({ value: s.id, label: s.title || s.id }))}
                />
                <Select
                    label="Container"
                    value={activeContainerId}
                    onChange={(value) => setActiveContainerId(value || "")}
                    disabled={activeSystemContainers.length === 0}
                    placeholder="Select Container..."
                    data={activeSystemContainers.map(c => ({ value: c.id, label: c.title || c.id }))}
                />
            </div>

            {isEditMode() && activeContainerId && (
                <div className="guided-actions">
                    <h3 className="text-sm font-medium text-[var(--color-text-secondary)] mb-2">
                        Add Component
                    </h3>
                    <Input
                        label="Component Name"
                        value={componentName}
                        onChange={(e) => setComponentName(e.target.value)}
                        placeholder="e.g., AuthController"
                    />
                    <Input
                        label="Technology"
                        value={componentTech}
                        onChange={(e) => setComponentTech(e.target.value)}
                        placeholder="e.g., Spring Boot"
                    />
                    <Button variant="primary" onClick={submitAddComponent} type="button">
                        <Plus size={16} className="mr-2" /> Add Component
                    </Button>
                </div>
            )}

            <div className="space-y-2 my-6">
                {activeContainerId ? (
                    activeContainerComponents.length === 0 ? (
                        <div className="text-center py-4 bg-[var(--color-surface)] text-[var(--color-text-tertiary)]">
                            No components defined.
                        </div>
                    ) : (
                        activeContainerComponents.map(c => (
                            <div key={c.id} className="p-2 border-b border-[var(--color-border)] last:border-0 flex justify-between">
                                <span>{c.title || c.id}</span>
                                <span className="text-xs text-[var(--color-text-secondary)]">{c.technology}</span>
                            </div>
                        ))
                    )
                ) : (
                    <div className="text-center py-4 text-[var(--color-text-tertiary)]">
                        Select a container to view components.
                    </div>
                )}
            </div>
            <div className="guided-checklist">
                <div className="check-item optional">
                    <Info size={14} />
                    <span>
                        {componentsWithTech > 0 ? `${componentsWithTech} components have technology` : "Document technologies"}
                    </span>
                </div>
            </div>
        </>
    );
}
