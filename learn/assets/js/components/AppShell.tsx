import React, { useEffect } from 'react';
import { createRoot } from 'react-dom/client';
import { TopNavigation } from './TopNavigation';
import { getSection } from '../utils/navigation';

export function AppShell() {
  const section = getSection();
  useEffect(() => {
    if (section === 'playground') {
      document.body.classList.add('playground-full');
    }
  }, [section]);

  return (
    <TopNavigation section={section} />
  );
}

export function mountAppShell(): void {
  let container = document.getElementById('sruja-app-root');
  if (!container) {
    const el = document.createElement('div');
    el.id = 'sruja-app-root';
    document.body.insertBefore(el, document.body.firstChild);
    container = el;
  }
  if ((container as any)._mounted) return;
  const root = createRoot(container);
  (container as any)._mounted = true;
  root.render(<AppShell />);
}
