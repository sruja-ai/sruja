// apps/studio-core/src/utils/commands.tsx
import React from 'react';
import {
  ZoomIn, ZoomOut, Maximize, Download, FileText, Copy,
  Save, Sidebar, Settings, User, Server, Box, Database, Layers, ArrowRight, Pencil
} from 'lucide-react';
import type { Command } from '../components/CommandPalette';

export function createCommands(
  handlers: {
    handleSave: () => void;
    handleCopyShareLink: () => void;
    handleZoomIn: () => void;
    handleZoomOut: () => void;
    handleFitToScreen: () => void;
    handleAddNode: (type: 'person' | 'system' | 'container' | 'datastore' | 'queue' | 'requirement' | 'adr' | 'deployment') => void;
    toggleAddRelation: () => void;
    setShowSidebar: (show: boolean) => void;
    setActiveSidebarPanel: (panel: 'explorer' | 'documentation' | 'shortcuts' | 'guide') => void;
    setShowProperties: (show: boolean) => void;
    showSidebar: boolean;
    activeSidebarPanel: 'explorer' | 'documentation' | 'shortcuts' | 'guide';
    showProperties: boolean;
    handleRename: () => void;
  }
): Command[] {
  const {
    handleSave,
    handleCopyShareLink,
    handleZoomIn,
    handleZoomOut,
    handleFitToScreen,
    handleAddNode,
    toggleAddRelation,
    setShowSidebar,
    setActiveSidebarPanel,
    setShowProperties,
    showSidebar,
    activeSidebarPanel,
    showProperties,
    handleRename,
  } = handlers;

  return [
    {
      id: 'rename-selected',
      label: 'Rename Selected',
      description: 'Rename the currently selected node',
      icon: <Pencil size={18} />,
      action: handleRename,
      keywords: ['rename', 'refactor', 'id', 'label'],
      category: 'Edit',
    },
    {
      id: 'save',
      label: 'Save',
      description: 'Save current diagram',
      icon: <Save size={18} />,
      action: handleSave,
      keywords: ['save', 'store'],
      category: 'File',
    },
    {
      id: 'copy-share-link',
      label: 'Copy Share Link',
      description: 'Copy URL containing current DSL',
      icon: <Copy size={18} />,
      action: handleCopyShareLink,
      keywords: ['share', 'link', 'url', 'copy'],
      category: 'File',
    },
    {
      id: 'zoom-in',
      label: 'Zoom In',
      description: 'Zoom in on the diagram',
      icon: <ZoomIn size={18} />,
      action: handleZoomIn,
      keywords: ['zoom', 'in', 'closer'],
      category: 'View',
    },
    {
      id: 'zoom-out',
      label: 'Zoom Out',
      description: 'Zoom out from the diagram',
      icon: <ZoomOut size={18} />,
      action: handleZoomOut,
      keywords: ['zoom', 'out', 'farther'],
      category: 'View',
    },
    {
      id: 'fit-screen',
      label: 'Fit to Screen',
      description: 'Fit entire diagram to viewport',
      icon: <Maximize size={18} />,
      action: handleFitToScreen,
      keywords: ['fit', 'screen', 'viewport', 'all'],
      category: 'View',
    },
    {
      id: 'add-person',
      label: 'Add Person',
      description: 'Add a new person to the diagram',
      icon: <User size={18} />,
      action: () => handleAddNode('person'),
      keywords: ['add', 'person', 'user', 'actor'],
      category: 'Add Element',
    },
    {
      id: 'add-system',
      label: 'Add System',
      description: 'Add a new system to the diagram',
      icon: <Server size={18} />,
      action: () => handleAddNode('system'),
      keywords: ['add', 'system', 'application'],
      category: 'Add Element',
    },
    {
      id: 'add-container',
      label: 'Add Container',
      description: 'Add a new container to the diagram',
      icon: <Box size={18} />,
      action: () => handleAddNode('container'),
      keywords: ['add', 'container', 'service'],
      category: 'Add Element',
    },
    {
      id: 'add-database',
      label: 'Add Database',
      description: 'Add a new database to the diagram',
      icon: <Database size={18} />,
      action: () => handleAddNode('datastore'),
      keywords: ['add', 'database', 'db', 'datastore', 'store'],
      category: 'Add Element',
    },
    {
      id: 'add-queue',
      label: 'Add Queue',
      description: 'Add a new queue to the diagram',
      icon: <Layers size={18} />,
      action: () => handleAddNode('queue'),
      keywords: ['add', 'queue', 'message'],
      category: 'Add Element',
    },
    {
      id: 'add-relation',
      label: 'Add Relation',
      description: 'Connect two elements',
      icon: <ArrowRight size={18} />,
      action: toggleAddRelation,
      keywords: ['add', 'relation', 'edge', 'connect', 'link'],
      category: 'Add Element',
    },
    {
      id: 'toggle-explorer',
      label: 'Toggle Explorer',
      description: 'Show/hide the model explorer panel',
      icon: <Sidebar size={18} />,
      action: () => {
        if (showSidebar && activeSidebarPanel === 'explorer') {
          setShowSidebar(false);
        } else {
          setShowSidebar(true);
          setActiveSidebarPanel('explorer');
        }
      },
      keywords: ['toggle', 'explorer', 'sidebar', 'panel'],
      category: 'View',
    },
    {
      id: 'toggle-properties',
      label: 'Toggle Properties',
      description: 'Show/hide the properties panel',
      icon: <Settings size={18} />,
      action: () => setShowProperties(!showProperties),
      keywords: ['toggle', 'properties', 'settings', 'panel'],
      category: 'View',
    },
  ];
}
