import { describe, it, expect } from 'vitest';
import React from 'react';
import { render, waitFor } from '@testing-library/react';
import { StudioStateProvider, useStudioState } from './StudioStateContext';

function Test() {
  const { sidebar, setSidebar, properties, setProperties } = useStudioState();
  return (
    <div>
      <button onClick={() => setSidebar({ ...sidebar, showSidebar: !sidebar.showSidebar })}>toggle</button>
      <button onClick={() => setSidebar({ ...sidebar, activePanel: 'documentation' })}>panel</button>
      <button onClick={() => setSidebar({ ...sidebar, width: 300 })}>width</button>
      <button onClick={() => setProperties({ showProperties: !properties.showProperties })}>props</button>
      <div data-testid="vis">{String(sidebar.showSidebar)}</div>
      <div data-testid="panel">{sidebar.activePanel}</div>
      <div data-testid="width">{String(sidebar.width)}</div>
      <div data-testid="props">{String(properties.showProperties)}</div>
    </div>
  );
}

describe('StudioStateContext', () => {
  it('persists sidebar and properties to localStorage', async () => {
    const { getByText } = render(<StudioStateProvider><Test /></StudioStateProvider>);
    getByText('toggle').click();
    getByText('panel').click();
    getByText('width').click();
    getByText('props').click();
    await waitFor(() => {
      expect(localStorage.getItem('studio-sidebar-width')).toBe('300');
      expect(localStorage.getItem('studio-properties-visible')).toBe('true');
    });
  });
});
