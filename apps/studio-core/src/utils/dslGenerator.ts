
/**
 * Generates a Sruja DSL fragment for a given architecture node.
 * This is used for the preview in the Properties Panel.
 */
/**
 * Generates a Sruja DSL fragment for a given architecture node.
 * This is used for the preview in the Properties Panel.
 */
export function generateDSLFragment(node: any): string {
    if (!node) return '';

    // Map node types to correct DSL keywords
    const typeMap: Record<string, string> = {
        'person': 'person',
        'system': 'system',
        'container': 'container',
        'component': 'component',
        'datastore': 'datastore',
        'queue': 'queue',
        'requirement': 'requirement',
        'adr': 'adr',
        'deployment': 'deployment',
    };

    const nodeType = node.type || '';
    const dslKeyword = typeMap[nodeType] || nodeType || 'system'; // Default to system if unknown
    const id = node.id || 'unknown';
    // Label is optional in DSL if ID is descriptive, but usually quoted
    const label = node.label ? ` "${node.label}"` : '';

    // Build body properties
    const lines: string[] = [];

    if (node.description) {
        lines.push(`  description "${node.description}"`);
    }

    // Technology is specific to containers and components
    if (node.technology && (nodeType === 'container' || nodeType === 'component')) {
        lines.push(`  technology "${node.technology}"`);
    }

    // Metadata
    if (node.metadata && node.metadata.length > 0) {
        lines.push(`  metadata {`);
        node.metadata.forEach((m: any) => {
            const val = m.value ? ` "${m.value}"` : '';
            lines.push(`    ${m.key}${val}`);
        });
        lines.push(`  }`);
    }

    // Connect block (handled via relations, usually separate or nested)
    // For this snippet, we might focus on the node definition itself.
    // If we wanted to show relations defined INSIDE this node:
    if (node.relations && node.relations.length > 0) {
        // This is a choice: do we show relations here?
        // Sruja DSL allows relations inside the node block.
        // For simplicity, let's include outgoing relations if they are part of the node structure
        // But usually relations are links. Let's skip for simple node preview to keep it clean,
        // unless requested. ticket says "DSL block for the selected element".
    }

    if (lines.length === 0) {
        return `${dslKeyword} ${id}${label}`;
    }

    return `${dslKeyword} ${id}${label} {\n${lines.join('\n')}\n}`;
}
