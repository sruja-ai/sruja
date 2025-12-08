// packages/viewer/src/style/colors.ts

import { Colors, getCssVar } from '@sruja/shared/utils/cssVars';

export const colors = {
  person: getCssVar('--color-primary-500'),
  personBorder: getCssVar('--color-primary-600'),
  system: getCssVar('--color-neutral-900'),
  systemBorder: getCssVar('--color-neutral-700'),
  container: Colors.background(),
  containerBorder: getCssVar('--color-neutral-700'),
  component: getCssVar('--color-neutral-100'),
  componentBorder: getCssVar('--color-neutral-400'),
  database: Colors.background(),
  databaseBorder: getCssVar('--color-neutral-600'),
  edge: Colors.neutral500(),
  edgeActive: Colors.info(),
  textDark: Colors.textPrimary(),
  textLight: '#ffffff',
} as const;
