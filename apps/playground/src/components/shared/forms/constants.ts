// apps/playground/src/components/shared/forms/constants.ts
import type { ListOption } from '@sruja/ui';

export const REQUIREMENT_TYPES: ListOption[] = [
    { id: 'functional', label: 'Functional' },
    { id: 'performance', label: 'Performance' },
    { id: 'security', label: 'Security' },
    { id: 'constraint', label: 'Constraint' },
    { id: 'reliability', label: 'Reliability' },
    { id: 'nonfunctional', label: 'Non-functional' },
];

export const ADR_STATUSES: ListOption[] = [
    { id: 'proposed', label: 'Proposed' },
    { id: 'accepted', label: 'Accepted' },
    { id: 'deprecated', label: 'Deprecated' },
    { id: 'superseded', label: 'Superseded' },
];
