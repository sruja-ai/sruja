import { Button } from "@sruja/ui";
import { DSLPanel } from "./DSLPanel";
import { JSONPanel } from "./JSONPanel";
import { MarkdownPanel } from "./MarkdownPanel";
import { Code, FileJson, FileText } from "lucide-react";
import "./CodePanel.css";
import { useUIStore } from "../../stores/uiStore";

export function CodePanel() {
  const activeTab = useUIStore((s) => s.codeTab);
  const setActiveTab = useUIStore((s) => s.setCodeTab);

  return (
    <div className="code-panel-container">
      <div className="code-tabs">
        <Button
          variant={activeTab === "dsl" ? "primary" : "ghost"}
          size="sm"
          className={`code-tab ${activeTab === "dsl" ? "active" : ""}`}
          onClick={() => setActiveTab("dsl")}
        >
          <Code size={16} />
          <span>Sruja DSL</span>
        </Button>
        <Button
          variant={activeTab === "json" ? "primary" : "ghost"}
          size="sm"
          className={`code-tab ${activeTab === "json" ? "active" : ""}`}
          onClick={() => setActiveTab("json")}
        >
          <FileJson size={16} />
          <span>JSON</span>
        </Button>
        <Button
          variant={activeTab === "markdown" ? "primary" : "ghost"}
          size="sm"
          className={`code-tab ${activeTab === "markdown" ? "active" : ""}`}
          onClick={() => setActiveTab("markdown")}
        >
          <FileText size={16} />
          <span>Markdown</span>
        </Button>
      </div>
      <div className="code-content">
        {activeTab === "dsl" && <DSLPanel key="dsl-panel" />}
        {activeTab === "json" && <JSONPanel key="json-panel" />}
        {activeTab === "markdown" && <MarkdownPanel key="markdown-panel" />}
      </div>
    </div>
  );
}
