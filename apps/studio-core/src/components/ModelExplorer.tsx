import React, { useMemo, useCallback } from 'react';
import { ChevronRight, ChevronDown, Box, Server, Database, User, Layers, FileText, ShieldCheck, Cloud, Search, X, AlertCircle } from 'lucide-react';
import { ArchitectureJSON } from '@sruja/viewer';
import { cn } from '@sruja/ui';

interface ModelExplorerProps {
    data: ArchitectureJSON;
    onSelect: (id: string) => void;
}

// Helper to check if a node has missing required fields
const checkNodeCompleteness = (node: any): boolean => {
    // Check if description is missing (common requirement)
    if (!node.description || node.description.trim() === '') {
        return true; // Has warning
    }
    // For containers/components, check technology
    if ((node.type === 'container' || node.type === 'component') && (!node.technology || node.technology.trim() === '')) {
        return true; // Has warning
    }
    return false; // No warning
};

export const ModelExplorer: React.FC<ModelExplorerProps> = React.memo(({ data, onSelect }) => {
    const [expanded, setExpanded] = React.useState<Set<string>>(new Set());
    const [searchQuery, setSearchQuery] = React.useState<string>('');

    const toggleExpand = useCallback((id: string, e: React.MouseEvent) => {
        e.stopPropagation();
        setExpanded(prev => {
            const newExpanded = new Set(prev);
            if (newExpanded.has(id)) {
                newExpanded.delete(id);
            } else {
                newExpanded.add(id);
            }
            return newExpanded;
        });
    }, []);

    const handleDragStart = useCallback((e: React.DragEvent, id: string, type: string) => {
        e.stopPropagation();
        e.dataTransfer.setData('application/json', JSON.stringify({
            type: 'existing-node',
            id,
            nodeType: type
        }));
        e.dataTransfer.effectAllowed = 'copy';
    }, []);

    const renderItem = (id: string, label: string, type: string, children?: React.ReactNode, node?: any) => {
        const isExpanded = expanded.has(id);
        const hasChildren = React.Children.count(children) > 0;
        const hasWarning = node ? checkNodeCompleteness(node) : false;

        let Icon = Box;
        if (type === 'system') Icon = Server;
        else if (type === 'person') Icon = User;
        else if (type === 'container') Icon = Box;
        else if (type === 'datastore') Icon = Database;
        else if (type === 'queue') Icon = Layers;
        else if (type === 'adr') Icon = FileText;
        else if (type === 'requirement') Icon = ShieldCheck;
        else if (type === 'deployment') Icon = Cloud;

        return (
            <div key={id} className="ml-0">
                <div
                    draggable
                    onDragStart={(e) => handleDragStart(e, id, type)}
                    className={cn(
                        "flex items-center py-0.5 pl-5 pr-2 cursor-pointer text-[0.8125rem] text-[var(--color-text-primary)] select-none transition-colors group",
                        "hover:bg-[var(--color-surface)]",
                        hasWarning && "bg-yellow-50/5"
                    )}
                    onClick={() => onSelect(id)}
                >
                    <div
                        className={cn(
                            "w-4 flex items-center justify-center -ml-4 pl-1",
                            hasChildren ? "cursor-pointer" : "cursor-default"
                        )}
                        onClick={(e) => hasChildren && toggleExpand(id, e)}
                    >
                        {hasChildren && (
                            isExpanded ? (
                                <ChevronDown size={12} className="text-[var(--color-text-secondary)]" />
                            ) : (
                                <ChevronRight size={12} className="text-[var(--color-text-secondary)]" />
                            )
                        )}
                    </div>
                    <Icon size={16} className="mr-1.5 text-[var(--color-text-secondary)] flex-shrink-0" />
                    <span className="whitespace-nowrap overflow-hidden text-ellipsis flex-1">
                        {label || id}
                    </span>
                    {hasWarning && (
                        <AlertCircle 
                            size={12} 
                            className="text-yellow-500 flex-shrink-0 ml-1" 
                            title="Missing description or required fields"
                        />
                    )}
                </div>
                {isExpanded && children && (
                    <div className="ml-0">
                        {children}
                    </div>
                )}
            </div>
        );
    };

    // Filter function to check if item matches search query (memoized)
    const matchesSearch = useCallback((id: string, label: string): boolean => {
        if (!searchQuery.trim()) return true;
        const query = searchQuery.toLowerCase();
        return id.toLowerCase().includes(query) || label.toLowerCase().includes(query);
    }, [searchQuery]);

    // Recursive function to filter and render items
    const renderFilteredItem = (id: string, label: string, type: string, children?: React.ReactNode): React.ReactNode | null => {
        const itemMatches = matchesSearch(id, label);
        const hasMatchingChildren = children && React.Children.toArray(children).some((child: any) => {
            if (child?.props?.id) {
                return matchesSearch(child.props.id, child.props.label || child.props.id);
            }
            return false;
        });

        // Show item if it matches or has matching children
        if (!itemMatches && !hasMatchingChildren) return null;

        return renderItem(id, label, type, children);
    };

    const arch = data.architecture;

    // Memoize filtered persons
    const filteredPersons = useMemo(() => {
        return arch?.persons?.filter(p => matchesSearch(p.id, p.label || p.id)) || [];
    }, [arch?.persons, matchesSearch]);

    // Memoize filtered systems with nested elements
    const filteredSystems = useMemo(() => {
        if (!arch?.systems) return [];
        return arch.systems.map(s => {
            const systemMatches = matchesSearch(s.id, s.label || s.id);
            const containers = s.containers?.filter(c => matchesSearch(s.id + '.' + c.id, c.label || c.id));
            const datastores = s.datastores?.filter(d => matchesSearch(s.id + '.' + d.id, d.label || d.id));
            const queues = s.queues?.filter(q => matchesSearch(s.id + '.' + q.id, q.label || q.id));
            const hasMatchingChildren = (containers && containers.length > 0) || (datastores && datastores.length > 0) || (queues && queues.length > 0);
            
            if (!systemMatches && !hasMatchingChildren) return null;
            
            return { system: s, containers, datastores, queues, systemMatches };
        }).filter(Boolean);
    }, [arch?.systems, matchesSearch]);

    // Memoize filtered requirements, ADRs, and deployments
    const filteredRequirements = useMemo(() => {
        return arch?.requirements?.filter(r => matchesSearch(r.id, r.title || r.id)) || [];
    }, [arch?.requirements, matchesSearch]);

    const filteredAdrs = useMemo(() => {
        return arch?.adrs?.filter(a => matchesSearch(a.id, a.title || a.id)) || [];
    }, [arch?.adrs, matchesSearch]);

    const filteredDeployments = useMemo(() => {
        return arch?.deployment?.filter(d => matchesSearch(d.id, d.label || d.id)) || [];
    }, [arch?.deployment, matchesSearch]);

    return (
        <div className="flex flex-col h-full">
            {/* Search Bar */}
            <div className="px-2 py-2 border-b border-[var(--color-border)] flex-shrink-0">
                <div className="relative">
                    <Search size={14} className="absolute left-2 top-1/2 -translate-y-1/2 text-[var(--color-text-tertiary)]" />
                    <input
                        type="text"
                        placeholder="Search nodes..."
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        className="w-full pl-8 pr-8 py-1.5 text-xs bg-[var(--color-background)] border border-[var(--color-border)] rounded text-[var(--color-text-primary)] placeholder:text-[var(--color-text-tertiary)] focus:outline-none focus:ring-1 focus:ring-[var(--color-info-500)]"
                    />
                    {searchQuery && (
                        <button
                            onClick={() => setSearchQuery('')}
                            className="absolute right-2 top-1/2 -translate-y-1/2 p-0.5 text-[var(--color-text-tertiary)] hover:text-[var(--color-text-primary)]"
                        >
                            <X size={12} />
                        </button>
                    )}
                </div>
            </div>

            {/* Explorer Content */}
            <div className="py-2 overflow-y-auto flex-1 text-[0.8125rem] font-sans min-h-0">
                {filteredPersons.map(p => renderItem(p.id, p.label || p.id, 'person', undefined, p))}

                {filteredSystems.map(({ system: s, containers, datastores, queues, systemMatches }) => {
                    return renderItem(s.id, s.label || s.id, 'system', (
                        <>
                            {containers?.map(c => {
                                const comps = c.components?.filter(comp => matchesSearch(s.id + '.' + c.id + '.' + comp.id, comp.label || comp.id));
                                const hasMatchingComps = comps && comps.length > 0;
                                const containerMatches = matchesSearch(s.id + '.' + c.id, c.label || c.id);
                                if (!containerMatches && !hasMatchingComps) return null;
                                return renderItem(s.id + '.' + c.id, c.label || c.id, 'container', (
                                    <>
                                        {comps?.map(comp => renderItem(s.id + '.' + c.id + '.' + comp.id, comp.label || comp.id, 'component', undefined, comp))}
                                    </>
                                ), c);
                            })}
                            {datastores?.map(d => renderItem(s.id + '.' + d.id, d.label || d.id, 'datastore', undefined, d))}
                            {queues?.map(q => renderItem(s.id + '.' + q.id, q.label || q.id, 'queue', undefined, q))}
                        </>
                    ), s);
                })}

                {filteredRequirements.map(r => renderItem(r.id, r.title || r.id, 'requirement', undefined, r))}
                {filteredAdrs.map(a => renderItem(a.id, a.title || a.id, 'adr', undefined, a))}
                {filteredDeployments.map(d => renderItem(d.id, d.label || d.id, 'deployment', undefined, d))}
                
                {searchQuery && (
                    <div className="px-4 py-8 text-center text-[var(--color-text-tertiary)] text-xs">
                        {arch?.persons?.length === 0 && arch?.systems?.length === 0 && (
                            <p>No nodes found matching "{searchQuery}"</p>
                        )}
                    </div>
                )}
            </div>
        </div>
    );
});
