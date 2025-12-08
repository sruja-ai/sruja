import React, { useState, useEffect, useMemo } from 'react';
import { Menu, Layout, Layers, Box, Component, Download, FileText, Edit } from 'lucide-react';
import { ThemeToggle, Logo, Breadcrumb, SearchBar, type SearchItem, Button } from '@sruja/ui';
import { ArchitectureJSON } from '@sruja/viewer';

interface TopBarProps {
    data: ArchitectureJSON;
    currentLevel: number;
    onSetLevel: (level: number) => void;
    onSearch: (query: string) => void;
    onSelectNode: (id: string) => void;
    breadcrumbs: { id: string; label: string }[];
    onBreadcrumbClick: (id: string) => void;
    onLayoutChange?: (name: string) => void;
    dragEnabled?: boolean;
    onToggleDrag?: () => void;
    onToggleSidebar?: () => void;
    onExport?: () => void;
    onEditInStudio?: () => void;
    onPreviewMarkdown?: () => void;
}

export function TopBar({
    data,
    currentLevel,
    onSetLevel,
    onSelectNode,
    breadcrumbs,
    onBreadcrumbClick,
    onLayoutChange,
    dragEnabled,
    onToggleDrag,
    onToggleSidebar,
    onExport,
    onEditInStudio,
    onPreviewMarkdown
}: TopBarProps) {
    const [searchQuery, setSearchQuery] = useState('');
    const [layout, setLayout] = useState('dagre');

    // Index all items for search
    const allItems = useMemo(() => {
        const arch = data.architecture || {};
        const items: SearchItem[] = [
            ...(arch.persons || []).map(p => ({ id: p.id, label: p.label || p.id, subLabel: 'Person' })),
            ...(arch.systems || []).map(s => ({ id: s.id, label: s.label || s.id, subLabel: 'System' })),
            ...(arch.requirements || []).map(r => ({ id: r.id, label: r.title || r.id, subLabel: 'Requirement' })),
            ...(arch.adrs || []).map(a => ({ id: a.id, label: a.title || a.id, subLabel: 'ADR' }))
        ];

        // Add containers with system context
        arch.systems?.forEach(system => {
            system.containers?.forEach(container => {
                items.push({
                    id: `${system.id}.${container.id}`,
                    label: container.label || container.id,
                    subLabel: `${system.label || system.id} > Container`
                });
            });
        });

        return items;
    }, [data]);

    // Filter search results
    const searchResults = useMemo(() => {
        if (!searchQuery.trim()) return [];
        const query = searchQuery.toLowerCase();
        return allItems.filter(item =>
            item.label.toLowerCase().includes(query) ||
            (item.subLabel && item.subLabel.toLowerCase().includes(query))
        ).slice(0, 10);
    }, [searchQuery, allItems]);

    const handleSearchSelect = (item: SearchItem | null) => {
        if (item) {
            onSelectNode(item.id);
            setSearchQuery('');
        }
    };

    return (
        <div className="top-bar">
            {/* Brand */}
            <div className="brand">
                {(() => {
                    const globalLogo = (window as any)?.SRUJA_BRAND_LOGO_URL;
                    const metaLogo = (data as any)?.metadata?.brandLogo;
                    const src = metaLogo || globalLogo;
                    if (src && typeof src === 'string') {
                        return <img className="brand-img" src={src} alt="Brand" />;
                    }
                    return (
                        <div className="brand-logo">
                            <Logo size={18} color="white" />
                        </div>
                    );
                })()}
                <div className="brand-name">{data?.metadata?.name || 'Sruja Architecture'}</div>
            </div>

            {/* Breadcrumbs */}
            <div className="breadcrumbs">
                <button
                    className="breadcrumb-item mobile-only"
                    onClick={() => onToggleSidebar && onToggleSidebar()}
                    style={{ display: 'none' }} // Handled by CSS media query
                >
                    <Menu size={16} />
                </button>
                <Breadcrumb
                    items={breadcrumbs}
                    onItemClick={onBreadcrumbClick}
                    onHomeClick={() => onBreadcrumbClick('root')}
                    className="flex-1"
                />
            </div>

            <div style={{ flex: 1 }} />

            {/* Center Controls (Level) */}
            <div className="level-controls">
                <button
                    className={`level-btn ${currentLevel === 1 ? 'active' : ''}`}
                    onClick={() => onSetLevel(1)}
                    title="Context Level"
                >
                    Context
                </button>
                <button
                    className={`level-btn ${currentLevel === 2 ? 'active' : ''}`}
                    onClick={() => onSetLevel(2)}
                    title="Container Level"
                >
                    Container
                </button>
                <button
                    className={`level-btn ${currentLevel === 3 ? 'active' : ''}`}
                    onClick={() => onSetLevel(3)}
                    title="Component Level"
                >
                    Component
                </button>
                <div style={{ width: 1, height: 16, backgroundColor: 'var(--border-color)', margin: '0 4px', alignSelf: 'center' }} />
                <select
                    value={layout}
                    onChange={(e) => {
                        const name = e.target.value;
                        setLayout(name);
                        onLayoutChange && onLayoutChange(name);
                    }}
                    style={{
                        padding: '4px 8px',
                        fontSize: 12,
                        borderRadius: 6,
                        border: 'none',
                        backgroundColor: 'transparent',
                        color: 'var(--text-secondary)',
                        cursor: 'pointer',
                        fontWeight: 500,
                        outline: 'none'
                    }}
                >
                    <option value="dagre">Dagre</option>
                    <option value="cose">CoSE</option>
                    <option value="fcose">fCoSE</option>
                    <option value="elk">ELK</option>
                </select>
            </div>

            <div style={{ flex: 1 }} />

            {/* Search */}
            <div className="search-container">
                <SearchBar
                    query={searchQuery}
                    onQueryChange={setSearchQuery}
                    results={searchResults}
                    onSelect={handleSearchSelect}
                    placeholder="Search architecture..."
                />
            </div>

            <div style={{ display: 'flex', gap: 12, alignItems: 'center', marginLeft: 16 }}>
                {onPreviewMarkdown && (
                    <Button
                        variant="ghost"
                        size="sm"
                        onClick={onPreviewMarkdown}
                        style={{ fontSize: 12, height: 28 }}
                    >
                        <FileText size={14} style={{ marginRight: 4 }} />
                        Preview Markdown
                    </Button>
                )}
                {onEditInStudio && (
                    <Button
                        variant="secondary"
                        size="sm"
                        onClick={onEditInStudio}
                        style={{ fontSize: 12, height: 28 }}
                    >
                        <Edit size={14} style={{ marginRight: 4 }} />
                        Edit in Studio
                    </Button>
                )}
                {onExport && (
                    <Button
                        variant="primary"
                        size="sm"
                        onClick={onExport}
                        style={{ fontSize: 12, height: 28 }}
                    >
                        <Download size={14} style={{ marginRight: 4 }} />
                        Export
                    </Button>
                )}
                <ThemeToggle iconOnly size="sm" />
                <Button
                    variant={dragEnabled ? 'primary' : 'ghost'}
                    size="sm"
                    onClick={() => onToggleDrag && onToggleDrag()}
                    style={{ fontSize: 12, height: 28 }}
                >
                    {dragEnabled ? 'Drag: On' : 'Drag: Off'}
                </Button>
            </div>
        </div>
    );
}
