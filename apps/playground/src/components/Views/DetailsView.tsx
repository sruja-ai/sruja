import { useState } from 'react';
import { RequirementsPanel } from '../Panels/RequirementsPanel';
import { ADRsPanel } from '../Panels/ADRsPanel';
import './DetailsView.css';

export function DetailsView() {
    const [activeSubTab, setActiveSubTab] = useState<'requirements' | 'adrs'>('requirements');

    return (
        <div className="details-view">
            <div className="details-subnav">
                <button
                    className={`subnav-item ${activeSubTab === 'requirements' ? 'active' : ''}`}
                    onClick={() => setActiveSubTab('requirements')}
                >
                    Requirements
                </button>
                <button
                    className={`subnav-item ${activeSubTab === 'adrs' ? 'active' : ''}`}
                    onClick={() => setActiveSubTab('adrs')}
                >
                    ADRs
                </button>
            </div>
            <div className="details-content">
                {activeSubTab === 'requirements' && <RequirementsPanel />}
                {activeSubTab === 'adrs' && <ADRsPanel />}
            </div>
        </div>
    );
}
