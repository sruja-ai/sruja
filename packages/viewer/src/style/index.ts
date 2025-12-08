// packages/viewer/src/style/index.ts
import { buildBaseNodeStyle } from './node-styles';
import { buildPersonStyle } from './node-styles';
import { buildSystemStyle } from './node-styles';
import { buildContainerStyle } from './node-styles';
import { buildDatastoreStyle } from './node-styles';
import { buildQueueStyle } from './node-styles';
import { buildEdgeStyle } from './node-styles';
import { buildInteractionStyles } from './node-styles';
import { buildParentStyles } from './node-styles';
import { buildHiddenNodeStyles } from './node-styles';

/**
 * Build complete Cytoscape style array
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function buildStyles(): any[] {
  return [
    buildBaseNodeStyle(),
    buildPersonStyle(),
    buildSystemStyle(),
    buildContainerStyle(),
    buildDatastoreStyle(),
    buildQueueStyle(),
    ...buildHiddenNodeStyles(),
    ...buildParentStyles(),
    buildEdgeStyle(),
    ...buildInteractionStyles(),
  ];
}
