// Example files configuration
// Pre-exported JSON files for robust loading (avoids WASM validation issues)

export interface ExampleFile {
    file: string;          // JSON filename
    name: string;
    order: number;
    category: string;
    description: string;
}

// Working pre-exported JSON examples
export const EXAMPLES: ExampleFile[] = [
    {
        file: 'sruja_architecture.json',
        name: 'Sruja Architecture',
        order: 0,
        category: 'showcase',
        description: 'The architecture of Sruja itself',
    },
    {
        file: 'ecommerce_platform.json',
        name: 'E-Commerce Platform',
        order: 1,
        category: 'showcase',
        description: 'Complete e-commerce with ADRs, flows, and requirements',
    },
    {
        file: 'c4_full.json',
        name: 'C4 Complete',
        order: 2,
        category: 'advanced',
        description: 'Complete C4 model example',
    },
];

/**
 * Fetch example JSON directly (pre-exported from CLI)
 */
export async function fetchExampleJson(filename: string): Promise<object> {
    const url = `/examples/${filename}`;

    const response = await fetch(url);
    if (!response.ok) {
        throw new Error(`Failed to fetch example: ${filename}`);
    }

    return response.json();
}
