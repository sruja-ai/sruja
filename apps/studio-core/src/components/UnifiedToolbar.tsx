// apps/studio-core/src/components/UnifiedToolbar.tsx
import React from 'react';
import { Button } from '@sruja/ui';
import {
  User, Server, Box, Database, Layers, ArrowRight,
  ZoomIn, ZoomOut, Maximize2, RotateCcw, RotateCw,
  HelpCircle, Download, Trash2,
  ShieldCheck, FileText, Cloud, Cpu
} from 'lucide-react';
import { useViewStore, type ViewStep } from '../stores/ViewStore';
import { Filter } from 'lucide-react';
import { Tooltip as CustomTooltip } from './Tooltip';
import { getDocUrl } from '@sruja/shared';

// Safe wrapper for getDocUrl
function safeGetDocUrl(type: string): string {
  try {
    return getDocUrl(type);
  } catch (error) {
    console.warn(`Failed to get doc URL for type: ${type}`, error);
    return `https://sruja.ai/docs/nodes/${type}`;
  }
}

interface UnifiedToolbarProps {
  // Node palette
  onAddNode: (type: 'person' | 'system' | 'container' | 'component' | 'datastore' | 'queue' | 'requirement' | 'adr' | 'deployment') => void;
  
  // Actions
  onToggleRelation: () => void;
  isAddingRelation: boolean;
  onDelete?: () => void;
  onUndo?: () => void;
  onRedo?: () => void;
  
  // Zoom & View
  zoomLevel: number;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onFitToScreen: () => void;
  currentLevel?: number;
  onSetLevel?: (level: number) => void;
  
  // Export & Help
  onExport?: (format: 'svg' | 'png') => void;
  onShowHelp?: () => void;
  
  // Other
  selectedNodeId?: string | null;
  onToggleCollapse?: () => void;
  onShare?: () => void;
}

export function UnifiedToolbar({
  onAddNode,
  onToggleRelation,
  isAddingRelation,
  onDelete,
  onUndo,
  onRedo,
  zoomLevel,
  onZoomIn,
  onZoomOut,
  onFitToScreen,
  currentLevel,
  onSetLevel,
  onExport,
  onShowHelp,
  selectedNodeId,
  onToggleCollapse,
  onShare,
}: UnifiedToolbarProps) {
  const { activeStep, filterByStep, setFilterByStep } = useViewStore();

  // Determine available node types based on step filter
  const getAvailableNodes = () => {
    if (filterByStep) {
      // Step-based filtering: show nodes based on active step
      switch (activeStep) {
        case 'context':
          return [
            { type: 'person' as const, icon: User, label: 'Person' },
            { type: 'system' as const, icon: Server, label: 'System' },
          ];
        case 'containers':
          return [
            { type: 'container' as const, icon: Box, label: 'Container' },
            { type: 'datastore' as const, icon: Database, label: 'Database' },
            { type: 'queue' as const, icon: Layers, label: 'Queue' },
          ];
        case 'components':
          return [
            { type: 'component' as const, icon: Cpu, label: 'Component' },
          ];
        default:
          return [];
      }
    } else {
      // Show all common nodes
      return [
        { type: 'person' as const, icon: User, label: 'Person' },
        { type: 'component' as const, icon: Cpu, label: 'Component' },
        { type: 'system' as const, icon: Server, label: 'System' },
        { type: 'container' as const, icon: Box, label: 'Container' },
        { type: 'datastore' as const, icon: Database, label: 'Database' },
        { type: 'queue' as const, icon: Layers, label: 'Queue' },
        { type: 'requirement' as const, icon: ShieldCheck, label: 'Requirement' },
        { type: 'adr' as const, icon: FileText, label: 'ADR' },
        { type: 'deployment' as const, icon: Cloud, label: 'Deployment' },
      ];
    }
  };

  const availableNodes = getAvailableNodes();

  return (
    <div className="px-4 py-2.5 flex flex-wrap items-center justify-between gap-3 border-b border-[var(--color-border)] bg-[var(--color-surface)] flex-shrink-0">
      {/* Left: Node Palette & Actions */}
      <div className="flex flex-wrap items-center gap-2">
        {/* Node Palette Buttons */}
        {availableNodes.length > 0 && (
          <>
            <div className="flex flex-wrap items-center gap-1.5">
              {availableNodes.map((node) => {
                const Icon = node.icon;
                return (
                  <CustomTooltip key={node.type} content={`Add ${node.label}\nDocs: ${safeGetDocUrl(node.type)}`}>
                    <Button
                      variant="secondary"
                      size="sm"
                      onClick={() => onAddNode(node.type)}
                      title={`Add ${node.label}`}
                      className="px-2"
                    >
                      <Icon size={16} />
                    </Button>
                  </CustomTooltip>
                );
              })}
            </div>
            <div className="w-px h-6 bg-[var(--color-border)]" />
          </>
        )}

        {/* Add Relation */}
        <CustomTooltip content={isAddingRelation ? 'Cancel Relation' : 'Add Relation'}>
          <Button
            variant={isAddingRelation ? 'primary' : 'secondary'}
            size="sm"
            onClick={onToggleRelation}
            title="Add Relation"
          >
            <ArrowRight size={16} />
          </Button>
        </CustomTooltip>

        {/* Step Filter Toggle */}
        <div className="w-px h-6 bg-[var(--color-border)]" />
        <CustomTooltip content={filterByStep ? 'Show All Nodes' : 'Filter by Step'}>
          <Button
            variant={filterByStep ? 'primary' : 'secondary'}
            size="sm"
            onClick={() => setFilterByStep(!filterByStep)}
            title={filterByStep ? 'Show All Nodes' : 'Filter by Step'}
            className="px-2"
          >
            <Filter size={16} />
          </Button>
        </CustomTooltip>

        {/* Level Selector */}
        {onSetLevel && (
          <>
            <div className="w-px h-6 bg-[var(--color-border)]" />
            <div className="flex items-center gap-1 bg-[var(--color-background)] rounded-md p-0.5 border border-[var(--color-border)]">
              <Button
                variant={currentLevel === 1 ? 'primary' : 'ghost'}
                size="sm"
                onClick={() => onSetLevel(1)}
                title="Level 1: System Context"
                className="px-2.5"
              >
                L1
              </Button>
              <Button
                variant={currentLevel === 2 ? 'primary' : 'ghost'}
                size="sm"
                onClick={() => onSetLevel(2)}
                title="Level 2: Containers"
                className="px-2.5"
              >
                L2
              </Button>
              <Button
                variant={currentLevel === 3 ? 'primary' : 'ghost'}
                size="sm"
                onClick={() => onSetLevel(3)}
                title="Level 3: Components"
                className="px-2.5"
              >
                L3
              </Button>
            </div>
          </>
        )}
      </div>

      {/* Right: Zoom, Undo/Redo, Actions */}
      <div className="flex flex-wrap items-center gap-2">
        {/* Undo/Redo */}
        {(onUndo || onRedo) && (
          <>
            {onUndo && (
              <Button
                variant="secondary"
                size="sm"
                onClick={onUndo}
                title="Undo (Ctrl+Z)"
                className="px-2"
              >
                <RotateCcw size={16} />
              </Button>
            )}
            {onRedo && (
              <Button
                variant="secondary"
                size="sm"
                onClick={onRedo}
                title="Redo (Ctrl+Y)"
                className="px-2"
              >
                <RotateCw size={16} />
              </Button>
            )}
            <div className="w-px h-6 bg-[var(--color-border)]" />
          </>
        )}

        {/* Zoom Controls */}
        <div className="flex items-center gap-1 bg-[var(--color-background)] rounded-md p-0.5 border border-[var(--color-border)]">
          <Button
            variant="ghost"
            size="sm"
            onClick={onZoomOut}
            title="Zoom Out (-)"
            className="px-2"
          >
            <ZoomOut size={16} />
          </Button>
          <span className="text-xs text-[var(--color-text-secondary)] px-2 font-medium min-w-[3rem] text-center">
            {Math.round(zoomLevel * 100)}%
          </span>
          <Button
            variant="ghost"
            size="sm"
            onClick={onZoomIn}
            title="Zoom In (+)"
            className="px-2"
          >
            <ZoomIn size={16} />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={onFitToScreen}
            title="Fit to Screen (0)"
            className="px-2"
          >
            <Maximize2 size={16} />
          </Button>
        </div>

        {/* Delete & Other Actions */}
        {(onDelete || onToggleCollapse || onShare) && (
          <>
            <div className="w-px h-6 bg-[var(--color-border)]" />
            <div className="flex items-center gap-1.5">
              {onToggleCollapse && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={onToggleCollapse}
                  title="Expand/Collapse Selected"
                  className="px-2"
                >
                  <Maximize2 size={16} />
                </Button>
              )}
              {onDelete && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={onDelete}
                  title="Delete Selected"
                  className="px-2 text-red-600 hover:text-red-700 hover:bg-red-50"
                >
                  <Trash2 size={16} />
                </Button>
              )}
              {onShare && (
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={onShare}
                  title="Share"
                >
                  <FileText size={16} />
                  <span className="ml-1">Share</span>
                </Button>
              )}
            </div>
          </>
        )}

        {/* Export & Help */}
        {(onExport || onShowHelp) && (
          <>
            <div className="w-px h-6 bg-[var(--color-border)]" />
            <div className="flex items-center gap-1.5">
              {onExport && (
                <div className="relative group">
                  <Button
                    variant="secondary"
                    size="sm"
                    title="Export Diagram"
                  >
                    <Download size={16} />
                  </Button>
                  <div className="absolute right-0 top-full mt-1 bg-[var(--color-background)] border border-[var(--color-border)] rounded-md shadow-lg opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-200 z-50 min-w-[120px]">
                    <button
                      onClick={() => onExport('svg')}
                      className="w-full text-left px-3 py-2 text-sm text-[var(--color-text-primary)] hover:bg-[var(--color-surface)] transition-colors"
                    >
                      Export as SVG
                    </button>
                    <button
                      onClick={() => onExport('png')}
                      className="w-full text-left px-3 py-2 text-sm text-[var(--color-text-primary)] hover:bg-[var(--color-surface)] transition-colors border-t border-[var(--color-border)]"
                    >
                      Export as PNG
                    </button>
                  </div>
                </div>
              )}
              {onShowHelp && (
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={onShowHelp}
                  title="Help & Keyboard Shortcuts"
                  className="px-2"
                >
                  <HelpCircle size={16} />
                </Button>
              )}
            </div>
          </>
        )}
      </div>
    </div>
  );
}
