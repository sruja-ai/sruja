import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { TopNavBar } from './TopNavBar';

const examples = [
  { file: 'a.sruja', name: 'A', description: 'ex', category: 'cat' },
  { file: 'b.sruja', name: 'B', description: 'ex2', category: '' },
] as any;

function renderBar(props: Partial<React.ComponentProps<typeof TopNavBar>> = {}) {
  const defaults = {
    examples,
    selectedExample: '',
    isLoadingExample: false,
    isWasmLoading: false,
    onExampleChange: () => {},
    onOpenStudio: () => {},
    onShare: () => {},
    shareCopied: false,
    validationStatus: { isValid: true, errors: 0, warnings: 0 },
    onErrorClick: () => {},
  } as any;
  return render(<TopNavBar {...defaults} {...props} />);
}

describe('TopNavBar', () => {
  it('calls onOpenStudio when button clicked', () => {
    const onOpenStudio = vi.fn();
    renderBar({ onOpenStudio });
    fireEvent.click(screen.getByText('Open in Studio'));
    expect(onOpenStudio).toHaveBeenCalled();
  });

  it('calls onShare and toggles label when shareCopied', () => {
    const onShare = vi.fn();
    const initial = {
      examples,
      selectedExample: '',
      isLoadingExample: false,
      isWasmLoading: false,
      onExampleChange: () => {},
      onOpenStudio: () => {},
      onShare,
      shareCopied: false,
      validationStatus: { isValid: true, errors: 0, warnings: 0 },
      onErrorClick: () => {},
    } as any;
    const { rerender } = render(<TopNavBar {...initial} />);
    fireEvent.click(screen.getByText('Share'));
    expect(onShare).toHaveBeenCalled();
    rerender(<TopNavBar {...{ ...initial, shareCopied: true }} />);
    expect(screen.getByText('Copied!')).toBeTruthy();
  });

  it('renders validation icon or error button', () => {
    const onErrorClick = vi.fn();
    renderBar({ validationStatus: { isValid: false, errors: 1, warnings: 0, lastError: 'x' }, onErrorClick });
    fireEvent.click(screen.getByTitle('Click to view errors'));
    expect(onErrorClick).toHaveBeenCalled();
  });

  it('disables example select while loading', () => {
    renderBar({ isLoadingExample: true });
    const sel = screen.getByTitle('Loading example...');
    expect((sel as HTMLSelectElement).disabled).toBe(true);
  });
});
