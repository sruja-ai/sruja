import { useMemo } from "react";
import { Eye, Download, Upload } from "lucide-react";
import type { ViewTab } from "../types";
import type { Command } from "../components/shared";

interface UseAppCommandsProps {
    activeTab: ViewTab;
    setActiveTab: (tab: ViewTab) => void;
    handleExport: () => void;
    handleImport: () => void;
    handleExportPNG: () => Promise<void>;
    handleExportSVG: () => Promise<void>;
}

export function useAppCommands({
    activeTab,
    setActiveTab,
    handleExport,
    handleImport,
    handleExportPNG,
    handleExportSVG,
}: UseAppCommandsProps) {
    return useMemo<Command[]>(() => {
        const cmds: Command[] = [
            {
                id: "tab-overview",
                label: "Go to Overview",
                description: "View architecture overview",
                icon: <Eye size={ 16} />,
            category: "navigation",
            action: () => setActiveTab("overview"),
            keywords: ["overview", "summary"],
      },
        {
            id: "tab-diagram",
            label: "Go to Diagram",
            description: "View architecture diagram",
            icon: <Eye size={ 16} />,
        category: "navigation",
        action: () => setActiveTab("diagram"),
        keywords: ["diagram", "visual", "graph"],
      },
{
    id: "tab-details",
        label: "Go to Details",
            description: "View detailed information",
                icon: <Eye size={ 16 } />,
    category: "navigation",
        action: () => setActiveTab("details"),
            keywords: ["details", "info"],
      },
{
    id: "tab-code",
        label: "Go to Code",
            description: "View architecture DSL",
                icon: <Eye size={ 16 } />,
    category: "navigation",
        action: () => setActiveTab("code"),
            keywords: ["code", "dsl"],
      },
{
    id: "tab-builder",
        label: "Go to Builder",
            description: "Guided architecture builder",
                icon: <Eye size={ 16 } />,
    category: "navigation",
        action: () => setActiveTab("builder"),
            keywords: ["builder", "wizard"],
      },
{
    id: "export-json",
        label: "Export .sruja",
            description: "Export architecture to file",
                icon: <Download size={ 16 } />,
    category: "export",
        action: handleExport,
            keywords: ["export", "save", "download"],
      },
{
    id: "import-json",
        label: "Import .sruja",
            description: "Import architecture from file",
                icon: <Upload size={ 16 } />,
    category: "actions",
        action: handleImport,
            keywords: ["import", "open", "load"],
      },
    ];

if (activeTab === "diagram") {
    cmds.push(
        {
            id: "export-png",
            label: "Export PNG",
            description: "Save diagram as PNG image",
            icon: <Download size={ 16} />,
        category: "export",
        action: () => void handleExportPNG(),
        keywords: ["export", "image", "png"],
        },
{
    id: "export-svg",
        label: "Export SVG",
            description: "Save diagram as SVG image",
                icon: <Download size={ 16 } />,
    category: "export",
        action: () => void handleExportSVG(),
            keywords: ["export", "image", "svg"],
        }
      );
    }

return cmds;
  }, [activeTab, handleExport, handleImport, handleExportPNG, handleExportSVG, setActiveTab]);
}
