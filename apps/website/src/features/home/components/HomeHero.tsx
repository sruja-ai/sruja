// apps/website/src/features/home/components/HomeHero.tsx
import { useState, useEffect } from "react";
import { Button, Logo, MantineProvider } from "@sruja/ui";
import "@sruja/ui/design-system/styles.css";
import AlgoliaSearch from "@/features/search/components/AlgoliaSearch";
import { getDesignerUrl } from "@/utils/designer-url";
import LiveSrujaBlock from "@/features/playground/components/LiveSrujaBlock";

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
        <h1>Write Code. Get Diagrams. Never Outdated.</h1>
        <p>
          Design, validate, and document your software architecture as code. Write simple text, get
          beautiful diagrams that stay in sync with your code.
        </p>

        {/* Interactive Demo */}
        <div
          style={{
            margin: "3rem 0 2rem",
            maxWidth: "900px",
            marginInline: "auto",
          }}
        >
          <p
            style={{
              fontSize: "1rem",
              color: "var(--color-text-primary)",
              marginBottom: "1.5rem",
              fontWeight: 600,
              textAlign: "center",
            }}
          >
            ✨ Try it live: Edit the code and watch the diagram update automatically
          </p>
          <LiveSrujaBlock
            initialDsl={`import { * } from 'sruja.ai/stdlib'

User = person "Customer"
App = system "E-commerce Platform" {
  Web = container "React App"
  API = container "Node.js API"
  DB = database "PostgreSQL"
}

User -> App.Web "Visits"
App.Web -> App.API "Calls"
App.API -> App.DB "Stores Data"

view index {
  include *
}
`}
          />
        </div>

        {/* Why Sruja? */}
        <div style={{ margin: "2.5rem 0 1.5rem", maxWidth: "700px", marginInline: "auto" }}>
          <p style={{ fontSize: "1.1rem", color: "var(--color-text-primary)", fontWeight: 500 }}>
            Traditional diagrams rot. Wikis get outdated. Sruja keeps your architecture live, valid,
            and version-controlled.
          </p>
        </div>

        <div className="hero-audiences">
          <div className="audience-card">
            <strong>🔄 Prevent Drift</strong>
            <p>
              Code and architecture stay in sync automatically. Bidirectional updates mean your
              diagrams never lie.
            </p>
          </div>
          <div className="audience-card">
            <strong>✅ Single Source of Truth</strong>
            <p>
              Version-controlled in Git. Keep your design, implementation, and documentation aligned
              in one place.
            </p>
          </div>
          <div className="audience-card">
            <strong>🛡️ Enforce Standards</strong>
            <p>
              Define governance rules as code. Automated linting ensures every service meets your
              architectural standards.
            </p>
          </div>
          <div className="audience-card">
            <strong>👁️ Visualize Anything</strong>
            <p>
              Generate Context, Container, Component, and Deployment views from a single model.
              Export to C4, Mermaid, and more.
            </p>
          </div>
        </div>

        <p>
          Try the <a href={getDesignerUrl()}>Sruja Designer</a> to see bidirectional sync in action,
          explore <a href="/docs/examples">real-world examples</a>, or start with our{" "}
          <a href="/courses">comprehensive courses</a>.
        </p>
        <div className="hero-actions">
          <Button
            variant="primary"
            onClick={() => (window.location.href = getDesignerUrl())}
            style={{ fontSize: "1.1rem", padding: "0.75rem 1.5rem" }}
          >
            🚀 Try Now in Designer
          </Button>
          <Button
            variant="secondary"
            onClick={() => (window.location.href = "/docs/getting-started")}
          >
            Get Started
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
