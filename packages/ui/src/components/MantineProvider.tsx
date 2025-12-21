// packages/ui/src/components/MantineProvider.tsx
import { MantineProvider as MantineCoreProvider } from '@mantine/core';
import type { MantineProviderProps as MantineCoreProviderProps } from '@mantine/core';
import type { ReactNode } from 'react';
import { srujaMantineTheme } from '../design-system/mantine-theme';

export interface MantineProviderProps extends Omit<MantineCoreProviderProps, 'children'> {
  children: ReactNode;
}

/**
 * MantineProvider wrapper for @sruja/ui
 * 
 * Provides Mantine context to the application with the shared Sruja theme.
 * Should be placed at the root of your application, typically wrapping your ThemeProvider.
 * 
 * The theme is automatically configured to use the shared design system.
 * You can override it by passing a custom `theme` prop.
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
export function MantineProvider({ children, theme, ...props }: MantineProviderProps) {
  return (
    <MantineCoreProvider theme={theme ?? srujaMantineTheme} {...props}>
      {children}
    </MantineCoreProvider>
  );
}
