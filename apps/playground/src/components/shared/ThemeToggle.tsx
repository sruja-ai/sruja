import { Moon, Sun } from 'lucide-react';
import { useThemeStore } from '../../stores';
import './ThemeToggle.css';

export function ThemeToggle() {
    const theme = useThemeStore((s) => s.theme);
    const toggleTheme = useThemeStore((s) => s.toggleTheme);

    const isDark = theme === 'dark' ||
        (theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);

    return (
        <button
            className="theme-toggle"
            onClick={toggleTheme}
            title={`Switch to ${isDark ? 'light' : 'dark'} mode`}
        >
            {isDark ? <Sun size={18} /> : <Moon size={18} />}
        </button>
    );
}
