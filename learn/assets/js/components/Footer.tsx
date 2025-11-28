import React from 'react';
import { createRoot } from 'react-dom/client';

export function Footer() {
  return (
    <footer className="site-footer border-t border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900">
      <div className="footer-container max-w-[1400px] mx-auto px-5 py-8 flex flex-col md:flex-row items-center justify-between gap-4">
        <div className="footer-links flex flex-wrap gap-4 text-sm">
          <a href="/docs/" className="no-underline text-slate-700 dark:text-slate-200 hover:text-violet-600 dark:hover:text-violet-400">Docs</a>
          <a href="/courses/" className="no-underline text-slate-700 dark:text-slate-200 hover:text-violet-600 dark:hover:text-violet-400">Courses</a>
          <a href="/tutorials/" className="no-underline text-slate-700 dark:text-slate-200 hover:text-violet-600 dark:hover:text-violet-400">Tutorials</a>
          <a href="/blogs/" className="no-underline text-slate-700 dark:text-slate-200 hover:text-violet-600 dark:hover:text-violet-400">Blogs</a>
          <a href="/community/" className="no-underline text-slate-700 dark:text-slate-200 hover:text-violet-600 dark:hover:text-violet-400">Community</a>
          <a href="https://github.com/sruja-ai/sruja" target="_blank" className="no-underline text-slate-700 dark:text-slate-200 hover:text-violet-600 dark:hover:text-violet-400">GitHub</a>
        </div>
        <div className="footer-copy text-slate-500 dark:text-slate-400 text-sm">
          © {new Date().getFullYear()} Sruja. All rights reserved.
        </div>
      </div>
    </footer>
  );
}

export function mountFooter(): void {
  let container = document.getElementById('sruja-footer-root');
  if (!container) {
    const el = document.createElement('div');
    el.id = 'sruja-footer-root';
    document.body.appendChild(el);
    container = el;
  }
  // @ts-ignore mark mounted to avoid remounts
  if ((container as any)._mounted) return;
  const root = createRoot(container);
  // @ts-ignore mark mounted to avoid remounts
  (container as any)._mounted = true;
  root.render(<Footer />);
}
