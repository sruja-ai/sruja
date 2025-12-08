import { X, Info, Tag } from 'lucide-react';
import { useArchitectureStore, useSelectionStore } from '../../stores';
import type { SystemJSON, ContainerJSON, ComponentJSON, PersonJSON } from '../../types';
import './DetailsPanel.css';

export function DetailsPanel() {
    const data = useArchitectureStore((s) => s.data);
    const selectedNodeId = useSelectionStore((s) => s.selectedNodeId);
    const selectNode = useSelectionStore((s) => s.selectNode);

    if (!selectedNodeId || !data) {
        return null;
    }

    // Find the selected node in the data
    const nodeInfo = findNode(data, selectedNodeId);

    if (!nodeInfo) {
        return null;
    }

    const { node, type } = nodeInfo;

    return (
        <div className="details-panel">
            <div className="details-header">
                <h3 className="details-title">{getLabel(node)}</h3>
                <button className="close-btn" onClick={() => selectNode(null)}>
                    <X size={16} />
                </button>
            </div>

            <div className="details-content">
                <div className="detail-row">
                    <span className="detail-label">Type</span>
                    <span className="detail-value type-badge">{type}</span>
                </div>

                <div className="detail-row">
                    <span className="detail-label">ID</span>
                    <span className="detail-value code">{node.id}</span>
                </div>

                {getDescription(node) && (
                    <div className="detail-section">
                        <div className="section-title">
                            <Info size={14} />
                            Description
                        </div>
                        <p className="description-text">{getDescription(node)}</p>
                    </div>
                )}

                {'technology' in node && node.technology && (
                    <div className="detail-row">
                        <span className="detail-label">Technology</span>
                        <span className="detail-value">{node.technology}</span>
                    </div>
                )}

                {'tags' in node && node.tags && node.tags.length > 0 && (
                    <div className="detail-section">
                        <div className="section-title">
                            <Tag size={14} />
                            Tags
                        </div>
                        <div className="tags-list">
                            {node.tags.map((tag, i) => (
                                <span key={i} className="tag">{tag}</span>
                            ))}
                        </div>
                    </div>
                )}

                {/* Child counts */}
                {'containers' in node && node.containers && node.containers.length > 0 && (
                    <div className="detail-row">
                        <span className="detail-label">Containers</span>
                        <span className="detail-value">{node.containers.length}</span>
                    </div>
                )}

                {'components' in node && node.components && node.components.length > 0 && (
                    <div className="detail-row">
                        <span className="detail-label">Components</span>
                        <span className="detail-value">{node.components.length}</span>
                    </div>
                )}
            </div>
        </div>
    );
}

// Helper functions
function findNode(
    data: ReturnType<typeof useArchitectureStore.getState>['data'],
    id: string
): { node: SystemJSON | ContainerJSON | ComponentJSON | PersonJSON; type: string } | null {
    if (!data) return null;
    const arch = data.architecture;

    // Check persons
    const person = arch.persons?.find((p) => p.id === id);
    if (person) return { node: person, type: 'Person' };

    // Check systems and their children
    for (const system of arch.systems ?? []) {
        if (system.id === id) return { node: system, type: 'System' };

        for (const container of system.containers ?? []) {
            if (container.id === id) return { node: container, type: 'Container' };

            for (const component of container.components ?? []) {
                if (component.id === id) return { node: component, type: 'Component' };
            }
        }
    }

    return null;
}

function getLabel(node: { id: string; label?: string }): string {
    return node.label ?? node.id;
}

function getDescription(node: { description?: string }): string | undefined {
    return node.description;
}
