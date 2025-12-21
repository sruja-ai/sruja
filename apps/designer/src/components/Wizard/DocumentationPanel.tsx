// apps/designer/src/components/Wizard/DocumentationPanel.tsx
// Contextual documentation panel for builder wizard

import { BookOpen, ExternalLink, ChevronDown, ChevronUp, ChevronRight } from "lucide-react";
import { useState } from "react";
import { Button } from "@sruja/ui";
import {
  getDocumentationTopics,
  getDocumentationUrl,
} from "../../utils/documentationService";
import "./DocumentationPanel.css";

interface DocumentationPanelProps {
  /** Builder step ID (goals, context, containers, components, flows) */
  stepId: string;
  /** Compact mode for sidebar */
  compact?: boolean;
}

/**
 * Documentation panel component for builder wizard.
 * 
 * Displays relevant documentation links for the current builder step.
 * Provides contextual help without leaving the builder.
 * 
 * @param props - Panel configuration
 * @param props.stepId - Current builder step ID
 * @param props.compact - Whether to render in compact mode (default: false)
 * @returns Documentation panel component
 * 
 * @example
 * ```tsx
 * <DocumentationPanel stepId="context" compact />
 * ```
 */
export function DocumentationPanel({
  stepId,
  compact = false,
}: DocumentationPanelProps) {
  const [isExpanded, setIsExpanded] = useState(!compact);
  const [expandedTopics, setExpandedTopics] = useState<Set<string>>(new Set());
  const topics = getDocumentationTopics(stepId);

  if (topics.length === 0) {
    return null;
  }

  const toggleTopic = (path: string) => {
    setExpandedTopics((prev) => {
      const next = new Set(prev);
      if (next.has(path)) {
        next.delete(path);
      } else {
        next.add(path);
      }
      return next;
    });
  };

  /**
   * Simple markdown to HTML converter for excerpts
   * Handles **bold**, *italic*, and basic formatting
   */
  const renderExcerpt = (text: string): string => {
    return text
      .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
      .replace(/\*(.+?)\*/g, "<em>$1</em>")
      .replace(/`(.+?)`/g, "<code>$1</code>");
  };

  if (compact) {
    return (
      <div className="documentation-panel compact">
        <Button
          variant="ghost"
          size="sm"
          className="doc-header-compact"
          onClick={() => setIsExpanded(!isExpanded)}
          aria-label="Toggle documentation"
          title="View documentation for this step"
        >
          <BookOpen size={16} />
          <span>📚 Documentation</span>
          {isExpanded ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
        </Button>

        {isExpanded && (
          <div className="doc-topics-compact">
            <div className="doc-intro-compact">
              Quick reference for this step
            </div>
            {topics.slice(0, 3).map((topic) => {
              const isTopicExpanded = expandedTopics.has(topic.path);
              const hasExcerpt = !!topic.excerpt;
              
              return (
                <div key={topic.path} className="doc-topic-item-compact">
                  <div className="doc-topic-header-compact">
                    <Button
                      variant="ghost"
                      size="sm"
                      className={`doc-topic-toggle ${hasExcerpt ? "" : "no-excerpt"}`}
                      onClick={() => hasExcerpt && toggleTopic(topic.path)}
                      disabled={!hasExcerpt}
                    >
                      {hasExcerpt && (
                        <ChevronRight
                          size={14}
                          className={`chevron-icon ${isTopicExpanded ? "expanded" : ""}`}
                        />
                      )}
                      <span className="doc-topic-title-compact">{topic.title}</span>
                    </Button>
                    <a
                      href={getDocumentationUrl(topic.path)}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="doc-external-link"
                      onClick={(e) => e.stopPropagation()}
                      title={`Open full ${topic.title} documentation`}
                    >
                      <ExternalLink size={12} />
                    </a>
                  </div>
                  {hasExcerpt && isTopicExpanded && (
                    <div className="doc-excerpt-compact">
                      <div
                        className="doc-excerpt-content"
                        dangerouslySetInnerHTML={{
                          __html: renderExcerpt(topic.excerpt || ""),
                        }}
                      />
                      <a
                        href={getDocumentationUrl(topic.path)}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="doc-learn-more"
                        onClick={(e) => e.stopPropagation()}
                      >
                        Learn more →
                      </a>
                    </div>
                  )}
                </div>
              );
            })}
            {topics.length > 3 && (
              <a
                href={getDocumentationUrl(topics[0].path.split("/")[0] || "getting-started")}
                target="_blank"
                rel="noopener noreferrer"
                className="doc-link-compact doc-link-more"
              >
                View all {topics.length} topics <ExternalLink size={12} />
              </a>
            )}
          </div>
        )}
      </div>
    );
  }

  return (
    <div className="documentation-panel">
      <div className="doc-header">
        <BookOpen size={20} />
        <h3>Documentation</h3>
      </div>

      <div className="doc-description">
        Learn more about this step and related concepts
      </div>

      <div className="doc-topics">
        {topics.map((topic) => (
          <a
            key={topic.path}
            href={getDocumentationUrl(topic.path)}
            target="_blank"
            rel="noopener noreferrer"
            className="doc-topic-card"
          >
            <div className="doc-topic-content">
              <strong className="doc-topic-title">{topic.title}</strong>
              {topic.description && (
                <span className="doc-topic-desc">{topic.description}</span>
              )}
            </div>
            <ExternalLink size={16} className="doc-topic-icon" />
          </a>
        ))}
      </div>
    </div>
  );
}
