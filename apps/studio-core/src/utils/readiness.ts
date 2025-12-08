
import { ArchitectureJSON, ArchitectureBody } from '@sruja/viewer';

export interface ReadinessItem {
    id: string;
    label: string;
    completed: boolean;
    nodeId?: string; // If clicking should focus something
}

export interface ReadinessReport {
    overallScore: number;
    items: ReadinessItem[];
}

export function calculateReadiness(archData: ArchitectureJSON | null): ReadinessReport {
    if (!archData) {
        return { overallScore: 0, items: [] };
    }

    const items: ReadinessItem[] = [];
    const arch = archData.architecture || {} as ArchitectureBody;

    // 1. Context Check (Person & System)
    const hasPerson = (arch.persons?.length || 0) > 0;
    items.push({
        id: 'has-person',
        label: 'Define at least one Person',
        completed: hasPerson
    });

    const hasSystem = (arch.systems?.length || 0) > 0;
    items.push({
        id: 'has-system',
        label: 'Define at least one System',
        completed: hasSystem
    });

    // 2. Container Checks
    let totalContainers = 0;
    let validContainers = 0;

    // Check top-level containers (if supported) or containers inside systems
    if (arch.systems) {
        arch.systems.forEach(sys => {
            if (sys.containers) {
                sys.containers.forEach(cont => {
                    totalContainers++;
                    if ((cont as any).technology) {
                        validContainers++;
                    } else {
                        items.push({
                            id: `cont-tech-${cont.id}`,
                            label: `Add technology to container "${cont.label || cont.id}"`,
                            completed: false,
                            nodeId: cont.id
                        });
                    }
                });
            }
        });
    }

    // 3. Component Checks (orphan check example)
    // For simplicity, let's just check description presence
    if (arch.systems) {
        arch.systems.forEach(sys => {
            if (sys.containers) {
                sys.containers.forEach(cont => {
                    if (cont.components) {
                        cont.components.forEach(comp => {
                            if (!comp.description) {
                                items.push({
                                    id: `comp-desc-${comp.id}`,
                                    label: `Add description to component "${comp.label || comp.id}"`,
                                    completed: false,
                                    nodeId: comp.id
                                });
                            }
                        });
                    }
                });
            }
        });
    }

    // Calculate Score
    // Base score from Context (50%) + Detail score (remaining 50%)
    let contextScore = 0;
    if (hasPerson) contextScore += 25;
    if (hasSystem) contextScore += 25;

    // Remaining 50% distributed among detailed tasks?
    // Let's keep it simple: Ratio of completed rules.
    // We have 2 fixed rules + N dynamic rules (where items are only added if INVALID).
    // This makes "completed items" hard to count because we only add negative items.

    // Approach: Total possible points logic.
    // Start with 100. Deduct for missing things?
    // Or: Count passed checks.

    // Let's refine:
    // We strictly added items for Failures (except the first two).
    // So current success count = (hasPerson ? 1 : 0) + (hasSystem ? 1 : 0).
    // Total checks = 2 + (failures).
    // This is weird.

    // Better Logic:
    // Fixed Goals: 
    // 1. Has Person
    // 2. Has System
    // Dynamic Goals:
    // For every container found, we add a generic "Container Technology" check.
    // For every component, "Component Description" check.

    // Re-run loops to add "Success" items too, or just compute score differently.

    // Let's just create a list of Todo items.
    // Score = max(0, 100 - (items.filter(i => !i.completed).length * 10)).
    // Capped at 100.

    const uncompletedCount = items.filter(i => !i.completed).length;
    const computedScore = Math.max(0, 100 - (uncompletedCount * 10));

    // If we have 0 persons and 0 systems, score starts at 80? That's generous.
    // Maybe weighted.
    // If !hasPerson or !hasSystem, max score is 50.

    let finalScore = computedScore;
    if (!hasPerson || !hasSystem) {
        finalScore = Math.min(finalScore, 40); // Cap low if basics missing
    }

    return {
        overallScore: finalScore,
        items: items.sort((a, b) => Number(a.completed) - Number(b.completed)) // Uncompleted first
    };
}
