// apps/designer/src/utils/modelToDsl.ts
// Convert SrujaModelDump to advanced DSL format with full feature support

import type {
  SrujaModelDump,
  RequirementDump,
  PolicyDump,
  ADRDump,
  ScenarioDump,
  FlowDump,
  ConstraintDump,
  ConventionDump,
} from "@sruja/shared";

// Local type aliases for this file
type ElementDump = NonNullable<SrujaModelDump["elements"]>[string];
type ViewDump = NonNullable<SrujaModelDump["views"]>[string];

export async function convertModelToDsl(model: SrujaModelDump): Promise<string> {
  if (!model) return "";

  const lines: string[] = [];

  // Specification block
  if (model.specification) {
    lines.push("specification {");
    if (model.specification.elements) {
      Object.keys(model.specification.elements).forEach((kind) => {
        lines.push(`  element ${kind}`);
      });
    }
    if (model.specification.tags) {
      Object.keys(model.specification.tags).forEach((tag) => {
        lines.push(`  tag ${tag}`);
      });
    }
    lines.push("}");
    lines.push("");
  }

  // Model block
  lines.push("model {");

  // Print elements in hierarchical order (top-level first, then nested)
  if (model.elements) {
    const elements = Object.values(model.elements) as any[];
    const topLevelElements = elements.filter((el: any) => !(el as any).parent);
    const nestedElements = elements.filter((el: any) => (el as any).parent);

    // Print top-level elements with their nested children
    topLevelElements.forEach((el: any) => {
      printElementWithChildren(el, elements, lines, "  ");
    });

    // Print any orphaned nested elements (shouldn't happen, but handle gracefully)
    nestedElements.forEach((el: any) => {
      if (!elements.find((e: any) => e.id === (el as any).parent)) {
        printElementWithChildren(el, elements, lines, "  ");
      }
    });
  }

  // Print relations with support for bidirectional and back arrows
  if (model.relations) {
    if (model.elements && Object.keys(model.elements).length > 0) {
      lines.push("");
    }
    model.relations.forEach((rel) => {
      printRelation(rel, lines, "  ");
    });
  }

  // Print Sruja extensions (requirements, policies, ADRs, scenarios, flows, etc.)
  if (model.sruja) {
    const hasExtensions =
      (model.sruja.requirements && model.sruja.requirements.length > 0) ||
      (model.sruja.policies && model.sruja.policies.length > 0) ||
      (model.sruja.adrs && model.sruja.adrs.length > 0) ||
      (model.sruja.scenarios && model.sruja.scenarios.length > 0) ||
      (model.sruja.flows && model.sruja.flows.length > 0) ||
      (model.sruja.constraints && model.sruja.constraints.length > 0) ||
      (model.sruja.conventions && model.sruja.conventions.length > 0);

    if (hasExtensions) {
      lines.push("");
    }

    // Requirements
    if (model.sruja.requirements) {
      model.sruja.requirements.forEach((req) => {
        printRequirement(req, lines, "  ");
      });
    }

    // Policies
    if (model.sruja.policies) {
      model.sruja.policies.forEach((policy) => {
        printPolicy(policy, lines, "  ");
      });
    }

    // ADRs
    if (model.sruja.adrs) {
      model.sruja.adrs.forEach((adr) => {
        printADR(adr, lines, "  ");
      });
    }

    // Scenarios
    if (model.sruja.scenarios) {
      model.sruja.scenarios.forEach((scenario) => {
        printScenario(scenario, lines, "  ");
      });
    }

    // Flows
    if (model.sruja.flows) {
      model.sruja.flows.forEach((flow) => {
        printFlow(flow, lines, "  ");
      });
    }

    // Constraints
    if (model.sruja.constraints) {
      model.sruja.constraints.forEach((constraint) => {
        printConstraint(constraint, lines, "  ");
      });
    }

    // Conventions
    if (model.sruja.conventions) {
      model.sruja.conventions.forEach((convention) => {
        printConvention(convention, lines, "  ");
      });
    }
  }

  lines.push("}");

  // Views block with proper include/exclude support
  if (model.views && Object.keys(model.views).length > 0) {
    lines.push("");
    lines.push("views {");
    Object.values(model.views).forEach((view) => {
      printView(view, lines, "  ");
    });
    lines.push("}");
  }

  return lines.join("\n");
}

function printElementWithChildren(
  el: ElementDump,
  allElements: ElementDump[],
  lines: string[],
  prefix: string
): void {
  // Find children of this element
  const children = allElements.filter((e: any) => (e as any).parent === (el as any).id);

  // Print element opening
  const elementName = extractElementName((el as any).id);
  lines.push(`${prefix}${elementName} = ${(el as any).kind} "${escapeString((el as any).title)}" {`);

  // Print element properties
  if ((el as any).description) {
    const desc = typeof (el as any).description === "string" ? (el as any).description : (el as any).description.txt;
    lines.push(`${prefix}  description "${escapeString(desc)}"`);
  }
  if ((el as any).technology) {
    lines.push(`${prefix}  technology "${escapeString((el as any).technology)}"`);
  }
  if ((el as any).tags && (el as any).tags.length > 0) {
    lines.push(`${prefix}  tags [${(el as any).tags.map((t: any) => `"${escapeString(t)}"`).join(", ")}]`);
  }
  if ((el as any).metadata && Object.keys((el as any).metadata).length > 0) {
    lines.push(`${prefix}  metadata {`);
    Object.entries((el as any).metadata).forEach(([key, value]: [string, any]) => {
      lines.push(`${prefix}    ${key} "${escapeString(String(value))}"`);
    });
    lines.push(`${prefix}  }`);
  }
  if ((el as any).links && (el as any).links.length > 0) {
    (el as any).links.forEach((link: any) => {
      if (link.title) {
        lines.push(`${prefix}  link "${escapeString(link.url)}" "${escapeString(link.title)}"`);
      } else {
        lines.push(`${prefix}  link "${escapeString(link.url)}"`);
      }
    });
  }

  // Print nested children
  if (children.length > 0) {
    children.forEach((child) => {
      printElementWithChildren(child, allElements, lines, prefix + "  ");
    });
  }

  lines.push(`${prefix}}`);
}

function extractElementName(id: string): string {
  // If ID contains dots, extract the last part for nested elements
  // e.g., "system.container" -> "container"
  const parts = id.split(".");
  return parts[parts.length - 1];
}

function printRelation(rel: any, lines: string[], prefix: string): void {
  // Support bidirectional (<->) and back arrows (<-)
  let arrow = "->";
  if (rel.kind === "bidirectional" || (rel.line as any) === "bidirectional") {
    arrow = "<->";
  } else if (rel.kind === "back" || (rel.line as any) === "back") {
    arrow = "<-";
  }

  // Handle source and target being either string (legacy) or FqnRef object (LikeC4)
  const sourceFqn = typeof rel.source === "string" ? rel.source : rel.source?.model || "";
  const targetFqn = typeof rel.target === "string" ? rel.target : rel.target?.model || "";

  if (!sourceFqn || !targetFqn) return;

  const title = rel.title ? ` "${escapeString(rel.title)}"` : "";
  const tech = rel.technology ? ` technology "${escapeString(rel.technology)}"` : "";
  const tags = rel.tags && rel.tags.length > 0
    ? ` tags [${rel.tags.map((t: any) => `"${escapeString(t)}"`).join(", ")}]`
    : "";

  lines.push(`${prefix}${sourceFqn} ${arrow} ${targetFqn}${title}${tech}${tags}`);
}

function printRequirement(req: RequirementDump, lines: string[], prefix: string): void {
  const type = req.type ? ` ${req.type}` : "";
  lines.push(`${prefix}requirement ${req.id}${type} "${escapeString(req.title)}"`);
  if (req.description) {
    lines.push(`${prefix}  description "${escapeString(req.description)}"`);
  }
  if (req.priority) {
    lines.push(`${prefix}  priority "${escapeString(req.priority)}"`);
  }
  if (req.status) {
    lines.push(`${prefix}  status "${escapeString(req.status)}"`);
  }
  if (req.elements && req.elements.length > 0) {
    lines.push(`${prefix}  elements [${req.elements.map((e) => e).join(", ")}]`);
  }
}

function printPolicy(policy: PolicyDump, lines: string[], prefix: string): void {
  const category = policy.category ? ` ${policy.category}` : "";
  lines.push(`${prefix}policy ${policy.id}${category} "${escapeString(policy.title)}"`);
  if (policy.description) {
    lines.push(`${prefix}  description "${escapeString(policy.description)}"`);
  }
  if (policy.enforcement) {
    lines.push(`${prefix}  enforcement "${escapeString(policy.enforcement)}"`);
  }
  if (policy.elements && policy.elements.length > 0) {
    lines.push(`${prefix}  elements [${policy.elements.map((e) => e).join(", ")}]`);
  }
}

function printADR(adr: ADRDump, lines: string[], prefix: string): void {
  lines.push(`${prefix}adr ${adr.id} "${escapeString(adr.title)}" {`);
  if (adr.status) {
    lines.push(`${prefix}  status "${escapeString(adr.status)}"`);
  }
  if (adr.context) {
    lines.push(`${prefix}  context "${escapeString(adr.context)}"`);
  }
  if (adr.decision) {
    lines.push(`${prefix}  decision "${escapeString(adr.decision)}"`);
  }
  if (adr.consequences) {
    lines.push(`${prefix}  consequences "${escapeString(adr.consequences)}"`);
  }
  if (adr.date) {
    lines.push(`${prefix}  date "${escapeString(adr.date)}"`);
  }
  if (adr.author) {
    lines.push(`${prefix}  author "${escapeString(adr.author)}"`);
  }
  lines.push(`${prefix}}`);
}

function printScenario(scenario: ScenarioDump, lines: string[], prefix: string): void {
  lines.push(`${prefix}scenario ${scenario.id} "${escapeString(scenario.title)}" {`);
  if (scenario.description) {
    lines.push(`${prefix}  description "${escapeString(scenario.description)}"`);
  }
  if (scenario.steps) {
    scenario.steps.forEach((step) => {
      if (step.from && step.to) {
        lines.push(
          `${prefix}  step "${escapeString(step.description)}" from ${step.from} to ${step.to}`
        );
      } else {
        lines.push(`${prefix}  step "${escapeString(step.description)}"`);
      }
    });
  }
  lines.push(`${prefix}}`);
}

function printFlow(flow: FlowDump, lines: string[], prefix: string): void {
  lines.push(`${prefix}flow ${flow.id} "${escapeString(flow.title)}" {`);
  if (flow.description) {
    lines.push(`${prefix}  description "${escapeString(flow.description)}"`);
  }
  if (flow.steps) {
    flow.steps.forEach((step) => {
      if (step.from && step.to) {
        lines.push(
          `${prefix}  step "${escapeString(step.description)}" from ${step.from} to ${step.to}`
        );
      } else {
        lines.push(`${prefix}  step "${escapeString(step.description)}"`);
      }
    });
  }
  lines.push(`${prefix}}`);
}

function printConstraint(constraint: ConstraintDump, lines: string[], prefix: string): void {
  const type = constraint.type ? ` ${constraint.type}` : "";
  lines.push(`${prefix}constraint ${constraint.id}${type} "${escapeString(constraint.description)}"`);
}

function printConvention(convention: ConventionDump, lines: string[], prefix: string): void {
  lines.push(`${prefix}convention ${convention.id} "${escapeString(convention.description)}"`);
}

function printView(view: ViewDump, lines: string[], prefix: string): void {
  const viewOf = (view as any).viewOf ? ` of ${(view as any).viewOf}` : "";
  lines.push(`${prefix}view ${view.id}${viewOf} {`);
  if (view.title) {
    lines.push(`${prefix}  title "${escapeString(view.title || "")}"`);
  }
  if (view.description) {
    const desc = typeof view.description === "string" ? view.description : (view.description as any)?.txt || "";
    if (desc) {
      lines.push(`${prefix}  description "${escapeString(desc)}"`);
    }
  }

  // Handle include/exclude patterns
  // Prioritize rules if they exist (new behavior from Go exporter)
  if (view.rules && Array.isArray(view.rules) && view.rules.length > 0) {
    view.rules.forEach((rule: any) => {
      if (rule.include) {
        if (rule.include.wildcard) {
          lines.push(`${prefix}  include *`);
        } else if (rule.include.elements && Array.isArray(rule.include.elements)) {
          rule.include.elements.forEach((el: string) => {
            lines.push(`${prefix}  include ${el}`);
          });
        }
      }
      if (rule.exclude) {
        if (rule.exclude.wildcard) {
          lines.push(`${prefix}  exclude *`);
        } else if (rule.exclude.elements && Array.isArray(rule.exclude.elements)) {
          rule.exclude.elements.forEach((el: string) => {
            lines.push(`${prefix}  exclude ${el}`);
          });
        }
      }
    });
  } else {
    // Legacy fallback or node-based inference
    const viewNodes = (view as any).nodes;
    if (viewNodes && viewNodes.length > 0) {
      // Extract unique element IDs from nodes
      const elementIds = [...new Set(viewNodes.map((n: any) => n.element))];
      if (elementIds.length > 0) {
        elementIds.forEach((id: any) => {
          lines.push(`${prefix}  include ${id}`);
        });
      }
    } else {
      lines.push(`${prefix}  include *`);
    }
  }

  if (view.tags && view.tags.length > 0) {
    lines.push(`${prefix}  tags [${view.tags.map((t) => `"${escapeString(t)}"`).join(", ")}]`);
  }

  lines.push(`${prefix}}`);
}

function escapeString(str: string): string {
  return str.replace(/\\/g, "\\\\").replace(/"/g, '\\"').replace(/\n/g, "\\n");
}
