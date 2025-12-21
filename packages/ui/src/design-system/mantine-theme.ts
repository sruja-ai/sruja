// packages/ui/src/design-system/mantine-theme.ts
// Shared Mantine theme configuration for all Sruja apps

import { createTheme } from '@mantine/core';

/**
 * Creates a Mantine theme that integrates with Sruja's design system.
 * 
 * This theme uses CSS variables from the shared design system, ensuring
 * consistency across all apps that use @sruja/ui.
 * 
 * @param options - Optional theme overrides
 * @returns Configured Mantine theme
 */
export function createSrujaMantineTheme() {
  return createTheme({
    // Use CSS variables from the shared design system
    // Mantine will automatically pick up these values
    primaryColor: 'violet',
    
    // Font families
    fontFamily: 'var(--font-family-sans)',
    fontFamilyMonospace: 'var(--font-family-mono)',
    
    // Headings
    headings: {
      fontFamily: 'var(--font-family-sans)',
      fontWeight: '600',
    },
    
    // Default radius
    defaultRadius: 'md',
    
    // Colors - Mantine will use CSS variables via colorScheme
    // The actual colors are defined in styles.css and managed by ThemeProvider
    colors: {
      // Map to our design system colors
      violet: [
        '#f5f3ff',
        '#ede9fe',
        '#ddd6fe',
        '#c4b5fd',
        '#a78bfa',
        '#8b5cf6',
        '#7c3aed', // primary-600
        '#6d28d9', // primary-700
        '#5b21b6',
        '#4c1d95',
      ],
    },
    
    // Shadows - use our design system shadows
    shadows: {
      xs: 'var(--shadow-sm)',
      sm: 'var(--shadow-sm)',
      md: 'var(--shadow-md)',
      lg: '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -4px rgba(0, 0, 0, 0.1)',
      xl: '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 8px 10px -6px rgba(0, 0, 0, 0.1)',
    },
    
    // Spacing scale
    spacing: {
      xs: 'var(--spacing-1)',
      sm: 'var(--spacing-2)',
      md: 'var(--spacing-4)',
      lg: 'var(--spacing-6)',
      xl: 'var(--spacing-8)',
    },
    
    // Border radius
    radius: {
      xs: 'var(--radius-sm)',
      sm: 'var(--radius-base)',
      md: 'var(--radius-md)',
      lg: 'var(--radius-lg)',
    },
    
    // Other defaults
    cursorType: 'pointer',
    respectReducedMotion: true,
  });
}

/**
 * Default shared Mantine theme instance.
 * Use this in MantineProvider to ensure consistency across apps.
 */
export const srujaMantineTheme = createSrujaMantineTheme();

