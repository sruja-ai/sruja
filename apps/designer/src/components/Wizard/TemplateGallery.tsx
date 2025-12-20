/**
 * TemplateGallery
 * Modal for selecting starter architecture templates
 */

import { useState, useEffect, useRef } from "react";
import { X, CheckCircle, Layers, Cloud, FileText } from "lucide-react";
import { Button } from "@sruja/ui";
import { templates, type Template } from "./templates";
import { useArchitectureStore } from "../../stores/architectureStore";
import "./TemplateGallery.css";

interface TemplateGalleryProps {
  isOpen: boolean;
  onClose: () => void;
}

export function TemplateGallery({ isOpen, onClose }: TemplateGalleryProps) {
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const loadFromModel = useArchitectureStore((s) => s.loadFromModel);
  const modalRef = useRef<HTMLDivElement>(null);
  const closeButtonRef = useRef<HTMLButtonElement>(null);

  // Focus trap and keyboard handling
  useEffect(() => {
    if (!isOpen) return;

    // Focus first template or close button on open
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

  if (!isOpen) return null;

  const categoryIcons = {
    basic: <FileText size={14} aria-hidden="true" />,
    intermediate: <Layers size={14} aria-hidden="true" />,
    advanced: <Cloud size={14} aria-hidden="true" />,
  };

  const getCategoryLabel = (cat: Template["category"]) => {
    switch (cat) {
      case "basic":
        return "Basic";
      case "intermediate":
        return "Intermediate";
      case "advanced":
        return "Advanced";
    }
  };

  const handleSelect = (template: Template) => {
    // Deep clone the architecture to avoid mutation
    const arch = JSON.parse(JSON.stringify(template.architecture));
    // Update timestamp
    if (arch.metadata) {
      arch.metadata.generated = new Date().toISOString();
    } else if (arch._metadata) {
      arch._metadata.generated = new Date().toISOString();
    }

    // Use loadFromModel to sync both JSON and DSL
    loadFromModel(arch, `template:${template.id}`);
    onClose();
  };

  return (
    <div className="template-gallery-overlay" onClick={onClose} role="presentation">
      <div
        ref={modalRef}
        className="template-gallery-modal"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
        aria-labelledby="template-gallery-title"
      >
        <div className="template-gallery-header">
          <h2 id="template-gallery-title">Choose a Template</h2>
          <p>Start with a pre-built architecture pattern</p>
          <Button
            ref={closeButtonRef}
            variant="ghost"
            size="sm"
            className="template-close-btn"
            onClick={onClose}
            aria-label="Close template gallery"
          >
            <X size={20} aria-hidden="true" />
          </Button>
        </div>

        <div className="template-grid" role="radiogroup" aria-label="Template options">
          {templates.map((template) => (
            <Button
              key={template.id}
              variant={selectedId === template.id ? "primary" : "ghost"}
              size="sm"
              className={`template-card ${selectedId === template.id ? "selected" : ""}`}
              onClick={() => setSelectedId(template.id)}
              onDoubleClick={() => handleSelect(template)}
              role="radio"
              aria-checked={selectedId === template.id}
              aria-label={`${template.name}: ${template.description}`}
            >
              <div className="template-icon" aria-hidden="true">
                {template.icon}
              </div>
              <div className="template-info">
                <h3>{template.name}</h3>
                <p>{template.description}</p>
              </div>
              <div className="template-category">
                {categoryIcons[template.category]}
                <span>{getCategoryLabel(template.category)}</span>
              </div>
              {selectedId === template.id && (
                <div className="template-selected-badge" aria-hidden="true">
                  <CheckCircle size={16} />
                </div>
              )}
            </Button>
          ))}
        </div>

        <div className="template-gallery-footer">
          <Button
            variant="ghost"
            className="template-cancel-btn"
            onClick={onClose}
            aria-label="Cancel and close"
          >
            Cancel
          </Button>
          <Button
            variant="primary"
            className="template-use-btn"
            disabled={!selectedId}
            onClick={() => {
              const template = templates.find((t) => t.id === selectedId);
              if (template) handleSelect(template);
            }}
            aria-label={selectedId ? "Use selected template" : "Select a template first"}
          >
            Use Template
          </Button>
        </div>
      </div>
    </div>
  );
}
