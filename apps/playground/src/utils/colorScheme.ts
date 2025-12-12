// C4 Model Color Constants
// Following Simon Brown's C4 model conventions
// Using BORDER-ONLY style (no fills) like official C4 diagrams

export const C4_COLORS = {
    // Person - Blue
    person: {
        bg: 'transparent',       // No fill
        border: '#08427B',       // Dark blue border
        text: '#08427B',         // Match border
    },
    // Internal System - Blue
    systemInternal: {
        bg: 'transparent',       // No fill
        border: '#1168BD',       // Blue border
        text: '#1168BD',         // Match border
    },
    // External System - Gray
    systemExternal: {
        bg: 'transparent',       // No fill
        border: '#999999',       // Gray border
        text: '#666666',         // Darker gray text
    },
    // Container - Lighter blue
    container: {
        bg: 'transparent',       // No fill
        border: '#438DD5',       // Light blue border
        text: '#438DD5',         // Match border
    },
    // Component - Lightest blue
    component: {
        bg: 'transparent',       // No fill
        border: '#85BBF0',       // Very light blue border
        text: '#5A8FC0',         // Slightly darker for readability
    },
    // Database - Container blue
    datastore: {
        bg: 'transparent',       // No fill
        border: '#438DD5',       // Light blue border
        text: '#438DD5',         // Match border
    },
    // Queue - Container blue
    queue: {
        bg: 'transparent',       // No fill
        border: '#438DD5',       // Light blue border
        text: '#438DD5',         // Match border
    },
} as const;

// Alternative FILLED style (legacy)
export const C4_COLORS_FILLED = {
    person: {
        bg: '#08427B',
        border: '#052E56',
        text: '#FFFFFF',
    },
    systemInternal: {
        bg: '#1168BD',
        border: '#0B4884',
        text: '#FFFFFF',
    },
    systemExternal: {
        bg: '#999999',
        border: '#6B6B6B',
        text: '#FFFFFF',
    },
    container: {
        bg: '#438DD5',
        border: '#2E6295',
        text: '#FFFFFF',
    },
    component: {
        bg: '#85BBF0',
        border: '#5D99C9',
        text: '#000000',
    },
    datastore: {
        bg: '#438DD5',
        border: '#2E6295',
        text: '#FFFFFF',
    },
    queue: {
        bg: '#438DD5',
        border: '#2E6295',
        text: '#FFFFFF',
    },
} as const;

export const EDGE_STYLES = {
    default: {
        stroke: '#707070',
        strokeWidth: 1.5,
    },
    selected: {
        stroke: '#333333',
        strokeWidth: 2,
    },
    animated: {
        stroke: '#1168BD',
        strokeWidth: 2,
        strokeDasharray: '5,5',
    },
    dimmed: {
        stroke: '#CCCCCC',
        strokeWidth: 1,
        opacity: 0.5,
    },
} as const;

// Style mode switch
export type ColorStyle = 'outline' | 'filled';
let currentStyle: ColorStyle = 'outline';

export function setColorStyle(style: ColorStyle) {
    currentStyle = style;
}

export function getColorStyle(): ColorStyle {
    return currentStyle;
}

// Get color scheme based on node type
export function getNodeColors(type: string, isExternal?: boolean) {
    const colors = currentStyle === 'outline' ? C4_COLORS : C4_COLORS_FILLED;

    switch (type) {
        case 'person':
            return colors.person;
        case 'system':
            return isExternal ? colors.systemExternal : colors.systemInternal;
        case 'container':
            return colors.container;
        case 'component':
            return colors.component;
        case 'datastore':
            return colors.datastore;
        case 'queue':
            return colors.queue;
        default:
            return colors.systemInternal;
    }
}

export function getTagStyles(tags?: string[]) {
    const style: Record<string, any> = {};
    if (!tags || tags.length === 0) return style;
    const t = new Set(tags.map((x) => x.toLowerCase()));
    if (t.has('deprecated')) style.borderStyle = 'dashed';
    if (t.has('critical')) style.boxShadow = '0 0 0 3px #ef4444';
    if (t.has('experimental')) style.boxShadow = '0 0 0 2px #f59e0b';
    return style;
}
