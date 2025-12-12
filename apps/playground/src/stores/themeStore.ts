import { create } from 'zustand';
import { persist } from 'zustand/middleware';

type Theme = 'light' | 'dark' | 'system';

interface ThemeState {
    theme: Theme;
    setTheme: (theme: Theme) => void;
    toggleTheme: () => void;
}

export const useThemeStore = create<ThemeState>()(
    persist(
        (set, get) => ({
            theme: 'system',

            setTheme: (theme) => {
                set({ theme });
                applyTheme(theme);
            },

            toggleTheme: () => {
                const current = get().theme;
                const next = current === 'dark' ? 'light' : 'dark';
                set({ theme: next });
                applyTheme(next);
            },
        }),
        {
            name: 'sruja-theme',
        }
    )
);

function applyTheme(theme: Theme) {
    const root = document.documentElement;

    if (theme === 'system') {
        const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
        root.setAttribute('data-theme', prefersDark ? 'dark' : 'light');
    } else {
        root.setAttribute('data-theme', theme);
    }
}

// Initialize theme on load
if (typeof window !== 'undefined') {
    const stored = localStorage.getItem('sruja-theme');
    if (stored) {
        try {
            const { state } = JSON.parse(stored);
            applyTheme(state.theme);
        } catch {
            applyTheme('system');
        }
    } else {
        applyTheme('system');
    }
}
