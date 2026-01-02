import { ChevronDown, ChevronRight, Edit, Plus } from "lucide-react";
import { Button } from "@sruja/ui";
import type { Element } from "@sruja/shared";

export interface NavTreeItemProps {
  element: Element;
  isExpanded: boolean;
  isSelected: boolean;
  hasChildren: boolean;
  onExpand: (id: string) => void;
  onDrillDown: (id: string, kind: string, parentId?: string) => void;
  onSelect?: (id: string, kind: string) => void;
  isEditMode: boolean;
  onEdit: (element: Element) => void;
  onAddChild?: (elementId: string) => void;
  children?: React.ReactNode;
}

export function NavTreeItem({
  element,
  isExpanded,
  isSelected,
  hasChildren,
  onExpand,
  onDrillDown,
  onSelect,
  isEditMode,
  onEdit,
  onAddChild,
  children,
}: NavTreeItemProps) {
  // Safe parent check - Element type from @sruja/shared might have strict parent type
  // Use simple any cast just for the parent access if strictly typed as FqnRef
  const parentId = (element as any).parent as string | undefined;

  return (
    <li className="nav-item-group">
      <div className={`nav-item-row ${isSelected ? "selected" : ""}`}>
        <div style={{ display: "flex", alignItems: "center", flex: 1, overflow: "hidden" }}>
          {hasChildren && (
            <Button
              variant="ghost"
              size="sm"
              className="nav-expand-btn"
              onClick={(e) => {
                e.stopPropagation();
                onExpand(element.id);
              }}
            >
              {isExpanded ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
            </Button>
          )}
          {!hasChildren && <div style={{ width: 24 }} />} {/* Spacer for alignment */}
          <span
            className="nav-item-label"
            onClick={() =>
              onSelect
                ? onSelect(element.id, element.kind)
                : onDrillDown(element.id, element.kind, parentId)
            }
            onDoubleClick={() => onDrillDown(element.id, element.kind, parentId)}
            title={element.title}
            style={{ cursor: "pointer" }}
          >
            {element.title}
          </span>
          {isEditMode && (
            <div className="nav-item-actions">
              <Button
                variant="ghost"
                size="sm"
                className="nav-edit-btn"
                onClick={() => onEdit(element)}
              >
                <Edit size={10} />
              </Button>
              {onAddChild && (
                <Button
                  variant="ghost"
                  size="sm"
                  className="nav-add-btn"
                  onClick={() => onAddChild(element.id)}
                  title="Add Child"
                >
                  <Plus size={10} />
                </Button>
              )}
            </div>
          )}
        </div>
      </div>

      {isExpanded && children && <ul className="nav-item-children">{children}</ul>}
    </li>
  );
}
