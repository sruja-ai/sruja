// apps/website/src/features/studio/components/StudioApp.tsx
import { ThemeProvider, PosthogProvider } from '@sruja/ui';
import '@sruja/ui/design-system/styles.css';
// Import studio-core CSS - now properly exported in package.json
import '@sruja/studio-core/src/index.css';
import { LoadingScreen } from '@sruja/studio-core/components/LoadingScreen';
import { lazy, Suspense, useEffect } from 'react';
import { Buffer } from 'buffer';

if (typeof window !== 'undefined') {
  window.Buffer = window.Buffer || Buffer;
}

const StudioAppCore = lazy(() => import('@sruja/studio-core'));

export default function StudioApp() {
  useEffect(() => {
    // Ensure theme is synced with data-theme attribute
    const updateTheme = () => {
      const theme = localStorage.getItem('sruja-theme-mode') || 'system';
      const isDark = theme === 'dark' || (theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);
      if (isDark) {
        document.documentElement.classList.add('dark');
        document.documentElement.classList.remove('light');
        document.documentElement.setAttribute('data-theme', 'dark');
      } else {
        document.documentElement.classList.add('light');
        document.documentElement.classList.remove('dark');
        document.documentElement.setAttribute('data-theme', 'light');
      }
    };

    updateTheme();
    
    // Listen for theme changes
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    mediaQuery.addEventListener('change', updateTheme);
    
    // Listen for storage changes (theme toggle)
    window.addEventListener('storage', updateTheme);
    
    // Custom event for theme changes
    window.addEventListener('theme-change', updateTheme);

    return () => {
      mediaQuery.removeEventListener('change', updateTheme);
      window.removeEventListener('storage', updateTheme);
      window.removeEventListener('theme-change', updateTheme);
    };
  }, []);

  return (
    <div id="root" style={{ width: '100%', height: '100%', overflow: 'hidden' }}>
      <PosthogProvider
        apiKey={(import.meta as any).env?.PUBLIC_POSTHOG_API_KEY || ''}
        host={(import.meta as any).env?.PUBLIC_POSTHOG_HOST || undefined}
      >
        <ThemeProvider defaultMode="system">
          <Suspense fallback={<LoadingScreen isLoading message="Loading Sruja Studio..." /> }>
            <StudioAppCore />
          </Suspense>
        </ThemeProvider>
      </PosthogProvider>
    </div>
  );
}
