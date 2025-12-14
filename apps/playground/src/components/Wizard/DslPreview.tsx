import { useMemo } from "react";
import { Code, ChevronDown, ChevronUp } from "lucide-react";
import { useState } from "react";
import "./DslPreview.css";

interface DslPreviewProps {
  dsl: string;
}

export function DslPreview({ dsl }: DslPreviewProps) {
  const [isExpanded, setIsExpanded] = useState(false);

  // Simple syntax highlighting for Sruja DSL
  const highlightedDsl = useMemo(() => {
    if (!dsl) return "";

    return (
      dsl
        // Keywords
        .replace(
          /\b(architecture|persons|systems|containers|components|relations|requirements|goals|overview|flows|scenarios|adrs|policies)\b/g,
          '<span class="dsl-keyword">$1</span>'
        )
        // Strings in quotes
        .replace(/"([^"]+)"/g, '<span class="dsl-string">"$1"</span>')
        // Arrows
        .replace(/->/g, '<span class="dsl-arrow">→</span>')
        // IDs (words before strings or braces)
        .replace(/(\w+)(\s+["{\[])/g, '<span class="dsl-id">$1</span>$2')
        // Comments
        .replace(/(\/\/[^\n]*)/g, '<span class="dsl-comment">$1</span>')
    );
  }, [dsl]);

  const previewLines = dsl.split("\n").slice(0, 5);
  const hasMore = dsl.split("\n").length > 5;

  return (
    <div className={`dsl-preview ${isExpanded ? "expanded" : ""}`}>
      <button className="dsl-preview-header" onClick={() => setIsExpanded(!isExpanded)}>
        <div className="dsl-preview-title">
          <Code size={16} />
          <span>DSL Preview</span>
          <span className="dsl-preview-hint">Learn as you build</span>
        </div>
        {hasMore && (isExpanded ? <ChevronUp size={16} /> : <ChevronDown size={16} />)}
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
                        /\b(architecture|persons|systems|containers|components|relations|requirements|goals|overview)\b/g,
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
