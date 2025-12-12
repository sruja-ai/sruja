import { useMemo, useState, useEffect } from 'react';
import { CheckCircle, FileText, Lock, Gauge, Shield, AlertCircle, Edit, Trash2 } from 'lucide-react';
import { useArchitectureStore, useUIStore, useSelectionStore } from '../../stores';
import { useFeatureFlagsStore } from '../../stores/featureFlagsStore';
import { EditRequirementForm, ConfirmDialog } from '../shared';
import { Input } from '@sruja/ui';
import type { RequirementJSON } from '../../types';
import './RequirementsPanel.css';

type RequirementType = 'functional' | 'performance' | 'security' | 'constraint' | 'reliability' | 'all';

const REQUIREMENT_TYPES: { type: RequirementType; label: string; icon: React.ReactNode; color: string }[] = [
    { type: 'all', label: 'All', icon: <FileText size={14} />, color: '#667eea' },
    { type: 'functional', label: 'Functional', icon: <CheckCircle size={14} />, color: '#22c55e' },
    { type: 'performance', label: 'Performance', icon: <Gauge size={14} />, color: '#f59e0b' },
    { type: 'security', label: 'Security', icon: <Shield size={14} />, color: '#ef4444' },
    { type: 'constraint', label: 'Constraint', icon: <Lock size={14} />, color: '#8b5cf6' },
    { type: 'reliability', label: 'Reliability', icon: <AlertCircle size={14} />, color: '#06b6d4' },
];

export function RequirementsPanel() {
    const data = useArchitectureStore((s) => s.data);
    const convertedJson = useArchitectureStore((s) => s.convertedJson);
    const isEditMode = useFeatureFlagsStore((s) => s.isEditMode);
    const [activeType, setActiveType] = useState<RequirementType>('all');
    const [searchQuery, setSearchQuery] = useState('');
    const [editRequirement, setEditRequirement] = useState<RequirementJSON | undefined>(undefined);
    const [showEditForm, setShowEditForm] = useState(false);
    const [deleteRequirement, setDeleteRequirement] = useState<RequirementJSON | undefined>(undefined);
    const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
    const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
    const pendingAction = useUIStore((s) => s.pendingAction);
    const clearPendingAction = useUIStore((s) => s.clearPendingAction);

    // Initial load action handler
    useEffect(() => {
        if (pendingAction === 'create-requirement') {
            setEditRequirement(undefined);
            setShowEditForm(true);
            clearPendingAction();
        }
    }, [pendingAction, clearPendingAction]);

    const selectedNodeId = useSelectionStore((s) => s.selectedNodeId);

    // Root-level requirements only - use converted JSON if available, otherwise fallback to data
    const requirements = useMemo(() => {
        const sourceData = convertedJson || data;

        if (!sourceData) {
            return [];
        }

        if (!sourceData.architecture) {
            return [];
        }

        const reqs: any = sourceData.architecture.requirements;

        if (Array.isArray(reqs) && reqs.length > 0) {
            return reqs;
        }

        if (reqs && typeof reqs === 'object' && !Array.isArray(reqs)) {
            return Object.values(reqs);
        }

        return [];
    }, [convertedJson, data]);

    const filteredRequirements = useMemo(() => {
        let filtered = requirements;

        // Filter by selected node
        if (selectedNodeId) {
            filtered = filtered.filter(r => r.tags?.includes(selectedNodeId));
        }

        // Filter by type
        if (activeType !== 'all') {
            filtered = filtered.filter(r => r.type?.toLowerCase() === activeType);
        }

        // Filter by search query
        if (searchQuery.trim()) {
            const query = searchQuery.toLowerCase();
            filtered = filtered.filter(r =>
                r.id?.toLowerCase().includes(query) ||
                r.title?.toLowerCase().includes(query) ||
                r.description?.toLowerCase().includes(query) ||
                r.type?.toLowerCase().includes(query)
            );
        }

        return filtered;
    }, [requirements, activeType, searchQuery, selectedNodeId]);

    const countByType = useMemo(() => {
        // Count based on CURRENTLY filtered list (by selection), but ignoring type filter
        // Actually, usually counts show total available.
        // If selection is active, counts should arguably reflect the selection subset.
        // Let's filter by selection first for counts.
        let baseList = requirements;
        if (selectedNodeId) {
            baseList = baseList.filter(r => r.tags?.includes(selectedNodeId));
        }

        const counts: Record<string, number> = { all: baseList.length };
        baseList.forEach(r => {
            const type = r.type?.toLowerCase() || 'other';
            counts[type] = (counts[type] || 0) + 1;
        });
        return counts;
    }, [requirements, selectedNodeId]);

    return (
        <div className="requirements-panel">
            <h3 className="requirements-title">
                <FileText size={18} />
                Requirements
                {requirements.length > 0 && (
                    <span className="requirements-count">{requirements.length}</span>
                )}
                {isEditMode() && (
                    <button
                        className="requirements-add-btn"
                        onClick={() => {
                            setEditRequirement(undefined);
                            setShowEditForm(true);
                        }}
                        title="Add Requirement"
                    >
                        <FileText size={14} />
                    </button>
                )}
            </h3>

            {!requirements.length ? (
                <div className="requirements-empty">
                    <FileText size={48} className="empty-icon" />
                    <p>No requirements defined</p>
                    <p className="empty-hint">Add requirements to your architecture DSL to see them here</p>
                </div>
            ) : (
                <>
                    {/* Selection Filter Info */}
                    {selectedNodeId && (
                        <div className="filter-info-banner" style={{
                            padding: '8px 12px',
                            background: 'var(--bg-secondary)',
                            borderBottom: '1px solid var(--border-color)',
                            fontSize: '0.85rem',
                            display: 'flex',
                            alignItems: 'center',
                            gap: '6px',
                            color: 'var(--text-secondary)'
                        }}>
                            <div style={{ width: 8, height: 8, borderRadius: '50%', background: 'var(--primary-color)' }}></div>
                            Filtered by selection: <strong>{selectedNodeId}</strong>
                        </div>
                    )}

                    {/* Search */}
                    <div className="requirements-search">
                        <Input
                            placeholder="Search requirements..."
                            value={searchQuery}
                            onChange={(e) => setSearchQuery(e.target.value)}
                        />
                    </div>

                    {/* Type Tabs */}
                    <div className="requirements-tabs">
                        {REQUIREMENT_TYPES.map(({ type, label, icon, color }) => (
                            <button
                                key={type}
                                className={`req-tab ${activeType === type ? 'active' : ''}`}
                                style={{ '--tab-color': color } as React.CSSProperties}
                                onClick={() => setActiveType(type)}
                            >
                                {icon}
                                <span>{label}</span>
                                {countByType[type] !== undefined && (
                                    <span className="tab-count">{countByType[type] || 0}</span>
                                )}
                            </button>
                        ))}
                    </div>

                    {/* Requirements List */}
                    <div className="requirements-list">
                        {filteredRequirements.map((req, index) => {
                            const typeConfig = REQUIREMENT_TYPES.find(t => t.type === req.type?.toLowerCase());
                            // Use index as fallback for key if id is missing or duplicate
                            const uniqueKey = req.id ? `${req.id}-${index}` : `req-${index}`;
                            return (
                                <div
                                    key={uniqueKey}
                                    className="requirement-card"
                                    style={{ '--req-color': typeConfig?.color || '#667eea' } as React.CSSProperties}
                                >
                                    <div className="req-header">
                                        <span className="req-id">{req.id || `REQ-${index + 1}`}</span>
                                        {req.type && (
                                            <span className="req-type-badge">
                                                {typeConfig?.icon}
                                                {req.type}
                                            </span>
                                        )}
                                        {isEditMode() && (
                                            <div className="req-actions">
                                                <button
                                                    className="req-edit-btn"
                                                    onClick={(e) => {
                                                        e.stopPropagation();
                                                        setEditRequirement(req);
                                                        setShowEditForm(true);
                                                    }}
                                                    title="Edit Requirement"
                                                >
                                                    <Edit size={14} />
                                                </button>
                                                <button
                                                    className="req-delete-btn"
                                                    onClick={(e) => {
                                                        e.stopPropagation();
                                                        setDeleteRequirement(req);
                                                        setShowDeleteConfirm(true);
                                                    }}
                                                    title="Delete Requirement"
                                                >
                                                    <Trash2 size={14} />
                                                </button>
                                            </div>
                                        )}
                                    </div>
                                    <p className="req-description">
                                        {req.title || req.description || 'No description'}
                                    </p>
                                </div>
                            );
                        })}
                    </div>
                </>
            )}

            <EditRequirementForm
                isOpen={showEditForm}
                onClose={() => {
                    setShowEditForm(false);
                    setEditRequirement(undefined);
                }}
                requirement={editRequirement}
            />
            <ConfirmDialog
                isOpen={showDeleteConfirm}
                onClose={() => {
                    setShowDeleteConfirm(false);
                    setDeleteRequirement(undefined);
                }}
                onConfirm={async () => {
                    if (deleteRequirement) {
                        await updateArchitecture((arch) => {
                            if (!arch.architecture) return arch;
                            const requirements = (arch.architecture.requirements || []).filter(
                                (r) => r.id !== deleteRequirement.id
                            );
                            return {
                                ...arch,
                                architecture: {
                                    ...arch.architecture,
                                    requirements,
                                },
                            };
                        });
                    }
                }}
                title="Delete Requirement"
                message={`Are you sure you want to delete requirement "${deleteRequirement?.id}"? This action cannot be undone.`}
                confirmLabel="Delete"
                variant="danger"
            />
        </div>
    );
}
