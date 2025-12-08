// apps/website/src/shared/components/ui/ThemeWrapper.tsx
import { ThemeProvider } from '@sruja/ui';
import type { ReactNode } from 'react';

interface ThemeWrapperProps {
  children: ReactNode;
}

export default function ThemeWrapper({ children }: ThemeWrapperProps) {
  return <ThemeProvider defaultMode="system">{children}</ThemeProvider>;
}

