import React from 'react';
import ReactDOM from 'react-dom/client';
import { ThemeProvider, PosthogProvider } from '@sruja/ui';
import '@sruja/ui/design-system/styles.css';
import App from './App';
import './index.css';

// Suppress console warnings from monaco-vscode-api about context attributes
// This is a known issue with React StrictMode and monaco-vscode-api
if (typeof window !== 'undefined') {
  const originalError = console.error;
  const originalWarn = console.warn;
  
  console.error = (...args: any[]) => {
    const message = args[0];
    if (typeof message === 'string' && message.includes('Element already has context attribute')) {
      // Suppress this specific warning from monaco-vscode-api
      return;
    }
    originalError.apply(console, args);
  };
  
  console.warn = (...args: any[]) => {
    const message = args[0];
    if (typeof message === 'string' && (
      message.includes('Could not create web worker') ||
      message.includes('Falling back to loading web worker code') ||
      message.includes('Workers disabled')
    )) {
      // Suppress worker warnings - we're intentionally using main thread
      return;
    }
    originalWarn.apply(console, args);
  };
  
  // Also suppress uncaught promise rejections from worker creation
  window.addEventListener('unhandledrejection', (event) => {
    if (event.reason && typeof event.reason === 'object' && event.reason.message && 
        event.reason.message.includes('Workers disabled')) {
      event.preventDefault();
    }
  });
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <PosthogProvider apiKey={import.meta.env.VITE_POSTHOG_KEY || ''} host={import.meta.env.VITE_POSTHOG_HOST || undefined}>
      <ThemeProvider defaultMode="system">
        <App />
      </ThemeProvider>
    </PosthogProvider>
  </React.StrictMode>,
);
