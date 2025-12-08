import { describe, it, expect } from 'vitest';
import { getNodeType, parseDslForNodeId } from '../nodeUtils';

const ARCH: any = {
  architecture: {
    persons: [{ id: 'User' }],
    requirements: [{ id: 'R1' }],
    adrs: [{ id: 'ADR1' }],
    deployment: [{ id: 'Prod' }],
    systems: [
      {
        id: 'Sys',
        containers: [
          { id: 'Web', components: [{ id: 'API' }] },
        ],
        datastores: [{ id: 'DB' }],
        queues: [{ id: 'Q' }],
      },
    ],
  },
};

describe('nodeUtils', () => {
  it('getNodeType infers types by id and arch', () => {
    expect(getNodeType({ id: 'Sys' }, ARCH)).toBe('system');
    expect(getNodeType({ id: 'Sys.Web' }, ARCH)).toBe('container');
    expect(getNodeType({ id: 'Sys.DB' }, ARCH)).toBe('datastore');
    expect(getNodeType({ id: 'Sys.Q' }, ARCH)).toBe('queue');
    expect(getNodeType({ id: 'Sys.Web.API' }, ARCH)).toBe('component');
    expect(getNodeType({ id: 'User' }, ARCH)).toBe('person');
    expect(getNodeType({ id: 'R1' }, ARCH)).toBe('requirement');
    expect(getNodeType({ id: 'ADR1' }, ARCH)).toBe('adr');
    expect(getNodeType({ id: 'Prod' }, ARCH)).toBe('deployment');
  });

  it('parseDslForNodeId matches ids near cursor including relation syntax', () => {
    const text = 'from Web to DB';
    // cursor on Web
    const id1 = parseDslForNodeId(text, 0, 6, ARCH);
    expect(id1).toBe('Web');
    // cursor on DB (position after space)
    const id2 = parseDslForNodeId(text, 0, 12, ARCH);
    expect(id2).toBe('DB');
  });
});
