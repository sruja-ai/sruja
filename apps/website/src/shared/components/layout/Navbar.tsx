// apps/website/src/shared/components/layout/Navbar.tsx
import { useState, useEffect, useRef } from "react";
import { Logo, ThemeToggle, MantineProvider, ThemeProvider } from "@sruja/ui";
import "@sruja/ui/design-system/styles.css";
import DocSearchBox from "@/features/search/components/DocSearchBox";

export default function Navbar() {
  const [learnMenuOpen, setLearnMenuOpen] = useState(false);
  const [isHydrated, setIsHydrated] = useState(false);
  const learnMenuRef = useRef<HTMLDivElement>(null);
  const closeTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    // Mark as hydrated after component mounts
    setIsHydrated(true);
  }, []);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (learnMenuOpen) setLearnMenuOpen(false);
      }
    };

    const handleClickOutside = (event: MouseEvent) => {
      if (learnMenuRef.current && !learnMenuRef.current.contains(event.target as Node)) {
        setLearnMenuOpen(false);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    document.addEventListener("mousedown", handleClickOutside);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      document.removeEventListener("mousedown", handleClickOutside);
      if (closeTimeoutRef.current) {
        clearTimeout(closeTimeoutRef.current);
      }
    };
  }, [learnMenuOpen]);

  const handleLearnMouseEnter = () => {
    if (closeTimeoutRef.current) {
      clearTimeout(closeTimeoutRef.current);
      closeTimeoutRef.current = null;
    }
    setLearnMenuOpen(true);
  };

  const handleLearnMouseLeave = () => {
    if (closeTimeoutRef.current) {
      clearTimeout(closeTimeoutRef.current);
      closeTimeoutRef.current = null;
    }
    closeTimeoutRef.current = setTimeout(() => {
      setLearnMenuOpen(false);
    }, 200);
  };

  return (
    <MantineProvider>
      <ThemeProvider defaultMode="system">
        <nav className="navbar">
        <div className="nav-container">
          <a href="/" className="nav-logo">
            <Logo size={32} />
            <span>Sruja</span>
          </a>
          <div className="nav-links">
            <a href="/docs">Docs</a>
            <div
              className="nav-dropdown"
              ref={learnMenuRef}
              onMouseEnter={handleLearnMouseEnter}
              onMouseLeave={handleLearnMouseLeave}
            >
              <a
                href="/learn"
                className="nav-dropdown-trigger"
                onClick={(e) => {
                  // Allow click to navigate, but also toggle dropdown on click
                  if (e.ctrlKey || e.metaKey) {
                    // Allow Ctrl/Cmd+click to open in new tab
                    return;
                  }
                  // On regular click, navigate to /learn (default link behavior)
                }}
              >
                Learn
                <svg
                  width="12"
                  height="12"
                  viewBox="0 0 12 12"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  style={{
                    marginLeft: "4px",
                    transform: learnMenuOpen ? "rotate(180deg)" : "rotate(0deg)",
                    transition: "transform 0.2s",
                  }}
                >
                  <path d="m3 4.5 3 3 3-3" />
                </svg>
              </a>
              {learnMenuOpen && (
                <div className="nav-dropdown-menu">
                  <a href="/courses" onClick={() => setLearnMenuOpen(false)}>
                    Courses
                  </a>
                  <a href="/tutorials" onClick={() => setLearnMenuOpen(false)}>
                    Tutorials
                  </a>
                  <a href="/blogs" onClick={() => setLearnMenuOpen(false)}>
                    Blogs
                  </a>
                  <a href="/challenges" onClick={() => setLearnMenuOpen(false)}>
                    Challenges
                  </a>
                </div>
              )}
            </div>
            <a href="/designer">Designer</a>
            <a href="https://github.com/sruja-ai/sruja" target="_blank" rel="noopener noreferrer">
              GitHub
            </a>
            <DocSearchBox />
            <ThemeToggle iconOnly size="sm" tabIndex={isHydrated ? 0 : -1} />
          </div>
        </div>
      </nav>
      </ThemeProvider>
    </MantineProvider>
  );
}
