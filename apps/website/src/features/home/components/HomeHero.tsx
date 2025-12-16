// apps/website/src/features/home/components/HomeHero.tsx
import { useState, useEffect } from "react";
import { Button, Logo } from "@sruja/ui";
import "@sruja/ui/design-system/styles.css";
import AlgoliaSearch from "@/features/search/components/AlgoliaSearch";

export default function HomeHero() {
  const [searchOpen, setSearchOpen] = useState(false);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        setSearchOpen(true);
      }
      if (e.key === "Escape" && searchOpen) {
        setSearchOpen(false);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [searchOpen]);

  return (
    <div className="hero">
      <div style={{ display: "flex", justifyContent: "center", marginBottom: 12 }}>
        <Logo size={56} />
      </div>
      <h1>Build Better Software Systems</h1>
      <p>
        <strong>Sruja</strong> is an open source architecture-as-code language for defining,
        visualizing, and validating software architecture. Built by and for the community, with a
        vision to evolve into a platform for live system review and architectural governance.
      </p>
      <p>
        Try the <a href="/designer">Sruja Designer</a> for interactive visualization. Studio is
        coming soon.
      </p>
      <div className="hero-actions">
        <Button variant="primary" onClick={() => (window.location.href = "/docs/getting-started")}>
          Get Started
        </Button>
        <Button variant="secondary" onClick={() => (window.location.href = "/designer")}>
          Open Designer
        </Button>
        <Button variant="outline" onClick={() => setSearchOpen(true)}>
          Search
        </Button>
      </div>
      <AlgoliaSearch isOpen={searchOpen} onClose={() => setSearchOpen(false)} />
    </div>
  );
}
