// C4 Model Color Constants
// Following Simon Brown's C4 model conventions

export const C4_COLORS = {
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

// Get color scheme based on node type
export function getNodeColors(type: string, isExternal?: boolean) {
    switch (type) {
        case 'person':
            return C4_COLORS.person;
        case 'system':
            return isExternal ? C4_COLORS.systemExternal : C4_COLORS.systemInternal;
        case 'system-boundary':
            return isExternal ? C4_COLORS.systemExternal : C4_COLORS.systemInternal;
        case 'container':
        case 'external-container':
            return C4_COLORS.container;
        case 'component':
        case 'external-component':
            return C4_COLORS.component;
        case 'datastore':
        case 'cache':
        case 'filesystem':
            return C4_COLORS.datastore;
        case 'queue':
        case 'topic':
            return C4_COLORS.queue;
        case 'deployment':
            // Deployment nodes use a distinct color
            return {
                bg: '#6B7280',
                border: '#4B5563',
                text: '#FFFFFF',
            };
        default:
            return C4_COLORS.systemInternal;
    }
}
