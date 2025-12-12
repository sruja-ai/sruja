import { useCallback, useState, useEffect } from "react";
import { Menu, Info, RefreshCw, Edit, Eye, Settings, Share2, Play, Plus } from "lucide-react";
import { ArchitectureCanvas } from "./components/Canvas";
import {
  NavigationPanel,
  DetailsPanel,
  OverviewPanel,
  GuidedBuilderPanel,
  CodePanel,
} from "./components/Panels";
import { Breadcrumb, ExamplesDropdown } from "./components/shared";
import { ThemeToggle } from "@sruja/ui";
import {
  useArchitectureStore,
  useSelectionStore,
  useUIStore,
  useFeatureFlagsStore,
} from "./stores";
import { getAllExamples, fetchExampleDsl } from "./examples";
import { convertDslToJson } from "./wasm";
import { convertJsonToDsl } from "./utils/jsonToDsl";
import { Logo, SrujaLoader } from "@sruja/ui";
import type { ArchitectureJSON, ViewTab } from "./types";
import { useTabCounts } from "./hooks/useTabCounts";
import { DetailsView } from "./components/Views/DetailsView";
import LZString from "lz-string";
import { ViewTabs } from "./components/ViewTabs";
import { FeatureSettingsDialog } from "./components/Overview/FeatureSettingsDialog";
import { useUrlState } from "./hooks/useUrlState";

const VALID_TABS: ViewTab[] = ["overview", "diagram", "details", "code", "guided"];

export default function App() {
  // Sync URL state (level, expanded nodes) with view store - must be called early
  useUrlState();

  // Use separate selectors to avoid getSnapshot infinite loop
  // Actions are stable references, so we can use them directly
  const data = useArchitectureStore((s) => s.data);
  const loadFromDSL = useArchitectureStore((s) => s.loadFromDSL);
  const reset = useArchitectureStore((s) => s.reset);
  const selectedNodeId = useSelectionStore((s) => s.selectedNodeId);
  // const selectNode = useSelectionStore((s) => s.selectNode); // selector not used currently
  const activeTab = useUIStore((s) => s.activeTab);
  const setActiveTab = useUIStore((s) => s.setActiveTab);
  const editMode = useFeatureFlagsStore((s) => s.editMode);
  const setEditMode = useFeatureFlagsStore((s) => s.setEditMode);

  const [isLoadingFile, setIsLoadingFile] = useState(false);
  const [isNavOpen, setIsNavOpen] = useState(false);
  const [isDetailsOpen, setIsDetailsOpen] = useState(false);
  const [showSettings, setShowSettings] = useState(false);

  // Initialize activeTab from URL on mount
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const tabParam = params.get("tab");

    // Map legacy tabs to new structure
    if (tabParam === "requirements" || tabParam === "adrs") {
      setActiveTab("details");
    } else if (VALID_TABS.includes(tabParam as ViewTab)) {
      setActiveTab(tabParam as ViewTab);
    } else {
      setActiveTab("diagram");
    }
  }, [setActiveTab]);

  // Sync URL when activeTab changes
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    if (params.get("tab") !== activeTab) {
      params.set("tab", activeTab);
      const newUrl = `${window.location.pathname}?${params.toString()}`;
      window.history.replaceState({}, "", newUrl);
    }
  }, [activeTab]);

  const counts = useTabCounts(data);
  const storeDslSource = useArchitectureStore((s) => s.dslSource);

  const handleShareHeader = useCallback(async () => {
    try {
      const dsl = storeDslSource || (data ? convertJsonToDsl(data) : "");
      if (!dsl) return;
      const compressed = LZString.compressToBase64(dsl);
      const encoded = encodeURIComponent(compressed);
      const url = new URL(window.location.href);
      url.searchParams.set("share", encoded);
      url.searchParams.set("tab", activeTab);
      await navigator.clipboard.writeText(url.toString());
    } catch (err) {
      console.error("Failed to generate share URL:", err);
    }
  }, [storeDslSource, data, activeTab]);

  // Load DSL from URL on mount (if share parameter exists)
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const shareParam = params.get("share");
    const dslParam = params.get("dsl");

    if (shareParam || dslParam) {
      setIsLoadingFile(true);
      try {
        const compressed = (shareParam || dslParam) as string;
        const decompressed = LZString.decompressFromBase64(decodeURIComponent(compressed));
        if (decompressed) {
          // Load DSL from URL
          convertDslToJson(decompressed)
            .then((json) => {
              if (json) {
                loadFromDSL(json as ArchitectureJSON, decompressed, undefined);
                setActiveTab("diagram");
              } else {
                console.error("Failed to parse DSL from URL");
              }
            })
            .finally(() => {
              setIsLoadingFile(false);
            });
          return;
        }
      } catch (err) {
        console.error("Failed to load DSL from URL:", err);
        setIsLoadingFile(false);
      }
    }
  }, [loadFromDSL, setActiveTab]);

  // Load demo data - load first example from shared examples (only if no URL DSL)
  const loadDemo = useCallback(async () => {
    // Don't load demo if URL has DSL
    const params = new URLSearchParams(window.location.search);
    if (params.get("share") || params.get("dsl")) {
      return;
    }

    try {
      setIsLoadingFile(true);
      try {
        const examples = await getAllExamples();
        const firstExample = examples[0];
        if (firstExample) {
          const content = await fetchExampleDsl(firstExample.file);
          if (firstExample.isDsl) {
            const json = await convertDslToJson(content);
            if (json) {
              loadFromDSL(json as ArchitectureJSON, content, firstExample.file);
              setActiveTab("diagram");
              return;
            }
          } else {
            const parsed = JSON.parse(content) as ArchitectureJSON;
            const dsl = convertJsonToDsl(parsed);
            loadFromDSL(parsed, dsl, firstExample.file);
            setActiveTab("diagram");
            return;
          }
        }
      } catch {}
      const fallbackDsl = `architecture "Demo" {
  persons {user "User" }
  systems {web "WebApp" }
  relations {user -> web "uses" }
}`;
      const fallbackJson = await convertDslToJson(fallbackDsl);
      if (fallbackJson) {
        loadFromDSL(fallbackJson as ArchitectureJSON, fallbackDsl, "fallback");
        setActiveTab("diagram");
      }
    } catch (err) {
      console.error("Failed to load demo:", err);
    } finally {
      setIsLoadingFile(false);
    }
  }, [loadFromDSL, setActiveTab]);

  // Auto-load demo on startup
  useEffect(() => {
    loadDemo();
  }, [loadDemo]);

  // Fix flex layout for React Flow container (CSS may not be applied immediately)
  useEffect(() => {
    const fixLayout = () => {
      const app = document.querySelector(".app") as HTMLElement | null;
      const appHeader = document.querySelector(".app-header") as HTMLElement | null;
      const appMain = document.querySelector(".app-main") as HTMLElement | null;
      const centerPanel = document.querySelector(".center-panel") as HTMLElement | null;
      const canvasContainer = document.querySelector(".canvas-container") as HTMLElement | null;

      if (app) {
        app.style.display = "flex";
        app.style.flexDirection = "column";
        app.style.height = "100vh";
      }

      if (appHeader) {
        appHeader.style.display = "flex";
        appHeader.style.flexDirection = "row";
        appHeader.style.alignItems = "center";
        appHeader.style.justifyContent = "space-between";
        appHeader.style.height = "56px";
        appHeader.style.flexShrink = "0";
      }

      const headerLeft = document.querySelector(".header-left") as HTMLElement | null;
      const headerCenter = document.querySelector(".header-center") as HTMLElement | null;
      const headerRight = document.querySelector(".header-right") as HTMLElement | null;

      if (headerLeft) {
        headerLeft.style.display = "flex";
        headerLeft.style.flex = "1";
        headerLeft.style.alignItems = "center";
      }

      if (headerCenter) {
        headerCenter.style.display = "flex";
        headerCenter.style.flex = "2";
        headerCenter.style.justifyContent = "center";
      }

      if (headerRight) {
        headerRight.style.display = "flex";
        headerRight.style.alignItems = "center";
        headerRight.style.gap = "8px";
      }

      if (appMain) {
        appMain.style.display = "flex";
        appMain.style.flex = "1 1 auto";
        appMain.style.minHeight = "0";
      }

      if (centerPanel) {
        centerPanel.style.display = "flex";
        centerPanel.style.flexDirection = "column";
        centerPanel.style.flex = "1 1 auto";
        centerPanel.style.minHeight = "0";
        centerPanel.style.overflow = "hidden";
      }

      if (canvasContainer) {
        canvasContainer.style.flex = "1 1 auto";
        canvasContainer.style.minHeight = "0";
      }
    };

    // Fix immediately and after a short delay to ensure DOM is ready
    fixLayout();
    const timeout = setTimeout(fixLayout, 100);

    return () => clearTimeout(timeout);
  }, []);

  return (
    <div className="app">
      {/* Header */}
      <header className="app-header">
        <div className="header-left">
          <button
            className="mobile-menu-btn"
            onClick={() => setIsNavOpen(!isNavOpen)}
            aria-label="Toggle navigation"
          >
            <Menu size={20} />
          </button>
          <div className="app-title">
            <Logo size={24} isLoading={isLoadingFile} />
            <span>Sruja Playground</span>
          </div>
        </div>
        <div className="header-center">{data && <Breadcrumb />}</div>
        <div className="header-right">
          <button
            className="action-btn"
            onClick={() => {
              // Clear localStorage explicitly
              localStorage.removeItem("sruja-architecture-data");
              localStorage.removeItem("architecture-visualizer-feature-flags");
              // Reset store state
              reset();
              // Reload the page
              window.location.reload();
            }}
            title="Clear cache and reload"
            aria-label="Clear cache and reload"
          >
            <RefreshCw size={18} />
            <span className="btn-label">Reload</span>
          </button>

          <div className="mode-toggle-group">
            <button
              className={`mode-btn ${editMode === "view" ? "active" : ""}`}
              onClick={() => setEditMode("view")}
              title="View Mode"
            >
              <Eye size={16} />
              <span className="btn-label">View</span>
            </button>
            <button
              className={`mode-btn ${editMode === "edit" ? "active" : ""}`}
              onClick={() => setEditMode("edit")}
              title="Edit Mode"
            >
              <Edit size={16} />
              <span className="btn-label">Edit</span>
            </button>
          </div>

          <button
            className="action-btn"
            onClick={handleShareHeader}
            title="Copy shareable URL"
            aria-label="Copy shareable URL"
          >
            <Share2 size={18} />
            <span className="btn-label">Share</span>
          </button>
          <ThemeToggle iconOnly />
          <ExamplesDropdown />
          <button
            className="action-btn"
            onClick={() => setShowSettings(true)}
            title="Feature Settings"
            aria-label="Feature Settings"
          >
            <Settings size={18} />
            <span className="btn-label">Settings</span>
          </button>
          {selectedNodeId && (
            <button
              className="mobile-menu-btn"
              onClick={() => setIsDetailsOpen(!isDetailsOpen)}
              aria-label="Toggle details"
            >
              <Info size={20} />
            </button>
          )}
        </div>
      </header>

      {/* Mobile Overlay */}
      {(isNavOpen || isDetailsOpen) && (
        <div
          className="mobile-overlay"
          onClick={() => {
            setIsNavOpen(false);
            setIsDetailsOpen(false);
          }}
        />
      )}

      {/* Main Content */}
      <main className="app-main">
        <div className={`navigation-panel-wrapper ${isNavOpen ? "open" : ""}`}>
          <NavigationPanel onClose={() => setIsNavOpen(false)} />
        </div>

        <div className={`center-panel ${editMode === "edit" ? "edit-mode" : ""}`}>
          {/* View Tabs */}
          {data && <ViewTabs activeTab={activeTab} onTabChange={setActiveTab} counts={counts} />}

          {/* Tab Content */}
          <div className="canvas-container">
            {!data && !isLoadingFile && (
              <div className="drop-zone">
                <Logo size={64} />
                <h2>Welcome to Sruja Architecture Visualizer</h2>
                <p>Select an example from the dropdown or try the demo architecture</p>
                <div className="empty-state-actions">
                  <button className="demo-btn large" onClick={loadDemo}>
                    <Play size={18} />
                    Load Demo Architecture
                  </button>
                  <button
                    className="upload-btn large"
                    onClick={() => {
                      // Creating new empty architecture
                      const emptyDsl = `architecture "New System" {
  // Start designing your system
  persons { user "User" }
  systems { system "System" }
}`;
                      convertDslToJson(emptyDsl).then((json) => {
                        if (json) loadFromDSL(json as ArchitectureJSON, emptyDsl, "new");
                      });
                    }}
                  >
                    <Plus size={18} />
                    Create New
                  </button>
                </div>
              </div>
            )}
            {isLoadingFile && (
              <div className="loading">
                <SrujaLoader size={64} />
                <p>Loading architecture...</p>
              </div>
            )}
            {data && activeTab === "overview" && <OverviewPanel />}
            {data && activeTab === "diagram" && (
              <ArchitectureCanvas dragEnabled={editMode === "edit"} />
            )}
            {data && activeTab === "guided" && <GuidedBuilderPanel />}
            {data && activeTab === "details" && <DetailsView />}
            {data && activeTab === "code" && <CodePanel />}
          </div>
        </div>

        {selectedNodeId && (
          <div className={`details-panel-wrapper ${isDetailsOpen ? "open" : ""}`}>
            <DetailsPanel onClose={() => setIsDetailsOpen(false)} />
          </div>
        )}
      </main>

      <FeatureSettingsDialog isOpen={showSettings} onClose={() => setShowSettings(false)} />
    </div>
  );
}
