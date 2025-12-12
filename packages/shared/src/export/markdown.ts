// packages/shared/src/export/markdown.ts
// TypeScript markdown exporter - ported from pkg/export/markdown
// Works with ArchitectureJSON from Go parser

import type {
  ArchitectureJSON,
  ArchitectureBody,
  SystemJSON,
  ScenarioJSON,
  RequirementJSON,
  DeploymentNodeJSON,
} from '../types/architecture';
import {
  generateSystemDiagram,
  generateSystemContainerDiagram,
  generateScenarioDiagram,
  generateDeploymentDiagram,
  extractMermaidConfig,
  type MermaidConfig,
} from './mermaid';

// MermaidConfig is now imported from mermaid.ts

export interface MarkdownExportOptions {
  includeTOC?: boolean;
  includeOverview?: boolean;
  includeSystems?: boolean;
  includeDeployments?: boolean;
  includePersons?: boolean;
  includeRequirements?: boolean;
  includeADRs?: boolean;
  includeScenarios?: boolean;
  includePolicies?: boolean;
  includeConstraints?: boolean;
  includeConventions?: boolean;
  includeFlows?: boolean;
  includeContracts?: boolean;
  includeDataConsistency?: boolean;
  includeFailureModes?: boolean;
  includeRelations?: boolean;
  includeQualityAttributes?: boolean;
  includeSecurity?: boolean;
  includeMetadata?: boolean;
  includeGlossary?: boolean;
  mermaidConfig?: MermaidConfig;
  headingLevel?: number;
}

const DEFAULT_OPTIONS: MarkdownExportOptions = {
  includeTOC: true,
  includeOverview: true,
  includeSystems: true,
  includeDeployments: true,
  includePersons: true,
  includeRequirements: true,
  includeADRs: true,
  includeScenarios: true,
  includePolicies: true,
  includeConstraints: true,
  includeConventions: true,
  includeFlows: true,
  includeContracts: true,
  includeDataConsistency: true,
  includeFailureModes: true,
  includeRelations: true,
  includeQualityAttributes: true,
  includeSecurity: true,
  includeMetadata: true,
  includeGlossary: true,
  headingLevel: 1,
  mermaidConfig: {
    direction: 'LR',
    theme: 'default',
  },
};

// Helper functions (kept for future use)
// function generateAnchor(text: string): string {
//   return text
//     .toLowerCase()
//     .replace(/\s+/g, '-')
//     .replace(/[()]/g, '')
//     .replace(/\./g, '');
// }

// function escapeMarkdown(s: string): string {
//   return s
//     .replace(/\\/g, '\\\\')
//     .replace(/`/g, '\\`')
//     .replace(/\*/g, '\\*')
//     .replace(/_/g, '\\_')
//     .replace(/\[/g, '\\[')
//     .replace(/\]/g, '\\]');
// }

// Helper: Write element (ID, label, description, technology)
function writeElement(
  sb: string[],
  id: string,
  label: string,
  desc?: string,
  technology?: string
): void {
  let line = `- **${id}**: ${label}`;
  if (technology) {
    line += ` (${technology})`;
  }
  if (desc) {
    line += ` - ${desc}`;
  }
  sb.push(line);
}

// extractMermaidConfig is now imported from mermaid.ts

// Generate TOC section
function renderTOC(arch: ArchitectureBody, options: MarkdownExportOptions): string {
  if (!options.includeTOC) return '';

  const items: string[] = [];
  items.push('## Table of Contents\n');

  // Always include common sections
  items.push('- [Executive Summary](#executive-summary)');
  if (options.includeOverview && (arch.systems?.length || arch.persons?.length)) {
    items.push('- [Architecture Overview (C4 L1)](#architecture-overview-c4-l1)');
  }
  if (options.includeSystems && arch.systems?.length) {
    items.push('- [Systems (C4 L2/L3)](#systems)');
  }
  if (options.includePersons && arch.persons?.length) {
    items.push('- [Persons](#persons)');
  }
  if (options.includeRequirements && arch.requirements?.length) {
    items.push('- [Requirements](#requirements)');
  }
  if (options.includeQualityAttributes) {
    items.push('- [Quality Attributes](#quality-attributes)');
  }
  if (options.includeSecurity) {
    items.push('- [Security](#security)');
  }
  if (options.includePolicies && arch.policies?.length) {
    items.push('- [Policies](#policies)');
  }
  if (options.includeFlows && arch.flows?.length) {
    items.push('- [Flows](#flows)');
  }
  if (options.includeContracts && arch.contracts?.length) {
    items.push('- [Integration Contracts](#integration-contracts)');
  }
  if (options.includeScenarios && arch.scenarios?.length) {
    items.push('- [Scenarios](#scenarios)');
  }
  if (options.includeRelations) {
    items.push('- [Relations](#relations)');
  }
  if (options.includeConstraints && arch.constraints?.length) {
    items.push('- [Constraints](#constraints)');
  }
  if (options.includeConventions && arch.conventions?.length) {
    items.push('- [Conventions](#conventions)');
  }
  if (options.includeADRs && arch.adrs?.length) {
    items.push('- [Architecture Decision Records (ADRs)](#architecture-decision-records-adrs)');
  }
  if (options.includeMetadata) {
    items.push('- [Document Metadata](#document-metadata)');
  }
  if (options.includeGlossary) {
    items.push('- [Glossary](#glossary)');
  }

  items.push('');
  return items.join('\n');
}

// Render executive summary (simplified)
function renderExecutiveSummary(arch: ArchitectureBody): string {
  const sections: string[] = [];
  sections.push('## Executive Summary\n');
  sections.push('### Architecture Highlights\n');
  sections.push('');

  const systemCount = arch.systems?.length || 0;
  if (systemCount > 0) {
    sections.push(`- Microservices architecture with ${systemCount} core service${systemCount > 1 ? 's' : ''}`);
  }

  sections.push('');
  return sections.join('\n');
}

// Render architecture overview with mermaid diagram
function renderOverview(
  arch: ArchitectureBody,
  config: MermaidConfig,
  mermaidGenerator?: (arch: ArchitectureBody, config: MermaidConfig) => string
): string {
  if (!arch.systems?.length && !arch.persons?.length) {
    return '';
  }

  const sections: string[] = [];
  sections.push('## Architecture Overview (C4 L1)\n');

  if (mermaidGenerator) {
    const diagram = mermaidGenerator(arch, config);
    if (diagram) {
      sections.push('```mermaid');
      sections.push(diagram);
      sections.push('```\n');
    }
  } else {
    // Use default mermaid generator
    const diagram = generateSystemDiagram(arch, config);
    if (diagram) {
      sections.push('```mermaid');
      sections.push(diagram);
      sections.push('```\n');
    }
  }

  return sections.join('\n');
}

// Render systems section
function renderSystems(
  arch: ArchitectureBody,
  config: MermaidConfig,
  mermaidGenerator?: (sys: SystemJSON, arch: ArchitectureBody, config: MermaidConfig) => string
): string {
  if (!arch.systems?.length) return '';

  const sections: string[] = [];
  sections.push('## Systems\n');

  for (const sys of arch.systems) {
    sections.push(`### ${sys.label || sys.id}\n`);
    if (sys.description) {
      sections.push(`${sys.description}\n`);
    }

    // Container view diagram
    if (sys.containers?.length) {
      sections.push('#### Container View (C4 L2)\n');
      const diagram = mermaidGenerator
        ? mermaidGenerator(sys, arch, config)
        : generateSystemContainerDiagram(sys, arch, config);
      if (diagram) {
        sections.push('```mermaid');
        sections.push(diagram);
        sections.push('```\n');
      }
    }

    // Containers list
    if (sys.containers?.length) {
      sections.push('#### Containers\n');
      for (const cont of sys.containers) {
        writeElement(sections, cont.id, cont.label || cont.id, cont.description, cont.technology);
      }
      sections.push('');
    }

    // Data stores
    if (sys.datastores?.length) {
      sections.push('#### Data Stores\n');
      for (const ds of sys.datastores) {
        writeElement(sections, ds.id, ds.label || ds.id, ds.description, ds.technology);
      }
      sections.push('');
    }
  }

  return sections.join('\n');
}

// Render persons section
function renderPersons(arch: ArchitectureBody): string {
  if (!arch.persons?.length) return '';

  const sections: string[] = [];
  sections.push('## Persons\n');

  for (const person of arch.persons) {
    writeElement(sections, person.id, person.label || person.id, person.description);
  }
  sections.push('');

  return sections.join('\n');
}

// Render relations section (simplified)
function renderRelations(arch: ArchitectureBody): string {
  if (!arch.relations?.length) return '';

  const sections: string[] = [];
  sections.push('## Relations\n');

  for (const rel of arch.relations) {
    const label = rel.label || rel.verb || 'uses';
    sections.push(`- ${rel.from} → ${rel.to}: ${label}`);
  }
  sections.push('');

  return sections.join('\n');
}

// Render requirements section
function renderRequirements(arch: ArchitectureBody): string {
  if (!arch.requirements?.length) return '';

  const sections: string[] = [];
  sections.push('## Requirements\n');

  // Group by type
  const byType: Record<string, RequirementJSON[]> = {};
  for (const req of arch.requirements) {
    const type = req.type || 'functional';
    if (!byType[type]) {
      byType[type] = [];
    }
    byType[type].push(req);
  }

  for (const [type, reqs] of Object.entries(byType)) {
    sections.push(`### ${type.charAt(0).toUpperCase() + type.slice(1)} Requirements\n`);
    for (const req of reqs) {
      sections.push(`- **${req.id}**: ${req.title || req.description || ''}`);
      if (req.tags?.length) {
        sections.push(`  - Tags: ${req.tags.join(', ')}`);
      }
    }
    sections.push('');
  }

  return sections.join('\n');
}

// Render ADRs section
function renderADRs(arch: ArchitectureBody): string {
  if (!arch.adrs?.length) return '';

  const sections: string[] = [];
  sections.push('## Architecture Decision Records (ADRs)\n');

  for (const adr of arch.adrs) {
    sections.push(`### ${adr.title || adr.id}\n`);
    if (adr.status) {
      sections.push(`**Status**: ${adr.status}\n`);
    }
    if (adr.context) {
      sections.push(`**Context**: ${adr.context}\n`);
    }
    if (adr.decision) {
      sections.push(`**Decision**: ${adr.decision}\n`);
    }
    if (adr.consequences) {
      sections.push(`**Consequences**: ${adr.consequences}\n`);
    }
    sections.push('');
  }

  return sections.join('\n');
}

// Render constraints section
function renderConstraints(arch: ArchitectureBody): string {
  if (!arch.constraints?.length) return '';

  const sections: string[] = [];
  sections.push('## Constraints\n');

  for (const c of arch.constraints) {
    sections.push(`- **${c.key}**: ${c.value}`);
  }
  sections.push('');

  return sections.join('\n');
}

// Render conventions section
function renderConventions(arch: ArchitectureBody): string {
  if (!arch.conventions?.length) return '';

  const sections: string[] = [];
  sections.push('## Conventions\n');

  for (const c of arch.conventions) {
    sections.push(`- **${c.key}**: ${c.value}`);
  }
  sections.push('');

  return sections.join('\n');
}

// Render flows section
function renderFlows(arch: ArchitectureBody): string {
  if (!arch.flows?.length) return '';

  const sections: string[] = [];
  sections.push('## Flows\n');

  for (const flow of arch.flows) {
    sections.push(`### ${flow.title || flow.label || flow.id}\n`);
    if (flow.description) {
      sections.push(`${flow.description}\n`);
    }
    if (flow.steps?.length) {
      sections.push('**Steps:**\n');
      for (const step of flow.steps) {
        sections.push(`- ${step.from} → ${step.to}${step.description ? `: ${step.description}` : ''}`);
      }
    }
    sections.push('');
  }

  return sections.join('\n');
}

// Render scenarios section
function renderScenarios(
  arch: ArchitectureBody,
  config: MermaidConfig,
  mermaidGenerator?: (scenario: ScenarioJSON, arch: ArchitectureBody, config: MermaidConfig) => string
): string {
  if (!arch.scenarios?.length) return '';

  const sections: string[] = [];
  sections.push('## Scenarios\n');

  for (const scenario of arch.scenarios) {
    sections.push(`### ${scenario.title || scenario.label || scenario.id}\n`);
    if (scenario.description) {
      sections.push(`${scenario.description}\n`);
    }

    // Mermaid sequence diagram
    const diagram = mermaidGenerator
      ? mermaidGenerator(scenario, arch, config)
      : generateScenarioDiagram(scenario, config);
    if (diagram) {
      sections.push('```mermaid');
      sections.push(diagram);
      sections.push('```\n');
    }

    if (scenario.steps?.length) {
      sections.push('**Steps:**\n');
      for (const step of scenario.steps) {
        sections.push(`- ${step.from} → ${step.to}${step.description ? `: ${step.description}` : ''}`);
      }
    }
    sections.push('');
  }

  return sections.join('\n');
}

// Render policies section
function renderPolicies(arch: ArchitectureBody): string {
  if (!arch.policies?.length) return '';

  const sections: string[] = [];
  sections.push('## Policies\n');

  for (const policy of arch.policies) {
    sections.push(`### ${policy.label || policy.id}\n`);
    if (policy.description) {
      sections.push(`${policy.description}\n`);
    }
    if (policy.category) {
      sections.push(`**Category**: ${policy.category}\n`);
    }
    sections.push('');
  }

  return sections.join('\n');
}

// Render contracts section (simplified)
function renderContracts(arch: ArchitectureBody): string {
  if (!arch.contracts?.length) return '';

  const sections: string[] = [];
  sections.push('## Integration Contracts\n');

  for (const contract of arch.contracts) {
    sections.push(`### ${contract.label || contract.id}\n`);
    if (contract.kind) {
      sections.push(`**Kind**: ${contract.kind}\n`);
    }
    sections.push('');
  }

  return sections.join('\n');
}

// Render deployments section
function renderDeployments(
  arch: ArchitectureBody,
  config: MermaidConfig,
  mermaidGenerator?: (deployment: DeploymentNodeJSON, arch: ArchitectureBody, config: MermaidConfig) => string
): string {
  if (!arch.deployment?.length) return '';

  const sections: string[] = [];
  sections.push('## Deployment Architecture\n');

  for (const deployment of arch.deployment) {
    sections.push(`### ${deployment.label || deployment.id}\n`);
    const diagram = mermaidGenerator
      ? mermaidGenerator(deployment, arch, config)
      : generateDeploymentDiagram(deployment, config);
    if (diagram) {
      sections.push('```mermaid');
      sections.push(diagram);
      sections.push('```\n');
    }
  }

  return sections.join('\n');
}

// Main export function
export function exportToMarkdown(
  archJson: ArchitectureJSON,
  options?: MarkdownExportOptions,
  mermaidGenerator?: {
    system?: (arch: ArchitectureBody, config: MermaidConfig) => string;
    container?: (sys: SystemJSON, arch: ArchitectureBody, config: MermaidConfig) => string;
    scenario?: (scenario: ScenarioJSON, arch: ArchitectureBody, config: MermaidConfig) => string;
    deployment?: (deployment: DeploymentNodeJSON, arch: ArchitectureBody, config: MermaidConfig) => string;
  }
): string {
  const opts = { ...DEFAULT_OPTIONS, ...options };
  const arch = archJson.architecture;
  const mermaidConfig = opts.mermaidConfig || extractMermaidConfig(arch);
  
  // Use provided mermaid generators or use default implementations
  const mermaidGen = mermaidGenerator || {
    system: (arch: ArchitectureBody, config: MermaidConfig) => generateSystemDiagram(arch, config),
    container: (sys: SystemJSON, _arch: ArchitectureBody, config: MermaidConfig) =>
      generateSystemContainerDiagram(sys, arch, config),
    scenario: (_scenario: ScenarioJSON, _arch: ArchitectureBody, config: MermaidConfig) =>
      generateScenarioDiagram(_scenario, config),
    deployment: (deployment: DeploymentNodeJSON, _arch: ArchitectureBody, config: MermaidConfig) =>
      generateDeploymentDiagram(deployment, config),
  };

  const sections: string[] = [];

  // Header
  sections.push(`# ${arch.name || archJson.metadata.name}\n`);
  if (arch.description) {
    sections.push(`${arch.description}\n`);
  }
  sections.push('');

  // TOC
  if (opts.includeTOC) {
    sections.push(renderTOC(arch, opts));
  }

  // Executive Summary
  sections.push(renderExecutiveSummary(arch));

  // Overview
  if (opts.includeOverview) {
    sections.push(renderOverview(arch, mermaidConfig, mermaidGen.system));
  }

  // Systems
  if (opts.includeSystems) {
    sections.push(renderSystems(arch, mermaidConfig, mermaidGen.container));
  }

  // Deployments
  if (opts.includeDeployments) {
    sections.push(renderDeployments(arch, mermaidConfig, mermaidGen.deployment));
  }

  // Persons
  if (opts.includePersons) {
    sections.push(renderPersons(arch));
  }

  // Requirements
  if (opts.includeRequirements) {
    sections.push(renderRequirements(arch));
  }

  // ADRs
  if (opts.includeADRs) {
    sections.push(renderADRs(arch));
  }

  // Scenarios
  if (opts.includeScenarios) {
    sections.push(renderScenarios(arch, mermaidConfig, mermaidGen.scenario));
  }

  // Policies
  if (opts.includePolicies) {
    sections.push(renderPolicies(arch));
  }

  // Constraints
  if (opts.includeConstraints) {
    sections.push(renderConstraints(arch));
  }

  // Conventions
  if (opts.includeConventions) {
    sections.push(renderConventions(arch));
  }

  // Flows
  if (opts.includeFlows) {
    sections.push(renderFlows(arch));
  }

  // Contracts
  if (opts.includeContracts) {
    sections.push(renderContracts(arch));
  }

  // Quality Attributes (placeholder)
  if (opts.includeQualityAttributes) {
    sections.push('## Quality Attributes\n\n');
  }

  // Security (placeholder)
  if (opts.includeSecurity) {
    sections.push('## Security\n\n');
  }

  // Relations
  if (opts.includeRelations) {
    sections.push(renderRelations(arch));
  }

  // Metadata (placeholder)
  if (opts.includeMetadata) {
    sections.push('## Document Metadata\n\n');
  }

  // Glossary (placeholder)
  if (opts.includeGlossary) {
    sections.push('## Glossary\n\n');
  }

  return sections.join('\n');
}
