import React, { useEffect, useState, useRef, useCallback, useMemo } from 'react';
import { ArchitectureJSON } from '@sruja/viewer';
import { findNode, updateNode } from '../utils/archUtils';
import { Settings, Type, Palette, Layout, Box, Circle, Database, Server, User, X } from 'lucide-react';
import { generateDSLFragment } from '../utils/dslGenerator';

interface PropertiesPanelProps {
    selectedNodeId: string | null;
    archData: ArchitectureJSON | null;
    onUpdate: (newData: ArchitectureJSON) => void;
    onClose?: () => void;
}


export const PropertiesPanel: React.FC<PropertiesPanelProps> = React.memo(({ selectedNodeId, archData, onUpdate, onClose }) => {
    const [node, setNode] = useState<any>(null);
    const debounceTimerRef = useRef<NodeJS.Timeout | null>(null);
    const pendingUpdateRef = useRef<{ field: string; value: string } | null>(null);
    const archDataRef = useRef<ArchitectureJSON | null>(null);
    const selectedNodeIdRef = useRef<string | null>(null);

    // Keep refs in sync
    useEffect(() => {
        archDataRef.current = archData;
        selectedNodeIdRef.current = selectedNodeId;
    }, [archData, selectedNodeId]);

    useEffect(() => {
        if (selectedNodeId && archData) {
            const found = findNode(archData, selectedNodeId);
            setNode(found);
        } else {
            setNode(null);
        }
    }, [selectedNodeId, archData]);

    // Cleanup debounce timer on unmount
    useEffect(() => {
        return () => {
            if (debounceTimerRef.current) {
                clearTimeout(debounceTimerRef.current);
            }
        };
    }, []);

    // Memoize metadata values lookup
    const metadataValues = useMemo(() => {
        if (!node?.metadata) return {};
        const values: Record<string, string> = {};
        node.metadata.forEach((m: any) => {
            values[m.key] = m.value || '';
        });
        return values;
    }, [node?.metadata]);

    const getMetadataValue = useCallback((key: string): string => {
        return metadataValues[key] || '';
    }, [metadataValues]);

    // Debounced update function for text fields (label, description, technology)
    const handleChange = useCallback((field: string, value: string) => {
        if (!archDataRef.current || !selectedNodeIdRef.current) return;

        // Update local state immediately for responsive UI
        setNode((prevNode: any) => {
            if (!prevNode) return prevNode;
            return { ...prevNode, [field]: value };
        });

        // Store pending update
        pendingUpdateRef.current = { field, value };

        // Clear existing timer
        if (debounceTimerRef.current) {
            clearTimeout(debounceTimerRef.current);
        }

        // Debounce the actual update (500ms delay)
        debounceTimerRef.current = setTimeout(() => {
            if (pendingUpdateRef.current && archDataRef.current && selectedNodeIdRef.current) {
                const newData = updateNode(archDataRef.current, selectedNodeIdRef.current, (n) => ({
                    ...n,
                    [pendingUpdateRef.current!.field]: pendingUpdateRef.current!.value
                }));
                onUpdate(newData);
                pendingUpdateRef.current = null;
            }
        }, 500);
    }, [onUpdate]);

    const handleMetadataChange = (key: string, value: string) => {
        if (!archData || !selectedNodeId) return;

        const newData = updateNode(archData, selectedNodeId, (n) => {
            const metadata = [...(n.metadata || [])];
            const index = metadata.findIndex((m: any) => m.key === key);

            if (index >= 0) {
                if (value) {
                    metadata[index] = { ...metadata[index], value };
                } else {
                    metadata.splice(index, 1);
                }
            } else if (value) {
                metadata.push({ key, value });
            }

            return { ...n, metadata };
        });
        onUpdate(newData);
    };


    const renderProperties = () => {
        if (!selectedNodeId || !node) {
            return (
                <div className="h-full flex flex-col items-center justify-center text-[var(--color-text-tertiary)] p-8 text-center">
                    <Settings className="w-12 h-12 mb-3 opacity-30" />
                    <p className="text-sm font-medium mb-1">No selection</p>
                    <p className="text-xs opacity-70">Select an element in the diagram to view and edit its properties</p>
                </div>
            );
        }

        return (
            <div className="p-4 space-y-6">
                {/* General */}
                <div className="space-y-4">
                    <h3 className="text-xs font-bold text-[var(--color-text-tertiary)] uppercase tracking-wider flex items-center gap-2">
                        <Type className="w-3 h-3" />
                        General
                    </h3>

                    <div className="space-y-1">
                        <label className="text-xs font-medium text-[var(--color-text-secondary)]">Label</label>
                        <input
                            type="text"
                            value={node.label || ''}
                            onChange={(e) => handleChange('label', e.target.value)}
                            className="w-full px-3 py-2 bg-[var(--color-background)] border border-[var(--color-border)] rounded-md text-sm text-[var(--color-text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] focus:border-transparent"
                        />
                    </div>

                    <div className="space-y-1">
                        <label className="text-xs font-medium text-[var(--color-text-secondary)]">Description</label>
                        <textarea
                            value={node.description || ''}
                            onChange={(e) => handleChange('description', e.target.value)}
                            rows={3}
                            className="w-full px-3 py-2 bg-[var(--color-background)] border border-[var(--color-border)] rounded-md text-sm text-[var(--color-text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] focus:border-transparent resize-none"
                        />
                    </div>
                </div>

                {/* Style */}
                <div className="space-y-4">
                    <h3 className="text-xs font-bold text-[var(--color-text-tertiary)] uppercase tracking-wider flex items-center gap-2">
                        <Palette className="w-3 h-3" />
                        Style
                    </h3>

                    <div className="space-y-1">
                        <label className="text-xs font-medium text-[var(--color-text-secondary)]">Color</label>
                        <div className="flex gap-2 items-center">
                            <input
                                type="color"
                                value={getMetadataValue('style.color') || '#ffffff'}
                                onChange={(e) => handleMetadataChange('style.color', e.target.value)}
                                className="w-8 h-8 p-0 border-0 rounded cursor-pointer"
                            />
                            <span className="text-xs text-[var(--color-text-secondary)] font-mono">
                                {getMetadataValue('style.color') || 'Default'}
                            </span>
                        </div>
                    </div>

                    <div className="space-y-1">
                        <label className="text-xs font-medium text-[var(--color-text-secondary)]">Shape</label>
                        <div className="grid grid-cols-4 gap-2">
                            {[
                                { id: 'rectangle', icon: Box, label: 'Box' },
                                { id: 'round-rectangle', icon: Layout, label: 'Round' },
                                { id: 'ellipse', icon: Circle, label: 'Oval' },
                                { id: 'barrel', icon: Database, label: 'DB' },
                                { id: 'cut-rectangle', icon: Server, label: 'Node' },
                                { id: 'bottom-round-rectangle', icon: User, label: 'User' },
                            ].map((shape) => (
                                <button
                                    key={shape.id}
                                    onClick={() => handleMetadataChange('style.shape', shape.id)}
                                    className={`p-2 rounded border flex flex-col items-center justify-center gap-1 hover:bg-[var(--color-surface)] transition-colors ${getMetadataValue('style.shape') === shape.id
                                        ? 'bg-[var(--color-primary-50)] border-[var(--color-primary)] text-[var(--color-primary)]'
                                        : 'bg-[var(--color-background)] border-[var(--color-border)] text-[var(--color-text-secondary)]'
                                        }`}
                                    title={shape.label}
                                >
                                    <shape.icon className="w-4 h-4" />
                                </button>
                            ))}
                        </div>
                    </div>

                    <div className="space-y-1">
                        <label className="text-xs font-medium text-[var(--color-text-secondary)]">Icon</label>
                        <select
                            value={getMetadataValue('style.icon') || ''}
                            onChange={(e) => handleMetadataChange('style.icon', e.target.value)}
                            className="w-full px-3 py-2 bg-[var(--color-background)] border border-[var(--color-border)] rounded-md text-sm text-[var(--color-text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] focus:border-transparent"
                        >
                            <option value="">None</option>
                            <option value="person">Person</option>
                            <option value="system">System</option>
                            <option value="container">Container</option>
                            <option value="database">Database</option>
                            <option value="cloud">Cloud</option>
                            <option value="server">Server</option>
                            <option value="mobile">Mobile</option>
                            <option value="browser">Browser</option>
                        </select>
                    </div>
                </div>

                {/* Metadata */}
                <div className="space-y-4">
                    <h3 className="text-xs font-bold text-[var(--color-text-tertiary)] uppercase tracking-wider flex items-center gap-2">
                        <Box className="w-3 h-3" />
                        Metadata
                    </h3>

                    {/* Technology (only for containers) */}
                    {(node.type === 'container' || node.type === 'component') && (
                        <div className="space-y-1">
                            <label className="text-xs font-medium text-[var(--color-text-secondary)]">Technology</label>
                            <input
                                type="text"
                                value={node.technology || ''}
                                onChange={(e) => handleChange('technology', e.target.value)}
                                placeholder="e.g. Go, React, PostgreSQL"
                                className="w-full px-3 py-2 bg-[var(--color-background)] border border-[var(--color-border)] rounded-md text-sm text-[var(--color-text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] focus:border-transparent"
                            />
                        </div>
                    )}

                    <div className="grid grid-cols-2 gap-4">
                        <div className="space-y-1">
                            <label className="text-xs font-medium text-[var(--color-text-secondary)]">Status</label>
                            <select
                                value={getMetadataValue('status') || ''}
                                onChange={(e) => handleMetadataChange('status', e.target.value)}
                                className="w-full px-3 py-2 bg-[var(--color-background)] border border-[var(--color-border)] rounded-md text-sm text-[var(--color-text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] focus:border-transparent"
                            >
                                <option value="">None</option>
                                <option value="draft">Draft</option>
                                <option value="review">Review</option>
                                <option value="approved">Approved</option>
                                <option value="deprecated">Deprecated</option>
                            </select>
                        </div>
                        <div className="space-y-1">
                            <label className="text-xs font-medium text-[var(--color-text-secondary)]">Owner</label>
                            <input
                                type="text"
                                value={getMetadataValue('owner') || ''}
                                onChange={(e) => handleMetadataChange('owner', e.target.value)}
                                placeholder="Team or Person"
                                className="w-full px-3 py-2 bg-[var(--color-background)] border border-[var(--color-border)] rounded-md text-sm text-[var(--color-text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] focus:border-transparent"
                            />
                        </div>
                    </div>
                </div>

                {/* DSL Preview */}
                <div className="space-y-2 pt-4 border-t border-[var(--color-border)]">
                    <h3 className="text-xs font-bold text-[var(--color-text-tertiary)] uppercase tracking-wider">
                        DSL Preview
                    </h3>
                    <div className="bg-gray-950 rounded-md p-3 overflow-x-auto">
                        <pre className="text-xs text-blue-300 font-mono leading-relaxed whitespace-pre">
                            {generateDSLFragment(node)}
                        </pre>
                    </div>
                </div>
            </div>
        );
    };

    return (
        <div className="h-full flex flex-col bg-[var(--color-surface)] border-l border-[var(--color-border)] overflow-y-auto">
            {/* Header with Close Button */}
            {onClose && (
                <div className="flex items-center justify-between px-4 py-2 border-b border-[var(--color-border)] bg-[var(--color-background)] flex-shrink-0">
                    <h3 className="text-sm font-medium text-[var(--color-text-primary)]">Properties</h3>
                    <button
                        onClick={onClose}
                        className="p-1 rounded text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] hover:bg-[var(--color-surface)] transition-colors"
                        title="Close Properties"
                    >
                        <X size={16} />
                    </button>
                </div>
            )}
            {/* Properties Content */}
            <div className="flex-1 overflow-y-auto">
                {renderProperties()}
            </div>
        </div>
    );
});
