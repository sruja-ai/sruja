import { buildDefinitionLinks, DefinitionResult } from './providers';
import { SrujaElement } from './wasm';

describe('buildDefinitionLinks', () => {
  it('returns targetSelectionRange matching element range', () => {
    const elements: SrujaElement[] = [
      {
        id: 'User',
        kind: 'element',
        title: 'Test User',
        range: { start: { line: 0, character: 0 }, end: { line: 2, character: 1 } },
      },
    ];

    const wordRange = { start: { line: 0, character: 8 }, end: { line: 0, character: 12 } };

    const result = buildDefinitionLinks('User', wordRange, elements);

    expect(result).toBeDefined();
    expect(result).toHaveLength(1);

    const link = result![0];
    expect(link.targetSelectionRange).toEqual({
      start: { line: 0, character: 0 },
      end: { line: 2, character: 1 },
    });
    expect(link.targetRange).toEqual({
      start: { line: 0, character: 0 },
      end: { line: 2, character: 1 },
    });
    expect(link.originSelectionRange).toEqual(wordRange);
  });

  it('returns targetSelectionRange for nested elements', () => {
    const elements: SrujaElement[] = [
      {
        id: 'Payment.ProcessPayment',
        kind: 'flow',
        title: 'Process',
        range: { start: { line: 1, character: 2 }, end: { line: 3, character: 3 } },
      },
    ];

    const wordRange = { start: { line: 1, character: 7 }, end: { line: 1, character: 23 } };

    const result = buildDefinitionLinks('ProcessPayment', wordRange, elements);

    expect(result).toBeDefined();
    expect(result).toHaveLength(1);

    const link = result![0];
    expect(link.targetSelectionRange).toEqual({
      start: { line: 1, character: 2 },
      end: { line: 3, character: 3 },
    });
  });

  it('matches element by full id', () => {
    const elements: SrujaElement[] = [
      {
        id: 'Payment.ProcessPayment',
        kind: 'flow',
        title: 'Process',
        range: { start: { line: 1, character: 2 }, end: { line: 3, character: 3 } },
      },
    ];

    const wordRange = { start: { line: 0, character: 0 }, end: { line: 0, character: 25 } };

    const result = buildDefinitionLinks('Payment.ProcessPayment', wordRange, elements);

    expect(result).toBeDefined();
    expect(result).toHaveLength(1);
  });

  it('returns undefined when element not found', () => {
    const elements: SrujaElement[] = [];

    const wordRange = { start: { line: 0, character: 8 }, end: { line: 0, character: 12 } };

    const result = buildDefinitionLinks('NonExistent', wordRange, elements);

    expect(result).toBeUndefined();
  });

  it('returns undefined for empty word', () => {
    const elements: SrujaElement[] = [
      {
        id: 'User',
        kind: 'element',
        title: 'Test User',
        range: { start: { line: 0, character: 0 }, end: { line: 2, character: 1 } },
      },
    ];

    const wordRange = { start: { line: 0, character: 8 }, end: { line: 0, character: 8 } };

    const result = buildDefinitionLinks('', wordRange, elements);

    expect(result).toBeUndefined();
  });
});
