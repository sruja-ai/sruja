// Navigation utilities
import type { Section, NavLink } from '../types';

export function getSection(): Section {
  const p = window.location.pathname;
  if (p.startsWith('/playground')) return 'playground';
  if (p.startsWith('/about')) return 'about';
  if (p.startsWith('/learn/')) return 'learn';
  if (p.startsWith('/courses/')) return 'courses';
  if (p.startsWith('/docs/')) return 'docs';
  if (p.startsWith('/tutorials/')) return 'tutorials';
  if (p.startsWith('/blogs/')) return 'blogs';
  if (p.startsWith('/community/')) return 'community';
  return 'home';
}

export function linksFor(section: Section): NavLink[] {
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

export function navLinksHTML(section: Section): string {
  const links = linksFor(section);
  return links.map(l => {
    const href = l.href.replace(/"/g, '&quot;');
    const label = l.label.replace(/</g, '&lt;').replace(/>/g, '&gt;');
    return `<a href="${href}">${label}</a>`;
  }).join('');
}

export function globalLinksHTML(section: Section): string {
  const links = linksFor(section);
  const learnActive = section === 'learn' ? ' class="active"' : '';
  return `
    <a href="/playground/"${section === 'playground' ? ' class="active"' : ''}>Playground</a>
    <div class="nav-item dropdown">
      <a href="/learn/"${learnActive}>Learn</a>
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
