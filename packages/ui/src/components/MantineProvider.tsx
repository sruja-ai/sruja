// packages/ui/src/components/MantineProvider.tsx
import { MantineProvider as MantineCoreProvider } from '@mantine/core';
import type { MantineProviderProps as MantineCoreProviderProps } from '@mantine/core';
import type { ReactNode } from 'react';

export interface MantineProviderProps extends Omit<MantineCoreProviderProps, 'children'> {
  children: ReactNode;
}

/**
 * MantineProvider wrapper for @sruja/ui
 * 
 * Provides Mantine context to the application. Should be placed at the root
 * of your application, typically wrapping your ThemeProvider.
 * 
 * @example
 * ```tsx
 * <MantineProvider>
 *   <ThemeProvider>
 *     <App />
 *   </ThemeProvider>
 * </MantineProvider>
 * ```
 */
export function MantineProvider({ children, ...props }: MantineProviderProps) {
  return (
    <MantineCoreProvider {...props}>
      {children}
    </MantineCoreProvider>
  );
}
