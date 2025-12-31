import {
  Menu,
  Info,
  RefreshCw,
  Edit,
  Eye,
  Settings,
  Share2,
  Plus,
  Upload,
  Download,
  Github,
  Globe,
} from "lucide-react";
import { Breadcrumb, ExamplesDropdown } from "./shared";
import { ThemeToggle, Button, Logo } from "@sruja/ui";
import type { SrujaModelDump } from "@sruja/shared";
import type { ViewTab } from "../types";
import { getWebsiteUrl } from "../utils/website-url";
import { PersonaSwitcher, type Persona } from "./PersonaSwitcher";

export interface HeaderProps {
  isNavOpen: boolean;
  setIsNavOpen: (open: boolean) => void;
  model: SrujaModelDump | null;
  showActions: boolean;
  setShowActions: (show: boolean) => void;
  activeTab: ViewTab;
  editMode: "view" | "edit";
  setEditMode: (mode: "view" | "edit") => void;
  setShowSettings: (show: boolean) => void;
  selectedNodeId: string | null;
  isDetailsOpen: boolean;
  setIsDetailsOpen: (open: boolean) => void;
  // Persona
  selectedPersona: Persona;
  onPersonaChange: (persona: Persona) => void;
  // Actions
  handleImport: () => void;
  handleExport: () => void;
  handleExportPNG: () => Promise<void>;
  handleExportSVG: () => Promise<void>;
  reloadFromDsl: () => Promise<void>;
  handleShareHeader: () => Promise<void>;
  handleCreateNewRemote: () => Promise<void>;
}

export function Header({
  setIsNavOpen,
  model,
  showActions,
  setShowActions,
  activeTab,
  editMode,
  setEditMode,
  setShowSettings,
  selectedNodeId,
  isDetailsOpen,
  setIsDetailsOpen,
  selectedPersona,
  onPersonaChange,
  handleImport,
  handleExport,
  handleExportPNG,
  handleExportSVG,
  reloadFromDsl,
  handleShareHeader,
  handleCreateNewRemote,
}: HeaderProps) {
  return (
    <header className="app-header">
      <div className="header-left">
        <Button
          variant="ghost"
          size="sm"
          className="mobile-menu-btn"
          onClick={() => setIsNavOpen(true)}
          aria-label="Open menu"
        >
          <Menu size={20} />
        </Button>
        <div className="app-branding">
          <Logo size={24} />
          <h1 className="app-title">
            Sruja<span>Designer</span>
          </h1>
        </div>
        <Breadcrumb />
      </div>

      <div className="header-center">
        <div className="project-pill">
          <span className="project-name">
            {(model?._metadata as any)?.name || "Untitled Architecture"}
          </span>
        </div>
      </div>

      <div className="header-right">
        <PersonaSwitcher
          selectedPersona={selectedPersona}
          onPersonaChange={onPersonaChange}
          className="header-persona-switcher"
        />
        <div className="actions-dropdown-container">
          <Button
            variant="secondary"
            size="sm"
            className="action-btn"
            onClick={() => setShowActions(!showActions)}
            aria-label="Actions"
            aria-expanded={showActions}
            title="Actions & Export"
          >
            <Menu size={18} />
            <span className="btn-label">Actions</span>
          </Button>
          {showActions && (
            <div className="actions-menu">
              <Button
                variant="ghost"
                size="sm"
                className="action-item"
                onClick={() => {
                  setShowActions(false);
                  handleImport();
                }}
                aria-label="Import .sruja file"
                title="Import existing .sruja file"
              >
                <Upload size={16} />
                Import .sruja
              </Button>
              <Button
                variant="ghost"
                size="sm"
                className="action-item"
                onClick={() => {
                  setShowActions(false);
                  handleExport();
                }}
                disabled={!model}
                aria-label="Export .sruja file"
                title="Export .sruja file"
              >
                <Download size={16} />
                Export .sruja
              </Button>
              {activeTab === "diagram" && (
                <>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="action-item"
                    onClick={() => {
                      setShowActions(false);
                      void handleExportPNG();
                    }}
                    disabled={!model}
                    aria-label="Export as PNG"
                    title="Export diagram as PNG image"
                  >
                    <Download size={16} />
                    Export PNG
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="action-item"
                    onClick={() => {
                      setShowActions(false);
                      void handleExportSVG();
                    }}
                    disabled={!model}
                    aria-label="Export as SVG"
                    title="Export diagram as SVG image"
                  >
                    <Download size={16} />
                    Export SVG
                  </Button>
                </>
              )}
              <Button
                variant="ghost"
                size="sm"
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
              </Button>
              <Button
                variant="ghost"
                size="sm"
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
              </Button>
              <Button
                variant="ghost"
                size="sm"
                className="action-item"
                onClick={() => {
                  setShowActions(false);
                  void handleCreateNewRemote();
                }}
                aria-label="Create new project"
                title="Create a new empty project"
              >
                <Plus size={16} />
                New Project
              </Button>

              <div
                className="menu-divider"
                style={{ margin: "4px 0", borderTop: "1px solid var(--border-color)" }}
              />

              <a
                href={getWebsiteUrl()}
                target="_blank"
                rel="noopener noreferrer"
                className="action-item-link"
                style={{ textDecoration: "none" }}
              >
                <Button
                  variant="ghost"
                  size="sm"
                  className="action-item"
                  style={{ width: "100%", justifyContent: "flex-start" }}
                >
                  <Globe size={16} />
                  Website
                </Button>
              </a>
              <a
                href="https://github.com/sruja-ai/sruja"
                target="_blank"
                rel="noopener noreferrer"
                className="action-item-link"
                style={{ textDecoration: "none" }}
              >
                <Button
                  variant="ghost"
                  size="sm"
                  className="action-item"
                  style={{ width: "100%", justifyContent: "flex-start" }}
                >
                  <Github size={16} />
                  GitHub
                </Button>
              </a>
            </div>
          )}
        </div>

        <div className="mode-toggle-group">
          <Button
            variant={editMode === "view" ? "primary" : "ghost"}
            size="sm"
            className={`mode-btn ${editMode === "view" ? "active" : ""}`}
            onClick={() => setEditMode("view")}
            title="View Mode - Browse and explore architecture"
            aria-pressed={editMode === "view"}
          >
            <Eye size={16} />
            <span className="mode-label">View</span>
          </Button>
          <Button
            variant={editMode === "edit" ? "primary" : "ghost"}
            size="sm"
            className={`mode-btn ${editMode === "edit" ? "active" : ""}`}
            onClick={() => setEditMode("edit")}
            title="Edit Mode - Modify and design architecture"
            aria-pressed={editMode === "edit"}
          >
            <Edit size={16} />
            <span className="mode-label">Edit</span>
          </Button>
        </div>
        <ThemeToggle iconOnly />
        <ExamplesDropdown />
        <Button
          variant="ghost"
          size="sm"
          className="action-btn icon-only"
          onClick={() => setShowSettings(true)}
          title="Feature Settings"
          aria-label="Feature Settings"
        >
          <Settings size={18} />
        </Button>
        {selectedNodeId && (
          <Button
            variant="ghost"
            size="sm"
            className="mobile-menu-btn"
            onClick={() => setIsDetailsOpen(!isDetailsOpen)}
            aria-label="Toggle details"
          >
            <Info size={20} />
          </Button>
        )}
      </div>
    </header>
  );
}
