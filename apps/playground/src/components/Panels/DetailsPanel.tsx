import { X, Info, Tag, ArrowDownLeft, ArrowUpRight, ShieldCheck, FileText, FileCode } from 'lucide-react';
import { useArchitectureStore, useSelectionStore, useUIStore } from '../../stores';
import type { SystemJSON, ContainerJSON, ComponentJSON, PersonJSON } from '../../types';
import './DetailsPanel.css';

interface DetailsPanelProps {
    onClose?: () => void;
}

export function DetailsPanel({ onClose }: DetailsPanelProps) {
    const data = useArchitectureStore((s) => s.data);
    const selectedNodeId = useSelectionStore((s) => s.selectedNodeId);
    const selectNode = useSelectionStore((s) => s.selectNode);
    const setActiveTab = useUIStore((s) => s.setActiveTab);

    if (!selectedNodeId || !data) {
        return null;
    }

    // Find the selected node in the data
    const nodeInfo = findNode(data, selectedNodeId);

    if (!nodeInfo) {
        return null;
    }

    const { node, type } = nodeInfo;

    // Calculate dependencies from relations
    const arch = data.architecture;
    const relations = arch.relations || [];

    // Find incoming relations (what uses this node)
    const incoming = relations
        .filter(r => r.to === selectedNodeId || r.to === node.id)
        .map(r => ({
            relation: r,
            source: findNodeById(data, r.from)
        }))
        .filter(x => x.source);

    // Find outgoing relations (what this node uses)
    const outgoing = relations
        .filter(r => r.from === selectedNodeId || r.from === node.id)
        .map(r => ({
            relation: r,
            target: findNodeById(data, r.to)
        }))
        .filter(x => x.target);

    // Find linked requirements and ADRs (if node has them)
    const linkedReqs = 'requirements' in node && Array.isArray(node.requirements) ? node.requirements : [];
    const linkedADRs = 'adrs' in node && Array.isArray(node.adrs) ? node.adrs : [];

    const getTypeIcon = (nodeType: string) => {
        switch (nodeType?.toLowerCase()) {
            case 'person': return <Info size={12} />;
            case 'system': return <Info size={12} />;
            case 'container': return <Info size={12} />;
            case 'component': return <Info size={12} />;
            default: return <Info size={12} />;
        }
    };

    return (
        <div className="details-panel">
            <div className="details-header">
                <h3 className="details-title">{getLabel(node)}</h3>
                <div className="details-actions">
                    <button
                        className="action-icon-btn"
                        onClick={() => setActiveTab('code')}
                        title="View Source"
                        aria-label="View Source"
                    >
                        <FileCode size={16} />
                    </button>
                    <button
                        className="close-btn"
                        onClick={() => {
                            selectNode(null);
                            onClose?.();
                        }}
                        aria-label="Close details"
                    >
                        <X size={16} />
                    </button>
                </div>
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

                {/* Dependency Analysis */}
                {(incoming.length > 0 || outgoing.length > 0) && (
                    <div className="detail-section">
                        <div className="section-title">
                            <Info size={14} />
                            Dependency Analysis
                        </div>

                        {incoming.length > 0 && (
                            <div className="dependency-group">
                                <h4 className="dep-header">
                                    <ArrowDownLeft size={12} />
                                    Incoming (Used By)
                                </h4>
                                <div className="dep-list">
                                    {incoming.map((inc, idx) => (
                                        <div
                                            key={idx}
                                            className="dep-item"
                                            onClick={() => selectNode(inc.relation.from)}
                                        >
                                            <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                                                {getTypeIcon(inc.source?.type || '')}
                                                <span className="dep-name">{inc.source ? getLabel(inc.source.node) : inc.relation.from}</span>
                                            </div>
                                            {(inc.relation.label || inc.relation.verb) && (
                                                <span className="dep-desc">{inc.relation.label || inc.relation.verb}</span>
                                            )}
                                        </div>
                                    ))}
                                </div>
                            </div>
                        )}

                        {outgoing.length > 0 && (
                            <div className="dependency-group">
                                <h4 className="dep-header">
                                    <ArrowUpRight size={12} />
                                    Outgoing (Uses)
                                </h4>
                                <div className="dep-list">
                                    {outgoing.map((out, idx) => (
                                        <div
                                            key={idx}
                                            className="dep-item"
                                            onClick={() => selectNode(out.relation.to)}
                                        >
                                            <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                                                {getTypeIcon(out.target?.type || '')}
                                                <span className="dep-name">{out.target ? getLabel(out.target.node) : out.relation.to}</span>
                                            </div>
                                            {(out.relation.label || out.relation.verb) && (
                                                <span className="dep-desc">{out.relation.label || out.relation.verb}</span>
                                            )}
                                        </div>
                                    ))}
                                </div>
                            </div>
                        )}
                    </div>
                )}

                {/* Traceability - Requirements & ADRs */}
                {(linkedReqs.length > 0 || linkedADRs.length > 0) && (
                    <div className="detail-section">
                        <div className="section-title">
                            <Info size={14} />
                            Traceability
                        </div>

                        {linkedReqs.length > 0 && (
                            <div className="trace-group">
                                <h4 className="trace-header">
                                    <ShieldCheck size={12} />
                                    Satisfies Requirements
                                </h4>
                                <div className="trace-list">
                                    {linkedReqs.map((reqId: string) => (
                                        <div key={reqId} className="trace-item">
                                            {reqId}
                                        </div>
                                    ))}
                                </div>
                            </div>
                        )}

                        {linkedADRs.length > 0 && (
                            <div className="trace-group">
                                <h4 className="trace-header">
                                    <FileText size={12} />
                                    Governed By ADRs
                                </h4>
                                <div className="trace-list">
                                    {linkedADRs.map((adrId: string) => (
                                        <div key={adrId} className="trace-item">
                                            {adrId}
                                        </div>
                                    ))}
                                </div>
                            </div>
                        )}
                    </div>
                )}
            </div>
        </div>
    );
}

// Helper functions
function findNode(
    data: Parameters<Parameters<typeof useArchitectureStore>[0]>[0]['data'],
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

// Helper to find node by ID (for dependency analysis)
function findNodeById(
    data: Parameters<Parameters<typeof useArchitectureStore>[0]>[0]['data'],
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
