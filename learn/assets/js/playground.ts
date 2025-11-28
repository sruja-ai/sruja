// Playground entry point
import { initPlayground } from './components/Playground';

// Initialize playground when DOM is ready
function init(): void {
  // Wait a bit for the HTML to be fully rendered
  const container = document.getElementById('playground-container');
  if (container) {
    const textarea = document.getElementById('sruja-input') as HTMLTextAreaElement | null;
    const initialCode = textarea?.value || '';
    // Initialize React in the existing container
    initPlayground('playground-container', initialCode);
  } else {
    // Retry if container not found yet
    setTimeout(init, 100);
  }
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  // Small delay to ensure DOM is ready
  setTimeout(init, 0);
}

