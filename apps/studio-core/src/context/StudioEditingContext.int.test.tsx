import { describe, it, expect } from 'vitest';
import React from 'react';
import { renderWithStudio, createMockViewer, createMockWasm } from '../test-utils/studioHarness';
import { waitFor } from '@testing-library/react';
import { useStudioEditing } from './StudioEditingContext';

function TestComponent() {
  const { editing, updateDsl, undo, redo } = useStudioEditing();
  return (
    <div>
      <button onClick={() => updateDsl('architecture "A" { system Sys {} }')}>update</button>
      <button onClick={() => undo()}>undo</button>
      <button onClick={() => redo()}>redo</button>
      <div data-testid="dsl">{editing.dsl}</div>
      <div data-testid="valid">{String(editing.validationStatus.isValid)}</div>
      <div data-testid="errors">{editing.validationStatus.errors}</div>
      <div data-testid="warnings">{editing.validationStatus.warnings}</div>
      <div data-testid="arch">{editing.archData ? 'ok' : 'null'}</div>
    </div>
  );
}

describe('StudioEditingContext integration', () => {
  it('updates DSL, parses to archData, and sets diagnostics', async () => {
    const viewer = createMockViewer();
    const wasm = createMockWasm();
    const { getByText, getByTestId } = renderWithStudio(<TestComponent />, { viewer, wasm });
    getByText('update').click();
    await waitFor(() => {
      expect(getByTestId('arch').textContent).toContain('ok');
      expect(getByTestId('valid').textContent).toContain('true');
      expect(getByTestId('warnings').textContent).toContain('1');
    });
  });

  it('supports undo/redo of DSL snapshots', async () => {
    const viewer = createMockViewer();
    const wasm = createMockWasm();
    const { getByText, getByTestId } = renderWithStudio(<TestComponent />, { viewer, wasm });
    getByText('update').click();
    await waitFor(() => { expect(getByTestId('dsl').textContent).toContain('architecture'); });
    getByText('undo').click();
    await waitFor(() => { expect(getByTestId('dsl').textContent).toBe(''); });
    getByText('redo').click();
    await waitFor(() => { expect(getByTestId('dsl').textContent).toContain('architecture'); });
  });
});
