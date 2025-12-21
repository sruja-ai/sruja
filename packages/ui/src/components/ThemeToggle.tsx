// packages/ui/src/components/ThemeToggle.tsx
import { Moon, Sun } from 'lucide-react';
import { useTheme } from './ThemeProvider';
import { Button, type ButtonProps } from './Button';

export interface ThemeToggleProps extends Omit<ButtonProps, 'onClick' | 'children'> {
  /** Show icon only (no text) */
  iconOnly?: boolean;
}

export function ThemeToggle({ iconOnly = false, variant = 'ghost', size = 'md', ...props }: ThemeToggleProps) {
  const { mode, toggle } = useTheme();
  const isDark = mode === 'dark' || (mode === 'system' && typeof window !== 'undefined' && window.matchMedia('(prefers-color-scheme: dark)').matches);

  return (
    <Button
      variant={variant}
      size={size}
      onClick={toggle}
      title={isDark ? 'Switch to light mode' : 'Switch to dark mode'}
      {...props}
    >
      {isDark ? (
        <Sun size={size === 'sm' ? 16 : size === 'lg' ? 20 : 18} />
      ) : (
        <Moon size={size === 'sm' ? 16 : size === 'lg' ? 20 : 18} />
      )}
      {!iconOnly && <span>{isDark ? 'Light' : 'Dark'}</span>}
    </Button>
  );
}

