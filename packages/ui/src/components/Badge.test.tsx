// packages/ui/src/components/Badge.test.tsx
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Badge } from './Badge';

describe('Badge', () => {
  it('should render badge with text', () => {
    render(<Badge>New</Badge>);
    expect(screen.getByText('New')).toBeInTheDocument();
  });

  it('should apply color classes', () => {
    const { container } = render(<Badge color="success">Success</Badge>);
    const badge = container.querySelector('span');
    // Check for success color-specific classes
    expect(badge?.className).toContain('bg-[#dcfce7]');
    expect(badge?.className).toContain('text-[#166534]');
  });

  it('should apply custom className', () => {
    const { container } = render(<Badge className="custom-badge">Custom</Badge>);
    const badge = container.querySelector('span');
    expect(badge?.className).toContain('custom-badge');
  });
});

