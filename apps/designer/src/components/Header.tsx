import {
  Menu,
  PanelLeft,
  PanelRight,
  MoreHorizontal,
  Share2,
  Plus,
  Upload,
  Download,
  Image,
  Code,
  Eye,
  Edit3,
  Hammer,
  Workflow,
  FileCode,
  FileText,
} from "lucide-react";
import { ExamplesDropdown } from "./shared";
import { ThemeToggle, Button, Logo } from "@sruja/ui";
import type { SrujaModelDump } from "@sruja/shared";
import type { ViewTab } from "../types";
import { useUIStore } from "../stores";
import { ViewTabs } from "./ViewTabs";
import "./Header.css";

export interface HeaderProps {
  isNavOpen: boolean;
  setIsNavOpen: (open: boolean) => void;
  model: SrujaModelDump | null;
  showActions: boolean;
  setShowActions: (show: boolean) => void;
  activeTab: ViewTab;
  setActiveTab: (tab: ViewTab) => void;
  editMode: "view" | "edit";
  setEditMode: (mode: "view" | "edit") => void;
  selectedNodeId: string | null;
  isDetailsOpen: boolean;
  setIsDetailsOpen: (open: boolean) => void;
  handleImport: () => void;
  handleExport: () => void;
  handleExportPNG: () => Promise<void>;
  handleExportSVG: () => Promise<void>;
  reloadFromDsl: () => Promise<void>;
  handleShareHeader: () => Promise<void>;
  handleCreateNewRemote: () => Promise<void>;
  onOpenCommandPalette: () => void;
}

export function Header({
  setIsNavOpen,
  model,
  showActions,
  setShowActions,
  activeTab,
  setActiveTab,
  editMode,
  setEditMode,
  handleImport,
  handleExport,
  handleExportPNG,
  handleExportSVG,
  reloadFromDsl,
  handleShareHeader,
  handleCreateNewRemote,
  onOpenCommandPalette,
}: HeaderProps) {
  return (
    <header className="app-header">
      {/* Left: Logo & Nav Toggle */}
      <div className="header-left">
        <div className="layout-controls">
          <Button
            variant="ghost"
            size="sm"
            className="action-btn icon-only"
            onClick={(e) => {
              e.preventDefault();
              e.stopPropagation();
              useUIStore.getState().toggleNavigation();
            }}
            title={
              useUIStore.getState().isNavigationVisible
                ? "Close Navigation (Alt/Opt + B)"
                : "Open Navigation (Alt/Opt + B)"
            }
            type="button"
          >
            <PanelLeft size={18} />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            className="mobile-menu-btn"
            onClick={(e) => {
              e.preventDefault();
              e.stopPropagation();
              setIsNavOpen(true);
            }}
            type="button"
          >
            <Menu size={20} />
          </Button>
        </div>
        <div className="logo-section">
          <Logo size={22} />
          <span className="app-title">Sruja</span>
        </div>
        <div className="divider-vertical" />
        <ExamplesDropdown />
      </div>

      {/* Center: Navigation Tabs */}
      <div className="header-center">
        {model && (
          <div
            className="header-tabs-container"
            style={{
              display: "flex",
              gap: "24px",
              alignItems: "center",
              width: "100%",
              justifyContent: "center",
            }}
          >
            {/* Editor Controls (Left) */}
            <div
              className="tab-group editor-group"
              style={{ display: "flex", gap: "8px", alignItems: "center" }}
            >
              <ViewTabs
                activeId={useUIStore((s) => s.activeEditor)}
                onTabChange={(id) =>
                  useUIStore
                    .getState()
                    .setActiveEditor(id === useUIStore.getState().activeEditor ? null : (id as any))
                }
                tabs={[
                  { id: "builder", icon: <Hammer size={16} />, label: "Builder" },
                  { id: "code", icon: <FileCode size={16} />, label: "Code" },
                ]}
              />
            </div>

            <div className="divider-vertical" style={{ height: "20px", margin: "0 8px" }} />

            {/* View Controls (Right) */}
            <div
              className="tab-group view-group"
              style={{ display: "flex", gap: "8px", alignItems: "center" }}
            >
              <ViewTabs
                activeId={useUIStore((s) => s.activeView)}
                onTabChange={(id) => useUIStore.getState().setActiveView(id as any)}
                tabs={[
                  { id: "diagram", icon: <Workflow size={16} />, label: "Diagram" },
                  { id: "docs", icon: <FileText size={16} />, label: "Docs" },
                ]}
              />
            </div>
          </div>
        )}
      </div>

      {/* Right: Actions & Tools */}
      <div className="header-right">
        {/* Search */}

        {/* Mode Toggle */}
        {activeTab !== "overview" && model && (
          <div className="mode-toggle-group">
            <button
              className={`mode-btn ${editMode === "view" ? "active" : ""}`}
              onClick={() => setEditMode("view")}
              title="View Mode"
            >
              <Eye size={15} />
            </button>
            <button
              className={`mode-btn ${editMode === "edit" ? "active" : ""}`}
              onClick={() => setEditMode("edit")}
              title="Edit Mode"
            >
              <Edit3 size={15} />
            </button>
          </div>
        )}

        {/* Primary Share Action */}
        <Button
          variant="ghost"
          size="sm"
          className="action-btn icon-only"
          onClick={handleShareHeader}
          title="Share Project"
        >
          <Share2 size={16} />
        </Button>

        {/* More Menu */}
        <div className="actions-dropdown-wrapper">
          <Button
            variant="ghost"
            size="sm"
            className="action-btn icon-only"
            onClick={() => setShowActions(!showActions)}
            title="More Actions"
          >
            <MoreHorizontal size={18} />
          </Button>

          {showActions && (
            <div className="actions-menu">
              <Button
                variant="ghost"
                className="menu-item"
                onClick={() => {
                  handleCreateNewRemote();
                  setShowActions(false);
                }}
              >
                <Plus size={16} /> New Project
              </Button>
              <Button
                variant="ghost"
                className="menu-item"
                onClick={() => {
                  handleImport();
                  setShowActions(false);
                }}
              >
                <Upload size={16} /> Import
              </Button>
              <div className="menu-divider" />

              <Button
                variant="ghost"
                className="menu-item"
                onClick={() => {
                  handleExport();
                  setShowActions(false);
                }}
              >
                <Download size={16} /> Export DSL
              </Button>
              <Button
                variant="ghost"
                className="menu-item"
                onClick={() => {
                  handleExportPNG();
                  setShowActions(false);
                }}
              >
                <Image size={16} /> Export PNG
              </Button>
              <Button
                variant="ghost"
                className="menu-item"
                onClick={() => {
                  handleExportSVG();
                  setShowActions(false);
                }}
              >
                <Code size={16} /> Export SVG
              </Button>
            </div>
          )}
        </div>

        <div className="divider-vertical" />

        <ThemeToggle iconOnly />

        <Button
          variant="ghost"
          size="sm"
          className="action-btn icon-only"
          onClick={(e) => {
            e.preventDefault();
            e.stopPropagation();
            useUIStore.getState().toggleInspector();
          }}
          title={
            useUIStore.getState().isInspectorVisible
              ? "Close Inspector (Alt/Opt + I)"
              : "Open Inspector (Alt/Opt + I)"
          }
          type="button"
        >
          <PanelRight size={18} />
        </Button>
      </div>
    </header>
  );
}
