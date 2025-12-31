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
        <h1>Architecture Editor with Live Code Sync</h1>
        <p>
          Edit diagrams or code - changes sync both ways automatically. Like Notion, but for
          architecture. Version-controlled, validated, beautiful.
        </p>

        {/* Value Propositions for Different Audiences */}
        <div className="hero-audiences">
          <div className="audience-card">
            <strong>🔄 Bidirectional Sync</strong>
            <p>Edit visually → Code updates. Edit code → Diagram updates. Real-time, both ways.</p>
          </div>
          <div className="audience-card">
            <strong>✅ For Everyone</strong>
            <p>
              Designers edit visually, developers edit code. Same source of truth,
              version-controlled in Git.
            </p>
          </div>
          <div className="audience-card">
            <strong>📊 Built-in Validation</strong>
            <p>
              Cycle detection, orphan detection, unique IDs, and custom governance rules out of the
              box.
            </p>
          </div>
          <div className="audience-card">
            <strong>🎯 Multiple Outputs</strong>
            <p>Export to JSON, Markdown, Mermaid, or keep as .sruja files in Git. Your choice.</p>
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
