// apps/studio-core/src/components/UnifiedLayout.tsx
import React, { useState } from 'react';
import { UnifiedToolbar } from './UnifiedToolbar';
import { ViewerPane } from './ViewerPane';
import { CollapsiblePropertiesPanel } from './CollapsiblePropertiesPanel';
import { CollapsibleSidebar } from './CollapsibleSidebar';
import { CollapsibleSection } from './CollapsibleSection';
import { Stepper } from './Stepper';
import { ModelExplorer } from './ModelExplorer';
import { DocumentationPanel } from './DocumentationPanel';
import { ShortcutsPanel } from './ShortcutsPanel';
import { GoalsPanel } from './GoalsPanel';
import { useKeyboardShortcuts } from '../hooks/useKeyboardShortcuts';
import { useStudioState } from '../context/StudioStateContext';
import { Footprints, List, BookOpen, Keyboard, X, PanelLeft, CheckSquare } from 'lucide-react';
import { Tooltip } from './Tooltip';
import { SectionErrorBoundary } from './SectionErrorBoundary';
import { ArchitectureJSON } from '@sruja/viewer';
import { cn } from '@sruja/ui';
import { SrujaMonacoEditor } from '@sruja/ui';
import { ViewerInstance } from '@sruja/viewer';

type ViewMode = 'editor' | 'split' | 'viewer';
type SidebarPanel = 'explorer' | 'steps' | 'documentation' | 'shortcuts' | 'goals';

interface UnifiedLayoutProps {
  // View management
  activeView: ViewMode;
  onViewChange: (view: ViewMode) => void;
  
  // DSL Editor
  dsl: string;
  onDslChange: (dsl: string) => void;
  monacoEditorRef: React.RefObject<any>;
  onMonacoSelectionChange: (editor: any) => void;
  
  // Viewer
  containerRef: React.RefObject<HTMLDivElement | null>;
  viewerRef: React.RefObject<ViewerInstance | null>;
  selectedNodeId: string | null;
  archData: ArchitectureJSON | null;
  onPropertiesUpdate: (updates: any) => void;
  
  // Node operations
  onAddNode: (type: 'person' | 'system' | 'container' | 'component' | 'datastore' | 'queue' | 'requirement' | 'adr' | 'deployment') => void;
  onToggleRelation: () => void;
  isAddingRelation: boolean;
  sourceNode: string | null;
  onDelete: () => void;
  onRename?: () => void;
  
  // View controls
  currentLevel?: number;
  onSetLevel: (level: number) => void;
  zoomLevel: number;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onFitToScreen: () => void;
  onToggleCollapse: () => void;
  
  // History
  onUndo?: () => void;
  onRedo?: () => void;
  
  // Other
  onShowHelp?: () => void;
  onExport?: (format: 'svg' | 'png') => void;
  onShare?: () => void;
  onExplorerSelect: (id: string) => void;
}

export const UnifiedLayout: React.FC<UnifiedLayoutProps> = ({
  activeView,
  onViewChange,
  dsl,
  onDslChange,
  monacoEditorRef,
  onMonacoSelectionChange,
  containerRef,
  viewerRef,
  selectedNodeId,
  archData,
  onPropertiesUpdate,
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
  onRename,
  onUndo,
  onRedo,
  onShowHelp,
  onExport,
  onShare,
  onExplorerSelect,
}) => {
  const { sidebar, setSidebar, documentation } = useStudioState();
  const [activeSidebarPanel, setActiveSidebarPanel] = useState<SidebarPanel>('explorer');
  const [showSidebar, setShowSidebar] = useState(true);

  // Keyboard shortcuts
  useKeyboardShortcuts({
    onUndo,
    onRedo,
    onDelete,
    onRename,
    onAddRelation: onToggleRelation,
    onZoomIn,
    onZoomOut,
    onFitToScreen,
    onEscape: () => {
      if (isAddingRelation) {
        onToggleRelation();
      }
    },
  });

  const sidebarPanels = [
    { id: 'explorer' as SidebarPanel, label: 'Explorer', icon: List },
    { id: 'steps' as SidebarPanel, label: 'Steps', icon: Footprints },
    { id: 'documentation' as SidebarPanel, label: 'Documentation', icon: BookOpen },
    { id: 'shortcuts' as SidebarPanel, label: 'Shortcuts', icon: Keyboard },
    { id: 'goals' as SidebarPanel, label: 'Goals', icon: CheckSquare },
  ];

  return (
    <div className="flex flex-col h-full w-full bg-[var(--color-background)] overflow-hidden">
      {/* Unified Toolbar */}
      <UnifiedToolbar
        onAddNode={onAddNode}
        onToggleRelation={onToggleRelation}
        isAddingRelation={isAddingRelation}
        currentLevel={currentLevel}
        onSetLevel={onSetLevel}
        zoomLevel={zoomLevel}
        onZoomIn={onZoomIn}
        onZoomOut={onZoomOut}
        onFitToScreen={onFitToScreen}
        onToggleCollapse={onToggleCollapse}
        onDelete={onDelete}
        onUndo={onUndo}
        onRedo={onRedo}
        onExport={onExport}
        onShowHelp={onShowHelp}
        selectedNodeId={selectedNodeId}
        onShare={onShare}
      />

      {/* Main Layout */}
      <div className="flex-1 flex min-w-0 relative overflow-hidden">
        {/* Left Sidebar */}
        {showSidebar && (
          <div className="flex flex-col border-r border-[var(--color-border)] bg-[var(--color-background)] flex-shrink-0 h-full overflow-hidden" style={{ width: `${sidebar.width}px` }}>
            {/* Sidebar Header with Tabs */}
            <div className="flex border-b border-[var(--color-border)] bg-[var(--color-surface)] flex-shrink-0">
              {sidebarPanels.map((panel) => {
                const Icon = panel.icon;
                return (
                  <Tooltip key={panel.id} content={panel.label}>
                    <button
                      onClick={() => setActiveSidebarPanel(panel.id)}
                      className={cn(
                        "p-2 transition-colors border-b-2 flex items-center justify-center",
                        activeSidebarPanel === panel.id
                          ? "text-[var(--color-text-primary)] border-[var(--color-info-500)] bg-[var(--color-background)]"
                          : "text-[var(--color-text-secondary)] border-transparent hover:text-[var(--color-text-primary)]"
                      )}
                    >
                      <Icon size={18} />
                    </button>
                  </Tooltip>
                );
              })}
              <div className="flex-1" />
              <Tooltip content="Close Sidebar">
                <button
                  onClick={() => setShowSidebar(false)}
                  className="p-2 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] transition-colors"
                >
                  <X size={16} />
                </button>
              </Tooltip>
            </div>

            {/* Panel Content */}
            <div className="flex-1 overflow-y-auto overflow-x-hidden min-h-0">
              {activeSidebarPanel === 'explorer' && (
                <SectionErrorBoundary sectionName="Model Explorer">
                  {archData ? (
                    <ModelExplorer data={archData} onSelect={onExplorerSelect} />
                  ) : (
                    <div className="p-8 text-center text-[var(--color-text-tertiary)]">
                      <List className="w-12 h-12 mx-auto mb-3 opacity-30" />
                      <p className="text-sm font-medium mb-1">No architecture loaded</p>
                      <p className="text-xs opacity-70">Parse your DSL to see the model explorer</p>
                    </div>
                  )}
                </SectionErrorBoundary>
              )}
              {activeSidebarPanel === 'steps' && (
                <SectionErrorBoundary sectionName="Stepper">
                  <Stepper archData={archData} />
                </SectionErrorBoundary>
              )}
              {activeSidebarPanel === 'documentation' && (
                <SectionErrorBoundary sectionName="Documentation Panel">
                  <DocumentationPanel
                  isOpen={true}
                  selectedNodeType={documentation.selectedNodeType}
                  selectedNodeId={documentation.selectedNodeId}
                  selectedNodeLabel={documentation.selectedNodeLabel}
                />
                </SectionErrorBoundary>
              )}
              {activeSidebarPanel === 'shortcuts' && (
                <SectionErrorBoundary sectionName="Shortcuts Panel">
                  <ShortcutsPanel />
                </SectionErrorBoundary>
              )}
              {activeSidebarPanel === 'goals' && (
                <SectionErrorBoundary sectionName="Goals Panel">
                  <GoalsPanel archData={archData} />
                </SectionErrorBoundary>
              )}
            </div>
          </div>
        )}

        {/* Main Content Area */}
        <div className="flex-1 flex flex-col min-w-0">
          {/* View Switcher Tabs */}
          <div className="flex border-b border-[var(--color-border)] bg-[var(--color-surface)]">
            {!showSidebar && (
              <Tooltip content="Show Sidebar">
                <button
                  onClick={() => setShowSidebar(true)}
                  className="px-3 py-2 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] transition-colors border-r border-[var(--color-border)]"
                  title="Show Sidebar"
                >
                  <PanelLeft size={18} />
                </button>
              </Tooltip>
            )}
            <button
              onClick={() => onViewChange('editor')}
              className={cn(
                "px-4 py-2 border-none bg-transparent cursor-pointer text-sm font-medium transition-colors border-b-2",
                activeView === 'editor'
                  ? "text-[var(--color-info-500)] border-[var(--color-info-500)]"
                  : "text-[var(--color-text-secondary)] border-transparent hover:text-[var(--color-text-primary)]"
              )}
            >
              Editor
            </button>
            <button
              onClick={() => onViewChange('split')}
              className={cn(
                "px-4 py-2 border-none bg-transparent cursor-pointer text-sm font-medium transition-colors border-b-2",
                activeView === 'split'
                  ? "text-[var(--color-info-500)] border-[var(--color-info-500)]"
                  : "text-[var(--color-text-secondary)] border-transparent hover:text-[var(--color-text-primary)]"
              )}
            >
              Split View
            </button>
            <button
              onClick={() => onViewChange('viewer')}
              className={cn(
                "px-4 py-2 border-none bg-transparent cursor-pointer text-sm font-medium transition-colors border-b-2",
                activeView === 'viewer'
                  ? "text-[var(--color-info-500)] border-[var(--color-info-500)]"
                  : "text-[var(--color-text-secondary)] border-transparent hover:text-[var(--color-text-primary)]"
              )}
            >
              Diagram
            </button>
          </div>

          {/* Content */}
          <div className="flex flex-1 overflow-hidden">
            {activeView === 'editor' ? (
              <SectionErrorBoundary sectionName="Editor">
                <div className="flex flex-1 flex-col" key="editor-view">
                  <SrujaMonacoEditor
                  key={`editor-${activeView}`}
                  value={dsl}
                  onChange={onDslChange}
                  height="100%"
                  onReady={(monaco, editor) => {
                    if (monacoEditorRef.current) {
                      monacoEditorRef.current = editor;
                    }
                    editor.onDidChangeCursorSelection(() => {
                      onMonacoSelectionChange(editor);
                    });
                  }}
                />
                </div>
              </SectionErrorBoundary>
            ) : activeView === 'split' ? (
              <>
                {/* Editor Pane */}
                <SectionErrorBoundary sectionName="Split Editor">
                  <div className="w-2/5 border-r border-[var(--color-border)] flex flex-col" key="split-editor">
                    <SrujaMonacoEditor
                    key={`split-editor-${activeView}`}
                    value={dsl}
                    onChange={onDslChange}
                    height="100%"
                    onReady={(monaco, editor) => {
                      if (monacoEditorRef.current) {
                        monacoEditorRef.current = editor;
                      }
                      editor.onDidChangeCursorSelection(() => {
                        onMonacoSelectionChange(editor);
                      });
                    }}
                    />
                  </div>
                </SectionErrorBoundary>
                {/* Viewer Pane */}
                <SectionErrorBoundary sectionName="Split Viewer">
                  <div className="flex flex-1 flex-col bg-[var(--color-background)] min-w-0">
                    <ViewerPane
                    containerRef={containerRef}
                    viewerRef={viewerRef}
                    isAddingRelation={isAddingRelation}
                    sourceNode={sourceNode}
                    currentLevel={currentLevel}
                    zoomLevel={zoomLevel}
                    onAddNode={onAddNode}
                    onToggleRelation={onToggleRelation}
                    onSetLevel={onSetLevel}
                    onZoomIn={onZoomIn}
                    onZoomOut={onZoomOut}
                    onFitToScreen={onFitToScreen}
                    onToggleCollapse={onToggleCollapse}
                    onDelete={onDelete}
                    showProperties={false}
                    onCloseProperties={() => {}}
                    selectedNodeId={selectedNodeId}
                    archData={archData}
                    onPropertiesUpdate={onPropertiesUpdate}
                    onShare={onShare}
                    onUndo={onUndo}
                    onRedo={onRedo}
                    onExport={onExport}
                    onShowHelp={onShowHelp}
                  />
                  </div>
                </SectionErrorBoundary>
              </>
            ) : (
              <ViewerPane
                containerRef={containerRef}
                viewerRef={viewerRef}
                isAddingRelation={isAddingRelation}
                sourceNode={sourceNode}
                currentLevel={currentLevel}
                zoomLevel={zoomLevel}
                onAddNode={onAddNode}
                onToggleRelation={onToggleRelation}
                onSetLevel={onSetLevel}
                onZoomIn={onZoomIn}
                onZoomOut={onZoomOut}
                onFitToScreen={onFitToScreen}
                onToggleCollapse={onToggleCollapse}
                onDelete={onDelete}
                showProperties={false}
                onCloseProperties={() => {}}
                selectedNodeId={selectedNodeId}
                archData={archData}
                onPropertiesUpdate={onPropertiesUpdate}
                onShare={onShare}
                onUndo={onUndo}
                onRedo={onRedo}
                onExport={onExport}
                onShowHelp={onShowHelp}
              />
            )}
          </div>
        </div>

        {/* Right Sidebar: Properties (Collapsible) */}
        <SectionErrorBoundary sectionName="Properties Panel">
          <CollapsiblePropertiesPanel
            selectedNodeId={selectedNodeId}
            archData={archData}
            onUpdate={onPropertiesUpdate}
            defaultWidth={320}
            minWidth={200}
            maxWidth={600}
          />
        </SectionErrorBoundary>
      </div>
    </div>
  );
};

