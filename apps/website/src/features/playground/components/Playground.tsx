// apps/website/src/features/playground/components/Playground.tsx
// Wrapper component to use playground in the website
import { lazy, Suspense } from 'react';
import { SrujaLoader, ThemeProvider } from '@sruja/ui';
import '@sruja/ui/design-system/styles.css';

// Lazy load the playground App component
// CSS files are imported within the App component itself
const PlaygroundApp = lazy(() =>
  import('@sruja/playground').then(m => ({ default: m.default || m.App }))
);

export default function Playground() {
  return (
    <ThemeProvider defaultMode="system">
      <Suspense fallback={
        <div className="flex items-center justify-center h-full w-full">
          <div className="text-center">
            <SrujaLoader size={48} />
            <p className="text-[var(--color-text-secondary)] mt-4">Loading Playground...</p>
          </div>
        </div>
      }>
        <PlaygroundApp />
      </Suspense>
    </ThemeProvider>
  );
}







