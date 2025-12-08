import React from 'react';
import { Button, Card, Badge } from '@sruja/ui';

interface OverviewProps {
  data: any;
  onExportPNG: () => void;
  onExportSVG: () => void;
  onDownloadJSON: () => void;
  onDownloadHTML?: () => void;
}

export function OverviewPage({ data, onExportPNG, onExportSVG, onDownloadJSON, onDownloadHTML }: OverviewProps) {
  const meta = data?.metadata || {};
  const arch = data?.architecture || {};

  const metrics = [
    { label: 'Systems', value: (arch.systems || []).length },
    { label: 'Containers', value: (arch.systems || []).reduce((sum: number, s: any) => sum + (s.containers?.length || 0), 0) + (arch.containers?.length || 0) },
    { label: 'Persons', value: (arch.persons || []).length },
    { label: 'Datastores', value: (arch.systems || []).reduce((sum: number, s: any) => sum + (s.datastores?.length || 0), 0) + (arch.datastores?.length || 0) },
    { label: 'Queues', value: (arch.systems || []).reduce((sum: number, s: any) => sum + (s.queues?.length || 0), 0) + (arch.queues?.length || 0) },
    { label: 'Relations', value: (arch.relations || []).length },
    { label: 'Scenarios', value: (arch.scenarios || []).length },
    { label: 'Requirements', value: (arch.requirements || []).length },
    { label: 'ADRs', value: (arch.adrs || []).length },
  ];

  return (
    <div className="page-content">
      <div className="overview-header">
        <div className="overview-title">
          <h1>{meta.name || 'Architecture'}</h1>
          <div className="overview-meta">
            <span className="meta-item">Version {meta.version || '1.0'}</span>
            {meta.generated && <span className="meta-sep">•</span>}
            {meta.generated && <span className="meta-item">Generated {new Date(meta.generated).toLocaleString()}</span>}
          </div>
        </div>
        <div className="overview-actions">
          <button className="primary-btn" onClick={onExportPNG}>Export PNG</button>
          <button className="secondary-btn" onClick={onExportSVG}>Export SVG</button>
          <button className="secondary-btn" onClick={onDownloadJSON}>Download JSON</button>
          {onDownloadHTML && (
            <button className="secondary-btn" onClick={onDownloadHTML}>Download HTML</button>
          )}
        </div>
      </div>

      <div className="metrics-grid">
        {metrics.map(m => (
          <div key={m.label} className="metric-card">
            <div className="metric-value">{m.value}</div>
            <div className="metric-label">{m.label}</div>
          </div>
        ))}
      </div>

      <div className="legend-section">
        <h2>Legend</h2>
        <div className="legend-grid">
          <div className="legend-item">
            <div className="legend-swatch person" />
            <div className="legend-text">Person</div>
          </div>
          <div className="legend-item">
            <div className="legend-swatch system" />
            <div className="legend-text">System</div>
          </div>
          <div className="legend-item">
            <div className="legend-swatch container" />
            <div className="legend-text">Container</div>
          </div>
          <div className="legend-item">
            <div className="legend-swatch datastore" />
            <div className="legend-text">Datastore</div>
          </div>
          <div className="legend-item">
            <div className="legend-swatch queue" />
            <div className="legend-text">Queue</div>
          </div>
        </div>
      </div>
    </div>
  );
}
