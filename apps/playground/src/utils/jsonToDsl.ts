import type {
  ArchitectureJSON,
  SystemJSON,
  ContainerJSON,
  ComponentJSON,
  RelationJSON,
  PersonJSON,
  DataStoreJSON,
  QueueJSON,
  RequirementJSON,
  ADRJSON,
  ScenarioJSON,
  FlowJSON,
  ScenarioStepJSON,
  OverviewJSON,
  PolicyJSON,
  ConstraintJSON,
  ConventionJSON,
  MetadataEntry,
} from "../types";

function esc(str?: string): string {
  return (str ?? "").replace(/"/g, '\\"');
}

function idOrLabel(id: string, label?: string): string {
  return label && label !== id ? `${id} "${esc(label)}"` : id;
}

function renderRelations(relations?: RelationJSON[]): string {
  if (!relations || relations.length === 0) return "";
  return relations
    .map((r) => {
      const meta: string[] = [];
      if (r.verb) meta.push(`verb "${esc(r.verb)}"`);
      if (r.label) meta.push(`label "${esc(r.label)}"`);
      if (r.technology) meta.push(`tech "${esc(r.technology)}"`);
      if (r.interaction) meta.push(`interaction ${r.interaction}`);
      return `  ${r.from} -> ${r.to}${meta.length ? " " + meta.join(" ") : ""}`;
    })
    .join("\n");
}

function renderContainers(containers?: ContainerJSON[]): string {
  if (!containers || containers.length === 0) return "";
  return containers
    .map((c) => {
      const head = `  container ${idOrLabel(c.id, c.label)}${c.technology ? ` tech "${esc(c.technology)}"` : ""}`;
      const desc = c.description ? `    description "${esc(c.description)}"` : "";
      const comps = (c.components ?? [])
        .map((k) => {
          const compHead = `    component ${idOrLabel(k.id, k.label)}${k.technology ? ` tech "${esc(k.technology)}"` : ""}`;
          const compDesc = k.description ? `      description "${esc(k.description)}"` : "";
          const compRels = renderRelations(k.relations)?.replace(/^ {2}/gm, "      ") ?? "";
          const compBody = [compDesc, compRels].filter(Boolean).join("\n");
          return compBody ? `${compHead} {\n${compBody}\n    }` : compHead;
        })
        .join("\n");
      const rels = renderRelations(c.relations);
      const bodyLines = [desc, comps, rels].filter(Boolean).join("\n");
      return bodyLines ? `${head} {\n${bodyLines}\n  }` : head;
    })
    .join("\n");
}

function renderComponents(components?: ComponentJSON[]): string {
  if (!components || components.length === 0) return "";
  return components
    .map((k) => {
      const head = `  component ${idOrLabel(k.id, k.label)}${k.technology ? ` tech "${esc(k.technology)}"` : ""}`;
      const desc = k.description ? `    description "${esc(k.description)}"` : "";
      const rels = renderRelations(k.relations);
      const body = [desc, rels].filter(Boolean).join("\n");
      return body ? `${head} {\n${body}\n  }` : head;
    })
    .join("\n");
}

function renderDatastores(datastores?: DataStoreJSON[]): string {
  if (!datastores || datastores.length === 0) return "";
  return datastores.map((d) => `  datastore ${idOrLabel(d.id, d.label)}`).join("\n");
}

function renderQueues(queues?: QueueJSON[]): string {
  if (!queues || queues.length === 0) return "";
  return queues.map((q) => `  queue ${idOrLabel(q.id, q.label)}`).join("\n");
}

function renderSystem(s: SystemJSON): string {
  const head = `system ${idOrLabel(s.id, s.label)}`;
  const desc = s.description ? `  description "${esc(s.description)}"` : "";
  const containers = renderContainers(s.containers);
  const comps = renderComponents(s.components);
  const stores = renderDatastores(s.datastores);
  const queues = renderQueues(s.queues);
  const rels = renderRelations(s.relations);
  const bodyLines = [desc, containers, comps, stores, queues, rels].filter(Boolean).join("\n");
  return bodyLines ? `${head} {\n${bodyLines}\n}` : head;
}

function renderPersons(persons?: PersonJSON[]): string {
  if (!persons || persons.length === 0) return "";
  return persons.map((p) => `person ${idOrLabel(p.id, p.label)}`).join("\n");
}

function renderRequirements(requirements?: RequirementJSON[]): string {
  if (!requirements || requirements.length === 0) return "";
  return requirements
    .map((r) => {
      const parts: string[] = [`requirement ${r.id}`];
      if (r.type) parts.push(r.type);
      if (r.title || r.description) {
        parts.push(`"${esc(r.title || r.description || "")}"`);
      }
      return `  ${parts.join(" ")}`;
    })
    .join("\n");
}

function renderADRs(adrs?: ADRJSON[]): string {
  if (!adrs || adrs.length === 0) return "";
  return adrs
    .map((a) => {
      const parts: string[] = [`adr ${a.id}`];
      if (a.title) parts.push(`"${esc(a.title)}"`);
      const bodyParts: string[] = [];
      if (a.status) bodyParts.push(`    status "${esc(a.status)}"`);
      if (a.context) bodyParts.push(`    context "${esc(a.context)}"`);
      if (a.decision) bodyParts.push(`    decision "${esc(a.decision)}"`);
      if (a.consequences) bodyParts.push(`    consequences "${esc(a.consequences)}"`);
      if (bodyParts.length > 0) {
        return `  ${parts.join(" ")} {\n${bodyParts.join("\n")}\n  }`;
      }
      return `  ${parts.join(" ")}`;
    })
    .join("\n");
}

function renderScenarioSteps(steps?: ScenarioStepJSON[]): string {
  if (!steps || steps.length === 0) return "";
  return steps
    .map((s) => {
      const parts: string[] = [`    ${s.from} -> ${s.to}`];
      if (s.description) parts.push(`"${esc(s.description)}"`);
      if (s.tags && s.tags.length > 0) parts.push(`[${s.tags.join(", ")}]`);
      return parts.join(" ");
    })
    .join("\n");
}

function renderScenarios(scenarios?: ScenarioJSON[]): string {
  if (!scenarios || scenarios.length === 0) return "";
  return scenarios
    .map((s) => {
      const parts: string[] = [`scenario ${s.id}`];
      if (s.title || s.label) parts.push(`"${esc(s.title || s.label || "")}"`);
      if (s.description) parts.push(`"${esc(s.description)}"`);
      const steps = renderScenarioSteps(s.steps);
      if (steps) {
        return `  ${parts.join(" ")} {\n${steps}\n  }`;
      }
      return `  ${parts.join(" ")}`;
    })
    .join("\n");
}

function renderFlows(flows?: FlowJSON[]): string {
  if (!flows || flows.length === 0) return "";
  return flows
    .map((f) => {
      const parts: string[] = [`flow ${f.id}`];
      if (f.title || f.label) parts.push(`"${esc(f.title || f.label || "")}"`);
      if (f.description) parts.push(`"${esc(f.description)}"`);
      const steps = renderScenarioSteps(f.steps);
      if (steps) {
        return `  ${parts.join(" ")} {\n${steps}\n  }`;
      }
      return `  ${parts.join(" ")}`;
    })
    .join("\n");
}

function renderOverview(overview?: OverviewJSON): string {
  if (!overview) return "";
  const parts: string[] = [];
  if (overview.summary) parts.push(`    summary "${esc(overview.summary)}"`);
  if (overview.audience) parts.push(`    audience "${esc(overview.audience)}"`);
  if (overview.scope) parts.push(`    scope "${esc(overview.scope)}"`);
  if (overview.goals && overview.goals.length > 0) {
    parts.push(`    goals [${overview.goals.map((g) => `"${esc(g)}"`).join(", ")}]`);
  }
  if (overview.nonGoals && overview.nonGoals.length > 0) {
    parts.push(`    nonGoals [${overview.nonGoals.map((ng) => `"${esc(ng)}"`).join(", ")}]`);
  }
  if (overview.risks && overview.risks.length > 0) {
    parts.push(`    risks [${overview.risks.map((r) => `"${esc(r)}"`).join(", ")}]`);
  }
  if (parts.length === 0) return "";
  return `  overview {\n${parts.join("\n")}\n  }`;
}

function renderPolicies(policies?: PolicyJSON[]): string {
  if (!policies || policies.length === 0) return "";
  return policies
    .map((p) => {
      const parts: string[] = [`policy ${p.id}`];
      if (p.label || p.description) parts.push(`"${esc(p.label || p.description || "")}"`);
      const bodyParts: string[] = [];
      if (p.category) bodyParts.push(`    category "${esc(p.category)}"`);
      if (p.enforcement) bodyParts.push(`    enforcement "${esc(p.enforcement)}"`);
      if (p.description && !p.label) bodyParts.push(`    description "${esc(p.description)}"`);
      if (bodyParts.length > 0) {
        return `  ${parts.join(" ")} {\n${bodyParts.join("\n")}\n  }`;
      }
      return `  ${parts.join(" ")}`;
    })
    .join("\n");
}

function renderConstraints(constraints?: ConstraintJSON[]): string {
  if (!constraints || constraints.length === 0) return "";
  const entries = constraints.map((c) => `    ${c.key} "${esc(c.value)}"`).join("\n");
  return `  constraints {\n${entries}\n  }`;
}

function renderConventions(conventions?: ConventionJSON[]): string {
  if (!conventions || conventions.length === 0) return "";
  const entries = conventions.map((c) => `    ${c.key} "${esc(c.value)}"`).join("\n");
  return `  conventions {\n${entries}\n  }`;
}

function renderMetadata(metadata?: MetadataEntry[]): string {
  if (!metadata || metadata.length === 0) return "";
  const entries: string[] = [];
  metadata.forEach((m) => {
    if (m.value) {
      entries.push(`    ${m.key} "${esc(m.value)}"`);
    } else if (m.array && m.array.length > 0) {
      entries.push(`    ${m.key} [${m.array.map((v) => `"${esc(v)}"`).join(", ")}]`);
    }
  });
  if (entries.length === 0) return "";
  return `  metadata {\n${entries.join("\n")}\n  }`;
}

export function convertJsonToDsl(arch: ArchitectureJSON): string {
  const lines: string[] = [];
  const name = arch.metadata?.name || arch.architecture?.name || "Architecture";
  lines.push(`architecture "${esc(name)}" {`);

  const archDesc = arch.architecture?.description;
  if (archDesc) {
    lines.push(`  description "${esc(archDesc)}"`);
  }

  const persons = renderPersons(arch.architecture?.persons);
  if (persons) lines.push(persons);

  const systems = arch.architecture?.systems?.map(renderSystem).join("\n") ?? "";
  if (systems) lines.push(systems);

  const topRels = renderRelations(arch.architecture?.relations);
  if (topRels) lines.push(topRels);

  const requirements = renderRequirements(arch.architecture?.requirements);
  if (requirements) lines.push(requirements);

  const adrs = renderADRs(arch.architecture?.adrs);
  if (adrs) lines.push(adrs);

  const scenarios = renderScenarios(arch.architecture?.scenarios);
  if (scenarios) lines.push(scenarios);

  const flows = renderFlows(arch.architecture?.flows);
  if (flows) lines.push(flows);

  const overview = renderOverview(arch.architecture?.overview);
  if (overview) lines.push(overview);

  const metadata = renderMetadata(arch.architecture?.archMetadata);
  if (metadata) lines.push(metadata);

  const policies = renderPolicies(arch.architecture?.policies);
  if (policies) lines.push(policies);

  const constraints = renderConstraints(arch.architecture?.constraints);
  if (constraints) lines.push(constraints);

  const conventions = renderConventions(arch.architecture?.conventions);
  if (conventions) lines.push(conventions);

  lines.push("}");
  return lines.join("\n");
}
