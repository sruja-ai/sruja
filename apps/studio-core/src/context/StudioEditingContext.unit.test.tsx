import { describe, it, expect } from 'vitest';
import React from 'react';
import { renderWithStudio, createMockViewer } from '../test-utils/studioHarness';
import { useStudioEditing } from './StudioEditingContext';

function Test({ api }: any) {
  const { editing, updateDsl } = useStudioEditing();
  return (
    <div>
      <button onClick={() => updateDsl('bad')}>bad</button>
      <div data-testid="valid">{String(editing.validationStatus.isValid)}</div>
      <div data-testid="error">{editing.validationStatus.lastError || ''}</div>
    </div>
  );
}

describe('StudioEditingContext updateDsl error path', () => {
  it('sets invalid status and lastError when parse fails', async () => {
    const viewer = createMockViewer();
    const wasm = { parseDslToJson: async () => { throw new Error('Parse error at line 2: expected "}"'); } } as any;
    const { getByText, findByTestId } = renderWithStudio(<Test />, { viewer, wasm });
    getByText('bad').click();
    const valid = await findByTestId('valid');
    expect(valid.textContent).toContain('false');
    const err = await findByTestId('error');
    expect(err.textContent?.includes('Parse error at line')).toBe(true);
  });
});
