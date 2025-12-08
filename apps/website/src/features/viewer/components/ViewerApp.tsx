// apps/website/src/features/viewer/components/ViewerApp.tsx
import { ThemeProvider } from '@sruja/ui';
import '@sruja/ui/design-system/styles.css';
import '../../../../../viewer-core/app/index.css';
import React, { lazy, Suspense, useEffect, useState } from 'react';
import { SrujaLoader } from '@sruja/ui';
import type { ArchitectureJSON } from '@sruja/viewer';

// Import the App component from viewer-core (sibling app in monorepo)
// From: apps/website/src/features/viewer/components/
// To: apps/viewer-core/app/App.tsx
const ViewerAppCore = lazy(() => 
  import('../../../../../viewer-core/app/App').then(m => ({ default: m.default }))
) as React.LazyExoticComponent<React.ComponentType<{ data: ArchitectureJSON | null }>>;

interface ViewerAppProps {
  data?: ArchitectureJSON | null;
  dataUrl?: string;
}

export default function ViewerApp({ data: initialData, dataUrl }: ViewerAppProps) {
  const [data, setData] = useState<ArchitectureJSON | null>(initialData || null);
  const [loading, setLoading] = useState(!!dataUrl);

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

  useEffect(() => {
    // Load data from URL if provided
    if (dataUrl && !data) {
      setLoading(true);
      fetch(dataUrl)
        .then(res => res.json())
        .then(json => {
          setData(json);
          setLoading(false);
        })
        .catch(err => {
          console.error('Failed to load architecture data:', err);
          setLoading(false);
        });
    }
  }, [dataUrl, data]);

  // Try to get data from script tag (for HTML exports)
  useEffect(() => {
    if (!data && !dataUrl) {
      const dataScript = document.getElementById('sruja-data');
      if (dataScript) {
        try {
          const parsed = JSON.parse(dataScript.textContent || '{}');
          setData(parsed);
        } catch (e) {
          console.error('Failed to parse architecture data from script tag:', e);
        }
      }
    }
  }, [data, dataUrl]);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full w-full">
        <div className="text-center">
          <SrujaLoader size={48} />
          <p className="mt-4 text-sm" style={{ color: 'var(--vscode-descriptionForeground)' }}>Loading architecture...</p>
        </div>
      </div>
    );
  }

  return (
    <div id="root" style={{ width: '100%', height: '100%', overflow: 'hidden' }}>
      <ThemeProvider defaultMode="system">
        <Suspense fallback={
          <div className="flex items-center justify-center h-full w-full">
            <div className="text-center">
              <SrujaLoader size={48} />
              <p className="mt-4 text-sm" style={{ color: 'var(--vscode-descriptionForeground)' }}>Loading Viewer...</p>
            </div>
          </div>
        }>
          <ViewerAppCore data={data || null} />
        </Suspense>
      </ThemeProvider>
    </div>
  );
}
