/**
 * SharePanel
 * Export architecture to multiple formats and share via URL
 */

import { useState, useCallback } from "react";
import { Share2, Link, Copy, Check, Download, FileJson, FileCode, FileText, X } from "lucide-react";
import { useArchitectureStore } from "../../stores/architectureStore";
import { convertJsonToDsl } from "../../utils/jsonToDsl";
import LZString from "lz-string";
import "./SharePanel.css";

interface SharePanelProps {
  isOpen: boolean;
  onClose: () => void;
}

export function SharePanel({ isOpen, onClose }: SharePanelProps) {
  const data = useArchitectureStore((s) => s.data);
  const dslSource = useArchitectureStore((s) => s.dslSource);

  const [copiedType, setCopiedType] = useState<string | null>(null);

  // Generate shareable URL
  const generateShareUrl = useCallback(() => {
    if (!dslSource && !data) return "";

    const dsl = dslSource || (data ? convertJsonToDsl(data) : "");
    const compressed = LZString.compressToBase64(dsl);
    const encoded = encodeURIComponent(compressed);
    const baseUrl = window.location.origin + window.location.pathname;
    return `${baseUrl}?share=${encoded}`;
  }, [data, dslSource]);

  const handleCopy = useCallback(async (type: string, content: string) => {
    try {
      await navigator.clipboard.writeText(content);
      setCopiedType(type);
      setTimeout(() => setCopiedType(null), 2000);
    } catch {
      // Fallback
      const textarea = document.createElement("textarea");
      textarea.value = content;
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand("copy");
      document.body.removeChild(textarea);
      setCopiedType(type);
      setTimeout(() => setCopiedType(null), 2000);
    }
  }, []);

  const handleDownload = useCallback((filename: string, content: string, mimeType: string) => {
    const blob = new Blob([content], { type: mimeType });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }, []);

  if (!isOpen) return null;

  const dsl = dslSource || (data ? convertJsonToDsl(data) : "");
  const json = data ? JSON.stringify(data, null, 2) : "";
  const shareUrl = generateShareUrl();
  const archName = data?.metadata?.name || "architecture";
  const safeName = archName.toLowerCase().replace(/[^a-z0-9]+/g, "-");

  return (
    <div className="share-panel-overlay" onClick={onClose}>
      <div className="share-panel-modal" onClick={(e) => e.stopPropagation()}>
        <div className="share-panel-header">
          <div className="share-panel-title">
            <Share2 size={20} />
            <h2>Share & Export</h2>
          </div>
          <button className="share-close-btn" onClick={onClose}>
            <X size={18} />
          </button>
        </div>

        <div className="share-panel-content">
          {/* Shareable Link */}
          <div className="share-section">
            <h3>
              <Link size={16} />
              Shareable Link
            </h3>
            <p>Anyone with this link can view your architecture</p>
            <div className="share-url-row">
              <input
                type="text"
                readOnly
                value={shareUrl}
                onClick={(e) => (e.target as HTMLInputElement).select()}
              />
              <button className="share-copy-btn" onClick={() => handleCopy("url", shareUrl)}>
                {copiedType === "url" ? <Check size={16} /> : <Copy size={16} />}
                {copiedType === "url" ? "Copied!" : "Copy"}
              </button>
            </div>
          </div>

          {/* Export Options */}
          <div className="share-section">
            <h3>
              <Download size={16} />
              Export
            </h3>
            <div className="export-grid">
              {/* DSL */}
              <div className="export-card">
                <div className="export-icon">
                  <FileCode size={24} />
                </div>
                <div className="export-info">
                  <h4>Sruja DSL</h4>
                  <p>.sruja file</p>
                </div>
                <div className="export-actions">
                  <button onClick={() => handleCopy("dsl", dsl)} title="Copy">
                    {copiedType === "dsl" ? <Check size={14} /> : <Copy size={14} />}
                  </button>
                  <button
                    onClick={() => handleDownload(`${safeName}.sruja`, dsl, "text/plain")}
                    title="Download"
                  >
                    <Download size={14} />
                  </button>
                </div>
              </div>

              {/* JSON */}
              <div className="export-card">
                <div className="export-icon">
                  <FileJson size={24} />
                </div>
                <div className="export-info">
                  <h4>JSON</h4>
                  <p>.json file</p>
                </div>
                <div className="export-actions">
                  <button onClick={() => handleCopy("json", json)} title="Copy">
                    {copiedType === "json" ? <Check size={14} /> : <Copy size={14} />}
                  </button>
                  <button
                    onClick={() => handleDownload(`${safeName}.json`, json, "application/json")}
                    title="Download"
                  >
                    <Download size={14} />
                  </button>
                </div>
              </div>

              {/* Markdown (DSL in code block) */}
              <div className="export-card">
                <div className="export-icon">
                  <FileText size={24} />
                </div>
                <div className="export-info">
                  <h4>Markdown</h4>
                  <p>DSL in code block</p>
                </div>
                <div className="export-actions">
                  <button
                    onClick={() => handleCopy("md", `# ${archName}\n\n\`\`\`sruja\n${dsl}\n\`\`\``)}
                    title="Copy"
                  >
                    {copiedType === "md" ? <Check size={14} /> : <Copy size={14} />}
                  </button>
                  <button
                    onClick={() =>
                      handleDownload(
                        `${safeName}.md`,
                        `# ${archName}\n\n\`\`\`sruja\n${dsl}\n\`\`\``,
                        "text/markdown"
                      )
                    }
                    title="Download"
                  >
                    <Download size={14} />
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
