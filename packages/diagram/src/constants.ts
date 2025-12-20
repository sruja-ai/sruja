// Node component constants
// Centralized values following FAANG best practices

export const NODE_DEFAULTS = {
    ICON_SIZE: { small: 16, medium: 20, large: 24 } as const,
    MIN_WIDTH: 140,
    MAX_WIDTH: 240,
    PADDING: { x: 20, y: 16 } as const,
    BORDER_RADIUS: 12,
} as const;

export const ANIMATION = {
    TRANSITION_DURATION: '0.2s',
    EASING: 'cubic-bezier(0.4, 0, 0.2, 1)',
    HOVER_LIFT: '-2px',
} as const;

export const Z_INDEX = {
    NODE: 1,
    NODE_HOVER: 10,
    BADGE: 15,
    HANDLE: 5,
} as const;

// Node type to icon size mapping
export const NODE_ICON_SIZES: Record<string, number> = {
    system: NODE_DEFAULTS.ICON_SIZE.large,
    container: NODE_DEFAULTS.ICON_SIZE.medium,
    component: NODE_DEFAULTS.ICON_SIZE.small + 2, // 18
    person: NODE_DEFAULTS.ICON_SIZE.medium,
    datastore: NODE_DEFAULTS.ICON_SIZE.medium,
    queue: NODE_DEFAULTS.ICON_SIZE.medium,
    topic: NODE_DEFAULTS.ICON_SIZE.medium,
    cache: NODE_DEFAULTS.ICON_SIZE.medium,
    filesystem: NODE_DEFAULTS.ICON_SIZE.medium,
    deployment: NODE_DEFAULTS.ICON_SIZE.medium,
} as const;
