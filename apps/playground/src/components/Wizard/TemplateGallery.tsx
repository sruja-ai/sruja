/**
 * TemplateGallery
 * Modal for selecting starter architecture templates
 */

import { useState } from "react";
import { X, CheckCircle, Layers, Cloud, FileText } from "lucide-react";
import { templates, type Template } from "./templates";
import { useArchitectureStore } from "../../stores/architectureStore";
import "./TemplateGallery.css";

interface TemplateGalleryProps {
  isOpen: boolean;
  onClose: () => void;
}

export function TemplateGallery({ isOpen, onClose }: TemplateGalleryProps) {
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);

  if (!isOpen) return null;

  const categoryIcons = {
    basic: <FileText size={14} />,
    intermediate: <Layers size={14} />,
    advanced: <Cloud size={14} />,
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
    arch.metadata.generated = new Date().toISOString();
    // Use updateArchitecture with a function that replaces the whole architecture
    updateArchitecture(() => arch);
    onClose();
  };

  return (
    <div className="template-gallery-overlay" onClick={onClose}>
      <div className="template-gallery-modal" onClick={(e) => e.stopPropagation()}>
        <div className="template-gallery-header">
          <h2>Choose a Template</h2>
          <p>Start with a pre-built architecture pattern</p>
          <button className="template-close-btn" onClick={onClose}>
            <X size={20} />
          </button>
        </div>

        <div className="template-grid">
          {templates.map((template) => (
            <button
              key={template.id}
              className={`template-card ${selectedId === template.id ? "selected" : ""}`}
              onClick={() => setSelectedId(template.id)}
              onDoubleClick={() => handleSelect(template)}
            >
              <div className="template-icon">{template.icon}</div>
              <div className="template-info">
                <h3>{template.name}</h3>
                <p>{template.description}</p>
              </div>
              <div className="template-category">
                {categoryIcons[template.category]}
                <span>{getCategoryLabel(template.category)}</span>
              </div>
              {selectedId === template.id && (
                <div className="template-selected-badge">
                  <CheckCircle size={16} />
                </div>
              )}
            </button>
          ))}
        </div>

        <div className="template-gallery-footer">
          <button className="template-cancel-btn" onClick={onClose}>
            Cancel
          </button>
          <button
            className="template-use-btn"
            disabled={!selectedId}
            onClick={() => {
              const template = templates.find((t) => t.id === selectedId);
              if (template) handleSelect(template);
            }}
          >
            Use Template
          </button>
        </div>
      </div>
    </div>
  );
}
