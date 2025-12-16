// apps/website/astro.config.mjs

import { defineConfig } from "astro/config";
import react from "@astrojs/react";
import mdx from "@astrojs/mdx";

import tailwindcss from "@tailwindcss/vite";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// https://astro.build/config
// Access Node.js process.env (available in Astro config context at build time)
const siteUrl = process.env.SITE_URL;
const baseUrl = process.env.BASE_URL;

export default defineConfig({
  site: siteUrl || "https://sruja.ai",
  base: baseUrl || "/",
  markdown: {
    syntaxHighlight: "shiki",
    shikiConfig: {
      wrap: true,
      themes: {
        light: "github-light",
        dark: "github-dark",
      },
      langs: [
        {
          id: "sruja",
          scopeName: "source.sruja",
          path: path.resolve(__dirname, "./syntaxes/sruja.tmLanguage.json"),
        },
      ],
    },
  },
  integrations: [react(), mdx()],
  output: "static",
  vite: {
    plugins: [tailwindcss()],
    server: {
      cors: true,
      watch: {
        // Watch workspace packages for changes
        ignored: ["!**/node_modules/@sruja/**"],
      },
      fs: {
        allow: [
          // Allow accessing files from the monorepo root (packages/apps)
          path.resolve(__dirname, "..", ".."),
        ],
      },
    },
    define: {
      global: "globalThis",
    },
    optimizeDeps: {
      include: [
        "react",
        "react-dom",
        "react-dom/client",
        "monaco-editor",
        "buffer",
        "algoliasearch/lite",
        "mermaid",
        "lz-string",
      ],
      exclude: ["@sruja/shared", "@sruja/ui", "@sruja/layout", "@sruja/diagram", "@sruja/designer"],
    },
    ssr: {
      // Static site - no SSR, but Vite still uses this config during build
      // Add packages to noExternal so Vite processes them (needed for CSS and module resolution)
      // React must remain external to prevent multiple instances
      noExternal: ["@sruja/ui", "@sruja/shared", "monaco-editor"],
      // Keep React external to ensure single instance
      external: ["react", "react-dom", "react/jsx-runtime", "react-dom/client"],
    },
    resolve: {
      conditions: ["import", "module", "browser", "default"],
      // Ensure CSS files are resolved as raw assets, not modules
      dedupe: [
        "react",
        "react-dom",
        "@sruja/ui",
        "@sruja/shared",
        "@sruja/diagram",
        "@sruja/designer",
      ],
      // Explicitly handle CSS imports from packages
      extensions: [".mjs", ".js", ".mts", ".ts", ".jsx", ".tsx", ".json", ".css"],
      alias: {
        // Map CSS import to actual file path
        "node:buffer": "buffer",
        // Feature-based path aliases
        "@": path.resolve(__dirname, "./src"),
        "@/features": path.resolve(__dirname, "./src/features"),
        "@/shared": path.resolve(__dirname, "./src/shared"),
      },
    },
    css: {
      postcss: {
        plugins: [],
      },
    },
    // Default CSS handling; Tailwind v4 Vite plugin processes imports
  },
});
