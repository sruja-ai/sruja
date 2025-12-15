import { useMemo, useState, useCallback } from "react";
import { Code, ChevronDown, ChevronUp, Copy, Check } from "lucide-react";
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
    } catch {
      // Fallback for older browsers
      const textarea = document.createElement("textarea");
      textarea.value = dsl;
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand("copy");
      document.body.removeChild(textarea);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  }, [dsl]);

  // Enhanced syntax highlighting for Sruja DSL
  const highlightedDsl = useMemo(() => {
    if (!dsl) return "";

    return (
      dsl
        // Block keywords (structure)
        .replace(
          /\b(architecture|system|container|component|person|datastore|queue)\b/g,
          '<span class="dsl-keyword">$1</span>'
        )
        // Governance keywords
        .replace(
          /\b(requirement|adr|policy|scenario|flow)\b/g,
          '<span class="dsl-keyword-gov">$1</span>'
        )
        // Property keywords
        .replace(
          /\b(description|tech|label|verb|interaction|tags|status|context|decision|consequences|summary|goals|nonGoals)\b/g,
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

  const previewLines = dsl.split("\n").slice(0, 5);
  const hasMore = dsl.split("\n").length > 5;
  const lineCount = dsl.split("\n").length;

  return (
    <div className={`dsl-preview ${isExpanded ? "expanded" : ""}`}>
      <button className="dsl-preview-header" onClick={() => setIsExpanded(!isExpanded)}>
        <div className="dsl-preview-title">
          <Code size={16} />
          <span>DSL Preview</span>
          <span className="dsl-preview-hint">{lineCount} lines</span>
        </div>
        <div className="dsl-preview-actions">
          <button
            className="dsl-copy-btn"
            onClick={(e) => {
              e.stopPropagation();
              handleCopy();
            }}
            title="Copy DSL"
          >
            {copied ? <Check size={14} /> : <Copy size={14} />}
          </button>
          {hasMore && (isExpanded ? <ChevronUp size={16} /> : <ChevronDown size={16} />)}
        </div>
      </button>

      <pre className="dsl-preview-code">
        <code
          dangerouslySetInnerHTML={{
            __html: isExpanded
              ? highlightedDsl
              : previewLines
                  .map((line) => {
                    return line
                      .replace(
                        /\b(architecture|system|container|component|person)\b/g,
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
