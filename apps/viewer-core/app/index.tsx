import React from 'react';
import { createRoot } from 'react-dom/client';
import { ThemeProvider } from '@sruja/ui';
import '@sruja/ui/design-system/styles.css';
import App from './App';
import './index.css';

// Initialize React app
const init = () => {
  const container = document.getElementById('root');
  if (container) {
    const root = createRoot(container);

    // Try to get data from window (injected) or script tag
    let data = (window as any).SRUJA_DATA;

    if (!data) {
      const dataScript = document.getElementById('sruja-data');
      if (dataScript) {
        try {
          data = JSON.parse(dataScript.textContent || '{}');
        } catch (e) {
          console.error('Failed to parse architecture data:', e);
        }
      }
    }

    root.render(
      <ThemeProvider defaultMode="system">
        <App data={data} />
      </ThemeProvider>
    );
  } else {
    console.error('Root element not found');
  }
};

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}

