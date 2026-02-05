import { useState } from "react";
import { Users, Plus, Trash2, Edit, Eye } from "lucide-react";
import { Button, Input } from "@sruja/ui";
import { useArchitectureStore } from "../../../stores/architectureStore";
import type { ElementDump, ParsedView } from "@sruja/shared";
import "../../Wizard/WizardSteps.css";

interface RolesViewsStepProps {
  onBack: () => void;
  onNext?: () => void;
  readOnly?: boolean;
}

export function RolesViewsStep({ onBack, onNext, readOnly = false }: RolesViewsStepProps) {
  const data = useArchitectureStore((s) => s.model);
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);

  // Get roles from elements
  const roles: ElementDump[] = data?.elements
    ? Object.values(data.elements).filter((e) => e.kind === "role")
    : [];

  // Get views
  const allViews: Record<string, ParsedView> = data?.views || {};
  const views = Object.values(allViews);

  const [newRoleId, setNewRoleId] = useState("");
  const [newRoleTitle, setNewRoleTitle] = useState("");
  const [newRoleDescription, setNewRoleDescription] = useState("");
  const [editingRole, setEditingRole] = useState<string | null>(null);

  const handleAddRole = () => {
    if (!newRoleId.trim() || !newRoleTitle.trim() || !data) return;

    // Create role element
    const roleElement: ElementDump = {
      id: newRoleId.trim(),
      kind: "role",
      title: newRoleTitle.trim(),
      description: newRoleDescription.trim(),
      tags: [],
      links: [],
      metadata: {},
    };

    updateArchitecture((model) => {
      if (!model) return model;
      return {
        ...model,
        elements: {
          ...model.elements,
          [newRoleId.trim()]: roleElement,
        },
      };
    });

    // Reset form
    setNewRoleId("");
    setNewRoleTitle("");
    setNewRoleDescription("");
  };

  const handleDeleteRole = (roleId: string) => {
    if (!data || readOnly) return;
    updateArchitecture((model) => {
      if (!model) return model;
      const newElements = { ...model.elements };
      delete newElements[roleId];
      return {
        ...model,
        elements: newElements,
      };
    });
  };

  const handleEditRole = (role: ElementDump) => {
    setEditingRole(role.id);
    setNewRoleId(role.id);
    setNewRoleTitle(role.title);
    setNewRoleDescription(role.description || "");
  };

  const handleUpdateRole = () => {
    if (!editingRole || !data) return;

    updateArchitecture((model) => {
      if (!model) return model;
      const existingRole = model.elements[editingRole];
      if (!existingRole) return model;

      return {
        ...model,
        elements: {
          ...model.elements,
          [editingRole]: {
            ...existingRole,
            title: newRoleTitle.trim(),
            description: newRoleDescription.trim(),
          },
        },
      };
    });

    // Reset form
    setEditingRole(null);
    setNewRoleId("");
    setNewRoleTitle("");
    setNewRoleDescription("");
  };

  const handleCancelEdit = () => {
    setEditingRole(null);
    setNewRoleId("");
    setNewRoleTitle("");
    setNewRoleDescription("");
  };

  // Group views by role tags
  const viewsByRole = new Map<string, ParsedView[]>();
  roles.forEach((role) => {
    const roleViews = views.filter((view) => view.tags?.includes(role.id));
    if (roleViews.length > 0) {
      viewsByRole.set(role.id, roleViews);
    }
  });

  return (
    <div className="wizard-step-content">
      <div className="step-header">
        <div className="step-icon">
          <Users size={24} />
        </div>
        <div className="step-header-content">
          <h2>Define Roles & Views</h2>
          <p>Optional: add role perspectives to tailor views for teams.</p>
        </div>
      </div>

      {/* Roles Section */}
      <div className="step-section">
        <div className="section-header">
          <h3>Roles</h3>
          {!readOnly && (
            <Button
              variant="primary"
              size="sm"
              onClick={editingRole ? handleUpdateRole : handleAddRole}
              disabled={!newRoleId.trim() || !newRoleTitle.trim()}
            >
              <Plus size={16} />
              {editingRole ? "Update Role" : "Add Role"}
            </Button>
          )}
        </div>

        {!readOnly && (
          <div className="form-group" style={{ marginBottom: "24px" }}>
            <Input
              placeholder="Role ID (e.g., devops, sre)"
              value={newRoleId}
              onChange={(e) => setNewRoleId(e.target.value)}
              disabled={!!editingRole}
            />
            <Input
              placeholder="Role Title (e.g., DevOps Team)"
              value={newRoleTitle}
              onChange={(e) => setNewRoleTitle(e.target.value)}
              style={{ marginTop: "8px" }}
            />
            <Input
              placeholder="Description (optional)"
              value={newRoleDescription}
              onChange={(e) => setNewRoleDescription(e.target.value)}
              style={{ marginTop: "8px" }}
            />
            {editingRole && (
              <div style={{ marginTop: "8px", display: "flex", gap: "8px" }}>
                <Button variant="ghost" size="sm" onClick={handleCancelEdit}>
                  Cancel
                </Button>
              </div>
            )}
          </div>
        )}

        <div className="items-list">
          {roles.length === 0 ? (
            <div className="empty-state">
              <Users size={48} style={{ opacity: 0.3 }} />
              <p>No roles defined yet. Create your first role to get started.</p>
            </div>
          ) : (
            roles.map((role) => (
              <div key={role.id} className="item-card">
                <div className="item-content">
                  <div className="item-header">
                    <h4>{role.title}</h4>
                    <span className="item-id">ID: {role.id}</span>
                  </div>
                  {role.description && <p className="item-description">{role.description}</p>}
                  {viewsByRole.has(role.id) && (
                    <div className="item-meta">
                      <Eye size={14} />
                      <span>{viewsByRole.get(role.id)?.length || 0} view(s)</span>
                    </div>
                  )}
                </div>
                {!readOnly && (
                  <div className="item-actions">
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleEditRole(role)}
                      title="Edit role"
                    >
                      <Edit size={16} />
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleDeleteRole(role.id)}
                      title="Delete role"
                    >
                      <Trash2 size={16} />
                    </Button>
                  </div>
                )}
              </div>
            ))
          )}
        </div>
      </div>

      {/* Views Section */}
      <div className="step-section" style={{ marginTop: "32px" }}>
        <div className="section-header">
          <h3>Views</h3>
          <p style={{ fontSize: "14px", color: "var(--color-text-secondary, #6b7280)" }}>
            Tag views with role IDs in the DSL to make them appear for specific roles.
          </p>
        </div>

        {views.length === 0 ? (
          <div className="empty-state">
            <Eye size={48} style={{ opacity: 0.3 }} />
            <p>No views defined yet. Create views in the DSL and tag them with role IDs.</p>
            <p style={{ fontSize: "12px", marginTop: "8px", opacity: 0.7 }}>
              Example:{" "}
              <code>
                view deployment_view {"{"} tags ["devops"] {"}"}
              </code>
            </p>
          </div>
        ) : (
          <div className="items-list">
            {views.map((view) => (
              <div key={view.id} className="item-card">
                <div className="item-content">
                  <div className="item-header">
                    <h4>{view.title || view.id}</h4>
                  </div>
                  {view.description && <p className="item-description">{view.description}</p>}
                  {view.tags && view.tags.length > 0 && (
                    <div className="item-meta">
                      <span>Tags: {view.tags.join(", ")}</span>
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Navigation */}
      <div className="step-actions">
        <Button variant="ghost" onClick={onBack}>
          Back
        </Button>
        {onNext && (
          <Button variant="secondary" onClick={onNext}>
            Skip for now →
          </Button>
        )}
      </div>
    </div>
  );
}
