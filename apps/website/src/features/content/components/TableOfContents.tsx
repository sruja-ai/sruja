// apps/website/src/features/content/components/TableOfContents.tsx
import { useEffect, useState } from 'react';
import './TableOfContents.css';

interface Heading {
  id: string;
  text: string;
  level: number;
}

export default function TableOfContents() {
  const [headings, setHeadings] = useState<Heading[]>([]);
  const [activeId, setActiveId] = useState<string>('');

  useEffect(() => {
    // Extract headings from the content
    const contentElement = document.querySelector('.content-body');
    if (!contentElement) return;

    const headingElements = contentElement.querySelectorAll('h2, h3, h4');
    const extractedHeadings: Heading[] = [];

    headingElements.forEach((heading) => {
      let id = heading.id;
      
      // Generate ID if not present
      if (!id) {
        const text = heading.textContent || '';
        id = text
          .toLowerCase()
          .replace(/\s+/g, '-')
          .replace(/[^a-z0-9-]/g, '')
          .replace(/-+/g, '-')
          .replace(/^-|-$/g, '');
        
        // Ensure unique ID
        let uniqueId = id;
        let counter = 1;
        while (document.getElementById(uniqueId)) {
          uniqueId = `${id}-${counter}`;
          counter++;
        }
        id = uniqueId;
        heading.id = id;
      }
      
      if (id) {
        extractedHeadings.push({
          id,
          text: heading.textContent || '',
          level: parseInt(heading.tagName.charAt(1)),
        });
      }
    });

    setHeadings(extractedHeadings);

    // Set up intersection observer for active heading
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            setActiveId(entry.target.id);
          }
        });
      },
      {
        rootMargin: '-100px 0px -66% 0px',
        threshold: 0,
      }
    );

    headingElements.forEach((heading) => {
      if (heading.id) {
        observer.observe(heading);
      }
    });

    return () => {
      observer.disconnect();
    };
  }, []);

  if (headings.length === 0) {
    return null;
  }

  const scrollToHeading = (id: string) => {
    const element = document.getElementById(id);
    if (element) {
      const offset = 100; // Account for sticky navbar
      const elementPosition = element.getBoundingClientRect().top;
      const offsetPosition = elementPosition + window.pageYOffset - offset;

      window.scrollTo({
        top: offsetPosition,
        behavior: 'smooth',
      });
    }
  };

  return (
    <nav className="toc">
      <div className="toc-header">
        <h3>On this page</h3>
      </div>
      <ul className="toc-list">
        {headings.map((heading) => (
          <li
            key={heading.id}
            className={`toc-item toc-level-${heading.level} ${activeId === heading.id ? 'active' : ''}`}
          >
            <a
              href={`#${heading.id}`}
              onClick={(e) => {
                e.preventDefault();
                scrollToHeading(heading.id);
              }}
              className="toc-link"
            >
              {heading.text}
            </a>
          </li>
        ))}
      </ul>
    </nav>
  );
}

