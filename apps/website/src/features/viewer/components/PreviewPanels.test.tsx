import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { PreviewPanels } from './PreviewPanels';

function renderPanels(props: Partial<React.ComponentProps<typeof PreviewPanels>> = {}) {
  const defaults = {
    previewFormat: 'json' as any,
    setPreviewFormat: () => {},
    activePane: 'split' as any,
    setActivePane: () => {},
    savePaneToStorage: () => {},
    viewerContainerRef: { current: null } as any,
    archData: { metadata: { name: 'A' } } as any,
    isParsingDsl: false,
    onDownloadJson: () => {},
    htmlPreview: '<html><body></body></html>',
    isGeneratingHtml: false,
    previewFrameRef: { current: null } as any,
    onDownloadHtml: () => {},
    markdownPreview: '# Title',
    isGeneratingMarkdown: false,
    onDownloadMarkdown: () => {},
    expandedMermaid: null,
    mermaidZoom: 1,
    onMermaidExpand: () => {},
    onMermaidZoomIn: () => {},
    onMermaidZoomOut: () => {},
    onMermaidClose: () => {},
  } as any;
  return render(<PreviewPanels {...defaults} {...props} />);
}

describe('PreviewPanels', () => {
  it('switches preview format via toolbar buttons', () => {
    const setPreviewFormat = vi.fn();
    renderPanels({ setPreviewFormat, previewFormat: 'json' as any });
    fireEvent.click(screen.getByTitle('HTML'));
    expect(setPreviewFormat).toHaveBeenCalledWith('preview');
    fireEvent.click(screen.getByTitle('Markdown'));
    expect(setPreviewFormat).toHaveBeenCalledWith('markdown');
    fireEvent.click(screen.getByTitle('Diagram'));
    expect(setPreviewFormat).toHaveBeenCalledWith('diagram');
  });

  it('opens preview in full screen and saves pane', () => {
    const setActivePane = vi.fn();
    const savePaneToStorage = vi.fn();
    renderPanels({ setActivePane, savePaneToStorage, previewFormat: 'markdown' as any });
    fireEvent.click(screen.getByTitle('Open preview in full screen'));
    expect(setActivePane).toHaveBeenCalledWith('markdown');
    expect(savePaneToStorage).toHaveBeenCalledWith('markdown');
  });

  it('enables Download when allowed and calls respective handler', () => {
    const onDownloadJson = vi.fn();
    renderPanels({ previewFormat: 'json' as any, onDownloadJson });
    fireEvent.click(screen.getByText('Download'));
    expect(onDownloadJson).toHaveBeenCalled();
  });
});
