// apps/website/src/features/home/components/HomeHero.tsx
import { useState, useEffect } from "react";
import { Button, Logo, MantineProvider } from "@sruja/ui";
import "@sruja/ui/design-system/styles.css";
import AlgoliaSearch from "@/features/search/components/AlgoliaSearch";
import { getDesignerUrl } from "@/utils/designer-url";

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
    <MantineProvider>
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

        {/* Value Propositions for Different Audiences */}
        <div className="hero-audiences">
          <div className="audience-card">
            <strong>For Students</strong>
            <p>
              Learn system design with real-world examples, production-ready patterns, and hands-on
              courses.
            </p>
          </div>
          <div className="audience-card">
            <strong>For Architects</strong>
            <p>
              Enforce standards, prevent drift, and scale governance across teams with
              policy-as-code.
            </p>
          </div>
          <div className="audience-card">
            <strong>For Product Teams</strong>
            <p>
              Link requirements to architecture, track SLOs, and align technical decisions with
              business goals.
            </p>
          </div>
          <div className="audience-card">
            <strong>For DevOps</strong>
            <p>
              Integrate architecture validation into CI/CD, automate documentation, and model
              deployments.
            </p>
          </div>
        </div>

        <p>
          Try the <a href={getDesignerUrl()}>Sruja Designer</a> for interactive visualization,
          explore <a href="/docs/examples">real-world examples</a> from fintech, healthcare, and
          e-commerce, or start with our <a href="/courses">comprehensive courses</a>.
        </p>
        <div className="hero-actions">
          <Button
            variant="primary"
            onClick={() => (window.location.href = "/docs/getting-started")}
          >
            Get Started
          </Button>
          <Button variant="secondary" onClick={() => (window.location.href = getDesignerUrl())}>
            Open Designer
          </Button>
          <Button variant="outline" onClick={() => (window.location.href = "/docs/examples")}>
            View Examples
          </Button>
          <Button variant="outline" onClick={() => setSearchOpen(true)}>
            Search
          </Button>
        </div>
        <AlgoliaSearch isOpen={searchOpen} onClose={() => setSearchOpen(false)} />
      </div>
    </MantineProvider>
  );
}
