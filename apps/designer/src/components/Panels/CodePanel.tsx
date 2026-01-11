import { DSLPanel } from "./DSLPanel";
import "./CodePanel.css";

/**
 * CodePanel - Shows only the DSL editor.
 * Markdown is now exposed as a separate "Docs" tab.
 */
export function CodePanel() {
  return (
    <div className="code-panel-container">
      <DSLPanel key="dsl-panel" />
    </div>
  );
}
