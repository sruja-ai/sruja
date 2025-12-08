import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { ErrorModal } from './ErrorModal';

describe('ErrorModal', () => {
  it('renders nothing when no lastError', () => {
    const { container } = render(<ErrorModal validationStatus={{ isValid: true, errors: 0, warnings: 0 }} onClose={() => {}} /> as any);
    expect(container.firstChild).toBeNull();
  });

  it('shows error message and counts, closes on overlay and button', () => {
    const onClose = vi.fn();
    const status = { isValid: false, errors: 2, warnings: 0, lastError: 'Parse error at line 2' } as any;
    render(<ErrorModal validationStatus={status} onClose={onClose} />);
    expect(screen.getByText('Validation Error')).toBeTruthy();
    expect(screen.getByText(/2 errors found/)).toBeTruthy();
    // click overlay
    fireEvent.click(screen.getByTitle('Close')); // header close button
    expect(onClose).toHaveBeenCalled();
  });
});
