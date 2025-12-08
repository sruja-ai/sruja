import React from 'react';
import {
    ArrowRightLeft, ShieldCheck, FileText,
    ArrowUpRight, ArrowDownLeft, X,
    Layers, Box, Database, User, Server
} from 'lucide-react';
import { ArchitectureJSON } from '@sruja/viewer';
import { Badge } from '@sruja/ui';

interface InspectorPanelProps {
    nodeId: string;
    data: ArchitectureJSON;
    onClose: () => void;
    onSelectNode: (id: string) => void;
    onSelectRequirement: (id: string) => void;
    onSelectADR: (id: string) => void;
}

export function InspectorPanel({
    nodeId,
    data,
    onClose,
    onSelectNode,
    onSelectRequirement,
    onSelectADR
}: InspectorPanelProps) {
    const arch = data.architecture || {};

    // Helper to find an item by ID
    const findItem = (id: string) => {
        const allItems = [
            ...(arch.persons || []).map(p => ({ ...p, type: 'Person' })),
            ...(arch.systems || []).map(s => ({ ...s, type: 'System' })),
            ...(arch.systems?.flatMap(s => s.containers || []) || []).map(c => ({ ...c, type: 'Container' })),
            ...(arch.systems?.flatMap(s => s.containers?.flatMap(c => c.components || []) || []) || []).map(c => ({ ...c, type: 'Component' })),
            ...(arch.requirements || []).map(r => ({ ...r, type: 'Requirement' })),
            ...(arch.adrs || []).map(a => ({ ...a, type: 'ADR' }))
        ];
        return allItems.find(i => i.id === id || (i.id && id.endsWith(`.${i.id}`))) as any;
    };

    const item = findItem(nodeId) as any;

    if (!item) return null;

    // Calculate Dependencies
    const relations = arch.relations || [];

    const incoming = relations.filter(r => r.to === nodeId).map(r => ({
        relation: r as any,
        source: findItem(r.from) as any
    })).filter(x => x.source);

    const outgoing = relations.filter(r => r.from === nodeId).map(r => ({
        relation: r as any,
        target: findItem(r.to) as any
    })).filter(x => x.target);

    // Find Linked Requirements & ADRs
    const linkedReqs = (item as any).requirements || [];
    const linkedADRs = (item as any).adrs || [];

    const getTypeIcon = (type: string) => {
        switch (type?.toLowerCase()) {
            case 'person': return <User size={14} />;
            case 'system': return <Server size={14} />;
            case 'container': return <Box size={14} />;
            case 'component': return <Layers size={14} />;
            case 'datastore': return <Database size={14} />;
            default: return <Box size={14} />;
        }
    };

    return (
        <div className="inspector-panel animate-slide-in-right">
            <div className="inspector-header">
                <div className="inspector-title">
                    <h2>{item.label || item.id}</h2>
                    <div className="inspector-type">
                        {item.type || 'Component'}
                    </div>
                </div>
                <button onClick={onClose} className="close-btn"><X size={18} /></button>
            </div>

            <div className="inspector-content">
                {/* Identity Section */}
                <section className="inspector-section">
                    <h3>Identity</h3>
                    <div className="prop-grid">
                        <div className="prop-row">
                            <label>ID</label>
                            <span style={{ fontFamily: 'var(--font-mono)', fontSize: 12 }}>{item.id}</span>
                        </div>
                        {item.technology && (
                            <div className="prop-row">
                                <label>Tech Stack</label>
                                <div>
                                    <span className="tech-tag">{item.technology}</span>
                                </div>
                            </div>
                        )}
                        {item.description && (
                            <div className="prop-row full-width">
                                <label>Description</label>
                                <p>{item.description}</p>
                            </div>
                        )}
                        {item.tags && item.tags.length > 0 && (
                            <div className="prop-row full-width">
                                <label>Tags</label>
                                <div style={{ display: 'flex', gap: 4, flexWrap: 'wrap' }}>
                                    {item.tags.map((tag: string) => (
                                        <Badge key={tag} color="neutral" size="sm">{tag}</Badge>
                                    ))}
                                </div>
                            </div>
                        )}
                    </div>
                </section>

                {/* Dependency Analysis */}
                <section className="inspector-section">
                    <h3>Dependency Analysis</h3>

                    <div className="dependency-group">
                        <h4 className="dep-header"><ArrowDownLeft size={14} /> Incoming (Used By)</h4>
                        {incoming.length === 0 ? (
                            <div className="empty-text">No incoming dependencies</div>
                        ) : (
                            <div className="dep-list">
                                {incoming.map((inc, idx) => (
                                    <div key={idx} className="dep-item" onClick={() => onSelectNode(inc.relation.from)}>
                                        <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                                            {getTypeIcon(inc.source?.type)}
                                            <span className="dep-name">{inc.source?.label || inc.relation.from}</span>
                                        </div>
                                        {inc.relation.description && <span className="dep-desc">{inc.relation.description}</span>}
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>

                    <div className="dependency-group">
                        <h4 className="dep-header"><ArrowUpRight size={14} /> Outgoing (Uses)</h4>
                        {outgoing.length === 0 ? (
                            <div className="empty-text">No outgoing dependencies</div>
                        ) : (
                            <div className="dep-list">
                                {outgoing.map((out, idx) => (
                                    <div key={idx} className="dep-item" onClick={() => onSelectNode(out.relation.to)}>
                                        <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                                            {getTypeIcon(out.target?.type)}
                                            <span className="dep-name">{out.target?.label || out.relation.to}</span>
                                        </div>
                                        {out.relation.description && <span className="dep-desc">{out.relation.description}</span>}
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                </section>

                {/* Traceability */}
                <section className="inspector-section">
                    <h3>Traceability</h3>

                    <div className="trace-group">
                        <h4 className="trace-header"><ShieldCheck size={14} /> Satisfies Requirements</h4>
                        {linkedReqs.length === 0 ? (
                            <div className="empty-text">No explicit requirements linked</div>
                        ) : (
                            <div className="trace-list">
                                {linkedReqs.map((reqId: string) => (
                                    <div key={reqId} className="trace-item" onClick={() => onSelectRequirement(reqId)}>
                                        {reqId}
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>

                    <div className="trace-group">
                        <h4 className="trace-header"><FileText size={14} /> Governed By ADRs</h4>
                        {linkedADRs.length === 0 ? (
                            <div className="empty-text">No ADRs linked</div>
                        ) : (
                            <div className="trace-list">
                                {linkedADRs.map((adrId: string) => (
                                    <div key={adrId} className="trace-item" onClick={() => onSelectADR(adrId)}>
                                        {adrId}
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                </section>
            </div>
        </div>
    );
}
