// apps/designer/src/components/RoleSwitcher.tsx
import { useEffect, useState } from "react";
import { Users, Code, Server, Shield, TrendingUp, Activity, type LucideIcon } from "lucide-react";
import { Button } from "@sruja/ui";
import { getArchitectureModel } from "../models/ArchitectureModel";
import { useArchitectureStore } from "../stores";
import "./RoleSwitcher.css";

// Default roles for backward compatibility and when no DSL tags are found
export type Role = "product" | "architect" | "devops" | "security" | "cto" | "sre" | string;

interface RoleConfig {
  id: string; // Allow any string, not just hardcoded Role types
  label: string;
  icon: LucideIcon;
  description: string;
}

const ROLES: RoleConfig[] = [
  {
    id: "product",
    label: "Product",
    icon: Users,
    description: "Feature library, user stories, requirements coverage",
  },
  {
    id: "architect",
    label: "Architect",
    icon: Code,
    description: "ADRs, policies, governance, anti-patterns",
  },
  {
    id: "devops",
    label: "DevOps",
    icon: Server,
    description: "Infrastructure, capacity, cost, deployments",
  },
  {
    id: "security",
    label: "Security",
    icon: Shield,
    description: "Trust boundaries, compliance, data flows",
  },
  {
    id: "cto",
    label: "CTO",
    icon: TrendingUp,
    description: "Health scores, risks, technical debt",
  },
  {
    id: "sre",
    label: "SRE",
    icon: Activity,
    description: "SLOs, error budgets, reliability",
  },
];

const STORAGE_KEY = "sruja-role-selection";

interface RoleSwitcherProps {
  selectedRole: Role;
  onRoleChange: (role: Role) => void;
  className?: string;
}

/**
 * Discover roles from the DSL by looking for role elements.
 * Falls back to default roles if none are found in the DSL.
 */
function useDiscoveredRoles(): RoleConfig[] {
  const model = useArchitectureStore((s) => s.model);
  const [discoveredRoles, setDiscoveredRoles] = useState<RoleConfig[]>([]);

  useEffect(() => {
    const archModel = getArchitectureModel();

    // Discover role elements from the DSL (elements with kind="role")
    const roleIds = archModel.discoverRoles();

    if (roleIds.length > 0) {
      // Use discovered role elements from DSL
      const nodes = archModel.getNodes();
      const discovered = roleIds.map((roleId: string) => {
        const element = nodes.get(roleId);
        const title = element?.title || roleId;

        // Try to find a matching default role for icon/description
        const defaultRole = ROLES.find((p) => p.id === roleId.toLowerCase());
        return {
          id: roleId,
          label: title,
          icon: defaultRole?.icon || Users, // Default icon
          description: element?.description || defaultRole?.description || `Views for ${title}`,
        };
      });
      setDiscoveredRoles(discovered);
    } else {
      // Fall back to default roles if no role elements found in DSL
      setDiscoveredRoles(ROLES);
    }
  }, [model]);

  return discoveredRoles.length > 0 ? discoveredRoles : ROLES;
}

export function RoleSwitcher({ selectedRole, onRoleChange, className = "" }: RoleSwitcherProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const availableRoles = useDiscoveredRoles();

  // Load role from localStorage on mount
  useEffect(() => {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved && availableRoles.some((p) => p.id === saved)) {
      onRoleChange(saved as Role);
    } else if (availableRoles.length > 0) {
      // Default to first available role
      onRoleChange(availableRoles[0].id);
    }
  }, [onRoleChange, availableRoles]);

  // Save role to localStorage when changed
  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, selectedRole);
  }, [selectedRole]);

  const currentRole =
    availableRoles.find((p) => p.id === selectedRole) || availableRoles[0] || ROLES[1];

  const handleRoleClick = (role: Role) => {
    onRoleChange(role);
    setIsExpanded(false);
  };

  return (
    <div className={`role-switcher ${className}`}>
      <div className="role-switcher-main">
        <Button
          variant="ghost"
          size="sm"
          className="role-switcher-button"
          onClick={() => setIsExpanded(!isExpanded)}
          aria-label={`Current role: ${currentRole.label}`}
          aria-expanded={isExpanded}
          title={`Switch role view (${currentRole.description})`}
        >
          <currentRole.icon size={16} />
          <span className="role-label">{currentRole.label}</span>
        </Button>
      </div>

      {isExpanded && (
        <>
          <div
            className="role-switcher-overlay"
            onClick={() => setIsExpanded(false)}
            aria-hidden="true"
          />
          <div className="role-switcher-menu">
            <div className="role-switcher-menu-header">
              <span className="role-switcher-menu-title">Switch Role View</span>
              <span className="role-switcher-menu-subtitle">
                Each role sees the same architecture through a different lens
              </span>
            </div>
            <div className="role-switcher-options">
              {availableRoles.map((role) => {
                const Icon = role.icon;
                const isSelected = role.id === selectedRole;
                return (
                  <button
                    key={role.id}
                    className={`role-option ${isSelected ? "selected" : ""}`}
                    onClick={() => handleRoleClick(role.id)}
                    aria-label={`Switch to ${role.label} view`}
                    aria-pressed={isSelected}
                  >
                    <Icon size={18} className="role-option-icon" />
                    <div className="role-option-content">
                      <span className="role-option-label">{role.label}</span>
                      <span className="role-option-description">{role.description}</span>
                    </div>
                    {isSelected && (
                      <span className="role-option-check" aria-hidden="true">
                        ✓
                      </span>
                    )}
                  </button>
                );
              })}
            </div>
          </div>
        </>
      )}
    </div>
  );
}
