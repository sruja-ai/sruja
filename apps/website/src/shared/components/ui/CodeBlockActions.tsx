import React, { useEffect } from "react";
import { createRoot } from "react-dom/client";
import { SrujaLoader, SrujaMonacoEditor } from "@sruja/ui";
import { initWasm, capture, convertDslToMermaid } from "@sruja/shared";
// Removed DiagramPreview import - using mermaid only for expand
import mermaid from "mermaid";
import * as LZString from "lz-string";

let mermaidInitialized = false;
function getMermaidTheme() {
  try {
    const mode = document.documentElement.getAttribute("data-theme") || "";
    return mode === "dark" ? "dark" : "default";
  } catch {
    return "default";
  }
}
function initMermaid() {
  try {
    mermaid.initialize({
      startOnLoad: false,
      theme: getMermaidTheme(),
      securityLevel: "loose",
      flowchart: {
        useMaxWidth: true,
        htmlLabels: true,
        subGraphTitleMargin: { top: 10, bottom: 20 },
      },
    });
    if (!mermaidInitialized) {
      try {
        window.addEventListener("theme-change", () => {
          try {
            mermaid.initialize({ theme: getMermaidTheme() });
          } catch (_e) {
            void 0;
          }
        });
      } catch (_e) {
        void 0;
      }
      mermaidInitialized = true;
    }
  } catch { }
}

/*
function sanitizeDslForDesigner(dsl: string): string {
  try {
    const lines = (dsl || "").split(/\r?\n/);
    const defined = new Set<string>();
    const base = (id: string) => (id || "").split(".").pop() || "";
    for (const line of lines) {
      const m = line.match(
        /^\s*(architecture|system|container|component|person|deployment|datastore|queue|external)\s+([A-Za-z0-9_.-]+)/i
      );
      if (m) {
        defined.add(base(m[2]));
      }
    }
    const missing = new Set<string>();
    const parseToken = (seg: string, takeFirstWord: boolean) => {
      const s = seg.trim();
      if (!s) return "";
      const word = takeFirstWord ? s.split(/\s+/)[0] || "" : s.split(/\s+/).pop() || "";
      const cleaned = word.replace(/[^A-Za-z0-9_.-]/g, "");
      return base(cleaned);
    };
    for (const line of lines) {
      const rm = line.match(/(.+?)(->|<-|=>|<=)(.+)/);
      if (rm) {
        const left = parseToken(rm[1], false);
        const right = parseToken(rm[3], true);
        if (left && !defined.has(left)) missing.add(left);
        if (right && !defined.has(right)) missing.add(right);
      }
    }
    if (missing.size === 0) return dsl;
    const header = Array.from(missing)
      .map((n) => `external ${n} "${n}"`)
      .join("\n");
    return `${header}\n\n${dsl}`;
  } catch {
    return dsl;
  }
}
*/

async function renderMermaid(preview: HTMLElement, dsl: string) {
  preview.innerHTML = "";
  const loaderContainer = document.createElement("div");
  loaderContainer.style.display = "flex";
  loaderContainer.style.alignItems = "center";
  loaderContainer.style.justifyContent = "center";
  loaderContainer.style.padding = "16px";
  const loaderRoot = createRoot(loaderContainer);
  preview.appendChild(loaderContainer);
  loaderRoot.render(
    React.createElement(
      "div",
      { style: { textAlign: "center", color: "var(--color-text-secondary)" } },
      React.createElement(SrujaLoader, { size: 32 }),
      React.createElement("div", { style: { marginTop: 8, fontSize: 12 } }, "Rendering diagram...")
    )
  );
  try {
    const normalize = (s: string) => {
      const basic = s
        .replace(/\u2192/g, "->")
        .replace(/[“”]/g, '"')
        .replace(/[’]/g, "'")
        .replace(/\u2013|\u2014/g, "-");
      return basic
        .split(/\r?\n/)
        .map((line) => line.replace(/^\s*\d+\s*[→:.-]\s?/, ""))
        .join("\n");
    };
    const input = normalize(dsl);
    let code: string | null = null;
    try {
      code = await convertDslToMermaid(input);
    } catch (mermaidError) {
      const errorMsg = mermaidError instanceof Error ? mermaidError.message : String(mermaidError);
      console.error("[CodeBlockActions] convertDslToMermaid error:", mermaidError);
      throw new Error(`Failed to generate mermaid diagram: ${errorMsg}`);
    }

    if (!code) {
      // Try to get more specific error information
      try {
        const api = await initWasm();
        if (!api) {
          throw new Error("WASM not initialized. Please refresh the page.");
        }
        // Try parsing to see if that's the issue
        const jsonStr = await api.parseDslToJson(input);
        if (!jsonStr || jsonStr.trim().length === 0) {
          throw new Error("Failed to parse DSL. Please check your syntax.");
        }
        // If parsing works but mermaid export fails, it's an exporter issue
        console.error(
          "[CodeBlockActions] Parsing succeeded but mermaid export returned null. JSON:",
          jsonStr.substring(0, 200)
        );
        throw new Error(
          "Mermaid export failed. The TypeScript exporter returned null. Please check browser console for details."
        );
      } catch (parseError) {
        const parseMsg = parseError instanceof Error ? parseError.message : String(parseError);
        throw new Error(`Failed to generate mermaid diagram: ${parseMsg}`);
      }
    }

    const firstLine = (code || "").split("\n")[0] || "";
    const metaMatch = firstLine.match(/^%%\s*node_count:\s*(\d+)\s*%%$/);
    let nodeCount = metaMatch ? parseInt(metaMatch[1], 10) : undefined;
    if (typeof nodeCount !== "number" || Number.isNaN(nodeCount)) {
      const lines = input.split(/\r?\n/);
      const nodeRegex =
        /^\s*(architecture|system|container|component|person|deployment|datastore|queue|external)\b/i;
      nodeCount = lines.filter((l) => nodeRegex.test(l)).length;
    }
    const rec = (() => {
      if (nodeCount <= 10) return { w: 600, h: 400 };
      if (nodeCount <= 20) return { w: 800, h: 600 };
      if (nodeCount <= 35) return { w: 1000, h: 700 };
      if (nodeCount <= 60) return { w: 1200, h: 800 };
      return { w: 1400, h: 900 };
    })();
    // Recommendations are automatically applied via styles below - no need to display to users
    const renderId = `sruja-` + Math.random().toString(36).slice(2);
    loaderRoot.unmount();
    preview.innerHTML = "";
    const inner = document.createElement("div");
    inner.style.transformOrigin = "top left";
    inner.style.display = "inline-block";
    inner.style.willChange = "transform";
    const result = await mermaid.render(renderId, code);
    inner.innerHTML = result?.svg || "";
    // Ensure SVG has proper overflow handling to prevent title clipping
    const svg = inner.querySelector("svg");
    if (svg) {
      svg.style.overflow = "visible";
    }
    preview.appendChild(inner);
    const host = preview;
    host.style.minHeight = `${rec.h}px`;
    const svgEl = inner.querySelector("svg") as SVGElement | null;
    if (svgEl) {
      svgEl.style.width = `${rec.w}px`;
      svgEl.style.maxWidth = "none";
      svgEl.style.height = "auto";
      svgEl.setAttribute("preserveAspectRatio", "xMidYMid meet");
    }
    inner.style.transform = `translate(0px, 0px) scale(1)`;
    preview.dataset.scale = "1";
    preview.dataset.tx = "0";
    preview.dataset.ty = "0";
    preview.dataset.inner = "1";
  } catch (e) {
    loaderRoot.unmount();
    const msg = e instanceof Error ? e.message : String(e);

    // Extract page context for better error messages
    const pageTitle = document.querySelector("h1")?.textContent || document.title || "Unknown Page";
    const sectionTitle = (() => {
      // Find the nearest heading before the code block
      const codeBlock = preview.closest("pre")?.parentElement;
      if (!codeBlock) return null;
      let element: Element | null = codeBlock.previousElementSibling;
      while (element) {
        if (element.tagName?.match(/^H[1-6]$/)) {
          return element.textContent || null;
        }
        element = element.previousElementSibling;
      }
      // Fallback: find any heading in the same section
      const parent = codeBlock.closest('article, section, main, [class*="content"]');
      if (parent) {
        const headings = parent.querySelectorAll("h2, h3, h4, h5, h6");
        if (headings.length > 0) {
          return headings[headings.length - 1].textContent || null;
        }
      }
      return null;
    })();

    const errorContext = sectionTitle ? `${pageTitle} > ${sectionTitle}` : pageTitle;

    const errorMessage = `Failed to render diagram in "${errorContext}": ${msg}`;
    preview.innerHTML = `<div style="padding:8px;color:var(--color-error-500)">${errorMessage}</div>`;

    // Track error to PostHog
    try {
      capture("diagram.render_error", {
        error_message: msg,
        page_title: pageTitle,
        section_title: sectionTitle || "",
        page_path: window.location.pathname,
        error_type: e instanceof Error ? e.constructor.name : typeof e,
        dsl_preview: dsl.substring(0, 200), // First 200 chars for context
      });
    } catch (trackError) {
      console.warn("Failed to track error to PostHog:", trackError);
    }
  }
}

function addToolbar(pre: HTMLElement, _codeEl: HTMLElement, dsl: string) {
  const toolbar = document.createElement("div");
  toolbar.style.display = "flex";
  toolbar.style.gap = "8px";
  toolbar.style.position = "absolute";
  toolbar.style.right = "12px";
  toolbar.style.top = "12px";
  toolbar.style.zIndex = "10";

  const btnStyle = {
    padding: "6px 10px",
    borderRadius: "6px",
    border: "1px solid var(--color-border)",
    background: "var(--color-background)",
    color: "var(--color-text-primary)",
    cursor: "pointer",
    fontSize: "12px",
  } as const;

  const showBtn = document.createElement("button");
  showBtn.textContent = "Show Diagram";
  Object.assign(showBtn.style, btnStyle);

  const viewBtn = document.createElement("button");
  viewBtn.textContent = "Open in Designer";
  Object.assign(viewBtn.style, btnStyle);
  let currentDsl = dsl;
  viewBtn.onclick = () => {
    const text = currentDsl || "";
    try {
      const b64 = encodeURIComponent(LZString.compressToBase64(text));
      window.open(`/designer#code=${b64}`, "_blank");
    } catch (e) {
      window.open(`/designer?code=${encodeURIComponent(text)}`, "_blank");
    }
    try {
      window.dispatchEvent(
        new CustomEvent("sruja:event", { detail: { type: "tutorial.open_designer" } })
      );
    } catch { }
  };

  toolbar.appendChild(showBtn);
  toolbar.appendChild(viewBtn);

  // Position wrapper
  const wrapper = document.createElement("div");
  wrapper.style.position = "relative";
  pre.parentElement?.insertBefore(wrapper, pre);
  const editorMount = document.createElement("div");
  editorMount.style.width = "100%";
  editorMount.style.maxHeight = "640px";
  editorMount.style.height = "auto";
  editorMount.style.minHeight = "200px";
  wrapper.appendChild(editorMount);
  const editorRoot = createRoot(editorMount);
  const getTheme = () =>
    document.documentElement.getAttribute("data-theme") === "dark" ? "vs-dark" : "vs";
  let currentTheme: "vs" | "vs-dark" | "hc-black" = getTheme() as any;

  // Calculate appropriate height based on content
  const calculateHeight = (content: string) => {
    const lines = content.split("\n").length;
    const minHeight = 200;
    const maxHeight = 640;
    const lineHeight = 20; // Approximate line height
    const calculatedHeight = Math.min(Math.max(lines * lineHeight + 40, minHeight), maxHeight);
    return `${calculatedHeight}px`;
  };

  const renderEditor = () => {
    const height = calculateHeight(currentDsl);
    editorMount.style.height = height;
    editorRoot.render(
      React.createElement(SrujaMonacoEditor, {
        value: currentDsl,
        onChange: (v) => {
          currentDsl = v || "";
          const newHeight = calculateHeight(currentDsl);
          editorMount.style.height = newHeight;
        },
        theme: currentTheme,
        options: {
          minimap: { enabled: false },
          wordWrap: "on",
          fontSize: 14,
          automaticLayout: true,
          scrollBeyondLastLine: false,
        },
        height: height,
      })
    );
  };
  renderEditor();
  try {
    window.addEventListener("theme-change", () => {
      currentTheme = getTheme() as any;
      renderEditor();
    });
  } catch (_e) {
    void 0;
  }
  pre.style.display = "none";
  wrapper.appendChild(pre);
  wrapper.appendChild(toolbar);

  // Preview container below code
  const preview = document.createElement("div");
  preview.style.border = "1px solid var(--color-border)";
  preview.style.borderRadius = "6px";
  preview.style.marginTop = "12px";
  preview.style.overflow = "auto";
  preview.style.background = "var(--color-background)";
  preview.style.maxHeight = "720px";
  preview.style.position = "relative";
  preview.style.display = "none";
  wrapper.appendChild(preview);

  const overlayControls = document.createElement("div");
  overlayControls.style.position = "absolute";
  overlayControls.style.bottom = "8px";
  overlayControls.style.right = "8px";
  overlayControls.style.display = "flex";
  overlayControls.style.gap = "6px";
  overlayControls.style.zIndex = "5";
  const mkBtn = (label: string) => {
    const b = document.createElement("button");
    Object.assign(b.style, btnStyle);
    b.textContent = label;
    return b;
  };
  const oZoomIn = mkBtn("+");
  const oZoomOut = mkBtn("-");
  const oReset = mkBtn("Reset");
  const oExpand = mkBtn("Expand");
  overlayControls.appendChild(oZoomIn);
  overlayControls.appendChild(oZoomOut);
  overlayControls.appendChild(oReset);
  overlayControls.appendChild(oExpand);
  overlayControls.style.display = "none";
  preview.appendChild(overlayControls);

  const content = document.createElement("div");
  content.style.width = "100%";
  content.style.height = "100%";
  content.style.position = "relative";
  preview.appendChild(content);

  const renderOnce = () => {
    if (preview.dataset.rendered === "1") return;
    preview.dataset.rendered = "1";
    initMermaid();
    renderMermaid(content, currentDsl);
    overlayControls.style.display = "flex";
  };

  // Show/Hide toggle
  showBtn.onclick = () => {
    const visible = preview.style.display !== "none";
    if (visible) {
      preview.style.display = "none";
      showBtn.textContent = "Show Diagram";
    } else {
      try {
        window.dispatchEvent(
          new CustomEvent("sruja:event", { detail: { type: "tutorial.diagram_show" } })
        );
      } catch (_e) {
        void 0;
      }
      renderOnce();
      preview.style.display = "block";
      showBtn.textContent = "Hide Diagram";
    }
  };

  const applyScale = (delta: number | null) => {
    const current = parseFloat(preview.dataset.scale || "1");
    const next = delta === null ? 1 : Math.max(0.25, Math.min(4, current + delta));
    const tx = parseFloat(preview.dataset.tx || "0");
    const ty = parseFloat(preview.dataset.ty || "0");
    applyTransform(next, tx, ty);
  };

  oZoomIn.onclick = () => {
    renderOnce();
    applyScale(0.25);
  };
  oZoomOut.onclick = () => {
    renderOnce();
    applyScale(-0.25);
  };
  oReset.onclick = () => {
    renderOnce();
    applyScale(null);
  };

  const openExpand = async (text: string) => {
    try {
      window.dispatchEvent(
        new CustomEvent("sruja:event", { detail: { type: "tutorial.diagram_expand" } })
      );
    } catch (_e) {
      void 0;
    }
    const overlay = document.createElement("div");
    overlay.style.position = "fixed";
    overlay.style.top = "0";
    overlay.style.left = "0";
    overlay.style.right = "0";
    overlay.style.bottom = "0";
    overlay.style.background = "var(--overlay-scrim)";
    overlay.style.zIndex = "10000";
    overlay.style.display = "flex";
    overlay.style.alignItems = "center";
    overlay.style.justifyContent = "center";
    const panel = document.createElement("div");
    panel.style.background = "var(--color-background)";
    panel.style.width = "90vw";
    panel.style.height = "85vh";
    panel.style.borderRadius = "8px";
    panel.style.boxShadow = "0 20px 25px -5px rgba(0,0,0,0.1), 0 10px 10px -5px rgba(0,0,0,0.04)";
    panel.style.position = "relative";
    const close = document.createElement("button");
    close.textContent = "Close";
    Object.assign(close.style, btnStyle);
    close.style.position = "absolute";
    close.style.top = "12px";
    close.style.right = "12px";
    const content = document.createElement("div");
    content.style.width = "100%";
    content.style.height = "100%";
    content.style.borderTopLeftRadius = "8px";
    content.style.borderTopRightRadius = "8px";
    content.style.overflow = "hidden";
    panel.appendChild(close);
    panel.appendChild(content);
    overlay.appendChild(panel);
    document.body.appendChild(overlay);
    close.onclick = () => {
      document.body.removeChild(overlay);
    };
    const overlayLoader = document.createElement("div");
    overlayLoader.style.display = "flex";
    overlayLoader.style.alignItems = "center";
    overlayLoader.style.justifyContent = "center";
    overlayLoader.style.height = "100%";
    const overlayRoot = createRoot(overlayLoader);
    content.appendChild(overlayLoader);
    overlayRoot.render(
      React.createElement(
        "div",
        { style: { textAlign: "center", color: "var(--color-text-tertiary)" } },
        React.createElement(SrujaLoader, { size: 32 }),
        React.createElement("div", { style: { marginTop: 8, fontSize: 12 } }, "Loading diagram...")
      )
    );
    try {
      // Just render mermaid diagram in expanded view (no diagram package)
      overlayRoot.unmount();
      content.innerHTML = "";
      content.style.padding = "20px";
      content.style.overflow = "auto";
      content.style.display = "flex";
      content.style.alignItems = "center";
      content.style.justifyContent = "center";

      const mermaidContainer = document.createElement("div");
      mermaidContainer.style.width = "100%";
      mermaidContainer.style.height = "100%";
      mermaidContainer.style.display = "flex";
      mermaidContainer.style.alignItems = "center";
      mermaidContainer.style.justifyContent = "center";
      content.appendChild(mermaidContainer);

      // Initialize mermaid if needed and render diagram
      initMermaid();
      await renderMermaid(mermaidContainer, text);
    } catch (e) {
      overlayRoot.unmount();
      const msg = e instanceof Error ? e.message : String(e);
      const pageTitle =
        document.querySelector("h1")?.textContent || document.title || "Unknown Page";
      const errorMessage = `Failed to open designer in "${pageTitle}": ${msg}`;
      content.innerHTML = `<div style="padding:16px;color:var(--color-error-500)">${errorMessage}</div>`;
      try {
        capture("designer.open_error", {
          error_message: msg,
          page_title: pageTitle,
          page_path: window.location.pathname,
          error_type: e instanceof Error ? e.constructor.name : typeof e,
        });
      } catch (trackError) {
        console.warn("Failed to track error to PostHog:", trackError);
      }
    }
  };
  oExpand.onclick = async () => {
    await openExpand(currentDsl);
  };

  let dragging = false;
  let startX = 0;
  let startY = 0;
  preview.addEventListener("mousedown", (e) => {
    const scale = parseFloat(preview.dataset.scale || "1");
    if (scale <= 1) return;
    dragging = true;
    startX = e.clientX;
    startY = e.clientY;
    e.preventDefault();
  });
  window.addEventListener("mouseup", () => {
    dragging = false;
  });
  window.addEventListener("mousemove", (e) => {
    if (!dragging) return;
    const scale = parseFloat(preview.dataset.scale || "1");
    const tx = parseFloat(preview.dataset.tx || "0");
    const ty = parseFloat(preview.dataset.ty || "0");
    const dx = e.clientX - startX;
    const dy = e.clientY - startY;
    startX = e.clientX;
    startY = e.clientY;
    applyTransform(scale, tx + dx, ty + dy);
  });

  // Auto-render once the block is in viewport
  // Disable auto-render; user controls visibility

  const applyTransform = (scale: number, tx: number, ty: number) => {
    const inner = content.firstElementChild as HTMLElement | null;
    if (!inner) return;
    inner.style.transform = `translate(${tx}px, ${ty}px) scale(${scale})`;
    preview.dataset.scale = String(scale);
    preview.dataset.tx = String(tx);
    preview.dataset.ty = String(ty);
  };
}

export default function CodeBlockActions() {
  useEffect(() => {
    const findBlocks = () => {
      // Find all code blocks - Shiki wraps them in pre.astro-code > code
      const candidates = Array.from(
        document.querySelectorAll("pre code, pre.astro-code code")
      ) as HTMLElement[];
      candidates.forEach((codeEl) => {
        const pre = codeEl.parentElement as HTMLElement;
        if (!pre) return;
        if (pre.dataset.srujaToolbar === "1") return;

        // Check code element class names
        const codeClass = codeEl.className || "";
        // Check for language class (Shiki uses 'language-<lang>' format)
        const langMatch = codeClass.match(/language-(\w+)|lang-(\w+)/i);
        const detectedLang = langMatch ? (langMatch[1] || langMatch[2]).toLowerCase() : null;

        // Also check data attributes
        const dataLang =
          codeEl.getAttribute("data-language") || pre.getAttribute("data-language") || "";

        // Determine if this is a Sruja code block
        // First check explicit language markers
        const hasExplicitLang =
          detectedLang === "sruja" ||
          dataLang.toLowerCase() === "sruja" ||
          /language-sruja|lang-sruja/i.test(codeClass) ||
          /language-sruja|lang-sruja/i.test(pre.className || "");

        // If no explicit language, check content for Sruja keywords (as fallback)
        const text = codeEl.textContent || "";
        const looksLikeSruja =
          !hasExplicitLang &&
          /\b(architecture|system|container|component|person|deployment|datastore|queue|external|scenario|story|flow)\b/i.test(
            text
          ) &&
          (/\b(->|<-|=>|<=)\b/.test(text) || /\b(description|technology|owner)\s*"/i.test(text));

        const isSruja = hasExplicitLang || looksLikeSruja;

        if (!isSruja) return;

        if (!text.trim()) return;

        addToolbar(pre, codeEl, text);
        pre.dataset.srujaToolbar = "1";
      });
    };

    findBlocks();

    const mo = new MutationObserver(() => findBlocks());
    mo.observe(document.body, { childList: true, subtree: true });
    return () => mo.disconnect();
  }, []);
  return null;
}
