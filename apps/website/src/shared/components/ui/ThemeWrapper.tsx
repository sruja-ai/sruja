// apps/website/src/shared/components/ui/ThemeWrapper.tsx
import { MantineProvider, ThemeProvider } from '@sruja/ui';
import type { ReactNode } from 'react';

interface ThemeWrapperProps {
  children: ReactNode;
}

export default function ThemeWrapper({ children }: ThemeWrapperProps) {
  return (
    <MantineProvider>
      <ThemeProvider defaultMode="system">{children}</ThemeProvider>
    </MantineProvider>
  );
}

