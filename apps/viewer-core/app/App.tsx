import { useEffect, useState, useRef } from 'react';
import { createViewer, type ArchitectureJSON } from '@sruja/viewer';
import { FileText, ShieldCheck, GitBranch, Layout,
  ChevronRight, ChevronDown, ExternalLink
} from 'lucide-react';
import { Footer, Button, Card, Badge } from '@sruja/ui';
import { InspectorPanel } from './components/InspectorPanel';
import { OverviewPage } from './components/OverviewPage';
import { TopBar } from './components/TopBar';
import { ExportDialog, type ExportOptions } from './components/ExportDialog';
import { MarkdownPreviewModal } from './components/MarkdownPreviewModal';
import { RequirementsPage } from './pages/RequirementsPage';
import { ADRsPage } from './pages/ADRsPage';
import { ScenariosPage } from './pages/ScenariosPage';
import { SystemsPage } from './pages/SystemsPage';
import { useScenarioPlayback } from './hooks/useScenarioPlayback';
import { exportDiagram } from './utils/exportUtils';
import { initWasm, type WasmApi } from '@sruja/shared';
import LZString from 'lz-string';
import { Colors } from '@sruja/shared/utils/cssVars';
import './App.css';
import './styles/sidebar.css';
import './styles/tabs.css';
import './styles/scenarios.css';
import './styles/inspector.css';
import './index.css';

interface AppProps {
  data: ArchitectureJSON | null;
}

export default function App({ data }: AppProps) {
  const [activeTab, setActiveTab] = useState<'overview' | 'diagram' | 'requirements' | 'adrs' | 'scenarios' | 'systems'>('overview');
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const [currentLevel, setCurrentLevel] = useState(1);
  const [showInspector, setShowInspector] = useState(false);
  const [expandedSections, setExpandedSections] = useState<Set<string>>(new Set(['persons', 'systems']));
  const [breadcrumbs, setBreadcrumbs] = useState<{ id: string; label: string }[]>([]);

  const viewerRef = useRef<any>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const wasmApiRef = useRef<WasmApi | null>(null);
  const [dragEnabled, setDragEnabled] = useState(true);
  const [sidebarVisible, setSidebarVisible] = useState(true);
  const [followCamera, setFollowCamera] = useState(false);
  const [animatedSequence, setAnimatedSequence] = useState(false);
  const [exportDialogOpen, setExportDialogOpen] = useState(false);
  const [toast, setToast] = useState<{ message: string; type: 'success' | 'error' | 'info' } | null>(null);
  const [markdownPreviewOpen, setMarkdownPreviewOpen] = useState(false);
  const [markdownContent, setMarkdownContent] = useState('');
  const [markdownPreviewMode, setMarkdownPreviewMode] = useState<'preview' | 'raw'>('preview');
  const [dslForMarkdown, setDslForMarkdown] = useState<string>('');

  const updateBreadcrumbs = (id: string | null) => {
    if (!id) {
      setBreadcrumbs([]);
      return;
    }

    const arch = data?.architecture || {};
    const newBreadcrumbs: { id: string; label: string }[] = [];

    // Helper to find node and build path
    // Search Systems
    for (const sys of arch.systems || []) {
      if (sys.id === id) {
        newBreadcrumbs.push({ id: sys.id, label: sys.label || sys.id });
        setBreadcrumbs(newBreadcrumbs);
        return;
      }
      // Search Containers
      for (const cont of sys.containers || []) {
        if (cont.id === id) {
          newBreadcrumbs.push({ id: sys.id, label: sys.label || sys.id });
          newBreadcrumbs.push({ id: cont.id, label: cont.label || cont.id });
          setBreadcrumbs(newBreadcrumbs);
          return;
        }
        // Search Components
        for (const comp of cont.components || []) {
          if (comp.id === id) {
            newBreadcrumbs.push({ id: sys.id, label: sys.label || sys.id });
            newBreadcrumbs.push({ id: cont.id, label: cont.label || cont.id });
            newBreadcrumbs.push({ id: comp.id, label: comp.label || comp.id });
            setBreadcrumbs(newBreadcrumbs);
            return;
          }
        }
      }
    }

    // Fallback for other types (Reqs, ADRs) or if not found in hierarchy
    const req = arch.requirements?.find(r => r.id === id);
    if (req) {
      newBreadcrumbs.push({ id: req.id, label: req.title || req.id });
      setBreadcrumbs(newBreadcrumbs);
      return;
    }

    const adr = arch.adrs?.find(a => a.id === id);
    if (adr) {
      newBreadcrumbs.push({ id: adr.id, label: adr.title || adr.id });
      setBreadcrumbs(newBreadcrumbs);
      return;
    }

    // If still not found, maybe it's a flat ID that matches a label? 
    setBreadcrumbs([{ id, label: id }]);
  };

  const handleBreadcrumbClick = (id: string) => {
    if (id === 'root') {
      setSelectedNodeId(null);
      setShowInspector(false);
      setBreadcrumbs([]);
      viewerRef.current?.reset();
      return;
    }

    handleNodeSelect(id);
  };

  useEffect(() => {
    if (!data || !containerRef.current) return;

    const viewer = createViewer({
      container: containerRef.current,
      data: data,
      onSelect: (id: string | null) => {
        setSelectedNodeId(id);
        if (id) {
          setShowInspector(true);
          updateBreadcrumbs(id);
        } else {
          setShowInspector(false);
        }
      }
    });

    viewerRef.current = viewer;
    viewer.init();

    return () => {
      viewer.destroy();
    };
  }, [data]);

  useEffect(() => {
    const mq = window.matchMedia('(max-width: 1024px)');
    const sync = () => setSidebarVisible(!mq.matches);
    sync();
    mq.addEventListener('change', sync);
    return () => mq.removeEventListener('change', sync);
  }, []);

  // Detect standalone HTML mode (no server, no WASM files, no export functionality)
  const isStandaloneHTML = typeof window !== 'undefined' && (
    document.location.protocol === 'file:' || 
    !document.getElementById('sruja-wasm-loader') ||
    // Check if we're in a self-contained HTML (all scripts embedded)
    document.querySelector('script[id="sruja-data"]') !== null && 
    document.querySelectorAll('script[src]').length === 0
  );

  // Initialize WASM for export functionality (optional - only needed for markdown export)
  useEffect(() => {
    if (typeof window === 'undefined') return;
    
    if (isStandaloneHTML) {
      console.info('Standalone HTML export detected - Export and WASM features disabled');
      wasmApiRef.current = null;
      return;
    }
    
    // Try to initialize WASM, but don't fail if it's not available
    const base = '/';
    initWasm({ base })
      .then((api) => {
        wasmApiRef.current = api;
      })
      .catch((err) => {
        // WASM is optional - only needed for markdown export and DSL conversion
        console.warn('WASM not available (optional for HTML export):', err);
        wasmApiRef.current = null;
      });
  }, [isStandaloneHTML]);

  const handleSetLevel = (level: number) => {
    if (viewerRef.current) {
      viewerRef.current.setLevel(level);
      setCurrentLevel(level);
    }
  };

  // Apply initial level/focus from URL params
  useEffect(() => {
    const applyFromUrl = () => {
      const params = new URLSearchParams(window.location.search);
      const levelParam = params.get('level');
      const focusParam = params.get('focus');
      if (levelParam) {
        const lvl = parseInt(levelParam, 10);
        if (!isNaN(lvl)) handleSetLevel(lvl);
      }
      if (focusParam && viewerRef.current) {
        viewerRef.current.setFocus({
          // container focus if it includes dot
          containerId: focusParam.includes('.') ? focusParam : undefined,
          systemId: focusParam.includes('.') ? undefined : focusParam
        });
      }
    };
    applyFromUrl();
  }, []);

  const handleNodeSelect = (id: string) => {
    if (viewerRef.current) {
      viewerRef.current.selectNode(id);
      setSelectedNodeId(id);
      setShowInspector(true);
      updateBreadcrumbs(id);

      // If it's a requirement or ADR, switch tabs
      const arch = data?.architecture || {};
      if (arch.requirements?.some(r => r.id === id)) {
        setActiveTab('requirements');
      } else if (arch.adrs?.some(a => a.id === id)) {
        setActiveTab('adrs');
      } else {
        setActiveTab('diagram');
      }
    }
  };

  const toggleSection = (section: string) => {
    const newExpanded = new Set(expandedSections);
    if (newExpanded.has(section)) {
      newExpanded.delete(section);
    } else {
      newExpanded.add(section);
    }
    setExpandedSections(newExpanded);
  };

  const highlightNodes = (nodeIds: string[]) => {
    if (!viewerRef.current?.cyInstance) return;
    const cy = viewerRef.current.cyInstance;

    // Clear previous highlights
    cy.elements().removeClass('highlight');
    cy.elements().style('background-color', '');
    cy.elements().style('border-color', '');
    cy.elements().style('border-width', '');

    // Highlight nodes
    nodeIds.forEach(nodeId => {
      let node = cy.getElementById(nodeId);
      if (node.length === 0 && data?.architecture.systems) {
        // Try to resolve (e.g., "WebApp" -> "ECommerce.WebApp")
        for (const system of data.architecture.systems) {
          const qualifiedId = `${system.id}.${nodeId}`;
          node = cy.getElementById(qualifiedId);
          if (node.length > 0) break;
        }
      }
      if (node && node.length > 0) {
        node.addClass('highlight');
        node.style('background-color', Colors.primary50());
        node.style('border-color', Colors.primary());
        node.style('border-width', '3px');
      }
    });

    if (followCamera) {
      const highlighted = cy.elements('.highlight');
      if (highlighted.length > 0) {
        cy.animate({ center: { eles: highlighted } }, { duration: 500 });
      }
    }
  };

  const showScenarioInDiagram = (scenarioId: string) => {
    setActiveTab('diagram');
    const scenario = data?.architecture.scenarios?.find(s => s.id === scenarioId);
    if (scenario?.steps) {
      const nodeIds = new Set<string>();
      scenario.steps.forEach(step => {
        nodeIds.add(step.from);
        nodeIds.add(step.to);
      });
      highlightNodes(Array.from(nodeIds));
    }
  };

  const { isPlaying, currentScenarioId, scenarioStepIndex, playScenario, stopScenario, nextScenarioStep, prevScenarioStep } = useScenarioPlayback({ data, viewerRef, followCamera });

  // Legacy export functions for backward compatibility (used by OverviewPage)
  const exportPNG = () => {
    const uri = viewerRef.current?.exportPNG();
    if (!uri) return;
    const a = document.createElement('a');
    a.href = uri;
    a.download = `${data?.metadata?.name || 'architecture'}.png`;
    document.body.appendChild(a);
    a.click();
    a.remove();
  };

  const exportSVG = () => {
    const svg = viewerRef.current?.exportSVG();
    if (!svg) return;
    const blob = new Blob([svg], { type: 'image/svg+xml' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${data?.metadata?.name || 'architecture'}.svg`;
    document.body.appendChild(a);
    a.click();
    a.remove();
    setTimeout(() => URL.revokeObjectURL(url), 500);
  };

  const downloadJSON = () => {
    if (!data) return;
    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${data?.metadata?.name || 'architecture'}.json`;
    document.body.appendChild(a);
    a.click();
    a.remove();
    setTimeout(() => URL.revokeObjectURL(url), 500);
  };

  // Preview markdown
  const handlePreviewMarkdown = async () => {
    if (!wasmApiRef.current || !data) {
      setToast({ message: 'Markdown preview requires WASM (not available in standalone HTML)', type: 'error' });
      setTimeout(() => setToast(null), 3000);
      return;
    }
    try {
      // Convert JSON to DSL first
      const dsl = await wasmApiRef.current.printJsonToDsl(JSON.stringify(data));
      setDslForMarkdown(dsl);
      
      // Then convert DSL to markdown
      const md = await wasmApiRef.current.dslToMarkdown(dsl);
      setMarkdownContent(md);
      setMarkdownPreviewOpen(true);
    } catch (e) {
      console.error('Markdown preview error:', e);
      setToast({ message: 'Failed to generate markdown', type: 'error' });
      setTimeout(() => setToast(null), 3000);
    }
  };

  // Generate "Edit in Studio" link
  const generateEditInStudioLink = async (): Promise<string | null> => {
    if (!wasmApiRef.current || !data) return null;
    try {
      // Convert JSON to DSL
      const dsl = await wasmApiRef.current.printJsonToDsl(JSON.stringify(data));
      const base = typeof window !== 'undefined' ? window.location.origin : '';
      const studioPath = '/studio';
      const compressed = LZString.compressToBase64(dsl);
      return `${base}${studioPath}#code=${encodeURIComponent(compressed)}`;
    } catch (err) {
      console.error('Failed to generate edit link:', err);
      return null;
    }
  };

  // Handle "Edit in Studio" click
  const handleEditInStudio = async () => {
    const link = await generateEditInStudioLink();
    if (link) {
      window.open(link, '_blank');
    } else {
      setToast({ message: 'Failed to generate edit link', type: 'error' });
      setTimeout(() => setToast(null), 3000);
    }
  };

  // New export handler using ExportDialog
  const handleExport = async (options: ExportOptions) => {
    if (!viewerRef.current || !data) {
      setToast({ message: 'Viewer not ready', type: 'error' });
      setTimeout(() => setToast(null), 3000);
      return;
    }

    // For markdown export, we need DSL - try to get it from WASM
    let dsl = '';
    if (options.format === 'markdown' && wasmApiRef.current) {
      try {
        dsl = await wasmApiRef.current.printJsonToDsl(JSON.stringify(data));
      } catch (err) {
        console.error('Failed to convert JSON to DSL for markdown export:', err);
        setToast({ message: 'Failed to generate DSL for markdown export', type: 'error' });
        setTimeout(() => setToast(null), 3000);
        setExportDialogOpen(false);
        return;
      }
    }

    await exportDiagram(
      viewerRef.current,
      wasmApiRef.current,
      dsl,
      data,
      options,
      (toast) => {
        setToast(toast);
        if (toast) {
          setTimeout(() => setToast(null), 3000);
        }
      }
    );
    setExportDialogOpen(false);
  };

  const downloadHTMLCdn = () => {
    if (!data) return;
    const json = JSON.stringify(data);
    const html = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Architecture: ${(data as any).metadata?.name || 'Architecture'}</title>
  <script crossorigin src="https://unpkg.com/react@19/umd/react.production.min.js"></script>
  <script crossorigin src="https://unpkg.com/react-dom@19/umd/react-dom.production.min.js"></script>
  <script src="https://unpkg.com/cytoscape@3.33.1/dist/cytoscape.min.js"></script>
  <script src="https://unpkg.com/cytoscape-dagre@2.5.0/cytoscape-dagre.js"></script>
  <style>
    body { margin: 0; padding: 0; font-family: system-ui, sans-serif; }
    #root { width: 100%; height: calc(100vh - 40px); }
    .sruja-footer {
      position: fixed;
      bottom: 0;
      left: 0;
      right: 0;
      background: rgba(255, 255, 255, 0.95);
      border-top: 1px solid #e2e8f0;
      padding: 8px 16px;
      text-align: center;
      font-size: 12px;
      color: #64748b;
      z-index: 1000;
    }
    .sruja-footer a {
      color: #3b82f6;
      text-decoration: none;
      font-weight: 500;
    }
    .sruja-footer a:hover {
      text-decoration: underline;
    }
  </style>
</head>
<body>
  <div id="root" style="width:100%;height:calc(100vh - 40px);"></div>
  <div class="sruja-footer">
    Powered by <a href="https://sruja.ai" target="_blank" rel="noopener noreferrer">Sruja</a> - Architecture as Code
  </div>
  <script id="sruja-data" type="application/json">${json.replace(/<\//g, '<\\/')}</script>
  <script src="https://cdn.sruja.ai/v1/viewer-core.js"></script>
  <script src="https://cdn.sruja.ai/v1/viewer-app.js"></script>
</body>
</html>`;
    const blob = new Blob([html], { type: 'text/html' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${data?.metadata?.name || 'architecture'}.cdn.html`;
    document.body.appendChild(a);
    a.click();
    a.remove();
    setTimeout(() => URL.revokeObjectURL(url), 500);
  };

  useEffect(() => {
    const cy = viewerRef.current?.cyInstance;
    if (!cy) return;
    if (dragEnabled) {
      cy.nodes().grabify();
      cy.userPanningEnabled(true);
    } else {
      cy.nodes().ungrabify();
    }
  }, [dragEnabled]);

  if (!data) {
    return (
      <div className="app-container" style={{ alignItems: 'center', justifyContent: 'center' }}>
        <div className="animate-pulse" style={{ textAlign: 'center', color: 'var(--text-muted)' }}>
          <div className="spinner spinner-lg" style={{ margin: '0 auto 16px' }} />
          <p>Loading architecture data...</p>
        </div>
      </div>
    );
  }

  const arch = data.architecture || {};

  return (
    <div className="app-container animate-fade-in">
      <TopBar
        data={data}
        currentLevel={currentLevel}
        onSetLevel={handleSetLevel}
        onSearch={() => { }} // Implemented inside TopBar
        onSelectNode={handleNodeSelect}
        breadcrumbs={breadcrumbs}
        onBreadcrumbClick={handleBreadcrumbClick}
        dragEnabled={dragEnabled}
        onToggleDrag={() => setDragEnabled((v) => !v)}
        onToggleSidebar={() => setSidebarVisible((v) => !v)}
        onExport={isStandaloneHTML ? undefined : () => setExportDialogOpen(true)}
        onEditInStudio={isStandaloneHTML ? undefined : handleEditInStudio}
        onPreviewMarkdown={isStandaloneHTML ? undefined : handlePreviewMarkdown}
      />

      <div className="app-main">
        {/* Sidebar */}
        <div className={`sidebar ${sidebarVisible ? 'visible' : ''}`}>
          <div className="sidebar-header">Model Explorer</div>
          <div className="sidebar-content">
            {/* Persons */}
            {arch.persons && arch.persons.length > 0 && (
              <div className="sidebar-section">
                <div
                  className="sidebar-section-header"
                  onClick={() => toggleSection('persons')}
                >
                  {expandedSections.has('persons') ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
                  <h3>Persons</h3>
                </div>
                {expandedSections.has('persons') && arch.persons.map(p => (
                  <div
                    key={p.id}
                    className={`sidebar-item ${selectedNodeId === p.id ? 'selected' : ''}`}
                    onClick={() => handleNodeSelect(p.id)}
                  >
                    <span style={{ fontSize: 14 }}>👤</span> {p.label || p.id}
                  </div>
                ))}
              </div>
            )}

            {/* Systems */}
            {arch.systems && arch.systems.length > 0 && (
              <div className="sidebar-section">
                <div
                  className="sidebar-section-header"
                  onClick={() => toggleSection('systems')}
                >
                  {expandedSections.has('systems') ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
                  <h3>Systems</h3>
                </div>
                {expandedSections.has('systems') && arch.systems.map(s => (
                  <div key={s.id}>
                    <div
                      className={`sidebar-item ${selectedNodeId === s.id ? 'selected' : ''}`}
                      onClick={() => handleNodeSelect(s.id)}
                    >
                      <span style={{ fontSize: 14 }}>🖥️</span> {s.label || s.id}
                    </div>
                    {s.containers && s.containers.map(c => (
                      <div
                        key={`${s.id}.${c.id}`}
                        className={`sidebar-item nested ${selectedNodeId === `${s.id}.${c.id}` ? 'selected' : ''}`}
                        onClick={() => handleNodeSelect(`${s.id}.${c.id}`)}
                      >
                        <span style={{ fontSize: 14 }}>📦</span> {c.label || c.id}
                      </div>
                    ))}
                  </div>
                ))}
              </div>
            )}

            {/* Requirements */}
            {arch.requirements && arch.requirements.length > 0 && (
              <div className="sidebar-section">
                <div
                  className="sidebar-section-header"
                  onClick={() => toggleSection('requirements')}
                >
                  {expandedSections.has('requirements') ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
                  <h3>Requirements</h3>
                </div>
                {expandedSections.has('requirements') && arch.requirements.map(r => (
                  <div
                    key={r.id}
                    className="sidebar-item"
                    onClick={() => {
                      setActiveTab('requirements');
                    }}
                  >
                    <span style={{ fontSize: 14 }}>📋</span> {r.title || r.id}
                  </div>
                ))}
              </div>
            )}

            {/* ADRs */}
            {arch.adrs && arch.adrs.length > 0 && (
              <div className="sidebar-section">
                <div
                  className="sidebar-section-header"
                  onClick={() => toggleSection('adrs')}
                >
                  {expandedSections.has('adrs') ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
                  <h3>ADRs</h3>
                </div>
                {expandedSections.has('adrs') && arch.adrs.map(a => (
                  <div
                    key={a.id}
                    className="sidebar-item"
                    onClick={() => setActiveTab('adrs')}
                  >
                    <span style={{ fontSize: 14 }}>📄</span> {a.title || a.id}
                  </div>
                ))}
              </div>
            )}

            {/* Scenarios */}
            {arch.scenarios && arch.scenarios.length > 0 && (
              <div className="sidebar-section">
                <div
                  className="sidebar-section-header"
                  onClick={() => toggleSection('scenarios')}
                >
                  {expandedSections.has('scenarios') ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
                  <h3>Scenarios</h3>
                </div>
                {expandedSections.has('scenarios') && arch.scenarios.map(s => (
                  <div
                    key={s.id}
                    className="sidebar-item"
                    onClick={() => setActiveTab('scenarios')}
                  >
                    <span style={{ fontSize: 14 }}>🎬</span> {s.title || s.label || s.id}
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>

        {/* Main Content */}
        <div className="main-content">
          {/* Tabs */}
          <div className="tabs">
            <button
              className={`tab ${activeTab === 'overview' ? 'active' : ''}`}
              onClick={() => setActiveTab('overview')}
            >
              Overview
            </button>
            <button
              className={`tab ${activeTab === 'diagram' ? 'active' : ''}`}
              onClick={() => setActiveTab('diagram')}
            >
              <Layout size={16} /> Diagram
            </button>
            <button
              className={`tab ${activeTab === 'systems' ? 'active' : ''}`}
              onClick={() => setActiveTab('systems')}
            >
              <Monitor size={16} /> Systems
            </button>
            <button
              className={`tab ${activeTab === 'requirements' ? 'active' : ''}`}
              onClick={() => setActiveTab('requirements')}
            >
              <ShieldCheck size={16} /> Requirements
            </button>
            <button
              className={`tab ${activeTab === 'adrs' ? 'active' : ''}`}
              onClick={() => setActiveTab('adrs')}
            >
              <FileText size={16} /> ADRs
            </button>
            <button
              className={`tab ${activeTab === 'scenarios' ? 'active' : ''}`}
              onClick={() => setActiveTab('scenarios')}
            >
              <GitBranch size={16} /> Scenarios
            </button>
          </div>

          {/* Tab Content */}
          <div className="tab-content-area">
            {activeTab === 'overview' && (
              <OverviewPage
                data={data}
                onExportPNG={isStandaloneHTML ? undefined : exportPNG}
                onExportSVG={isStandaloneHTML ? undefined : exportSVG}
                onDownloadJSON={isStandaloneHTML ? undefined : downloadJSON}
                onDownloadHTML={isStandaloneHTML ? undefined : downloadHTMLCdn}
                isStandalone={isStandaloneHTML}
              />
            )}
            {activeTab === 'diagram' && (
              <div className="diagram-tab">
                <div ref={containerRef} id="sruja-app" className="viewer-container" />
              </div>
            )}

            {activeTab === 'systems' && (
              <SystemsPage systems={arch.systems || []} onSelect={handleNodeSelect} />
            )}

            {activeTab === 'requirements' && (
              <RequirementsPage requirements={arch.requirements || []} onHighlight={highlightNodes} />
            )}

            {activeTab === 'adrs' && (
              <ADRsPage adrs={arch.adrs || []} onHighlight={highlightNodes} />
            )}

            {activeTab === 'scenarios' && (
              <div className="scenarios-split">
                <div className="scenarios-left">
                  <ScenariosPage
                    scenarios={arch.scenarios || []}
                    onShowInDiagram={showScenarioInDiagram}
                    onPlay={playScenario}
                    onStop={stopScenario}
                    onNextStep={nextScenarioStep}
                    onPrevStep={prevScenarioStep}
                    isPlaying={isPlaying}
                    currentScenarioId={currentScenarioId}
                    stepIndex={scenarioStepIndex}
                    followCamera={followCamera}
                    onToggleFollowCamera={() => setFollowCamera(v => !v)}
                    animated={animatedSequence}
                    onToggleAnimated={() => setAnimatedSequence(v => !v)}
                  />
                </div>
                <div className="scenarios-right">
                  <div className="diagram-tab">
                    <div ref={containerRef} id="sruja-app" className="viewer-container" />
                  </div>
                </div>
              </div>
            )}
          </div>

          {/* Inspector Panel */}
          {showInspector && selectedNodeId && (
            <InspectorPanel
              nodeId={selectedNodeId}
              data={data}
              onClose={() => setShowInspector(false)}
              onSelectNode={handleNodeSelect}
              onSelectRequirement={(id) => {
                setActiveTab('requirements');
              }}
              onSelectADR={(id) => {
                setActiveTab('adrs');
              }}
            />
          )}
        </div>
      </div>

      <Footer
        leftContent={<span style={{ fontWeight: 500 }}>Architecture Viewer</span>}
        centerContent={<span style={{ color: 'var(--text-secondary)' }}>{data?.metadata?.name || 'Sruja Architecture'}</span>}
        rightContent={
          <a
            href="https://sruja.ai"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-2 px-3 py-1.5 rounded-md border border-[var(--color-border)] text-sm text-[var(--color-text-primary)] bg-[var(--color-background)] no-underline hover:bg-[var(--color-surface)] transition-colors"
            style={{ fontSize: '14px' }}
            title="Open Sruja Website"
          >
            <ExternalLink size={18} />
            <span>Website</span>
          </a>
        }
      />

        {!isStandaloneHTML && (
          <ExportDialog
            isOpen={exportDialogOpen}
            defaultFilename={data?.metadata?.name || 'architecture'}
            onExport={handleExport}
            onCancel={() => setExportDialogOpen(false)}
            wasmAvailable={!!wasmApiRef.current}
          />
        )}

      <MarkdownPreviewModal
        isOpen={markdownPreviewOpen}
        content={markdownContent}
        mode={markdownPreviewMode}
        onClose={() => setMarkdownPreviewOpen(false)}
        onModeChange={setMarkdownPreviewMode}
        onCopy={() => {
          navigator.clipboard.writeText(markdownContent);
          setToast({ message: 'Copied to clipboard', type: 'success' });
          setTimeout(() => setToast(null), 3000);
        }}
        dsl={dslForMarkdown}
      />

      {toast && (
        <div
          style={{
            position: 'fixed',
            bottom: 20,
            right: 20,
            padding: '12px 20px',
            borderRadius: '8px',
            backgroundColor: toast.type === 'error' ? Colors.error() : toast.type === 'success' ? Colors.success() : Colors.info(),
            color: 'white',
            boxShadow: '0 4px 6px -1px rgba(0, 0, 0, 0.1)',
            zIndex: 1001,
            animation: 'fadeIn 0.2s',
          }}
        >
          {toast.message}
        </div>
      )}

      {toast && (
        <style>{`
          @keyframes fadeIn {
            from { opacity: 0; transform: translateY(10px); }
            to { opacity: 1; transform: translateY(0); }
          }
        `}</style>
      )}
    </div>
  );
}
