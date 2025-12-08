import { useMemo } from 'react';
import { ChevronDown, ChevronRight, Building2, Box, User, Play } from 'lucide-react';
import { useArchitectureStore, useViewStore, useSelectionStore } from '../../stores';
import type { SystemJSON, FlowJSON } from '../../types';
import './NavigationPanel.css';

export function NavigationPanel() {
    const data = useArchitectureStore((s) => s.data);
    const currentLevel = useViewStore((s) => s.currentLevel);
    const focusedSystemId = useViewStore((s) => s.focusedSystemId);
    const drillDown = useViewStore((s) => s.drillDown);
    const goToRoot = useViewStore((s) => s.goToRoot);
    const setActiveFlow = useSelectionStore((s) => s.setActiveFlow);

    const systems = useMemo(() => data?.architecture.systems ?? [], [data]);
    const persons = useMemo(() => data?.architecture.persons ?? [], [data]);
    const flows = useMemo(() => data?.architecture.flows ?? [], [data]);

    if (!data) {
        return (
            <div className="navigation-panel">
                <div className="panel-empty">Load an architecture to see navigation</div>
            </div>
        );
    }

    return (
        <div className="navigation-panel">
            {/* Level Selector */}
            <div className="nav-section">
                <div className="nav-section-title">View Level</div>
                <div className="level-buttons">
                    <button
                        className={`level-btn ${currentLevel === 'L1' ? 'active' : ''}`}
                        onClick={goToRoot}
                    >
                        L1 Context
                    </button>
                    <button
                        className={`level-btn ${currentLevel === 'L2' ? 'active' : ''}`}
                        disabled={!focusedSystemId}
                    >
                        L2 Container
                    </button>
                    <button
                        className={`level-btn ${currentLevel === 'L3' ? 'active' : ''}`}
                        disabled={currentLevel !== 'L3'}
                    >
                        L3 Component
                    </button>
                </div>
            </div>

            {/* Systems Tree */}
            <div className="nav-section">
                <div className="nav-section-title">
                    <Building2 size={14} />
                    Systems ({systems.length})
                </div>
                <ul className="nav-tree">
                    {systems.map((system) => (
                        <SystemTreeItem
                            key={system.id}
                            system={system}
                            isSelected={focusedSystemId === system.id}
                            onSelect={() => drillDown(system.id, 'system')}
                        />
                    ))}
                </ul>
            </div>

            {/* Persons */}
            {persons.length > 0 && (
                <div className="nav-section">
                    <div className="nav-section-title">
                        <User size={14} />
                        Actors ({persons.length})
                    </div>
                    <ul className="nav-tree">
                        {persons.map((person) => (
                            <li key={person.id} className="nav-item">
                                <User size={12} />
                                <span>{person.label ?? person.id}</span>
                            </li>
                        ))}
                    </ul>
                </div>
            )}

            {/* Flows */}
            {flows.length > 0 && (
                <div className="nav-section">
                    <div className="nav-section-title">
                        <Play size={14} />
                        Flows ({flows.length})
                    </div>
                    <ul className="nav-tree">
                        {flows.map((flow) => (
                            <li
                                key={flow.id}
                                className="nav-item clickable"
                                onClick={() => setActiveFlow(flow)}
                            >
                                <Play size={12} />
                                <span>{flow.title ?? flow.label ?? flow.id}</span>
                            </li>
                        ))}
                    </ul>
                </div>
            )}
        </div>
    );
}

interface SystemTreeItemProps {
    system: SystemJSON;
    isSelected: boolean;
    onSelect: () => void;
}

function SystemTreeItem({ system, isSelected, onSelect }: SystemTreeItemProps) {
    const hasContainers = (system.containers?.length ?? 0) > 0;

    return (
        <li className={`nav-item ${isSelected ? 'selected' : ''}`}>
            <button className="nav-item-btn" onClick={onSelect}>
                {hasContainers ? <ChevronRight size={12} /> : <span className="spacer" />}
                <Building2 size={12} />
                <span className="nav-item-label">{system.label ?? system.id}</span>
                {hasContainers && (
                    <span className="nav-item-count">{system.containers?.length}</span>
                )}
            </button>
        </li>
    );
}
