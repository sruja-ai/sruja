// Course State Management using localStorage

const COURSE_KEY = 'sruja_course_state';

// Build section-specific links
// Build section-specific links
function getSection() {
    const p = window.location.pathname;
    if (p.startsWith('/playground')) return 'playground';
    if (p.startsWith('/about')) return 'about';
    if (p.startsWith('/resources/') || p.startsWith('/docs/') || p.startsWith('/courses/') || p.startsWith('/tutorials/') || p.startsWith('/blogs/')) return 'resources';
    if (p.startsWith('/community/')) return 'community';
    return 'home';
}

function linksFor(section) {
    switch (section) {
        case 'courses':
            return [
                { href: '/courses/system-design-101/', label: 'Course Home' },
                { href: '/courses/system-design-101/module-1-fundamentals/', label: 'Modules' },
                { href: '/courses/quiz/', label: 'Quiz' }
            ];
        case 'docs':
            return [
                { href: '/docs/getting-started/', label: 'Getting Started' },
                { href: '/docs/concepts/', label: 'Concepts' },
                { href: '/docs/reference/', label: 'Reference' },
                { href: '/docs/cli/', label: 'CLI' }
            ];
        case 'tutorials':
            return [
                { href: '/tutorials/', label: 'Tutorials Home' }
            ];
        case 'blogs':
            return [
                { href: '/blogs/', label: 'Blog Home' }
            ];
        default:
            return [
                { href: '/courses/system-design-101/', label: 'Courses' },
                { href: '/docs/', label: 'Docs' },
                { href: '/tutorials/', label: 'Tutorials' },
                { href: '/blogs/', label: 'Blogs' }
            ];
    }
}

function navLinksHTML(section) {
    const links = linksFor(section);
    return links.map(l => `<a href="${l.href}">${l.label}</a>`).join('');
}

function globalLinksHTML(section) {
    const resourcesActive = section === 'resources' ? ' class="active"' : '';
    return `
        <a href="/playground/"${section === 'playground' ? ' class="active"' : ''}>Playground</a>
        <div class="nav-item dropdown">
            <a href="/resources/"${resourcesActive}>Resources</a>
            <div class="nav-dropdown">
                <a href="/docs/">Docs</a>
                <a href="/courses/">Courses</a>
                <a href="/tutorials/">Tutorials</a>
                <a href="/blogs/">Blogs</a>
            </div>
        </div>
        <a href="/about/"${section === 'about' ? ' class="active"' : ''}>About</a>
        <a href="/community/"${section === 'community' ? ' class="active"' : ''}>Community</a>
    `;
}

// Inject Top Navigation Bar (section-aware)
function injectTopNav() {
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
    document.body.insertBefore(div.firstElementChild, document.body.firstChild);

    const toggle = document.querySelector('.nav-toggle');
    const mobileMenu = document.querySelector('.nav-mobile-menu');
    if (toggle && mobileMenu) {
        toggle.addEventListener('click', () => {
            mobileMenu.classList.toggle('active');
        });
    }

    // If WASM is initializing, show spinning logo immediately
    if (window.srujaWasmInitializing) {
        const logo = document.querySelector('.nav-logo');
        if (logo) logo.classList.add('spin');
    }

    // Fallback: if nav-links somehow empty, rebuild with defaults
    const navLinks = document.querySelector('.nav-links');
    if (navLinks) {
        const anchors = navLinks.querySelectorAll('a[href]');
        if (anchors.length < 3) {
            navLinks.innerHTML = `${globalLinksHTML(section)}<a href="https://github.com/sruja-ai/sruja" target="_blank" class="nav-github">GitHub</a>`;
        }
        // Ensure active highlight for current section
        const setActive = (sel) => {
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
        // Empty-state suggestions for centered search
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
            const q = (input.value || '').trim();
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

function filterSidebarBySection() {
    const section = getSection();
    if (section === 'home') return;
    const prefixMap = {
        resources: ['/resources/', '/docs/', '/courses/', '/tutorials/', '/blogs/'],
        community: '/community/'
    };
    const prefixes = Array.isArray(prefixMap[section]) ? prefixMap[section] : [prefixMap[section]];
    const topItems = document.querySelectorAll('.book-menu nav > ul > li');
    topItems.forEach(li => {
        const a = li.querySelector('a[href]');
        if (!a) return;
        const href = a.getAttribute('href');
        if (!prefixes.some(pre => href.startsWith(pre))) {
            li.classList.add('hidden');
        } else {
            li.classList.remove('hidden');
        }
    });

    // Fallback: if sidebar becomes empty or nearly empty, render default top-level items
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
            // Hide original nav list to avoid duplication
            const originalUL = content.querySelector('nav > ul');
            if (originalUL) originalUL.classList.add('fallback-hidden');
            // Highlight current top-level item
            const activeHref = section === 'resources' ? '/resources/' : `/${section}/`;
            const active = fb.querySelector(`a[href="${activeHref}"]`);
            if (active) active.classList.add('active');
        }
    }
}

function setupCollapsibleSidebar() {
    const isLanding = (p) => ['/', '/index.html', '/docs/', '/courses/', '/tutorials/', '/blogs/', '/community/'].includes(p);
    const path = window.location.pathname;
    // Only make nested items collapsible; keep root sections visible
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

    // Collapse nested by default; roots stay visible
    document.querySelectorAll('.book-menu li li.expanded').forEach(li => li.classList.remove('expanded'));
    document.querySelectorAll('.book-menu li li .sruja-collapse-toggle').forEach(t => t.textContent = '▸');

    // Expand only the active branch on content pages
    const active = document.querySelector('.book-menu a.active') || document.querySelector(`.book-menu a[href="${path}"]`);
    if (active && !isLanding(path)) {
        let li = active.closest('li');
        while (li) {
            if (li.classList.contains('collapsible')) {
                li.classList.add('expanded');
                const t = li.querySelector(':scope > .sruja-collapse-toggle');
                if (t) t.textContent = '▾';
            }
            li = li.parentElement && li.parentElement.closest('li');
        }
    }
}

// Run injection
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', injectTopNav);
    document.addEventListener('DOMContentLoaded', filterSidebarBySection);
    document.addEventListener('DOMContentLoaded', setupCollapsibleSidebar);
    document.addEventListener('DOMContentLoaded', injectFooter);
} else {
    injectTopNav();
    filterSidebarBySection();
    setupCollapsibleSidebar();
    injectFooter();
}

// Inject Custom CSS
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

function injectFooter() {
    if (document.querySelector('.site-footer')) return;
    const footerHTML = `
    <footer class="site-footer">
      <div class="footer-container">
        <div class="footer-links">
          <a href="/docs/">Docs</a>
          <a href="/courses/">Courses</a>
          <a href="/tutorials/">Tutorials</a>
          <a href="/blogs/">Blogs</a>
          <a href="/community/">Community</a>
          <a href="https://github.com/sruja-ai/sruja" target="_blank">GitHub</a>
        </div>
        <div class="footer-copy">© ${new Date().getFullYear()} Sruja. All rights reserved.</div>
      </div>
    </footer>`;
    const div = document.createElement('div');
    div.innerHTML = footerHTML;
    document.body.appendChild(div.firstElementChild);
}

// Initialize state if not present
function getCourseState() {
    const state = localStorage.getItem(COURSE_KEY);
    if (state) {
        return JSON.parse(state);
    }
    return {
        visited: [],
        quizResults: {},
        lastVisited: null
    };
}

function saveCourseState(state) {
    localStorage.setItem(COURSE_KEY, JSON.stringify(state));
}

// Track page visit
function trackPageVisit() {
    const path = window.location.pathname;
    // Only track course pages
    if (!path.includes('/courses/')) {
        return;
    }

    const state = getCourseState();
    if (!state.visited.includes(path)) {
        state.visited.push(path);
    }
    state.lastVisited = path;
    saveCourseState(state);
    updateProgressUI();
}

// Update UI with progress
function updateProgressUI() {
    const state = getCourseState();
    // Example: Add a checkmark to sidebar links
    const links = document.querySelectorAll('a');
    links.forEach(link => {
        const href = link.getAttribute('href');
        if (href && state.visited.some(v => href.endsWith(v) || v.endsWith(href))) {
            link.classList.add('visited');
            // Optional: Add checkmark icon
            // link.innerHTML += ' ✓'; 
        }
    });

    // Show "Resume Course" button on course home
    const resumeContainer = document.getElementById('resume-course-container');
    if (resumeContainer && state.lastVisited) {
        resumeContainer.innerHTML = `<a href="${state.lastVisited}" class="btn btn-primary">Resume Course</a>`;
    }
}

// Quiz Logic
function submitQuiz(quizId, answers) {
    const state = getCourseState();
    state.quizResults[quizId] = answers;
    saveCourseState(state);
}

document.addEventListener('DOMContentLoaded', () => {
    trackPageVisit();
});

// =============================
// Sruja WASM initialization
// =============================
window.srujaWasmReady = window.srujaWasmReady || false;
window.srujaWasmInitializing = window.srujaWasmInitializing || false;

function setLogoSpin(on) {
    const logo = document.querySelector('.nav-logo');
    if (logo) {
        if (on) logo.classList.add('spin');
        else logo.classList.remove('spin');
    }
}

function loadScript(src, onload) {
    const s = document.createElement('script');
    s.src = src;
    s.async = true;
    if (onload) s.onload = onload;
    document.head.appendChild(s);
}

function initSrujaWasm() {
    if (window.srujaWasmInitializing || window.srujaWasmReady) return;
    window.srujaWasmInitializing = true;
    setLogoSpin(true);
    loadScript('/wasm_exec.js', () => {
        if (typeof Go === 'undefined') {
            console.error('Go WASM runtime not available');
            setLogoSpin(false);
            window.srujaWasmInitializing = false;
            return;
        }
        try {
            const go = new Go();
            WebAssembly.instantiateStreaming(fetch('/sruja.wasm'), go.importObject)
                .then(result => {
                    go.run(result.instance);
                    window.srujaWasmReady = true;
                    window.srujaWasmInitializing = false;
                    setLogoSpin(false);
                })
                .catch(err => console.error('Failed to init Sruja WASM', err));
        } catch (e) {
            console.error('WASM init error', e);
            setLogoSpin(false);
            window.srujaWasmInitializing = false;
        }
    });
}

// =============================
// Enhance Sruja code blocks
// =============================
function slugify(s) {
    return s.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '') || 'snippet';
}

function enhanceSrujaBlocks() {
    const blocks = document.querySelectorAll('pre > code.language-sruja');
    const pageSlug = slugify(window.location.pathname.replace(/\//g, '-'));
    blocks.forEach((code, idx) => {
        const pre = code.parentElement;
        if (pre.dataset.enhanced === 'true') return;
        pre.dataset.enhanced = 'true';

        const wrapper = document.createElement('div');
        wrapper.className = 'sruja-code-wrapper';

        const toolbar = document.createElement('div');
        toolbar.className = 'sruja-code-toolbar';

        const btnCopy = document.createElement('button');
        btnCopy.className = 'sruja-btn sruja-btn-copy';
        btnCopy.title = 'Copy';
        btnCopy.innerHTML = '📋';

        const btnEdit = document.createElement('button');
        btnEdit.className = 'sruja-btn sruja-btn-edit';
        btnEdit.title = 'Edit';
        btnEdit.innerHTML = '✏️';

        const btnRun = document.createElement('button');
        btnRun.className = 'sruja-btn sruja-btn-run';
        btnRun.title = 'Run';
        btnRun.innerHTML = '▶';


        const output = document.createElement('div');
        output.className = 'sruja-run-output';
        output.style.display = 'none';

        // Insert DOM
        pre.parentNode.insertBefore(wrapper, pre);
        wrapper.appendChild(pre);
        wrapper.appendChild(toolbar);
        toolbar.appendChild(btnCopy);
        toolbar.appendChild(btnEdit);
        toolbar.appendChild(btnRun);
        wrapper.appendChild(output);

        // Helper to get current source (code or editor)
        let editor = null;
        const currentSource = () => (editor ? editor.value : code.textContent);

        // Copy
        btnCopy.addEventListener('click', () => {
            const text = currentSource();
            navigator.clipboard && navigator.clipboard.writeText(text);
            btnCopy.classList.add('success');
            setTimeout(() => btnCopy.classList.remove('success'), 1200);
        });

        // Edit toggle
        btnEdit.addEventListener('click', () => {
            if (!editor) {
                editor = document.createElement('textarea');
                editor.className = 'sruja-editor';
                editor.value = code.textContent;
                pre.style.display = 'none';
                wrapper.insertBefore(editor, toolbar.nextSibling);
                btnEdit.classList.add('active');
            } else {
                // collapse back to highlighted view, keep edited text
                code.textContent = editor.value;
                editor.remove();
                editor = null;
                pre.style.display = '';
                btnEdit.classList.remove('active');
            }
        });

        // Run
        btnRun.addEventListener('click', () => {
            initSrujaWasm();
            if (!srujaWasmReady || typeof compileSruja === 'undefined') {
                output.style.display = 'block';
                output.innerText = 'WASM not ready';
                return;
            }
            output.style.display = 'block';
            output.innerHTML = '';
            try {
                const filename = `page-${pageSlug}-snippet-${idx}.sruja`;
                const result = compileSruja(currentSource(), filename);
                if (result && result.error) {
                    output.innerText = result.error;
                } else if (result && result.svg) {
                    output.innerHTML = result.svg;
                } else {
                    output.innerText = 'No output';
                }
            } catch (e) {
                output.innerText = 'Internal Error: ' + e;
                }
        });

        btnRun.disabled = false;
        btnRun.title = 'Run';
    });
}

document.addEventListener('DOMContentLoaded', () => {
    initSrujaWasm();
    enhanceSrujaBlocks();
});
    // Theme toggle
    const rootEl = document.documentElement;
    function applyTheme(theme) {
        rootEl.classList.remove('theme-dark', 'theme-light');
        if (theme === 'dark') rootEl.classList.add('theme-dark');
        else if (theme === 'light') rootEl.classList.add('theme-light');
        localStorage.setItem('sruja_theme', theme);
        const btn = document.querySelector('.theme-toggle');
        if (btn) btn.textContent = theme === 'dark' ? '☀️' : '🌙';
    }
    const savedTheme = localStorage.getItem('sruja_theme');
    if (savedTheme) applyTheme(savedTheme);
    const themeBtn = document.querySelector('.theme-toggle');
    if (themeBtn) {
        themeBtn.addEventListener('click', () => {
            const next = (localStorage.getItem('sruja_theme') === 'dark') ? 'light' : 'dark';
            applyTheme(next);
        });
    }
