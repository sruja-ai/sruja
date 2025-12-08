// apps/studio-core/src/components/ViewerToolbar.tsx
import React from 'react';
import { Button } from '@sruja/ui';
import {
  User, Server, Box, Database, Layers, ArrowRight,
  ZoomIn, ZoomOut, Maximize, Maximize2, Trash2,
  ShieldCheck, FileText, Cloud, Cpu
} from 'lucide-react';
import { Tooltip as CustomTooltip } from './Tooltip';
import { getDocUrl } from '@sruja/shared';

// Safe wrapper for getDocUrl to handle potential errors
function safeGetDocUrl(type: string): string {
  try {
    return getDocUrl(type);
  } catch (error) {
    console.warn(`Failed to get doc URL for type: ${type}`, error);
    return `https://sruja.ai/docs/nodes/${type}`;
  }
}

interface ViewerToolbarProps {
  onAddNode: (type: 'person' | 'system' | 'container' | 'component' | 'datastore' | 'queue' | 'requirement' | 'adr' | 'deployment') => void;
  onToggleRelation: () => void;
  isAddingRelation: boolean;
  sourceNode: string | null;
  currentLevel?: number;
  onSetLevel: (level: number) => void;
  zoomLevel: number;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onFitToScreen: () => void;
  onToggleCollapse: () => void;
  onDelete: () => void;
  onShare?: () => void;
}

export function ViewerToolbar({
  onAddNode,
  onToggleRelation,
  isAddingRelation,
  sourceNode,
  currentLevel,
  onSetLevel,
  zoomLevel,
  onZoomIn,
  onZoomOut,
  onFitToScreen,
  onToggleCollapse,
  onDelete,
  onShare,
}: ViewerToolbarProps) {
  return (
    <div className="px-5 py-3 border-b border-[var(--color-border)] bg-[var(--color-background)]/80 backdrop-blur supports-[backdrop-filter]:shadow-sm">
      <div className="flex flex-wrap items-center justify-between gap-4">
        {/* Left: Add Elements Group */}
        <div className="flex flex-wrap items-center gap-3">
          <div className="flex flex-wrap items-center gap-1.5">
            <CustomTooltip content={`Add Person\nDocs: ${safeGetDocUrl('person')}`}>
              <Button variant="secondary" size="sm" onClick={() => onAddNode('person')} title="Add Person">
                <User size={16} />
              </Button>
            </CustomTooltip>
            <CustomTooltip content={`Add Component\nDocs: ${safeGetDocUrl('component')}`}>
              <Button variant="secondary" size="sm" onClick={() => onAddNode('component')} title="Add Component">
                <Cpu size={16} />
              </Button>
            </CustomTooltip>
            <CustomTooltip content={`Add System\nDocs: ${safeGetDocUrl('system')}`}>
              <Button variant="secondary" size="sm" onClick={() => onAddNode('system')} title="Add System">
                <Server size={16} />
              </Button>
            </CustomTooltip>
            <CustomTooltip content={`Add Container\nDocs: ${safeGetDocUrl('container')}`}>
              <Button variant="secondary" size="sm" onClick={() => onAddNode('container')} title="Add Container">
                <Box size={16} />
              </Button>
            </CustomTooltip>
            <CustomTooltip content={`Add Database\nDocs: ${safeGetDocUrl('datastore')}`}>
              <Button variant="secondary" size="sm" onClick={() => onAddNode('datastore')} title="Add Database">
                <Database size={16} />
              </Button>
            </CustomTooltip>
            <CustomTooltip content={`Add Queue\nDocs: ${safeGetDocUrl('queue')}`}>
              <Button variant="secondary" size="sm" onClick={() => onAddNode('queue')} title="Add Queue">
                <Layers size={16} />
              </Button>
            </CustomTooltip>
          </div>

          <div className="hidden sm:block w-px h-6 bg-[var(--color-border)]"></div>

          <div className="flex flex-wrap items-center gap-1.5">
            <CustomTooltip content={`Add Requirement\nDocs: ${safeGetDocUrl('requirement')}`}>
              <Button variant="secondary" size="sm" onClick={() => onAddNode('requirement')} title="Add Requirement">
                <ShieldCheck size={16} />
              </Button>
            </CustomTooltip>
            <CustomTooltip content={`Add ADR\nDocs: ${safeGetDocUrl('adr')}`}>
              <Button variant="secondary" size="sm" onClick={() => onAddNode('adr')} title="Add ADR">
                <FileText size={16} />
              </Button>
            </CustomTooltip>
            <CustomTooltip content={`Add Deployment Node\nDocs: ${safeGetDocUrl('deployment')}`}>
              <Button variant="secondary" size="sm" onClick={() => onAddNode('deployment')} title="Add Deployment Node">
                <Cloud size={16} />
              </Button>
            </CustomTooltip>
          </div>

          <div className="hidden sm:block w-px h-6 bg-[var(--color-border)]"></div>

          <CustomTooltip content={isAddingRelation ? (sourceNode ? 'Select Target' : 'Connect') : 'Connect (hover node to see connection points)'}>
            <Button
              variant={isAddingRelation ? 'primary' : 'secondary'}
              size="sm"
              onClick={onToggleRelation}
              title="Add Relation (or hover over a node and drag from connection handles)"
            >
              <ArrowRight size={16} />
            </Button>
          </CustomTooltip>
        </div>

        {/* Center: View Controls */}
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-1 bg-[var(--color-surface)] rounded-md p-0.5">
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
          {/* Focus selectors – optionally render based on level */}
          <div className="hidden md:flex items-center gap-2">
            {/* Placeholder: actual system/container dropdowns can be added later */}
          </div>
        </div>

        {/* Right: Zoom & Actions */}
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-1 bg-[var(--color-surface)] rounded-md p-0.5">
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
              onClick={onZoomOut}
              title="Zoom Out (-)"
              className="px-2"
            >
              <ZoomOut size={16} />
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onClick={onFitToScreen}
              title="Fit to Screen (0)"
              className="px-2"
            >
              <Maximize size={16} />
            </Button>
            <span className="text-xs text-[var(--color-text-secondary)] px-2 font-medium min-w-[3rem] text-center">
              {Math.round(zoomLevel * 100)}%
            </span>
          </div>

          <div className="w-px h-6 bg-[var(--color-border)]"></div>

        <div className="flex items-center gap-1.5">
          <Button variant="ghost" size="sm" onClick={onToggleCollapse} title="Expand/Collapse Selected">
            <Maximize2 size={16} />
          </Button>
          <Button variant="ghost" size="sm" onClick={onDelete} title="Delete Selected" className="text-red-600 hover:text-red-700 hover:bg-red-50">
            <Trash2 size={16} />
          </Button>
          {onShare && (
            <Button variant="secondary" size="sm" onClick={onShare} title="Copy shareable URL">
              {/* Using FileText icon already imported; alternatively use Copy */}
              <FileText size={16} />
              <span className="ml-1">Share</span>
            </Button>
          )}
        </div>
        </div>
      </div>
    </div>
  );
}
