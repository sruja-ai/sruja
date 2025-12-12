// apps/playground/src/components/ViewTabs.tsx
import { Layout, Eye, FileCode, Compass, List } from 'lucide-react';
import { Badge } from '@sruja/ui';
import type { ViewTab } from '../types';
import { useFeatureFlagsStore } from '../stores';

interface ViewTabsProps {
    activeTab: ViewTab;
    onTabChange: (tab: ViewTab) => void;
    counts: {
        requirements: number;
        adrs: number;
    };
}

export function ViewTabs({ activeTab, onTabChange, counts }: ViewTabsProps) {
    const editMode = useFeatureFlagsStore((s) => s.editMode);

    return (
        <div className="view-tabs">
            <button
                className={`view-tab ${activeTab === 'overview' ? 'active' : ''}`}
                onClick={() => onTabChange('overview')}
            >
                <Eye size={16} />
                Overview
            </button>
            <button
                className={`view-tab ${activeTab === 'diagram' ? 'active' : ''}`}
                onClick={() => onTabChange('diagram')}
            >
                <Layout size={16} />
                Diagram
            </button>
            <button
                className={`view-tab ${activeTab === 'details' ? 'active' : ''}`}
                onClick={() => onTabChange('details')}
            >
                <List size={16} />
                Details
                {(counts.requirements + counts.adrs) > 0 && (
                    <Badge color="brand">
                        {counts.requirements + counts.adrs}
                    </Badge>
                )}
            </button>
            <button
                className={`view-tab ${activeTab === 'code' ? 'active' : ''}`}
                onClick={() => onTabChange('code')}
            >
                <FileCode size={16} />
                Code
            </button>
            {editMode === 'edit' && (
                <button
                    className={`view-tab ${activeTab === 'guided' ? 'active' : ''}`}
                    onClick={() => onTabChange('guided')}
                >
                    <Compass size={16} />
                    Guided
                </button>
            )}
        </div>
    );
}
