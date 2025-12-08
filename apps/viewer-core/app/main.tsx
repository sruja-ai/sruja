// Standalone entry point - auto-initializes when script loads
(function() {
  // Wait for DOM and dependencies
  function init() {
    if (typeof React === 'undefined' || typeof ReactDOM === 'undefined') {
      setTimeout(init, 50);
      return;
    }

    const React = (window as any).React;
    const ReactDOM = (window as any).ReactDOM;
    try {
      const prefersDark = window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
      document.documentElement.setAttribute('data-theme', prefersDark ? 'dark' : 'light');
    } catch {}
    
    // Dynamic import to avoid bundling issues
    import('./App').then(({ default: App }) => {
      import('@sruja/ui/design-system/styles.css');
      import('./index.css');
      
      // Get architecture data from embedded script tag
      const dataScript = document.getElementById('sruja-data');
      let architectureData = null;

      if (dataScript) {
        try {
          architectureData = JSON.parse(dataScript.textContent || '{}');
        } catch (e) {
          console.error('Failed to parse architecture data:', e);
        }
      }

      const root = document.getElementById('root');
      if (root) {
        ReactDOM.createRoot(root).render(
          React.createElement(React.StrictMode, null,
            React.createElement(App, { data: architectureData })
          )
        );
      }
    }).catch(err => {
      console.error('Failed to load app:', err);
    });
  }

  // Start initialization
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
