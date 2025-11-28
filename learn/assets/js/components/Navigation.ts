// Navigation injection
import { getSection, globalLinksHTML } from '../utils/navigation';

export function injectTopNav(): void {
  const section = getSection();
  if (section === 'playground') {
    document.body.classList.add('playground-full');
  }
  const navHTML = `
    <nav class="site-top-nav">
      <div class="nav-container">
        <a href="/" class="nav-brand">
          <img src="/sruja-logo.svg" alt="Sruja Logo" class="nav-logo">
          <span>${section === 'docs' ? 'Sruja Docs' : section === 'courses' ? 'Sruja Courses' : section === 'resources' ? 'Sruja Resources' : 'Sruja'}</span>
        </a>
        <div class="nav-search-center"></div>
        <div class="nav-links">
          ${globalLinksHTML(section)}
          <a href="https://github.com/sruja-ai/sruja" target="_blank" class="nav-github">GitHub</a>
          <button class="theme-toggle" aria-label="Toggle Theme">🌙</button>
        </div>
        <button class="nav-toggle" aria-label="Toggle Menu">☰</button>
      </div>
      <div class="nav-mobile-menu">
        ${globalLinksHTML(section)}
      </div>
    </nav>
  `;

  const div = document.createElement('div');
  div.innerHTML = navHTML;
  if (div.firstElementChild) {
    document.body.insertBefore(div.firstElementChild, document.body.firstChild);
  }

  const toggle = document.querySelector('.nav-toggle');
  const mobileMenu = document.querySelector('.nav-mobile-menu');
  if (toggle && mobileMenu) {
    toggle.addEventListener('click', () => {
      mobileMenu.classList.toggle('active');
    });
  }

  if (window.srujaWasmInitializing) {
    const logo = document.querySelector('.nav-logo');
    if (logo) logo.classList.add('spin');
  }

  const navLinks = document.querySelector('.nav-links');
  if (navLinks) {
    const anchors = navLinks.querySelectorAll('a[href]');
    if (anchors.length < 3) {
      navLinks.innerHTML = `${globalLinksHTML(section)}<a href="https://github.com/sruja-ai/sruja" target="_blank" class="nav-github">GitHub</a>`;
    }
    const setActive = (sel: string) => {
      const a = navLinks.querySelector(sel);
      if (a) a.classList.add('active');
    };
    if (section === 'resources') setActive('a[href="/resources/"]');
    if (section === 'community') setActive('a[href="/community/"]');
    if (section === 'about') setActive('a[href="/about/"]');
    if (section === 'playground') setActive('a[href="/playground/"]');
  }

  // Move Hugo Book search into top nav
  const sidebarSearch = document.querySelector('.book-search');
  const navSearchHolder = document.querySelector('.nav-search-center');
  if (sidebarSearch && navSearchHolder) {
    navSearchHolder.appendChild(sidebarSearch);
    const suggestionsData = [
      { href: '/docs/getting-started/', label: 'Getting Started' },
      { href: '/docs/cli/', label: 'CLI' },
      { href: '/docs/concepts/', label: 'Concepts' },
      { href: '/courses/', label: 'Courses' },
      { href: '/playground/', label: 'Playground' }
    ];
    const sugg = document.createElement('div');
    sugg.className = 'nav-search-suggestions';
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
        sugg.classList.add('active');
      } else {
        sugg.classList.remove('active');
      }
    };
    if (input) {
      input.addEventListener('input', showSuggestions);
      input.addEventListener('focus', showSuggestions);
      input.addEventListener('blur', () => setTimeout(() => sugg.classList.remove('active'), 150));
    }
  }

  const resourcesLink = document.querySelector('.nav-item.dropdown > a[href="/resources/"]');
  if (resourcesLink) {
    resourcesLink.addEventListener('click', (e) => {
      e.preventDefault();
      window.location.href = '/resources/';
    });
  }
}

export function filterSidebarBySection(): void {
  const section = getSection();
  if (section === 'home') return;
  const prefixMap: Record<string, string[] | string> = {
    resources: ['/resources/', '/docs/', '/courses/', '/tutorials/', '/blogs/'],
    community: '/community/'
  };
  const prefixes = Array.isArray(prefixMap[section]) ? prefixMap[section] as string[] : [prefixMap[section] as string];
  const topItems = document.querySelectorAll('.book-menu nav > ul > li');
  topItems.forEach(li => {
    const a = li.querySelector('a[href]');
    if (!a) return;
    const href = a.getAttribute('href');
    if (!href) return;
    if (!prefixes.some(pre => href.startsWith(pre))) {
      li.classList.add('hidden');
    } else {
      li.classList.remove('hidden');
    }
  });

  const visible = Array.from(document.querySelectorAll('.book-menu nav > ul > li'))
    .filter(li => !li.classList.contains('hidden'));
  if (visible.length <= 1) {
    const content = document.querySelector('.book-menu .book-menu-content');
    if (content && !document.querySelector('.book-menu-fallback')) {
      const fb = document.createElement('div');
      fb.className = 'book-menu-fallback';
      fb.innerHTML = `
        <ul class="fallback-links">
          <li><a href="/about/">About</a></li>
          <li><a href="/docs/">Docs</a></li>
          <li><a href="/courses/">Courses</a></li>
          <li><a href="/community/">Community</a></li>
          <li><a href="/tutorials/">Tutorials</a></li>
          <li><a href="/blogs/">Blogs</a></li>
        </ul>`;
      content.insertBefore(fb, content.firstChild);
      const originalUL = content.querySelector('nav > ul');
      if (originalUL) originalUL.classList.add('fallback-hidden');
      const activeHref = section === 'resources' ? '/resources/' : `/${section}/`;
      const active = fb.querySelector(`a[href="${activeHref}"]`);
      if (active) active.classList.add('active');
    }
  }
}

export function setupCollapsibleSidebar(): void {
  const isLanding = (p: string) => ['/', '/index.html', '/docs/', '/courses/', '/tutorials/', '/blogs/', '/community/'].includes(p);
  const path = window.location.pathname;
  const nodes = document.querySelectorAll('.book-menu nav li li');
  nodes.forEach(li => {
    const hasChildren = !!li.querySelector(':scope > ul');
    if (!hasChildren) return;
    li.classList.add('collapsible');
    li.classList.remove('expanded');
    if (!li.querySelector(':scope > .sruja-collapse-toggle')) {
      const anchor = li.querySelector(':scope > a, :scope > span');
      const toggle = document.createElement('span');
      toggle.className = 'sruja-collapse-toggle';
      toggle.textContent = '▸';
      toggle.addEventListener('click', (e) => {
        e.stopPropagation();
        li.classList.toggle('expanded');
        toggle.textContent = li.classList.contains('expanded') ? '▾' : '▸';
      });
      if (anchor) anchor.after(toggle);
    }
  });

  document.querySelectorAll('.book-menu li li.expanded').forEach(li => li.classList.remove('expanded'));
  document.querySelectorAll('.book-menu li li .sruja-collapse-toggle').forEach(t => t.textContent = '▸');

  const active = document.querySelector('.book-menu a.active') || document.querySelector(`.book-menu a[href="${path}"]`);
  if (active && !isLanding(path)) {
    let li = active.closest('li');
    while (li) {
      if (li.classList.contains('collapsible')) {
        li.classList.add('expanded');
        const t = li.querySelector(':scope > .sruja-collapse-toggle');
        if (t) t.textContent = '▾';
      }
      li = li.parentElement?.closest('li') || null;
    }
  }
}

// Footer rendering moved to React component Footer.tsx
