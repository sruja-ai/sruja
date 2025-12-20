/**
 * SharePanel
 * Export architecture to multiple formats and share via URL
 */

import { useState, useCallback, useEffect, useRef } from "react";
import { Share2, Link, Copy, Check, Download, FileJson, FileCode, FileText, X } from "lucide-react";
import { Button, Input } from "@sruja/ui";
import { useArchitectureStore } from "../../stores/architectureStore";
// import { convertJsonToDsl } from "../../utils/jsonToDsl"; // Removed
import { convertModelToDsl } from "../../utils/modelToDsl";
import { firebaseShareService } from "../../utils/firebaseShareService";
import LZString from "lz-string";
import "./SharePanel.css";

interface SharePanelProps {
  isOpen: boolean;
  onClose: () => void;
}

export function SharePanel({ isOpen, onClose }: SharePanelProps) {
  const data = useArchitectureStore((state) => state.likec4Model);
  const dslSource = useArchitectureStore((s) => s.dslSource);
  const modalRef = useRef<HTMLDivElement>(null);
  const closeButtonRef = useRef<HTMLButtonElement>(null);

  const [copiedType, setCopiedType] = useState<string | null>(null);

  // Focus trap and keyboard handling
  useEffect(() => {
    if (!isOpen) return;

    // Focus close button on open
    closeButtonRef.current?.focus();

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
        return;
      }

      // Focus trap
      if (e.key === "Tab" && modalRef.current) {
        const focusableElements = modalRef.current.querySelectorAll<HTMLElement>(
          'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
        );
        const firstEl = focusableElements[0];
        const lastEl = focusableElements[focusableElements.length - 1];

        if (e.shiftKey && document.activeElement === firstEl) {
          e.preventDefault();
          lastEl?.focus();
        } else if (!e.shiftKey && document.activeElement === lastEl) {
          e.preventDefault();
          firstEl?.focus();
        }
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, onClose]);

  // Generate shareable URL
  const generateShareUrl = useCallback(async () => {
    if (!dslSource && !data) return "";

    const dsl = dslSource || (data ? await convertModelToDsl(data) : "");
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

  const [dsl, setDsl] = useState<string>("");
  const [shareUrl, setShareUrl] = useState<string>("");

  useEffect(() => {
    const loadDsl = async () => {
      if (dslSource) {
        setDsl(dslSource);
      } else if (data) {
        try {
          const generatedDsl = await convertModelToDsl(data);
          setDsl(generatedDsl);
        } catch (error) {
          setDsl(`// Error generating DSL: ${error instanceof Error ? error.message : String(error)}`);
        }
      } else {
        setDsl("");
      }
    };
    void loadDsl();
  }, [dslSource, data]);

  useEffect(() => {
    const loadShareUrl = async () => {
      const url = await generateShareUrl();
      setShareUrl(url);
    };
    void loadShareUrl();
  }, [generateShareUrl]);

  if (!isOpen) return null;

  const json = data ? JSON.stringify(data, null, 2) : "";
  const archName = data?._metadata?.name || "architecture";
  const safeName = archName.toLowerCase().replace(/[^a-z0-9]+/g, "-");

  return (
    <div className="share-panel-overlay" onClick={onClose} role="presentation">
      <div
        ref={modalRef}
        className="share-panel-modal"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
        aria-labelledby="share-panel-title"
      >
        <div className="share-panel-header">
          <div className="share-panel-title">
            <Share2 size={20} aria-hidden="true" />
            <h2 id="share-panel-title">Share & Export</h2>
          </div>
          <Button
            ref={closeButtonRef}
            variant="ghost"
            size="sm"
            className="share-close-btn"
            onClick={onClose}
            aria-label="Close share panel"
          >
            <X size={18} aria-hidden="true" />
          </Button>
        </div>

        <div className="share-panel-content">
          {/* Shareable Link */}
          <div className="share-section">
            <h3>
              <Link size={16} aria-hidden="true" />
              Shareable Link
            </h3>
            <p>Anyone with this link can view your architecture</p>
            <div className="share-url-row">
              <Input
                type="text"
                readOnly
                value={
                  firebaseShareService.isProjectUrl(window.location.href)
                    ? window.location.href
                    : shareUrl
                }
                onClick={(e) => (e.target as HTMLInputElement).select()}
                aria-label="Shareable URL"
                className="share-url-input"
              />
              <Button
                className="share-copy-btn"
                onClick={() =>
                  handleCopy(
                    "url",
                    firebaseShareService.isProjectUrl(window.location.href)
                      ? window.location.href
                      : shareUrl
                  )
                }
                aria-label={copiedType === "url" ? "URL copied" : "Copy URL"}
              >
                {copiedType === "url" ? (
                  <Check size={16} aria-hidden="true" />
                ) : (
                  <Copy size={16} aria-hidden="true" />
                )}
                {copiedType === "url" ? "Copied!" : "Copy"}
              </Button>
            </div>
            {firebaseShareService.isProjectUrl(window.location.href) && (
              <p className="share-note">
                Use this link to collaborate. Changes will be saved to this project.
              </p>
            )}
          </div>

          {/* Export Options */}
          <div className="share-section">
            <h3>
              <Download size={16} aria-hidden="true" />
              Export
            </h3>
            <div className="export-grid" role="list">
              {/* DSL */}
              <div className="export-card" role="listitem">
                <div className="export-icon">
                  <FileCode size={24} aria-hidden="true" />
                </div>
                <div className="export-info">
                  <h4>Sruja DSL</h4>
                  <p>.sruja file</p>
                </div>
                <div className="export-actions">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleCopy("dsl", dsl)}
                    aria-label={copiedType === "dsl" ? "DSL copied" : "Copy DSL"}
                  >
                    {copiedType === "dsl" ? (
                      <Check size={14} aria-hidden="true" />
                    ) : (
                      <Copy size={14} aria-hidden="true" />
                    )}
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleDownload(`${safeName}.sruja`, dsl, "text/plain")}
                    aria-label="Download DSL file"
                  >
                    <Download size={14} aria-hidden="true" />
                  </Button>
                </div>
              </div>

              {/* JSON */}
              <div className="export-card" role="listitem">
                <div className="export-icon">
                  <FileJson size={24} aria-hidden="true" />
                </div>
                <div className="export-info">
                  <h4>JSON</h4>
                  <p>.json file</p>
                </div>
                <div className="export-actions">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleCopy("json", json)}
                    aria-label={copiedType === "json" ? "JSON copied" : "Copy JSON"}
                  >
                    {copiedType === "json" ? (
                      <Check size={14} aria-hidden="true" />
                    ) : (
                      <Copy size={14} aria-hidden="true" />
                    )}
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleDownload(`${safeName}.json`, json, "application/json")}
                    aria-label="Download JSON file"
                  >
                    <Download size={14} aria-hidden="true" />
                  </Button>
                </div>
              </div>

              {/* Markdown (DSL in code block) */}
              <div className="export-card" role="listitem">
                <div className="export-icon">
                  <FileText size={24} aria-hidden="true" />
                </div>
                <div className="export-info">
                  <h4>Markdown</h4>
                  <p>DSL in code block</p>
                </div>
                <div className="export-actions">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleCopy("md", `# ${archName}\n\n\`\`\`sruja\n${dsl}\n\`\`\``)}
                    aria-label={copiedType === "md" ? "Markdown copied" : "Copy Markdown"}
                  >
                    {copiedType === "md" ? (
                      <Check size={14} aria-hidden="true" />
                    ) : (
                      <Copy size={14} aria-hidden="true" />
                    )}
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() =>
                      handleDownload(
                        `${safeName}.md`,
                        `# ${archName}\n\n\`\`\`sruja\n${dsl}\n\`\`\``,
                        "text/markdown"
                      )
                    }
                    aria-label="Download Markdown file"
                  >
                    <Download size={14} aria-hidden="true" />
                  </Button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
