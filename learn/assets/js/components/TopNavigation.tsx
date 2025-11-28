// Top Navigation Component - Replaces injectTopNav DOM manipulation
import React, { useState, useEffect } from 'react';
import { Button } from './ui/button';
import { getSection, linksFor } from '../utils/navigation';
import { toggleTheme } from '../utils/theme';

interface TopNavigationProps {
  section: string;
}

export function TopNavigation({ section }: TopNavigationProps) {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [logoSpinning, setLogoSpinning] = useState(false);
  const links = linksFor(section as any);

  useEffect(() => {
    if (window.srujaWasmInitializing) {
      setLogoSpinning(true);
    }
    const checkWasm = setInterval(() => {
      if (!window.srujaWasmInitializing) {
        setLogoSpinning(false);
        clearInterval(checkWasm);
      }
    }, 100);
    return () => clearInterval(checkWasm);
  }, []);

  // Move Hugo Book search into nav inside component lifecycle
  useEffect(() => {
    const sidebarSearch = document.querySelector('.book-search');
    const navSearchHolder = document.getElementById('nav-search-holder');
    if (sidebarSearch && navSearchHolder) {
      navSearchHolder.classList.add('mr-4','md:mr-8');
      navSearchHolder.appendChild(sidebarSearch);

      const suggestionsData = [
        { href: '/docs/getting-started/', label: 'Getting Started' },
        { href: '/docs/cli/', label: 'CLI' },
        { href: '/docs/concepts/', label: 'Concepts' },
        { href: '/courses/', label: 'Courses' },
        { href: '/playground/', label: 'Playground' }
      ];
      const sugg = document.createElement('div');
      sugg.className = 'nav-search-suggestions mt-2 flex gap-2 flex-wrap';
      suggestionsData.forEach(s => {
        const a = document.createElement('a');
        a.className = 'suggestion-chip';
        a.href = s.href;
        a.textContent = s.label;
        sugg.appendChild(a);
      });
      navSearchHolder.appendChild(sugg);

      const input = document.getElementById('book-search-input');
      const results = document.getElementById('book-search-results');
      const showSuggestions = () => {
        if (!input) return;
        const q = (input as HTMLInputElement).value.trim();
        const hasResults = results && results.children && results.children.length > 0;
        if (q.length === 0 && !hasResults) {
          (sugg as HTMLDivElement).classList.add('active');
        } else {
          (sugg as HTMLDivElement).classList.remove('active');
        }
      };
      if (input) {
        input.addEventListener('input', showSuggestions);
        input.addEventListener('focus', showSuggestions);
        input.addEventListener('blur', () => setTimeout(() => (sugg as HTMLDivElement).classList.remove('active'), 150));
      }
    }
  }, []);

  const getTitle = () => {
    if (section === 'docs') return 'Sruja Docs';
    if (section === 'courses') return 'Sruja Courses';
    if (section === 'resources') return 'Sruja Resources';
    return 'Sruja';
  };

  const isActive = (href: string) => {
    const path = window.location.pathname;
    if (href === '/resources/') return section === 'resources';
    return path === href || path.startsWith(href);
  };

  return (
    <nav className="site-top-nav fixed top-0 left-0 w-full h-16 bg-white/80 dark:bg-slate-900/80 backdrop-blur-sm border-b border-slate-200 dark:border-slate-700 z-[1000] flex items-center justify-center">
      <div className="nav-container w-full max-w-[1400px] px-5 flex items-center justify-between h-full">
        <a href="/" className="nav-brand flex items-center gap-2.5 no-underline font-bold text-xl text-violet-600 dark:text-violet-400">
          <img 
            src="/sruja-logo.svg" 
            alt="Sruja Logo" 
            className={`nav-logo h-8 w-8 ${logoSpinning ? 'animate-spin' : ''}`}
          />
          <span>{getTitle()}</span>
        </a>
        
        <div className="nav-search-center flex-1 flex items-center justify-center relative mr-8 md:mr-12">
          <div id="nav-search-holder"></div>
        </div>

        <div className="nav-links hidden md:flex gap-6 items-center">
          <a 
            href="/playground/" 
            className={`no-underline text-slate-500 dark:text-slate-400 font-medium text-sm transition-colors hover:text-violet-600 dark:hover:text-violet-400 ${isActive('/playground/') ? 'text-violet-600 dark:text-violet-400 font-semibold' : ''}`}
          >
            Playground
          </a>
          <div className="nav-item dropdown relative inline-flex items-center">
            <a 
              href="/resources/" 
              className={`no-underline text-slate-500 dark:text-slate-400 font-medium text-sm transition-colors hover:text-violet-600 dark:hover:text-violet-400 ${isActive('/resources/') ? 'text-violet-600 dark:text-violet-400 font-semibold' : ''}`}
            >
              Resources
            </a>
            <div className="nav-dropdown hidden absolute top-full left-0 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-600 rounded-lg shadow-lg p-2 min-w-[180px] z-[1200] group-hover:block hover:block">
              <a href="/docs/" className="block py-1.5 text-slate-700 dark:text-slate-300 no-underline hover:text-violet-600 dark:hover:text-violet-400">Docs</a>
              <a href="/courses/" className="block py-1.5 text-slate-700 dark:text-slate-300 no-underline hover:text-violet-600 dark:hover:text-violet-400">Courses</a>
              <a href="/tutorials/" className="block py-1.5 text-slate-700 dark:text-slate-300 no-underline hover:text-violet-600 dark:hover:text-violet-400">Tutorials</a>
              <a href="/blogs/" className="block py-1.5 text-slate-700 dark:text-slate-300 no-underline hover:text-violet-600 dark:hover:text-violet-400">Blogs</a>
            </div>
          </div>
          <a 
            href="/about/" 
            className={`no-underline text-slate-500 dark:text-slate-400 font-medium text-sm transition-colors hover:text-violet-600 dark:hover:text-violet-400 ${isActive('/about/') ? 'text-violet-600 dark:text-violet-400 font-semibold' : ''}`}
          >
            About
          </a>
          <a 
            href="/community/" 
            className={`no-underline text-slate-500 dark:text-slate-400 font-medium text-sm transition-colors hover:text-violet-600 dark:hover:text-violet-400 ${isActive('/community/') ? 'text-violet-600 dark:text-violet-400 font-semibold' : ''}`}
          >
            Community
          </a>
          <a 
            href="https://github.com/sruja-ai/sruja" 
            target="_blank" 
            className="nav-github px-3 py-1.5 bg-slate-100 dark:bg-slate-800 rounded-md text-slate-700 dark:text-slate-200 no-underline text-sm font-medium hover:bg-slate-200 dark:hover:bg-slate-700"
          >
            GitHub
          </a>
          <Button
            variant="ghost"
            size="icon"
            className="theme-toggle"
            aria-label="Toggle Theme"
            onClick={() => toggleTheme()}
          >
            🌙
          </Button>
        </div>

        <Button
          variant="ghost"
          size="icon"
          className="nav-toggle md:hidden"
          aria-label="Toggle Menu"
          onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
        >
          ☰
        </Button>
      </div>

      {mobileMenuOpen && (
        <div className="nav-mobile-menu absolute top-16 left-0 w-full bg-white dark:bg-slate-800 border-b border-slate-200 dark:border-slate-700 flex flex-col py-2.5 md:hidden">
          <a href="/playground/" className="px-5 py-3 no-underline text-slate-700 dark:text-slate-200 border-b border-slate-100 dark:border-slate-700">Playground</a>
          <a href="/resources/" className="px-5 py-3 no-underline text-slate-700 dark:text-slate-200 border-b border-slate-100 dark:border-slate-700">Resources</a>
          <a href="/about/" className="px-5 py-3 no-underline text-slate-700 dark:text-slate-200 border-b border-slate-100 dark:border-slate-700">About</a>
          <a href="/community/" className="px-5 py-3 no-underline text-slate-700 dark:text-slate-200 border-b border-slate-100 dark:border-slate-700">Community</a>
          <a href="https://github.com/sruja-ai/sruja" target="_blank" className="px-5 py-3 no-underline text-slate-700 dark:text-slate-200">GitHub</a>
        </div>
      )}
    </nav>
  );
}
