import { useCallback, useState, useEffect, useRef, useMemo } from "react";
import {
  Menu,
  Info,
  RefreshCw,
  Edit,
  Eye,
  Settings,
  Share2,
  Play,
  Plus,
  Upload,
  Download,
} from "lucide-react";
import { ArchitectureCanvas, type ArchitectureCanvasRef } from "./components/Canvas";
import "./App.css";
import { NavigationPanel, DetailsPanel, CodePanel } from "./components/Panels";
import { BuilderWizard } from "./components/Wizard";
import {
  Breadcrumb,
  ExamplesDropdown,
  CommandPalette,
  ShortcutsModal,
  type Command,
} from "./components/shared";
import { ThemeToggle, Button } from "@sruja/ui";
import {
  useArchitectureStore,
  useSelectionStore,
  useUIStore,
  useFeatureFlagsStore,
  useHistoryStore,
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
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
import { firebaseShareService } from "./utils/firebaseShareService";
import { getFirebaseConfig } from "./config/firebase";

const VALID_TABS: ViewTab[] = ["overview", "diagram", "details", "code", "builder"];

export default function App() {
  // Sync URL state (level, expanded nodes) with view store - must be called early
  useUrlState();

  // Use separate selectors to avoid getSnapshot infinite loop
  // Actions are stable references, so we can use them directly
  const data = useArchitectureStore((s) => s.data);
  const loadFromDSL = useArchitectureStore((s) => s.loadFromDSL);
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const selectedNodeId = useSelectionStore((s) => s.selectedNodeId);
  const undo = useHistoryStore((s) => s.undo);
  const redo = useHistoryStore((s) => s.redo);
  // const selectNode = useSelectionStore((s) => s.selectNode); // selector not used currently
  const activeTab = useUIStore((s) => s.activeTab);
  const setActiveTab = useUIStore((s) => s.setActiveTab);
  const editMode = useFeatureFlagsStore((s) => s.editMode);
  const setEditMode = useFeatureFlagsStore((s) => s.setEditMode);

  const [isLoadingFile, setIsLoadingFile] = useState(false);
  const [isNavOpen, setIsNavOpen] = useState(false);
  const [isDetailsOpen, setIsDetailsOpen] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [showActions, setShowActions] = useState(false);
  const [showCommandPalette, setShowCommandPalette] = useState(false);
  const [showShortcuts, setShowShortcuts] = useState(false);
  const actionsRef = useRef<HTMLDivElement | null>(null);
  const canvasRef = useRef<ArchitectureCanvasRef>(null);

  // Initialize Firebase on mount
  useEffect(() => {
    const config = getFirebaseConfig();
    if (config) {
      firebaseShareService.initialize(config).catch((err) => {
        console.error("Failed to initialize Firebase:", err);
      });
    }
  }, []);

  // Autosave: Save to Firebase when DSL changes (debounced)
  // Note: storeDslSource and data are declared later, so this effect will be defined after they're available

  // Initialize activeTab from URL on mount
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const tabParam = params.get("tab");

    // Map legacy tabs to new structure
    if (tabParam === "requirements" || tabParam === "adrs") {
      setActiveTab("details");
    } else if (tabParam === "guided") {
      // Legacy "guided" tab renamed to "builder"
      setActiveTab("builder");
    } else if (tabParam === "overview") {
      // Overview merged into Builder/Details
      setActiveTab(editMode === "edit" ? "builder" : "diagram");
    } else if (VALID_TABS.includes(tabParam as ViewTab)) {
      setActiveTab(tabParam as ViewTab);
    } else {
      // Default: Builder in edit mode, Diagram in view mode
      setActiveTab(editMode === "edit" ? "builder" : "diagram");
    }
  }, [setActiveTab, editMode]);

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

  // Autosave: Save to Firebase when DSL changes (debounced)
  useEffect(() => {
    const { projectId } = firebaseShareService.parseUrl(window.location.href);
    const currentProjectId = firebaseShareService.getCurrentProjectId();

    // Only autosave if we're on a project URL and have data
    if (!projectId || projectId !== currentProjectId || !data || !storeDslSource) {
      return;
    }

    // Debounce saves (wait 2 seconds after last change)
    const timeoutId = setTimeout(async () => {
      try {
        const dsl = storeDslSource || convertJsonToDsl(data);
        await firebaseShareService.saveProject(dsl);
        // Silent save - no user notification
      } catch (err) {
        console.error("Autosave failed:", err);
        // Don't show error to user for autosave failures
      }
    }, 2000);

    return () => clearTimeout(timeoutId);
  }, [storeDslSource, data]);

  const handleShareHeader = useCallback(async () => {
    try {
      const dsl = storeDslSource || (data ? convertJsonToDsl(data) : "");
      if (!dsl) return;

      // Check if we're on a project URL (/designer/{projectId})
      const { projectId, keyBase64 } = firebaseShareService.parseUrl(window.location.href);
      const currentProjectId = firebaseShareService.getCurrentProjectId();

      let shareUrl: string;

      if (projectId && keyBase64 && projectId === currentProjectId) {
        // We're on an existing project URL - update it (save to Firebase)
        try {
          await firebaseShareService.saveProject(dsl);
          // URL stays the same - just copy current URL
          shareUrl = window.location.href;
        } catch (err) {
          console.error("Failed to save project:", err);
          // Fallback: create new project
          const newProject = await firebaseShareService.createNewProject();
          await firebaseShareService.saveProject(dsl);
          shareUrl = await firebaseShareService.buildShareUrl(
            newProject.projectId,
            newProject.keyBase64
          );
          // Update URL to new project
          window.history.replaceState({}, "", shareUrl);
        }
      } else {
        // Create new project
        const newProject = await firebaseShareService.createNewProject();
        await firebaseShareService.saveProject(dsl);
        shareUrl = await firebaseShareService.buildShareUrl(
          newProject.projectId,
          newProject.keyBase64
        );
        // Update URL to new project
        window.history.replaceState({}, "", shareUrl);
      }

      await navigator.clipboard.writeText(shareUrl);

      // Show warning message (per spec)
      alert(
        "Link copied to clipboard!\n\n" +
          "⚠️ Anyone with this link can view and edit.\n" +
          "⚠️ We cannot recover it if you lose it.\n\n" +
          "💡 Tip: Export your DSL to keep a personal backup."
      );
    } catch (err) {
      console.error("Failed to generate share URL:", err);
      alert("Failed to share. Please ensure Firebase is configured.");
    }
  }, [storeDslSource, data]);

  const handleExport = useCallback(() => {
    if (!data) return;
    const dsl = storeDslSource || convertJsonToDsl(data);
    const blob = new Blob([dsl], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `${data.architecture?.name || "architecture"}.sruja`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }, [data, storeDslSource]);

  const handleExportPNG = useCallback(async () => {
    if (!canvasRef.current || !data) return;
    try {
      await canvasRef.current.exportAsPNG();
    } catch (error) {
      console.error("Failed to export PNG:", error);
      alert("Failed to export PNG. Please ensure the diagram is loaded.");
    }
  }, [data]);

  const handleExportSVG = useCallback(async () => {
    if (!canvasRef.current || !data) return;
    try {
      await canvasRef.current.exportAsSVG();
    } catch (error) {
      console.error("Failed to export SVG:", error);
      alert("Failed to export SVG. Please ensure the diagram is loaded.");
    }
  }, [data]);

  const handleImport = useCallback(() => {
    fileInputRef.current?.click();
  }, []);

  const reloadFromDsl = useCallback(async () => {
    localStorage.removeItem("architecture-visualizer-feature-flags");
    const { dslSource, currentExampleFile } = useArchitectureStore.getState();
    if (dslSource) {
      try {
        const json = await convertDslToJson(dslSource);
        if (json) {
          await loadFromDSL(json as ArchitectureJSON, dslSource, currentExampleFile ?? undefined);
          setActiveTab("diagram");
        }
      } catch {}
    }
  }, [loadFromDSL, setActiveTab]);

  const handleFileChange = useCallback(
    async (event: React.ChangeEvent<HTMLInputElement>) => {
      const file = event.target.files?.[0];
      if (!file) return;

      setIsLoadingFile(true);
      try {
        const text = await file.text();
        const json = await convertDslToJson(text);
        if (json) {
          // Import creates a new project (per spec: "creates new project with new projectId + key")
          const newProject = await firebaseShareService.createNewProject();
          await firebaseShareService.saveProject(text);

          // Update URL to new project
          const shareUrl = await firebaseShareService.buildShareUrl(
            newProject.projectId,
            newProject.keyBase64
          );
          window.history.replaceState({}, "", shareUrl);

          // Load the DSL
          loadFromDSL(json as ArchitectureJSON, text, file.name);
        }
      } catch (err) {
        console.error("Failed to load file:", err);
        alert("Failed to load file. Please ensure it's a valid .sruja file.");
      } finally {
        setIsLoadingFile(false);
        if (fileInputRef.current) {
          fileInputRef.current.value = "";
        }
      }
    },
    [loadFromDSL]
  );

  const handleCreateNew = useCallback(async () => {
    try {
      // Create new empty project (per spec: initialize empty DSL template)
      const emptyDsl = `architecture "New Project" {
  persons {}
  systems {}
  relations {}
}`;

      const json = await convertDslToJson(emptyDsl);
      if (json) {
        // Create new Firebase project but don't save yet (per spec: "Do NOT write to Firebase yet")
        const newProject = await firebaseShareService.createNewProject();

        // Update URL to new project
        const shareUrl = await firebaseShareService.buildShareUrl(
          newProject.projectId,
          newProject.keyBase64
        );
        window.history.replaceState({}, "", shareUrl);

        // Load the empty DSL (will be saved on first change via autosave)
        loadFromDSL(json as ArchitectureJSON, emptyDsl, undefined);
        setActiveTab("diagram");
      }
    } catch (err) {
      console.error("Failed to create new project:", err);
      alert("Failed to create new project. Please try again.");
    }
  }, [loadFromDSL, setActiveTab]);

  // Load project from URL on mount (if /designer/{projectId}#k={key} format)
  // Uses real-time sync so multiple users can collaborate
  useEffect(() => {
    const { projectId, keyBase64 } = firebaseShareService.parseUrl(window.location.href);
    const dslParam = new URLSearchParams(window.location.search).get("dsl"); // Legacy param
    const codeParam = new URLSearchParams(window.location.search).get("code"); // Legacy param

    // Handle new encrypted project format: /designer/{projectId}#k={key}
    // Use real-time listener for automatic sync across multiple users
    if (projectId && keyBase64) {
      setIsLoadingFile(true);

      // Track if we've done the initial load to prevent showing loading state on every update
      let hasInitialLoad = false;
      let lastReceivedDsl: string | null = null;

      // Set up real-time listener
      const unsubscribe = firebaseShareService.loadProjectRealtime(
        projectId,
        keyBase64,
        async (dsl: string) => {
          try {
            // Skip if this is the same DSL we just received (avoid unnecessary re-renders)
            if (lastReceivedDsl === dsl) {
              return;
            }
            lastReceivedDsl = dsl;

            const json = await convertDslToJson(dsl);
            if (json) {
              // Load the DSL (will update store)
              loadFromDSL(json as ArchitectureJSON, dsl, undefined);
              if (!hasInitialLoad) {
                setActiveTab("diagram");
                setIsLoadingFile(false);
                hasInitialLoad = true;
              }
            } else {
              if (!hasInitialLoad) {
                console.error("Failed to parse DSL from project");
                alert("Failed to parse project. The link may be corrupted.");
                setIsLoadingFile(false);
              }
            }
          } catch (err) {
            if (!hasInitialLoad) {
              console.error("Failed to load project from URL:", err);
              const errorMessage = err instanceof Error ? err.message : "Unknown error";
              if (errorMessage.includes("Cannot decrypt")) {
                alert("Cannot decrypt project. Invalid key or corrupted data.");
              } else if (errorMessage.includes("not found")) {
                alert("Project not found. It may have been deleted or the link is invalid.");
              } else {
                alert(`Failed to load project: ${errorMessage}`);
              }
              setIsLoadingFile(false);
            }
          }
        }
      );

      // Cleanup listener on unmount or when project changes
      return () => {
        unsubscribe();
      };
    }

    // Handle legacy code=<compressed> format (direct compressed DSL sharing)
    if (codeParam) {
      setIsLoadingFile(true);
      try {
        const decompressed = LZString.decompressFromBase64(decodeURIComponent(codeParam));
        if (decompressed) {
          convertDslToJson(decompressed)
            .then((json) => {
              if (json) {
                loadFromDSL(json as ArchitectureJSON, decompressed, undefined);
                setActiveTab("diagram");
              } else {
                console.error("Failed to parse DSL from code");
              }
            })
            .finally(() => {
              setIsLoadingFile(false);
            });
          return;
        }
      } catch (err) {
        console.error("Failed to load code from URL:", err);
        setIsLoadingFile(false);
        return;
      }
    }

    // Handle dsl param (legacy)
    if (dslParam) {
      setIsLoadingFile(true);
      try {
        const decompressed = LZString.decompressFromBase64(decodeURIComponent(dslParam));
        if (decompressed) {
          convertDslToJson(decompressed)
            .then((json) => {
              if (json) {
                loadFromDSL(json as ArchitectureJSON, decompressed, undefined);
                setActiveTab("diagram");
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
  }, [loadFromDSL, setActiveTab, storeDslSource]);

  // Load demo data - load first example from shared examples (only if no URL DSL/project)
  const loadDemo = useCallback(async () => {
    // Don't load demo if URL has project, DSL, or legacy share params
    const { projectId } = firebaseShareService.parseUrl(window.location.href);
    const params = new URLSearchParams(window.location.search);
    if (projectId || params.get("share") || params.get("dsl") || params.get("code")) {
      return;
    }

    try {
      setIsLoadingFile(true);
      try {
        const examples = await getAllExamples();

        // Check for specific example in URL
        const exampleParam = params.get("example");
        let targetExample = examples[0];

        if (exampleParam) {
          const found = examples.find((ex) => ex.file === exampleParam);
          if (found) {
            targetExample = found;
          }
        }

        if (targetExample) {
          const content = await fetchExampleDsl(targetExample.file);
          if (targetExample.isDsl) {
            const json = await convertDslToJson(content);
            if (json) {
              loadFromDSL(json as ArchitectureJSON, content, targetExample.file);
              setActiveTab("diagram");
              return;
            }
          } else {
            const parsed = JSON.parse(content) as ArchitectureJSON;
            const dsl = convertJsonToDsl(parsed);
            loadFromDSL(parsed, dsl, targetExample.file);
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

  // Keyboard shortcuts
  useKeyboardShortcuts([
    {
      key: "s",
      ctrlKey: true,
      action: useCallback(() => {
        if (activeTab === "diagram" && canvasRef.current && data) {
          void handleExportPNG();
        } else if (data) {
          handleExport();
        }
      }, [activeTab, data, handleExportPNG, handleExport]),
      description: "Save/Export",
    },
    {
      key: "o",
      ctrlKey: true,
      action: handleImport,
      description: "Open/Import",
    },
    {
      key: "z",
      ctrlKey: true,
      shiftKey: false,
      action: useCallback(() => {
        const previousState = undo();
        if (previousState && data) {
          void updateArchitecture(() => previousState);
        }
      }, [undo, data, updateArchitecture]),
      description: "Undo",
    },
    {
      key: "z",
      ctrlKey: true,
      shiftKey: true,
      action: useCallback(() => {
        const nextState = redo();
        if (nextState && data) {
          void updateArchitecture(() => nextState);
        }
      }, [redo, data, updateArchitecture]),
      description: "Redo",
    },
    {
      key: "y",
      ctrlKey: true,
      action: useCallback(() => {
        // Cmd+Y for redo (Windows/Linux convention)
        const nextState = redo();
        if (nextState && data) {
          void updateArchitecture(() => nextState);
        }
      }, [redo, data, updateArchitecture]),
      description: "Redo (Alt)",
    },
    {
      key: "k",
      ctrlKey: true,
      action: useCallback(() => {
        setShowCommandPalette(true);
      }, []),
      description: "Command Palette",
    },
    {
      key: "?",
      action: useCallback(() => {
        setShowShortcuts(true);
      }, []),
      description: "Show Keyboard Shortcuts",
    },
    {
      key: "Escape",
      action: useCallback(() => {
        setShowActions(false);
        setShowSettings(false);
      }, []),
      description: "Close dialogs",
      preventDefault: false,
    },
  ]);

  // Build command palette commands
  const commandPaletteCommands: Command[] = useMemo(() => {
    const cmds: Command[] = [
      {
        id: "tab-overview",
        label: "Go to Overview",
        description: "View architecture overview",
        icon: <Eye size={16} />,
        category: "navigation",
        action: () => setActiveTab("overview"),
        keywords: ["overview", "summary"],
      },
      {
        id: "tab-diagram",
        label: "Go to Diagram",
        description: "View architecture diagram",
        icon: <Eye size={16} />,
        category: "navigation",
        action: () => setActiveTab("diagram"),
        keywords: ["diagram", "visual", "graph"],
      },
      {
        id: "tab-details",
        label: "Go to Details",
        description: "View detailed information",
        icon: <Eye size={16} />,
        category: "navigation",
        action: () => setActiveTab("details"),
        keywords: ["details", "info"],
      },
      {
        id: "tab-code",
        label: "Go to Code",
        description: "View DSL code",
        icon: <Download size={16} />,
        category: "navigation",
        action: () => setActiveTab("code"),
        keywords: ["code", "dsl", "source"],
      },
      {
        id: "tab-builder",
        label: "Go to Builder",
        description: "Step-by-step architecture builder",
        icon: <Edit size={16} />,
        category: "navigation",
        action: () => setActiveTab("builder"),
        keywords: ["builder", "wizard", "guide"],
      },
      {
        id: "import",
        label: "Import File",
        description: "Import .sruja file",
        icon: <Upload size={16} />,
        category: "actions",
        action: handleImport,
        keywords: ["import", "open", "load", "file"],
      },
      {
        id: "export-sruja",
        label: "Export .sruja",
        description: "Export as .sruja file",
        icon: <Download size={16} />,
        category: "export",
        action: handleExport,
        keywords: ["export", "save", "sruja", "download"],
      },
      {
        id: "export-png",
        label: "Export PNG",
        description: "Export diagram as PNG image",
        icon: <Download size={16} />,
        category: "export",
        action: () => {
          if (activeTab === "diagram") {
            void handleExportPNG();
          } else {
            setActiveTab("diagram");
            setTimeout(() => void handleExportPNG(), 100);
          }
        },
        keywords: ["export", "png", "image", "picture"],
      },
      {
        id: "export-svg",
        label: "Export SVG",
        description: "Export diagram as SVG image",
        icon: <Download size={16} />,
        category: "export",
        action: () => {
          if (activeTab === "diagram") {
            void handleExportSVG();
          } else {
            setActiveTab("diagram");
            setTimeout(() => void handleExportSVG(), 100);
          }
        },
        keywords: ["export", "svg", "vector"],
      },
      {
        id: "share",
        label: "Share",
        description: "Copy shareable URL",
        icon: <Share2 size={16} />,
        category: "actions",
        action: () => void handleShareHeader(),
        keywords: ["share", "url", "link", "copy"],
      },
      {
        id: "settings",
        label: "Settings",
        description: "Open settings",
        icon: <Settings size={16} />,
        category: "settings",
        action: () => setShowSettings(true),
        keywords: ["settings", "preferences", "config"],
      },
    ];

    return cmds;
  }, [
    setActiveTab,
    handleImport,
    handleExport,
    handleExportPNG,
    handleExportSVG,
    handleShareHeader,
    activeTab,
  ]);

  // Close Actions menu on outside click
  useEffect(() => {
    const onDocClick = (e: MouseEvent) => {
      if (!actionsRef.current) return;
      const target = e.target as Node;
      if (showActions && target && !actionsRef.current.contains(target)) {
        setShowActions(false);
      }
    };
    document.addEventListener("click", onDocClick, true);
    return () => document.removeEventListener("click", onDocClick, true);
  }, [showActions]);

  // Fix flex layout for React Flow container (CSS may not be applied immediately)
  useEffect(() => {
    const fixLayout = () => {
      const app = document.querySelector(".app") as HTMLElement | null;
      const appMain = document.querySelector(".app-main") as HTMLElement | null;
      const centerPanel = document.querySelector(".center-panel") as HTMLElement | null;
      const canvasContainer = document.querySelector(".canvas-container") as HTMLElement | null;

      if (app) {
        app.style.display = "flex";
        app.style.flexDirection = "column";
        app.style.height = "100vh";
      }

      // Respect CSS-driven header layout to keep responsiveness intact

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
            <span>Sruja Designer</span>
          </div>
        </div>
        <div className="header-center">{data && <Breadcrumb />}</div>
        <div className="header-right">
          <input
            ref={fileInputRef}
            type="file"
            accept=".sruja"
            onChange={handleFileChange}
            style={{ display: "none" }}
            aria-label="Import file"
          />
          <div ref={actionsRef} style={{ position: "relative", display: "inline-block" }}>
            <Button
              variant="ghost"
              size="md"
              onClick={() => setShowActions((v) => !v)}
              aria-label="Actions"
              title="Actions"
              style={{ paddingLeft: "10px", paddingRight: "10px", minWidth: "36px" }}
            >
              Actions
            </Button>
            {showActions && (
              <div
                className="actions-menu"
                style={{
                  position: "absolute",
                  top: "calc(100% + 4px)",
                  right: 0,
                  width: 240,
                  zIndex: 10000,
                }}
              >
                <button
                  className="action-item"
                  onClick={() => {
                    setShowActions(false);
                    handleImport();
                  }}
                  aria-label="Import .sruja file"
                  title="Import .sruja file"
                >
                  <Upload size={16} />
                  Import .sruja
                </button>
                <button
                  className="action-item"
                  onClick={() => {
                    setShowActions(false);
                    handleExport();
                  }}
                  disabled={!data}
                  aria-label="Export .sruja file"
                  title="Export .sruja file"
                >
                  <Download size={16} />
                  Export .sruja
                </button>
                {activeTab === "diagram" && (
                  <>
                    <button
                      className="action-item"
                      onClick={() => {
                        setShowActions(false);
                        void handleExportPNG();
                      }}
                      disabled={!data}
                      aria-label="Export as PNG"
                      title="Export diagram as PNG image"
                    >
                      <Download size={16} />
                      Export PNG
                    </button>
                    <button
                      className="action-item"
                      onClick={() => {
                        setShowActions(false);
                        void handleExportSVG();
                      }}
                      disabled={!data}
                      aria-label="Export as SVG"
                      title="Export diagram as SVG image"
                    >
                      <Download size={16} />
                      Export SVG
                    </button>
                  </>
                )}
                <button
                  className="action-item"
                  onClick={() => {
                    setShowActions(false);
                    void reloadFromDsl();
                  }}
                  aria-label="Refresh from source"
                  title="Refresh from source"
                >
                  <RefreshCw size={16} />
                  Refresh from source
                </button>
                <button
                  className="action-item"
                  onClick={() => {
                    setShowActions(false);
                    void handleShareHeader();
                  }}
                  aria-label="Copy shareable URL"
                  title="Copy shareable URL - Anyone with the link can view and edit. We cannot recover it if you lose it."
                >
                  <Share2 size={16} />
                  Share
                </button>
                <button
                  className="action-item"
                  onClick={() => {
                    setShowActions(false);
                    void handleCreateNew();
                  }}
                  aria-label="Create new project"
                  title="Create a new empty project"
                >
                  <Plus size={16} />
                  New Project
                </button>
              </div>
            )}
          </div>

          <div className="mode-toggle-group">
            <button
              className={`mode-btn ${editMode === "view" ? "active" : ""}`}
              onClick={() => setEditMode("view")}
              title="View Mode - Browse and explore architecture"
              aria-pressed={editMode === "view"}
            >
              <Eye size={16} />
              <span className="mode-label">View</span>
            </button>
            <button
              className={`mode-btn ${editMode === "edit" ? "active" : ""}`}
              onClick={() => setEditMode("edit")}
              title="Edit Mode - Modify and design architecture"
              aria-pressed={editMode === "edit"}
            >
              <Edit size={16} />
              <span className="mode-label">Edit</span>
            </button>
          </div>
          <ThemeToggle iconOnly />
          <ExamplesDropdown />
          <button
            className="action-btn icon-only"
            onClick={() => setShowSettings(true)}
            title="Feature Settings"
            aria-label="Feature Settings"
          >
            <Settings size={18} />
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
                <h2>Design, visualize, and govern your architecture</h2>
                <p>
                  Start from a demo or template, or import a <code>.sruja</code> file.
                </p>
                <div className="empty-state-actions">
                  <button className="demo-btn large" onClick={loadDemo}>
                    <Play size={18} />
                    Try a Demo
                  </button>
                  <button
                    className="upload-btn large"
                    onClick={() => {
                      // Creating new empty architecture
                      const emptyDsl = `architecture "New System" {
  persons { user "User" }
  systems { web "WebApp" }
  relations { user -> web "uses" }
}`;
                      const emptyJson: ArchitectureJSON = {
                        metadata: {
                          name: "New System",
                          version: "1.0.0",
                          generated: new Date().toISOString(),
                        },
                        architecture: {
                          persons: [{ id: "user", label: "User" }],
                          systems: [{ id: "web", label: "WebApp", containers: [] }],
                          relations: [{ from: "user", to: "web", verb: "uses" }],
                        },
                        navigation: {
                          levels: ["L1", "L2", "L3"],
                          scenarios: [],
                          flows: [],
                        },
                      };
                      loadFromDSL(emptyJson, emptyDsl, "new");
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
            {/* Overview tab removed - content consolidated into Builder and Details */}
            {data && activeTab === "diagram" && (
              <div role="tabpanel" id="tabpanel-diagram" aria-labelledby="tab-diagram">
                <ArchitectureCanvas ref={canvasRef} dragEnabled={editMode === "edit"} />
              </div>
            )}
            {data && activeTab === "builder" && (
              <div role="tabpanel" id="tabpanel-builder" aria-labelledby="tab-builder">
                <BuilderWizard />
              </div>
            )}
            {data && activeTab === "details" && (
              <div role="tabpanel" id="tabpanel-details" aria-labelledby="tab-details">
                <DetailsView />
              </div>
            )}
            {data && activeTab === "code" && (
              <div role="tabpanel" id="tabpanel-code" aria-labelledby="tab-code">
                <CodePanel />
              </div>
            )}
          </div>
        </div>

        {selectedNodeId && (
          <div className={`details-panel-wrapper ${isDetailsOpen ? "open" : ""}`}>
            <DetailsPanel onClose={() => setIsDetailsOpen(false)} />
          </div>
        )}
      </main>

      <FeatureSettingsDialog isOpen={showSettings} onClose={() => setShowSettings(false)} />

      {/* Command Palette */}
      <CommandPalette
        isOpen={showCommandPalette}
        onClose={() => setShowCommandPalette(false)}
        commands={commandPaletteCommands}
      />

      {/* Shortcuts Modal */}
      <ShortcutsModal
        isOpen={showShortcuts}
        onClose={() => setShowShortcuts(false)}
        shortcuts={[
          { keys: ["Ctrl", "S"], description: "Save/Export", category: "actions" },
          { keys: ["Ctrl", "O"], description: "Open/Import file", category: "actions" },
          { keys: ["Ctrl", "Z"], description: "Undo", category: "editing" },
          { keys: ["Ctrl", "Shift", "Z"], description: "Redo", category: "editing" },
          { keys: ["Ctrl", "Y"], description: "Redo (Alt)", category: "editing" },
          { keys: ["Ctrl", "K"], description: "Command Palette", category: "navigation" },
          { keys: ["?"], description: "Show Keyboard Shortcuts", category: "general" },
          { keys: ["Esc"], description: "Close dialogs/menus", category: "general" },
        ]}
      />
    </div>
  );
}
