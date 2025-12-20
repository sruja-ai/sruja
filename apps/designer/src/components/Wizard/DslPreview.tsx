import { useMemo, useState, useCallback } from "react";
import { Code, ChevronDown, ChevronUp, Copy, Check } from "lucide-react";
import { Button } from "@sruja/ui";
import "./DslPreview.css";

interface DslPreviewProps {
  dsl: string;
}

export function DslPreview({ dsl }: DslPreviewProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(dsl);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      // Clipboard API failed - show error or fallback UI
      console.warn("Failed to copy to clipboard:", err);
      // Optionally: show a toast/notification to user
    }
  }, [dsl]);

  // Enhanced syntax highlighting for Sruja DSL
  const highlightedDsl = useMemo(() => {
    if (!dsl) return "";

    return (
      dsl
        // Top-level block keywords (advanced DSL)
        .replace(
          /\b(specification|model|views)\b/g,
          '<span class="dsl-keyword-block">$1</span>'
        )
        // Element type keywords
        .replace(
          /\b(element|person|system|container|component|database|queue|datastore)\b/g,
          '<span class="dsl-keyword">$1</span>'
        )
        // View keywords
        .replace(
          /\b(view|include|exclude|of)\b/g,
          '<span class="dsl-keyword-view">$1</span>'
        )
        // Governance keywords
        .replace(
          /\b(requirement|adr|policy|scenario|flow|constraint|convention)\b/g,
          '<span class="dsl-keyword-gov">$1</span>'
        )
        // Property keywords
        .replace(
          /\b(description|technology|tags|status|context|decision|consequences|summary|goals|nonGoals|metadata|link|title|priority|enforcement|date|author)\b/g,
          '<span class="dsl-property">$1</span>'
        )
        // ADR status values with colors
        .replace(
          /status\s+"(proposed|draft)"/g,
          'status <span class="dsl-status-proposed">"$1"</span>'
        )
        .replace(
          /status\s+"(accepted|approved)"/g,
          'status <span class="dsl-status-accepted">"$1"</span>'
        )
        .replace(
          /status\s+"(deprecated|superseded|rejected)"/g,
          'status <span class="dsl-status-deprecated">"$1"</span>'
        )
        // Requirement types
        .replace(
          /\b(functional|nonfunctional|performance|security|compliance)\b/g,
          '<span class="dsl-req-type">$1</span>'
        )
        // Strings in quotes (but not already colored)
        .replace(/"([^"]+)"/g, (match, content) => {
          // Skip if already wrapped in span
          if (match.includes("dsl-status") || match.includes("dsl-req")) return match;
          return `<span class="dsl-string">"${content}"</span>`;
        })
        // Arrows
        .replace(/->/g, '<span class="dsl-arrow">→</span>')
        // Tag arrays
        .replace(/\[([^\]]+)\]/g, '<span class="dsl-tags">[$1]</span>')
        // Comments
        .replace(/(\/\/[^\n]*)/g, '<span class="dsl-comment">$1</span>')
    );
  }, [dsl]);

  // Handle empty or invalid DSL
  if (!dsl || dsl.trim().length === 0) {
    return (
      <div className={`dsl-preview ${isExpanded ? "expanded" : ""}`}>
        <div className="dsl-preview-header">
          <div className="dsl-preview-title">
            <Code size={16} aria-hidden="true" />
            <span>DSL Preview</span>
          </div>
        </div>
        <div className="dsl-preview-code">
          <code>// No DSL content available</code>
        </div>
      </div>
    );
  }

  const previewLines = dsl.split("\n").slice(0, 5);
  const hasMore = dsl.split("\n").length > 5;
  const lineCount = dsl.split("\n").length;

  return (
    <div className={`dsl-preview ${isExpanded ? "expanded" : ""}`}>
      <div
        className="dsl-preview-header"
        onClick={() => setIsExpanded(!isExpanded)}
        role="button"
        tabIndex={0}
        onKeyDown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            setIsExpanded(!isExpanded);
          }
        }}
        aria-expanded={isExpanded}
        aria-label="Toggle DSL preview"
      >
        <div className="dsl-preview-title">
          <Code size={16} aria-hidden="true" />
          <span>DSL Preview</span>
          <span className="dsl-preview-hint">{lineCount} lines</span>
        </div>
        <div className="dsl-preview-actions">
          <Button
            variant="ghost"
            size="sm"
            className="dsl-copy-btn"
            onClick={(e) => {
              e.stopPropagation();
              handleCopy();
            }}
            aria-label={copied ? "Copied" : "Copy DSL"}
          >
            {copied ? (
              <Check size={14} aria-hidden="true" />
            ) : (
              <Copy size={14} aria-hidden="true" />
            )}
          </Button>
          {hasMore &&
            (isExpanded ? (
              <ChevronUp size={16} aria-hidden="true" />
            ) : (
              <ChevronDown size={16} aria-hidden="true" />
            ))}
        </div>
      </div>

      <pre className="dsl-preview-code">
        <code
          dangerouslySetInnerHTML={{
            __html: isExpanded
              ? highlightedDsl
              : previewLines
                  .map((line) => {
                    return line
                      .replace(
                        /\b(specification|model|views|element|system|container|component|person|database|queue)\b/g,
                        '<span class="dsl-keyword">$1</span>'
                      )
                      .replace(/"([^"]+)"/g, '<span class="dsl-string">"$1"</span>');
                  })
                  .join("\n") +
                (hasMore && !isExpanded ? '\n<span class="dsl-more">...</span>' : ""),
          }}
        />
      </pre>
    </div>
  );
}
