import React, { useState, useEffect, useRef } from "react";
import "./SplitLayout.css";

interface SplitLayoutProps {
  leftContent: React.ReactNode;
  rightContent: React.ReactNode;
  minLeftWidth?: number;
  minRightWidth?: number;
  defaultSplit?: number; // Check if user has preference
  isLeftVisible?: boolean;
  onCollapse?: () => void;
  onExpand?: () => void;
}

export function SplitLayout({
  leftContent,
  rightContent,
  minLeftWidth = 300,
  minRightWidth = 400,
  defaultSplit = 40, // 40% left, 60% right
  isLeftVisible = true,
  onCollapse,
  onExpand,
}: SplitLayoutProps) {
  const [splitPos, setSplitPos] = useState(defaultSplit);
  const [isResizing, setIsResizing] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  // Restore split position from local storage
  useEffect(() => {
    const saved = localStorage.getItem("sruja-split-layout-pos");
    if (saved) {
      setSplitPos(parseFloat(saved));
    }
  }, []);

  // Sync with prop changes - ensure component reacts to isLeftVisible changes
  useEffect(() => {
    // This ensures the component re-renders when isLeftVisible changes
  }, [isLeftVisible]);

  const startResizing = (e: React.MouseEvent) => {
    // Don't start resizing if clicking on the collapse button
    const target = e.target as HTMLElement;
    if (target.closest(".split-collapse-btn")) {
      return;
    }
    setIsResizing(true);
    e.preventDefault();
  };

  // Removed stopResizing - now handled in useEffect

  useEffect(() => {
    if (!isResizing) return;

    const resize = (e: MouseEvent) => {
      if (containerRef.current) {
        const containerWidth = containerRef.current.offsetWidth;
        const newLeftWidth = e.clientX - containerRef.current.getBoundingClientRect().left;
        let newSplitPos = (newLeftWidth / containerWidth) * 100;

        // Constraints
        const minLeftPercent = (minLeftWidth / containerWidth) * 100;
        const maxLeftPercent = 100 - (minRightWidth / containerWidth) * 100;

        if (newSplitPos < minLeftPercent) newSplitPos = minLeftPercent;
        if (newSplitPos > maxLeftPercent) newSplitPos = maxLeftPercent;

        setSplitPos(newSplitPos);
      }
    };

    const handleMouseUp = () => {
      setIsResizing(false);
      // Save preference
      if (containerRef.current) {
        const currentSplit = splitPos;
        localStorage.setItem("sruja-split-layout-pos", currentSplit.toString());
      }
    };

    window.addEventListener("mousemove", resize);
    window.addEventListener("mouseup", handleMouseUp);

    return () => {
      window.removeEventListener("mousemove", resize);
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, [isResizing, minLeftWidth, minRightWidth, splitPos]);

  return (
    <div className={`split-layout-container ${isResizing ? "resizing" : ""}`} ref={containerRef}>
      {isLeftVisible ? (
        <>
          <div className="split-pane left-pane" style={{ width: `${splitPos}%` }}>
            {leftContent}
          </div>
          <div className="split-resizer" onMouseDown={startResizing} title="Drag to resize">
            <div className="resizer-handle" />
            {onCollapse && (
              <button
                className="split-collapse-btn"
                onClick={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  onCollapse();
                }}
                onMouseDown={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                }}
                onMouseUp={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                }}
                title="Collapse panel"
                type="button"
              >
                <svg
                  width="12"
                  height="12"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <polyline points="15 18 9 12 15 6"></polyline>
                </svg>
              </button>
            )}
          </div>
        </>
      ) : (
        /* Collapsed Gutter */
        <div className="split-collapsed-gutter">
          {onExpand && (
            <button className="split-expand-btn" onClick={onExpand} title="Expand panel">
              <svg
                width="12"
                height="12"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <polyline points="9 18 15 12 9 6"></polyline>
              </svg>
            </button>
          )}
        </div>
      )}

      <div
        className="split-pane right-pane"
        style={{
          width: isLeftVisible ? `${100 - splitPos}%` : undefined,
          flex: isLeftVisible ? undefined : 1,
        }}
      >
        {rightContent}
      </div>
    </div>
  );
}
