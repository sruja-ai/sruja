// Main entry point for Sruja Learn App
import { mountAppShell } from './components/AppShell';
import { mountFooter } from './components/Footer';
import { filterSidebarBySection, setupCollapsibleSidebar } from './components/Navigation';
import { initSrujaWasm } from './utils/wasm';
import { initSrujaCodeBlocks } from './components/SrujaCodeBlock';
import { initTheme } from './utils/theme';
import { trackPageVisit } from './utils/course-state';

// Initialize WASM state
window.srujaWasmReady = window.srujaWasmReady || false;
window.srujaWasmInitializing = window.srujaWasmInitializing || false;

function init(): void {
  mountAppShell();
  filterSidebarBySection();
  setupCollapsibleSidebar();
  mountFooter();
  initSrujaWasm();
  initSrujaCodeBlocks();
  initTheme();
  trackPageVisit();

  // Inject custom CSS
  const cssFiles = ['/css/theme.css'];
  if (window.location.pathname === '/' || window.location.pathname === '/index.html') {
    cssFiles.push('/css/home.css');
  }

  cssFiles.forEach(file => {
    const link = document.createElement('link');
    link.rel = 'stylesheet';
    link.href = file;
    document.head.appendChild(link);
  });
}

// Run initialization
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}
