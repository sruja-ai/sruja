// Top Navigation Component - Replaces injectTopNav DOM manipulation
import React, { useState, useEffect, useRef, useCallback } from 'react';
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
  const menuButtonRef = useRef<HTMLButtonElement>(null);
  
  // Ensure no external event listeners interfere
  useEffect(() => {
    const button = menuButtonRef.current;
    if (!button) return;
    
    // Stop any event propagation at the document level that might interfere
    const handleDocumentClick = (e: MouseEvent) => {
      if (button.contains(e.target as Node)) {
        e.stopImmediatePropagation();
      }
    };
    
    document.addEventListener('click', handleDocumentClick, { capture: true });
    
    return () => {
      document.removeEventListener('click', handleDocumentClick, { capture: true });
    };
  }, []);
  
  const handleMenuToggle = useCallback((e: React.MouseEvent | React.TouchEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (e.nativeEvent) {
      e.nativeEvent.stopImmediatePropagation();
    }
    setMobileMenuOpen((prev) => !prev);
  }, []);

  useEffect(() => {
    // Check namespaced API first, fallback to legacy
    const isInitializing = window.sruja?.wasmInitializing || window.srujaWasmInitializing;
    if (isInitializing) {
      setLogoSpinning(true);
    }
    const checkWasm = setInterval(() => {
      const stillInitializing = window.sruja?.wasmInitializing || window.srujaWasmInitializing;
      if (!stillInitializing) {
        setLogoSpinning(false);
        clearInterval(checkWasm);
      }
    }, 100);
    return () => clearInterval(checkWasm);
  }, []);

  // Prevent body scroll when mobile menu is open
  useEffect(() => {
    if (mobileMenuOpen) {
      document.body.style.overflow = 'hidden';
      // Close menu on escape key
      const handleEscape = (e: KeyboardEvent) => {
        if (e.key === 'Escape') {
          setMobileMenuOpen(false);
        }
      };
      document.addEventListener('keydown', handleEscape);
      return () => {
        document.body.style.overflow = '';
        document.removeEventListener('keydown', handleEscape);
      };
    } else {
      document.body.style.overflow = '';
    }
  }, [mobileMenuOpen]);

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
    if (section === 'learn') return 'Sruja Learn';
    return 'Sruja';
  };

  const buildBreadcrumbs = () => {
    const path = window.location.pathname;
    const crumbs: { href: string; label: string }[] = [];
    const push = (href: string, label: string) => crumbs.push({ href, label });
    if (section === 'docs' && path.startsWith('/docs/')) {
      push('/docs/', 'Docs');
      const parts = path.replace(/^\/docs\//, '').split('/').filter(Boolean);
      let acc = '/docs/';
      parts.forEach(p => {
        acc += `${p}/`;
        const label = p.replace(/-/g, ' ').replace(/\b\w/g, c => c.toUpperCase());
        push(acc, label);
      });
    } else if (section === 'courses' && path.startsWith('/courses/')) {
      push('/courses/', 'Courses');
      const parts = path.replace(/^\/courses\//, '').split('/').filter(Boolean);
      let acc = '/courses/';
      parts.forEach(p => {
        acc += `${p}/`;
        const label = p.replace(/-/g, ' ').replace(/\b\w/g, c => c.toUpperCase());
        push(acc, label);
      });
    }
    return crumbs;
  };

  const isActive = (href: string) => {
    const path = window.location.pathname;
    if (href === '/learn/') return section === 'learn' || path.startsWith('/learn/');
    return path === href || path.startsWith(href);
  };

  return (
    <nav className="site-top-nav fixed top-0 left-0 w-full h-16 bg-white/80 dark:bg-slate-900/80 backdrop-blur-sm border-b border-slate-200 dark:border-slate-700 z-[1000] flex items-center justify-center">
      <div className="nav-container w-full max-w-[1400px] px-5 flex items-center justify-between h-full">
        <a href="/" className="nav-brand flex items-center gap-2.5 no-underline font-bold text-xl text-violet-600 dark:text-violet-400 flex-shrink-1 min-w-0">
          <img 
            src="/sruja-logo.svg" 
            alt="Sruja Logo" 
            className={`nav-logo h-8 w-8 flex-shrink-0 ${logoSpinning ? 'animate-spin' : ''}`}
          />
          <span className="flex-shrink-0 whitespace-nowrap truncate max-w-[200px] sm:max-w-none">{getTitle()}</span>
          <span className="pre-alpha-badge hidden sm:inline-flex items-center gap-1 px-2 py-0.5 bg-violet-600 text-white text-xs font-bold rounded-full whitespace-nowrap">
            <span className="w-1.5 h-1.5 bg-white rounded-full animate-pulse"></span>
            PRE-ALPHA
          </span>
          {(section === 'docs' || section === 'courses') && (
            <nav className="ml-2 sm:ml-4 text-xs text-slate-500 dark:text-slate-400 hidden sm:flex items-center gap-1" aria-label="Breadcrumb">
              {buildBreadcrumbs().map((c, i) => (
                <span key={`${c.href}-${i}`} className="flex items-center gap-1">
                  <a href={c.href} className="no-underline hover:underline">{c.label}</a>
                  {i < buildBreadcrumbs().length - 1 && <span>/</span>}
                </span>
              ))}
            </nav>
          )}
        </a>
        
        <div className="nav-search-center hidden md:flex flex-1 items-center justify-center relative mr-8 md:mr-12">
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
              href="/learn/" 
              className={`no-underline text-slate-500 dark:text-slate-400 font-medium text-sm transition-colors hover:text-violet-600 dark:hover:text-violet-400 ${isActive('/learn/') ? 'text-violet-600 dark:text-violet-400 font-semibold' : ''}`}
            >
              Learn
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

        <button
          ref={menuButtonRef}
          type="button"
          data-sruja-mobile-toggle="true"
          className="nav-toggle md:hidden min-w-[44px] min-h-[44px] flex-shrink-0 z-[1100] relative inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-white transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-slate-950 focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 dark:ring-offset-slate-950 dark:focus-visible:ring-slate-300 hover:bg-slate-100 hover:text-slate-900 dark:hover:bg-slate-800 dark:hover:text-slate-50 h-10 w-10"
          aria-label="Toggle Menu"
          aria-expanded={mobileMenuOpen}
          onClick={handleMenuToggle}
          onTouchEnd={handleMenuToggle}
          style={{ pointerEvents: 'auto', cursor: 'pointer', touchAction: 'manipulation', WebkitTapHighlightColor: 'transparent', zIndex: 1100 }}
        >
          {mobileMenuOpen ? '✕' : '☰'}
        </button>
      </div>

      {/* Mobile menu overlay */}
      {mobileMenuOpen && (
        <div 
          className="fixed inset-0 bg-black/50 z-[1050] md:hidden nav-mobile-overlay"
          style={{ 
            display: 'block',
            top: 'var(--nav-height, 64px)' 
          }}
          onClick={() => setMobileMenuOpen(false)}
        />
      )}
      
      {mobileMenuOpen && (
        <div 
          className="nav-mobile-menu fixed left-0 w-full bg-white dark:bg-slate-800 border-b border-slate-200 dark:border-slate-700 flex flex-col py-2.5 md:hidden z-[1100] overflow-y-auto"
          data-sruja-mobile-menu="true"
          style={{ 
            top: 'var(--nav-height, 64px)',
            maxHeight: 'calc(100vh - var(--nav-height, 64px))',
            zIndex: 1100
          }}
        >
          <a 
            href="/playground/" 
            className="px-5 py-3 no-underline text-slate-700 dark:text-slate-200 border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-700 transition-colors"
            onClick={() => setMobileMenuOpen(false)}
          >
            Playground
          </a>
          <div className="px-5 py-3 border-b border-slate-100 dark:border-slate-700">
            <div className="text-slate-600 dark:text-slate-400 text-xs font-semibold uppercase mb-2">Learn</div>
            <a href="/docs/" onClick={() => setMobileMenuOpen(false)} className="block py-2 text-slate-700 dark:text-slate-200 no-underline hover:text-violet-600 dark:hover:text-violet-400">Docs</a>
            <a href="/courses/" onClick={() => setMobileMenuOpen(false)} className="block py-2 text-slate-700 dark:text-slate-200 no-underline hover:text-violet-600 dark:hover:text-violet-400">Courses</a>
            <a href="/tutorials/" onClick={() => setMobileMenuOpen(false)} className="block py-2 text-slate-700 dark:text-slate-200 no-underline hover:text-violet-600 dark:hover:text-violet-400">Tutorials</a>
            <a href="/blogs/" onClick={() => setMobileMenuOpen(false)} className="block py-2 text-slate-700 dark:text-slate-200 no-underline hover:text-violet-600 dark:hover:text-violet-400">Blogs</a>
          </div>
          <a 
            href="/about/" 
            className="px-5 py-3 no-underline text-slate-700 dark:text-slate-200 border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-700 transition-colors"
            onClick={() => setMobileMenuOpen(false)}
          >
            About
          </a>
          <a 
            href="/community/" 
            className="px-5 py-3 no-underline text-slate-700 dark:text-slate-200 border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-700 transition-colors"
            onClick={() => setMobileMenuOpen(false)}
          >
            Community
          </a>
          <a 
            href="https://github.com/sruja-ai/sruja" 
            target="_blank" 
            className="px-5 py-3 no-underline text-slate-700 dark:text-slate-200 hover:bg-slate-50 dark:hover:bg-slate-700 transition-colors"
            onClick={() => setMobileMenuOpen(false)}
          >
            GitHub
          </a>
          <div className="px-5 py-3 border-t border-slate-100 dark:border-slate-700">
            <Button
              variant="ghost"
              className="w-full justify-start theme-toggle"
              aria-label="Toggle Theme"
              onClick={() => {
                toggleTheme();
                setMobileMenuOpen(false);
              }}
            >
              🌙 Toggle Theme
            </Button>
          </div>
        </div>
      )}
    </nav>
  );
}
