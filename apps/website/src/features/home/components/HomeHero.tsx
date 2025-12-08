// apps/website/src/features/home/components/HomeHero.tsx
import { useState, useEffect } from 'react';
import { Button, Logo } from '@sruja/ui';
import '@sruja/ui/design-system/styles.css';
import AlgoliaSearch from '@/features/search/components/AlgoliaSearch';

export default function HomeHero() {
  const [searchOpen, setSearchOpen] = useState(false);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        setSearchOpen(true);
      }
      if (e.key === 'Escape' && searchOpen) {
        setSearchOpen(false);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [searchOpen]);

  return (
    <div className="hero">
      <div style={{ display: 'flex', justifyContent: 'center', marginBottom: 12 }}>
        <Logo size={56} />
      </div>
      <h1>Build Better Software Systems</h1>
      <p>
        <strong>Sruja</strong> is the developer-friendly language for defining, visualizing, and validating software architecture. 
        Our vision is to bring governance to software architecture and assist AI-driven development.
      </p>
      <p>
        Visit <a href="/studio">Studio</a> for interactive modeling.
      </p>
      <div className="hero-actions">
        <Button variant="primary" onClick={() => (window.location.href = '/docs/intro')}>
          Get Started
        </Button>
        <Button variant="secondary" onClick={() => (window.location.href = '/studio')}>
          Open Studio
        </Button>
        <Button variant="outline" onClick={() => setSearchOpen(true)}>
          Search
        </Button>
      </div>
      <AlgoliaSearch 
        isOpen={searchOpen} 
        onClose={() => setSearchOpen(false)} 
      />
    </div>
  );
}

