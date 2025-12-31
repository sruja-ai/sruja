// apps/designer/src/components/Product/FeatureLibrary.tsx
import { useState, useMemo } from "react";
import { Search, Package } from "lucide-react";
import { Input, Button, Badge } from "@sruja/ui";
import {
  FEATURE_TEMPLATES,
  type FeatureTemplate,
  getFeatureTemplatesByCategory,
} from "../../data/featureTemplates";
import "./FeatureLibrary.css";

interface FeatureLibraryProps {
  onFeatureSelect?: (feature: FeatureTemplate) => void;
  onFeatureDragStart?: (feature: FeatureTemplate) => void;
}

export function FeatureLibrary({ onFeatureSelect, onFeatureDragStart }: FeatureLibraryProps) {
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedCategory, setSelectedCategory] = useState<FeatureTemplate["category"] | "all">(
    "all"
  );

  const categories: Array<{ value: FeatureTemplate["category"] | "all"; label: string }> = [
    { value: "all", label: "All" },
    { value: "ecommerce", label: "E-commerce" },
    { value: "saas", label: "SaaS" },
    { value: "platform", label: "Platform" },
    { value: "api", label: "API" },
    { value: "data", label: "Data" },
    { value: "auth", label: "Auth" },
    { value: "other", label: "Other" },
  ];

  const filteredFeatures = useMemo(() => {
    let features =
      selectedCategory === "all"
        ? FEATURE_TEMPLATES
        : getFeatureTemplatesByCategory(selectedCategory);

    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      features = features.filter(
        (f) => f.name.toLowerCase().includes(query) || f.description.toLowerCase().includes(query)
      );
    }

    return features;
  }, [searchQuery, selectedCategory]);

  const handleFeatureClick = (feature: FeatureTemplate) => {
    onFeatureSelect?.(feature);
  };

  const handleDragStart = (e: React.DragEvent, feature: FeatureTemplate) => {
    e.dataTransfer.setData("application/feature", JSON.stringify(feature));
    e.dataTransfer.effectAllowed = "copy";
    onFeatureDragStart?.(feature);
  };

  return (
    <div className="feature-library">
      <div className="feature-library-header">
        <h3 className="feature-library-title">
          <Package size={18} />
          Feature Library
        </h3>
      </div>

      <div className="feature-library-filters">
        <div className="feature-library-search">
          <Search size={16} className="search-icon" />
          <Input
            type="text"
            placeholder="Search features..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="feature-search-input"
          />
        </div>

        <div className="feature-library-categories">
          {categories.map((cat) => (
            <Button
              key={cat.value}
              variant={selectedCategory === cat.value ? "primary" : "ghost"}
              size="sm"
              onClick={() => setSelectedCategory(cat.value)}
              className="category-button"
            >
              {cat.label}
            </Button>
          ))}
        </div>
      </div>

      <div className="feature-library-list">
        {filteredFeatures.length === 0 ? (
          <div className="feature-library-empty">
            <p>No features found matching your search.</p>
          </div>
        ) : (
          filteredFeatures.map((feature) => (
            <div
              key={feature.id}
              className="feature-item"
              draggable
              onDragStart={(e) => handleDragStart(e, feature)}
              onClick={() => handleFeatureClick(feature)}
              title={`Drag to diagram or click to view details`}
            >
              <div className="feature-item-header">
                <h4 className="feature-item-name">{feature.name}</h4>
                <Badge color="neutral" className="feature-item-category">
                  {feature.category}
                </Badge>
              </div>
              <p className="feature-item-description">{feature.description}</p>
              <div className="feature-item-components">
                <span className="feature-item-components-label">
                  {feature.requiredComponents.length} required component
                  {feature.requiredComponents.length !== 1 ? "s" : ""}
                </span>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
