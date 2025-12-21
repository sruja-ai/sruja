// packages/ui/src/components/Button.test.tsx
import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Button } from './Button';

describe('Button', () => {
  it('should render button with text', () => {
    render(<Button>Click me</Button>);
    expect(screen.getByRole('button', { name: 'Click me' })).toBeInTheDocument();
  });

  it('should handle click events', async () => {
    const handleClick = vi.fn();
    const user = userEvent.setup();
    
    render(<Button onClick={handleClick}>Click me</Button>);
    const button = screen.getByRole('button', { name: 'Click me' });
    
    await user.click(button);
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('should be disabled when disabled prop is true', () => {
    render(<Button disabled>Disabled</Button>);
    expect(screen.getByRole('button', { name: 'Disabled' })).toBeDisabled();
  });

  it('should not call onClick when disabled', async () => {
    const handleClick = vi.fn();
    const user = userEvent.setup();
    
    render(<Button onClick={handleClick} disabled>Disabled</Button>);
    const button = screen.getByRole('button', { name: 'Disabled' });
    
    await user.click(button);
    expect(handleClick).not.toHaveBeenCalled();
  });

  it('should apply variant classes', () => {
    const { container } = render(<Button variant="primary">Primary</Button>);
    const button = container.querySelector('button');
    // Check for variant-specific classes
    expect(button?.className).toContain('bg-[var(--color-primary)]');
  });

  it('should apply size classes', () => {
    const { container } = render(<Button size="lg">Large</Button>);
    const button = container.querySelector('button');
    // Check for size-specific classes
    expect(button?.className).toContain('px-7');
    expect(button?.className).toContain('text-xl');
  });
});

