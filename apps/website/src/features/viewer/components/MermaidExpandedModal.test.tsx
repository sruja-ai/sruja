import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { MermaidExpandedModal } from './MermaidExpandedModal';

const expandedMermaid = { svg: '<svg width="100" height="50"><rect width="100" height="50"/></svg>', code: 'graph TD; A-->B' } as any;

describe('MermaidExpandedModal', () => {
  it('calls zoom in/out and close handlers', () => {
    const onZoomIn = vi.fn();
    const onZoomOut = vi.fn();
    const onClose = vi.fn();
    render(<MermaidExpandedModal expandedMermaid={expandedMermaid} zoom={1} onZoomIn={onZoomIn} onZoomOut={onZoomOut} onClose={onClose} />);
    fireEvent.click(screen.getByTitle('Zoom in'));
    expect(onZoomIn).toHaveBeenCalled();
    fireEvent.click(screen.getByTitle('Zoom out'));
    expect(onZoomOut).toHaveBeenCalled();
    fireEvent.click(screen.getByTitle('Close'));
    expect(onClose).toHaveBeenCalled();
  });

  it('disables zoom buttons at limits', () => {
    const { rerender } = render(<MermaidExpandedModal expandedMermaid={expandedMermaid} zoom={3} onZoomIn={() => {}} onZoomOut={() => {}} onClose={() => {}} />);
    expect((screen.getByTitle('Zoom in') as HTMLButtonElement).disabled).toBe(true);
    rerender(<MermaidExpandedModal expandedMermaid={expandedMermaid} zoom={0.5} onZoomIn={() => {}} onZoomOut={() => {}} onClose={() => {}} />);
    expect((screen.getByTitle('Zoom out') as HTMLButtonElement).disabled).toBe(true);
  });
});
