// packages/ui/src/components/SrujaLoader.test.tsx
import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import { SrujaLoader } from './SrujaLoader';

describe('SrujaLoader', () => {
  it('should render loader', () => {
    const { container } = render(<SrujaLoader />);
    const loader = container.querySelector('.sruja-loader');
    expect(loader).toBeInTheDocument();
  });

  it('should apply custom size', () => {
    const { container } = render(<SrujaLoader size={64} />);
    const loader = container.querySelector('.sruja-loader');
    expect(loader).toHaveStyle({ width: '64px', height: '64px' });
  });

  it('should apply custom className', () => {
    const { container } = render(<SrujaLoader className="custom-class" />);
    const loader = container.querySelector('.sruja-loader');
    expect(loader?.className).toContain('custom-class');
  });

  it('should render SVG content', () => {
    const { container } = render(<SrujaLoader />);
    const svg = container.querySelector('svg');
    expect(svg).toBeInTheDocument();
    expect(svg?.getAttribute('viewBox')).toBe('0 0 300 300');
  });

  it('should use default size when not provided', () => {
    const { container } = render(<SrujaLoader />);
    const loader = container.querySelector('.sruja-loader');
    expect(loader).toHaveStyle({ width: '48px', height: '48px' });
  });
});




