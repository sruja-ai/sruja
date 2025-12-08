// Status bar component showing diagram stats, validation, zoom, etc.

import React from 'react';
import { ArchitectureJSON } from '@sruja/viewer';
import { useStudioState } from '../context/StudioStateContext';
import { CheckCircle2, AlertCircle, ZoomIn } from 'lucide-react';

interface StatusBarProps {
  archData: ArchitectureJSON | null;
  zoomLevel?: number;
  lastSaved?: Date | null;
  validationStatus?: {
    isValid: boolean;
    errors?: number;
    warnings?: number;
    lastError?: string;
  };
  currentLevel?: number;
  onValidationClick?: () => void;
}

export const StatusBar: React.FC<StatusBarProps> = ({
  archData,
  zoomLevel,
  lastSaved,
  validationStatus,
  currentLevel,
  onValidationClick,
}) => {
  const { setSidebar } = useStudioState();
  const getNodeCount = () => {
    if (!archData?.architecture) return 0;
    const arch = archData.architecture;
    let count = 0;
    if (arch.persons) count += arch.persons.length;
    if (arch.systems) {
      count += arch.systems.length;
      arch.systems.forEach((sys) => {
        if (sys.containers) count += sys.containers.length;
        if (sys.datastores) count += sys.datastores.length;
        if (sys.queues) count += sys.queues.length;
      });
    }
    if (arch.containers) count += arch.containers.length;
    if (arch.datastores) count += arch.datastores.length;
    if (arch.queues) count += arch.queues.length;
    return count;
  };

  const getEdgeCount = () => {
    if (!archData?.architecture?.relations) return 0;
    return archData.architecture.relations.length;
  };

  const formatLastSaved = () => {
    if (!lastSaved) return 'Never saved';
    const now = new Date();
    const diff = now.getTime() - lastSaved.getTime();
    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);

    if (seconds < 10) return 'Just saved';
    if (seconds < 60) return `Saved ${seconds}s ago`;
    if (minutes < 60) return `Saved ${minutes}m ago`;
    if (hours < 24) return `Saved ${hours}h ago`;
    return `Saved ${Math.floor(hours / 24)}d ago`;
  };

  const getLevelLabel = () => {
    if (!currentLevel) return '';
    const labels = ['', 'Level 1: Context', 'Level 2: Containers', 'Level 3: Components'];
    return labels[currentLevel] || '';
  };

  const nodeCount = getNodeCount();
  const edgeCount = getEdgeCount();

  // Guide progress (L1/L2/L3)
  const arch = archData?.architecture;
  const persons = arch?.persons?.length || 0;
  const systems = arch?.systems?.length || 0;
  const containers = (arch?.systems || []).reduce((acc: number, s: any) => acc + (s.containers?.length || 0) + (s.datastores?.length || 0) + (s.queues?.length || 0), 0);
  const components = (arch?.systems || []).reduce((acc: number, s: any) => acc + (s.containers || []).reduce((a: number, c: any) => a + (c.components?.length || 0), 0), 0);
  const l1Done = persons > 0 && systems > 0;
  const l2Done = containers > 0;
  const l3Done = components > 0;
  const guidePct = Math.round(((l1Done ? 1 : 0) + (l2Done ? 1 : 0) + (l3Done ? 1 : 0)) / 3 * 100);

  return (
    <div
      className="h-7 border-t border-[var(--color-border)] bg-[var(--color-surface)] flex items-center px-3 text-[11px] text-[var(--color-text-secondary)] gap-4 font-sans"
    >
      {/* Diagram Stats */}
      <div className="flex items-center gap-1">
        <span className="font-medium text-[var(--color-text-primary)]">
          {nodeCount} {nodeCount === 1 ? 'node' : 'nodes'}
        </span>
        <span className="text-[var(--color-border)]">•</span>
        <span className="font-medium text-[var(--color-text-primary)]">
          {edgeCount} {edgeCount === 1 ? 'edge' : 'edges'}
        </span>
      </div>

      {/* Validation Status */}
      {validationStatus && (
        <>
          <div className="w-px h-4 bg-[var(--color-border)]" />
          <div 
            className={`flex items-center gap-1 ${validationStatus.lastError ? 'cursor-pointer hover:opacity-80' : 'cursor-default'}`}
            title={validationStatus.isValid ? 'DSL is valid' : (validationStatus.lastError || 'DSL has validation errors')}
            onClick={validationStatus.lastError && onValidationClick ? onValidationClick : undefined}
          >
            {validationStatus.isValid ? (
              <>
                <CheckCircle2 size={12} className="text-[var(--color-success-500)]" />
                <span className="text-[var(--color-success-500)]">Valid</span>
              </>
            ) : (
              <>
                <AlertCircle size={12} className="text-[var(--color-error-500)]" />
                <span className="text-[var(--color-error-500)]">
                  {validationStatus.errors || 0} {validationStatus.errors === 1 ? 'error' : 'errors'}
                </span>
                {validationStatus.warnings && validationStatus.warnings > 0 && (
                  <>
                    <span className="text-[var(--color-warning-500)] ml-1">
                      {validationStatus.warnings} {validationStatus.warnings === 1 ? 'warning' : 'warnings'}
                    </span>
                  </>
                )}
              </>
            )}
          </div>
        </>
      )}

      {/* Zoom Level */}
      {zoomLevel !== undefined && (
        <>
          <div className="w-px h-4 bg-[var(--color-border)]" />
          <div className="flex items-center gap-1">
            <ZoomIn size={12} className="text-[var(--color-text-secondary)]" />
            <span>{Math.round(zoomLevel * 100)}%</span>
          </div>
        </>
      )}

      {/* Current Level */}
      {currentLevel && (
        <>
          <div className="w-px h-4 bg-[var(--color-border)]" />
          <span>{getLevelLabel()}</span>
        </>
      )}

      {/* Guide Progress (click to open Guide) */}
      <div className="w-px h-4 bg-[var(--color-border)]" />
      <button
        className="flex items-center gap-2 px-2 py-[2px] rounded bg-[var(--color-background)] border border-[var(--color-border)] hover:bg-[var(--color-surface)] transition-colors cursor-pointer"
        title="Architecture Guide Progress (click to open)"
        onClick={() => setSidebar((prev) => ({ ...prev, showSidebar: true, activePanel: 'guide' }))}
      >
        <span className="text-[11px]">Guide</span>
        <div className="w-20 h-2 rounded bg-[var(--color-border)] overflow-hidden">
          <div
            className="h-2"
            style={{
              width: `${guidePct}%`,
              background: 'linear-gradient(90deg, var(--color-primary) 0%, var(--color-info-500) 100%)'
            }}
          />
        </div>
      </button>

      {/* Last Saved */}
      <div className="ml-auto flex items-center gap-1">
        <span>{formatLastSaved()}</span>
      </div>
    </div>
  );
};
