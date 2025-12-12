// packages/shared/src/export/mermaid.ts
// TypeScript mermaid exporter - ported from pkg/export/mermaid
// Works with ArchitectureJSON from Go parser

import type {
  ArchitectureJSON,
  ArchitectureBody,
  SystemJSON,
  ContainerJSON,
  PersonJSON,
  DataStoreJSON,
  QueueJSON,
  RelationJSON,
  ScenarioJSON,
  DeploymentNodeJSON,
} from '../types/architecture';
// Note: ComponentJSON and ScenarioStepJSON are used via arrays (container.components, scenario.steps)
// but TypeScript doesn't detect them as used, so we import them implicitly

export interface MermaidConfig {
  layout?: string;
  theme?: string;
  look?: string;
  direction?: string;
  useFrontmatter?: boolean;
}

export type DiagramType = 'graph' | 'sequence' | 'deployment';
export type ViewType = 'system' | 'container' | 'component' | 'all';

export interface MermaidExporterOptions {
  diagramType?: DiagramType;
  viewType?: ViewType;
  systemID?: string;
  containerID?: string;
  scenarioID?: string;
  deploymentID?: string;
  mermaidConfig?: MermaidConfig;
}

// Styles
const PERSON_STYLE = 'fill:#ffcccc,stroke:#333,stroke-width:2px,color:#000';
const SYSTEM_STYLE = 'fill:#cce5ff,stroke:#333,stroke-width:2px,color:#000';
const CONTAINER_STYLE = 'fill:#cce5ff,stroke:#333,stroke-width:2px,color:#000';
const DATABASE_STYLE = 'fill:#ccffcc,stroke:#333,stroke-width:2px,color:#000';
const QUEUE_STYLE = 'fill:#ffe5cc,stroke:#333,stroke-width:2px,color:#000';
const EXTERNAL_STYLE = 'fill:#eeeeee,stroke:#666,stroke-width:2px,color:#000,stroke-dasharray: 3 3';
const COMPONENT_STYLE = 'fill:#e6f7ff,stroke:#333,stroke-width:2px,color:#000';

// LookupIndex for O(1) access to architecture elements
interface LookupIndex {
  systems: Map<string, SystemJSON>;
  persons: Map<string, PersonJSON>;
  containers: Map<string, ContainerJSON>;
  datastores: Map<string, DataStoreJSON>;
  queues: Map<string, QueueJSON>;
  containerToSystem: Map<string, string>;
}

function newLookupIndex(arch: ArchitectureBody): LookupIndex {
  const idx: LookupIndex = {
    systems: new Map(),
    persons: new Map(),
    containers: new Map(),
    datastores: new Map(),
    queues: new Map(),
    containerToSystem: new Map(),
  };

  if (!arch) return idx;

  // Index persons
  for (const p of arch.persons || []) {
    idx.persons.set(p.id, p);
  }

  // Index systems and their containers/datastores/queues
  for (const sys of arch.systems || []) {
    idx.systems.set(sys.id, sys);
    for (const c of sys.containers || []) {
      idx.containers.set(c.id, c);
      idx.containerToSystem.set(c.id, sys.id);
    }
    for (const d of sys.datastores || []) {
      idx.datastores.set(d.id, d);
      idx.containerToSystem.set(d.id, sys.id);
    }
    for (const q of sys.queues || []) {
      idx.queues.set(q.id, q);
      idx.containerToSystem.set(q.id, sys.id);
    }
  }

  return idx;
}

// Extract mermaid config from architecture
export function extractMermaidConfig(arch: ArchitectureBody): MermaidConfig {
  const config: MermaidConfig = {
    layout: 'elk',
    theme: 'default',
    look: '',
    direction: 'LR',
    useFrontmatter: false,
  };

  if (!arch) return config;

  // Check style block
  if (arch.style) {
    const layout = arch.style.mermaid_layout;
    if (layout) {
      const lower = layout.toLowerCase();
      if (lower === 'elk' || lower === 'dagre') {
        config.layout = lower;
      } else if (['lr', 'tb', 'bt', 'rl'].includes(lower)) {
        config.direction = lower.toUpperCase();
      }
    }
    if (arch.style.mermaid_direction) {
      const dir = arch.style.mermaid_direction.toLowerCase();
      if (['lr', 'tb', 'bt', 'rl'].includes(dir)) {
        config.direction = dir.toUpperCase();
      }
    }
    if (arch.style.mermaid_theme) {
      config.theme = arch.style.mermaid_theme;
    }
    if (arch.style.mermaid_look) {
      config.look = arch.style.mermaid_look;
    }
    if (arch.style.mermaid_frontmatter) {
      config.useFrontmatter = arch.style.mermaid_frontmatter.toLowerCase() === 'true';
    }
  }

  // Check metadata
  if (arch.archMetadata) {
    for (const meta of arch.archMetadata) {
      if (!meta.value) continue;
      const val = meta.value;
      switch (meta.key) {
        case 'mermaid_layout':
          const lower = val.toLowerCase();
          if (lower === 'elk' || lower === 'dagre') {
            config.layout = lower;
          } else if (['lr', 'tb', 'bt', 'rl'].includes(lower)) {
            config.direction = lower.toUpperCase();
          }
          break;
        case 'mermaid_direction':
          const dir = val.toLowerCase();
          if (['lr', 'tb', 'bt', 'rl'].includes(dir)) {
            config.direction = dir.toUpperCase();
          }
          break;
        case 'mermaid_theme':
          config.theme = val;
          break;
        case 'mermaid_look':
          config.look = val;
          break;
        case 'mermaid_frontmatter':
          config.useFrontmatter = val.toLowerCase() === 'true';
          break;
      }
    }
  }

  // Check properties
  if (arch.properties?.mermaid_frontmatter) {
    config.useFrontmatter = arch.properties.mermaid_frontmatter.toLowerCase() === 'true';
  }

  return config;
}

// Helper functions
function sanitizeNodeID(id: string): string {
  const buf: string[] = [];
  for (const r of id) {
    if ((r >= 'a' && r <= 'z') || (r >= 'A' && r <= 'Z') || (r >= '0' && r <= '9')) {
      buf.push(r);
    } else if (r === '_' || r === ' ' || r === '-' || r === '.') {
      buf.push(r === '_' ? r : '_');
    }
    // Drop unsupported characters
  }
  return buf.join('');
}

function escapeQuotes(s: string): string {
  return s.replace(/"/g, '\\"');
}

function formatNodeLabel(
  label: string | undefined,
  id: string,
  description: string | undefined,
  technology: string | undefined
): string {
  const parts: string[] = [label || id];
  if (technology) {
    parts.push(`(${technology})`);
  }
  if (description) {
    const desc = description.length > 50 ? description.substring(0, 47) + '...' : description;
    parts.push(desc);
  }
  return parts.join('\\n');
}

function graphDirection(config: MermaidConfig): string {
  const dir = (config.direction || 'LR').toUpperCase();
  return ['LR', 'TB', 'BT', 'RL'].includes(dir) ? dir : 'LR';
}

function writeMermaidConfig(sections: string[], config: MermaidConfig): void {
  if (config.useFrontmatter) {
    sections.push('---');
    sections.push('config:');
    if (config.layout) {
      sections.push(`  layout: ${config.layout}`);
    }
    if (config.theme && config.theme !== 'default') {
      sections.push(`  theme: ${config.theme}`);
    }
    if (config.direction) {
      sections.push(`  direction: ${config.direction.toLowerCase()}`);
    }
    if (config.look) {
      sections.push(`  look: ${config.look}`);
    }
    sections.push('  flowchart:');
    sections.push('    subGraphTitleMargin:');
    sections.push('      top: 10');
    sections.push('      bottom: 20');
    sections.push('---');
  } else {
    const theme = (config.theme || 'default').trim();
    sections.push(
      `%%{init: { "theme": "${theme}", "flowchart": { "htmlLabels": true, "nodeSpacing": 50, "rankSpacing": 50, "subGraphTitleMargin": { "top": 10, "bottom": 20 } }, "themeVariables": { "fontSize": "16px", "textHeight": 20 } }}%%`
    );
  }
}

function writeMermaidStyles(sections: string[]): void {
  sections.push(`    classDef personStyle ${PERSON_STYLE}`);
  sections.push(`    classDef systemStyle ${SYSTEM_STYLE}`);
  sections.push(`    classDef containerStyle ${CONTAINER_STYLE}`);
  sections.push(`    classDef databaseStyle ${DATABASE_STYLE}`);
  sections.push(`    classDef queueStyle ${QUEUE_STYLE}`);
  sections.push(`    classDef externalStyle ${EXTERNAL_STYLE}`);
  sections.push(`    classDef componentStyle ${COMPONENT_STYLE}`);
  sections.push('');
}

function relationLabel(rel: RelationJSON): string {
  const label = rel.verb || rel.label || '';
  return escapeQuotes(label);
}

function renderEdge(sections: string[], from: string, to: string, label: string): void {
  const fromID = sanitizeNodeID(from);
  const toID = sanitizeNodeID(to);
  if (label) {
    sections.push(`    ${fromID} -->|${label}| ${toID}`);
  } else {
    sections.push(`    ${fromID} --> ${toID}`);
  }
}

// Get container technology (from first item with technology)
function getContainerTechnology(cont: ContainerJSON): string | undefined {
  // In JSON, technology is directly on container
  return cont.technology;
}

// Render container node
function renderContainerNode(
  sections: string[],
  idx: LookupIndex,
  containerID: string,
  defaultLabel: string,
  indent: string
): void {
  const nodeID = sanitizeNodeID(containerID);
  let label = defaultLabel || containerID;
  let styleName = 'containerStyle';
  let shapeOpen = '[';
  let shapeClose = ']';

  const ds = idx.datastores.get(containerID);
  const q = idx.queues.get(containerID);
  const cont = idx.containers.get(containerID);

  if (ds) {
    styleName = 'databaseStyle';
    shapeOpen = '[("';
    shapeClose = '")]';
    label = formatNodeLabel(ds.label, ds.id, ds.description, ds.technology);
  } else if (q) {
    styleName = 'queueStyle';
    label = formatNodeLabel(q.label, q.id, q.description, q.technology);
  } else if (cont) {
    const tech = getContainerTechnology(cont);
    label = formatNodeLabel(cont.label, cont.id, cont.description, tech);
  }

  label = escapeQuotes(label);

  if (styleName === 'databaseStyle') {
    sections.push(`${indent}${nodeID}${shapeOpen}${label}${shapeClose}`);
  } else {
    sections.push(`${indent}${nodeID}${shapeOpen}"${label}"${shapeClose}`);
  }
  sections.push(`${indent}class ${nodeID} ${styleName}`);
}

// Generate system diagram (C4 L1)
export function generateSystemDiagram(arch: ArchitectureBody, config: MermaidConfig): string {
  const idx = newLookupIndex(arch);
  const sections: string[] = [];

  writeMermaidConfig(sections, config);
  sections.push(`graph ${graphDirection(config)}`);
  sections.push('');
  writeMermaidStyles(sections);

  // Empty architecture check
  const hasElements =
    (arch.systems?.length || 0) +
      (arch.persons?.length || 0) +
      (arch.containers?.length || 0) +
      (arch.datastores?.length || 0) +
      (arch.queues?.length || 0) >
    0;

  if (!hasElements && arch.name) {
    const archID = sanitizeNodeID(arch.name);
    const archLabel = escapeQuotes(arch.name);
    sections.push(`    ${archID}["${archLabel}"]`);
    sections.push(`    class ${archID} systemStyle`);
    return sections.join('\n');
  }

  // Track containers to show
  const containerNodes = new Set<string>();
  const addContainerNode = (id: string) => containerNodes.add(id);

  // Helper to split dotted IDs
  const splitDotted = (s: string): [string, string, boolean] => {
    const firstDot = s.indexOf('.');
    if (firstDot === -1) return [s, s, false];
    const sys = s.substring(0, firstDot);
    const lastDot = s.lastIndexOf('.');
    const leaf = s.substring(lastDot + 1);
    return [sys, leaf, true];
  };

  // Analyze relations to determine which containers to show
  for (const rel of arch.relations || []) {
    const fromStr = rel.from;
    const toStr = rel.to;
    const [fromSys, fromContainer, fromHasDot] = splitDotted(fromStr);
    const [toSys, toContainer, toHasDot] = splitDotted(toStr);

    if (fromHasDot && toHasDot) {
      if (fromSys === toSys) {
        if (fromContainer !== toContainer && idx.systems.has(fromSys)) {
          addContainerNode(fromContainer);
          addContainerNode(toContainer);
        }
      } else {
        if (idx.systems.has(fromSys)) addContainerNode(fromContainer);
        if (idx.systems.has(toSys)) addContainerNode(toContainer);
      }
    } else if (fromHasDot && idx.systems.has(fromSys)) {
      addContainerNode(fromContainer);
    } else if (toHasDot && idx.systems.has(toSys)) {
      addContainerNode(toContainer);
    }
  }

  // Ensure all defined containers/datastores/queues are shown
  for (const sys of arch.systems || []) {
    for (const c of sys.containers || []) addContainerNode(c.id);
    for (const d of sys.datastores || []) addContainerNode(d.id);
    for (const q of sys.queues || []) addContainerNode(q.id);
  }

  // Render persons
  for (const person of arch.persons || []) {
    const nodeID = sanitizeNodeID(person.id);
    let label = formatNodeLabel(person.label, person.id, person.description, undefined);
    label = escapeQuotes(label);
    sections.push(`    ${nodeID}["${label}"]`);
    sections.push(`    class ${nodeID} personStyle`);
  }

  // Group containers by system
  const systemContainers = new Map<string, string[]>();
  for (const containerID of containerNodes) {
    const sysID = idx.containerToSystem.get(containerID);
    if (sysID) {
      const list = systemContainers.get(sysID) || [];
      list.push(containerID);
      systemContainers.set(sysID, list);
    }
  }

  // Render systems with containers
  for (const sys of arch.systems || []) {
    const nodeID = sanitizeNodeID(sys.id);
    const containerIDs = systemContainers.get(sys.id);

    if (containerIDs && containerIDs.length > 0) {
      let label = formatNodeLabel(sys.label, sys.id, undefined, undefined);
      label = escapeQuotes(label);
      sections.push(`    subgraph ${nodeID}["${label}"]`);
      for (const containerID of containerIDs) {
        renderContainerNode(sections, idx, containerID, '', '        ');
      }
      sections.push('    end');
    } else {
      let label = formatNodeLabel(sys.label, sys.id, sys.description, undefined);
      label = escapeQuotes(label);
      sections.push(`    ${nodeID}["${label}"]`);
      sections.push(`    class ${nodeID} systemStyle`);
    }
  }

  // Render standalone containers
  for (const containerID of containerNodes) {
    if (!idx.containerToSystem.has(containerID)) {
      renderContainerNode(sections, idx, containerID, '', '    ');
    }
  }

  sections.push('');

  // Render edges (deduplicated)
  const edgeSet = new Set<string>();

  for (const rel of arch.relations || []) {
    const resolveNode = (fullID: string): [string, boolean] => {
      const lastDot = fullID.lastIndexOf('.');
      const leaf = lastDot !== -1 ? fullID.substring(lastDot + 1) : fullID;

      if (containerNodes.has(leaf)) {
        return [sanitizeNodeID(leaf), true];
      }
      if (idx.persons.has(fullID) || idx.systems.has(fullID)) {
        return [sanitizeNodeID(fullID), true];
      }
      // Fallback to system
      const parts = fullID.split('.');
      if (parts.length >= 2) {
        return [sanitizeNodeID(parts[0]), true];
      }
      const sys = idx.containerToSystem.get(fullID);
      if (sys) {
        return [sanitizeNodeID(sys), true];
      }
      return ['', false];
    };

    const [fromID, okF] = resolveNode(rel.from);
    const [toID, okT] = resolveNode(rel.to);

    if (okF && okT && fromID !== toID) {
      const label = relationLabel(rel);
      const key = `${fromID}->${toID}:${label}`;
      if (!edgeSet.has(key)) {
        edgeSet.add(key);
        renderEdge(sections, fromID, toID, label);
      }
    }
  }

  return sections.join('\n');
}

// Generate system container diagram (C4 L2)
export function generateSystemContainerDiagram(
  sys: SystemJSON,
  arch: ArchitectureBody,
  config: MermaidConfig
): string {
  const sections: string[] = [];

  writeMermaidConfig(sections, config);
  sections.push(`graph ${graphDirection(config)}`);
  sections.push('');
  writeMermaidStyles(sections);

  const sysID = sanitizeNodeID(sys.id);
  let sysLabel = formatNodeLabel(sys.label, sys.id, undefined, undefined);
  sysLabel = escapeQuotes(sysLabel);
  sections.push(`    subgraph ${sysID}["${sysLabel}"]`);

  // Render containers
  for (const cont of sys.containers || []) {
    const contID = sanitizeNodeID(cont.id);
    const tech = getContainerTechnology(cont);

    if ((cont.components?.length || 0) > 0) {
      let contLabel = formatNodeLabel(cont.label, cont.id, undefined, tech);
      contLabel = escapeQuotes(contLabel);
      sections.push(`        subgraph ${contID}["${contLabel}"]`);
      for (const comp of cont.components || []) {
        const compID = sanitizeNodeID(comp.id);
        let compLabel = formatNodeLabel(comp.label, comp.id, comp.description, comp.technology);
        compLabel = escapeQuotes(compLabel);
        sections.push(`            ${compID}["${compLabel}"]`);
        sections.push(`            class ${compID} componentStyle`);
      }
      sections.push('        end');
    } else {
      let contLabel = formatNodeLabel(cont.label, cont.id, cont.description, tech);
      contLabel = escapeQuotes(contLabel);
      sections.push(`        ${contID}["${contLabel}"]`);
      sections.push(`        class ${contID} containerStyle`);
    }
  }

  // Render datastores and queues
  for (const ds of sys.datastores || []) {
    const nodeID = sanitizeNodeID(ds.id);
    let label = formatNodeLabel(ds.label, ds.id, ds.description, ds.technology);
    label = escapeQuotes(label);
    sections.push(`        ${nodeID}[("${label}")]`);
    sections.push(`        class ${nodeID} databaseStyle`);
  }
  for (const q of sys.queues || []) {
    const nodeID = sanitizeNodeID(q.id);
    let label = formatNodeLabel(q.label, q.id, q.description, q.technology);
    label = escapeQuotes(label);
    sections.push(`        ${nodeID}["${label}"]`);
    sections.push(`        class ${nodeID} queueStyle`);
  }
  sections.push('    end');
  sections.push('');

  // Build component to container map
  const compToContainer = new Map<string, string>();
  for (const cont of sys.containers || []) {
    for (const comp of cont.components || []) {
      compToContainer.set(comp.id, cont.id);
    }
  }

  // Resolve system element
  const resolveSystemElement = (name: string): [string, boolean] => {
    const parts = name.split('.');
    const candidate = parts[parts.length - 1];

    // Check components
    for (const cont of sys.containers || []) {
      for (const comp of cont.components || []) {
        if (comp.id === candidate) return [candidate, true];
      }
    }

    // Check containers
    for (const c of sys.containers || []) {
      if (c.id === candidate) return [candidate, true];
    }

    // Check datastores
    for (const ds of sys.datastores || []) {
      if (ds.id === candidate) return [candidate, true];
    }

    // Check queues
    for (const q of sys.queues || []) {
      if (q.id === candidate) return [candidate, true];
    }

    // Fallback to container if component mapped
    const parent = compToContainer.get(candidate);
    if (parent) return [parent, true];

    return ['', false];
  };

  // Collect edges
  const edgeSet = new Set<string>();
  const addEdge = (fromRaw: string, toRaw: string, lbl: string): void => {
    const [fromName, okFrom] = resolveSystemElement(fromRaw);
    const [toName, okTo] = resolveSystemElement(toRaw);
    if (!okFrom || !okTo) return;
    const key = `${fromName}=>${toName}|${lbl}`;
    edgeSet.add(key);
  };

  for (const rel of sys.relations || []) {
    addEdge(rel.from, rel.to, relationLabel(rel));
  }
  if (arch) {
    for (const rel of arch.relations || []) {
      addEdge(rel.from, rel.to, relationLabel(rel));
    }
  }
  for (const cont of sys.containers || []) {
    for (const rel of cont.relations || []) {
      addEdge(rel.from, rel.to, relationLabel(rel));
    }
    for (const comp of cont.components || []) {
      for (const rel of comp.relations || []) {
        addEdge(rel.from, rel.to, relationLabel(rel));
      }
    }
  }

  // Render edges
  for (const key of edgeSet) {
    const parts = key.split('=>');
    if (parts.length !== 2) continue;
    const fromName = parts[0];
    const seg = parts[1].split('|');
    const toName = seg[0];
    const label = seg.length > 1 ? seg[1] : '';
    renderEdge(sections, fromName, toName, label);
  }

  // TODO: Render external relations (simplified for now)
  return sections.join('\n');
}

// Generate container component diagram (C4 L3)
export function generateContainerComponentDiagram(
  container: ContainerJSON,
  sys: SystemJSON,
  _arch: ArchitectureBody,
  config: MermaidConfig
): string {
  const sections: string[] = [];

  writeMermaidConfig(sections, config);
  sections.push(`graph ${graphDirection(config)}`);
  sections.push('');
  writeMermaidStyles(sections);

  // Render component nodes
  const compIDs = new Set<string>();
  for (const comp of container.components || []) {
    const compID = sanitizeNodeID(comp.id);
    let compLabel = formatNodeLabel(comp.label, comp.id, comp.description, comp.technology);
    compLabel = escapeQuotes(compLabel);
    sections.push(`    ${compID}["${compLabel}"]`);
    sections.push(`    class ${compID} componentStyle`);
    compIDs.add(comp.id);
  }

  // Build system element IDs map
  const sysElementIDs = new Map<string, string>();
  for (const ds of sys.datastores || []) {
    sysElementIDs.set(ds.id, 'db');
  }
  for (const q of sys.queues || []) {
    sysElementIDs.set(q.id, 'queue');
  }
  for (const c of sys.containers || []) {
    sysElementIDs.set(c.id, 'container');
  }

  // Add external nodes
  const externalAdded = new Set<string>();
  const addExternalNode = (name: string): string => {
    const parts = name.split('.');
    const base = parts[parts.length - 1];
    const kind = sysElementIDs.get(base);
    if (kind && !externalAdded.has(base)) {
      const id = sanitizeNodeID(base);
      const safeBase = escapeQuotes(base);
      switch (kind) {
        case 'db':
          sections.push(`    ${id}[("${safeBase}")]`);
          sections.push(`    class ${id} databaseStyle`);
          break;
        case 'queue':
          sections.push(`    ${id}["${safeBase}"]`);
          sections.push(`    class ${id} queueStyle`);
          break;
        case 'container':
          sections.push(`    ${id}["${safeBase}"]`);
          sections.push(`    class ${id} containerStyle`);
          break;
      }
      externalAdded.add(base);
      return base;
    }
    return '';
  };

  // Collect edges
  const edgeSet = new Set<string>();
  for (const rel of container.relations || []) {
    const fromParts = rel.from.split('.');
    const toParts = rel.to.split('.');
    const from = fromParts[fromParts.length - 1];
    const to = toParts[toParts.length - 1];
    let fromNode = compIDs.has(from) ? from : '';
    let toNode = compIDs.has(to) ? to : '';
    if (!fromNode) fromNode = addExternalNode(rel.from);
    if (!toNode) toNode = addExternalNode(rel.to);
    if (fromNode && toNode) {
      const key = `${fromNode}=>${toNode}|${relationLabel(rel)}`;
      edgeSet.add(key);
    }
  }

  // Render edges
  for (const key of edgeSet) {
    const parts = key.split('=>');
    if (parts.length !== 2) continue;
    const from = parts[0];
    const seg = parts[1].split('|');
    const to = seg[0];
    const label = seg.length > 1 ? seg[1] : '';
    renderEdge(sections, from, to, label);
  }

  // TODO: Render external relations (simplified for now)
  return sections.join('\n');
}

// Generate scenario diagram (sequence)
export function generateScenarioDiagram(scenario: ScenarioJSON, config: MermaidConfig): string {
  const sections: string[] = [];
  writeMermaidConfig(sections, config);
  sections.push('sequenceDiagram');

  // Collect participants
  const participants = new Set<string>();
  for (const step of scenario.steps || []) {
    participants.add(step.from);
    participants.add(step.to);
  }

  // Render participants
  for (const participant of participants) {
    sections.push(`    participant ${sanitizeNodeID(participant)}`);
  }

  // Render steps
  for (const step of scenario.steps || []) {
    const fromID = sanitizeNodeID(step.from);
    const toID = sanitizeNodeID(step.to);
    const label = step.description || 'interaction';
    sections.push(`    ${fromID}->>${toID}: ${label}`);
  }

  return sections.join('\n');
}

// Generate deployment diagram
export function generateDeploymentDiagram(
  root: DeploymentNodeJSON,
  config: MermaidConfig
): string {
  const sections: string[] = [];
  writeMermaidConfig(sections, config);
  sections.push(`graph ${graphDirection(config)}`);
  sections.push('');

  // Recursive helper to add deployment subgraphs
  const addDeploymentSubgraphs = (node: DeploymentNodeJSON, indent: number): void => {
    const indentStr = '    '.repeat(indent);
    const nodeID = sanitizeNodeID(node.id);
    let label = node.label || node.id;
    label = escapeQuotes(label);

    if (label.includes(' ') || label.includes('-') || label !== nodeID) {
      sections.push(`${indentStr}subgraph ${nodeID}["${label}"]`);
    } else {
      sections.push(`${indentStr}subgraph ${nodeID}[${label}]`);
    }

    // Note: ContainerInstances and Infrastructure not in JSON types yet
    // This is a simplified version
    sections.push(`${indentStr}end`);
  };

  addDeploymentSubgraphs(root, 0);
  return sections.join('\n');
}

// Main exporter class
export class MermaidExporter {
  diagramType: DiagramType = 'graph';
  viewType: ViewType = 'system';
  systemID?: string;
  containerID?: string;
  scenarioID?: string;
  deploymentID?: string;
  mermaidConfig: MermaidConfig = { direction: 'LR', theme: 'default' };

  constructor(options?: MermaidExporterOptions) {
    if (options) {
      this.diagramType = options.diagramType || 'graph';
      this.viewType = options.viewType || 'system';
      this.systemID = options.systemID;
      this.containerID = options.containerID;
      this.scenarioID = options.scenarioID;
      this.deploymentID = options.deploymentID;
      this.mermaidConfig = options.mermaidConfig || { direction: 'LR', theme: 'default' };
    }
  }

  export(archJson: ArchitectureJSON): string {
    const arch = archJson.architecture;
    if (!arch) {
      throw new Error('architecture is nil');
    }

    const config = this.extractMermaidConfig(arch);
    let code = '';

    switch (this.diagramType) {
      case 'sequence':
        code = this.exportSequence(arch);
        break;
      case 'deployment':
        code = this.exportDeployment(arch);
        break;
      default:
        code = this.exportGraph(arch, config);
    }

    const count = this.countNodes(arch);
    const header = `%% node_count: ${count} %%\n`;
    return header + code;
  }

  private exportGraph(arch: ArchitectureBody, config: MermaidConfig): string {
    switch (this.viewType) {
      case 'container':
        if (!this.systemID) {
          throw new Error('system ID required for container view');
        }
        const sys = this.findSystem(arch, this.systemID);
        if (!sys) {
          throw new Error(`system not found: ${this.systemID}`);
        }
        return generateSystemContainerDiagram(sys, arch, config);

      case 'component':
        if (!this.systemID || !this.containerID) {
          throw new Error('system ID and container ID required for component view');
        }
        const system = this.findSystem(arch, this.systemID);
        if (!system) {
          throw new Error(`system not found: ${this.systemID}`);
        }
        const container = this.findContainer(system, this.containerID);
        if (!container) {
          throw new Error(`container not found: ${this.containerID} in system ${this.systemID}`);
        }
        return generateContainerComponentDiagram(container, system, arch, config);

      case 'all':
      case 'system':
      default:
        return generateSystemDiagram(arch, config);
    }
  }

  private exportSequence(arch: ArchitectureBody): string {
    if (!this.scenarioID) {
      if (!arch.scenarios || arch.scenarios.length === 0) {
        throw new Error('no scenarios found in architecture');
      }
      this.scenarioID = arch.scenarios[0].id;
    }

    const scenario = this.findScenario(arch, this.scenarioID);
    if (!scenario) {
      throw new Error(`scenario not found: ${this.scenarioID}`);
    }

    const config = this.extractMermaidConfig(arch);
    return generateScenarioDiagram(scenario, config);
  }

  private exportDeployment(_arch: ArchitectureBody): string {
    if (!this.deploymentID) {
      if (!_arch.deployment || _arch.deployment.length === 0) {
        throw new Error('no deployment nodes found in architecture');
      }
      this.deploymentID = _arch.deployment[0].id;
    }

    const deployment = this.findDeployment(_arch, this.deploymentID);
    if (!deployment) {
      throw new Error(`deployment node not found: ${this.deploymentID}`);
    }

    const config = this.extractMermaidConfig(_arch);
    return generateDeploymentDiagram(deployment, config);
  }

  private extractMermaidConfig(arch: ArchitectureBody): MermaidConfig {
    const cfg = extractMermaidConfig(arch);
    // Overlay explicit config
    if (this.mermaidConfig.layout) cfg.layout = this.mermaidConfig.layout;
    if (this.mermaidConfig.theme) cfg.theme = this.mermaidConfig.theme;
    if (this.mermaidConfig.look) cfg.look = this.mermaidConfig.look;
    if (this.mermaidConfig.direction) cfg.direction = this.mermaidConfig.direction;
    if (this.mermaidConfig.useFrontmatter !== undefined) {
      cfg.useFrontmatter = this.mermaidConfig.useFrontmatter;
    }
    return cfg;
  }

  private findSystem(arch: ArchitectureBody, id: string): SystemJSON | undefined {
    return arch.systems?.find((s) => s.id === id);
  }

  private findContainer(sys: SystemJSON, id: string): ContainerJSON | undefined {
    return sys.containers?.find((c) => c.id === id);
  }

  private findScenario(arch: ArchitectureBody, id: string): ScenarioJSON | undefined {
    return arch.scenarios?.find((s) => s.id === id);
  }

  private findDeployment(arch: ArchitectureBody, id: string): DeploymentNodeJSON | undefined {
    return arch.deployment?.find((d) => d.id === id);
  }

  private countNodes(arch: ArchitectureBody): number {
    if (!arch) return 0;

    switch (this.diagramType) {
      case 'sequence':
        if (!this.scenarioID && arch.scenarios && arch.scenarios.length > 0) {
          const scenario = arch.scenarios[0];
          const seen = new Set<string>();
          for (const step of scenario.steps || []) {
            seen.add(step.from);
            seen.add(step.to);
          }
          return seen.size;
        }
        const scenario = this.findScenario(arch, this.scenarioID || '');
        if (!scenario) return 0;
        const participants = new Set<string>();
        for (const step of scenario.steps || []) {
          participants.add(step.from);
          participants.add(step.to);
        }
        return participants.size;

      case 'deployment':
        // Simplified - deployment counting would need full deployment node structure
        return 1;

      default:
        switch (this.viewType) {
          case 'component':
            const sys = this.findSystem(arch, this.systemID || '');
            if (!sys) {
              return (arch.persons?.length || 0) + (arch.systems?.length || 0);
            }
            let n = 0;
            if (this.containerID) {
              const cont = this.findContainer(sys, this.containerID);
              if (cont) {
                n += cont.components?.length || 0;
              }
            }
            n += (sys.containers?.length || 0) + (sys.datastores?.length || 0) + (sys.queues?.length || 0);
            return n;

          case 'container':
            const system = this.findSystem(arch, this.systemID || '');
            if (!system) {
              return (arch.persons?.length || 0) + (arch.systems?.length || 0);
            }
            return (
              (system.containers?.length || 0) +
              (system.datastores?.length || 0) +
              (system.queues?.length || 0)
            );

          case 'all':
            let total = (arch.persons?.length || 0) + (arch.systems?.length || 0);
            for (const s of arch.systems || []) {
              total +=
                (s.containers?.length || 0) + (s.datastores?.length || 0) + (s.queues?.length || 0);
            }
            return total;

          default:
            let count = (arch.persons?.length || 0) + (arch.systems?.length || 0);
            for (const s of arch.systems || []) {
              count +=
                (s.containers?.length || 0) + (s.datastores?.length || 0) + (s.queues?.length || 0);
            }
            return count;
        }
    }
  }
}

// Convenience functions matching Go API
export function generateSystemDiagramForArch(
  archJson: ArchitectureJSON,
  config?: MermaidConfig
): string {
  const arch = archJson.architecture;
  const cfg = config || extractMermaidConfig(arch);
  return generateSystemDiagram(arch, cfg);
}

export function generateSystemContainerDiagramForArch(
  sys: SystemJSON,
  archJson: ArchitectureJSON,
  config?: MermaidConfig
): string {
  const arch = archJson.architecture;
  const cfg = config || extractMermaidConfig(arch);
  return generateSystemContainerDiagram(sys, arch, cfg);
}

export function generateContainerComponentDiagramForArch(
  container: ContainerJSON,
  sys: SystemJSON,
  archJson: ArchitectureJSON,
  config?: MermaidConfig
): string {
  const arch = archJson.architecture;
  const cfg = config || extractMermaidConfig(arch);
  return generateContainerComponentDiagram(container, sys, arch, cfg);
}

export function generateScenarioDiagramForArch(
  scenario: ScenarioJSON,
  archJson: ArchitectureJSON,
  config?: MermaidConfig
): string {
  const arch = archJson.architecture;
  const cfg = config || extractMermaidConfig(arch);
  return generateScenarioDiagram(scenario, cfg);
}

export function generateDeploymentDiagramForArch(
  deployment: DeploymentNodeJSON,
  archJson: ArchitectureJSON,
  config?: MermaidConfig
): string {
  const arch = archJson.architecture;
  const cfg = config || extractMermaidConfig(arch);
  return generateDeploymentDiagram(deployment, cfg);
}
