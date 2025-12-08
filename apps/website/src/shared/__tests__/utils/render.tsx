// apps/website/src/shared/__tests__/utils/render.tsx
import { render as rtlRender, type RenderOptions } from '@testing-library/react';
import type { ReactElement } from 'react';
import ThemeWrapper from '@/shared/components/ui/ThemeWrapper';

export function render(ui: ReactElement, options?: Omit<RenderOptions, 'wrapper'>) {
  return rtlRender(ui, {
    wrapper: ThemeWrapper,
    ...options,
  });
}

// Re-export everything from testing-library
export * from '@testing-library/react';
