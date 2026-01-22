// apps/website/astro.config.mjs

// Suppress glob-loader duplicate ID warnings (false positives from hot reload)
if (typeof console !== "undefined" && console.warn) {
  const originalWarn = console.warn;
  const suppressedPattern = /\[glob-loader\].*Duplicate id/;

  console.warn = (...args) => {
    const message = args[0]?.toString() || "";
    if (suppressedPattern.test(message)) {
      return; // Suppress this warning
    }
    originalWarn.apply(console, args);
  };
}

import { defineConfig } from "astro/config";
import react from "@astrojs/react";
import mdx from "@astrojs/mdx";
import { nodePolyfills } from "vite-plugin-node-polyfills";

import tailwindcss from "@tailwindcss/vite";
import path from "path";
import { fileURLToPath } from "url";
import fs from "fs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Plugin to watch example files for HMR
function watchExampleFiles() {
  return {
    name: "watch-example-files",
    configureServer(server) {
      // Watch example files for HMR
      const examplesRootPath = path.resolve(__dirname, "../../examples");
      const websiteExamplesPath = path.resolve(__dirname, "public/examples");

      const watchExampleFiles = (dir, basePath) => {
        if (!fs.existsSync(dir)) return null;

        // Watch directory recursively
        const watcher = fs.watch(dir, { recursive: true }, (eventType, filename) => {
          if (!filename) return;

          // Only watch .sruja and .json files
          if (!filename.endsWith(".sruja") && !filename.endsWith(".json")) {
            return;
          }

          const fullPath = path.join(dir, filename);

          // Skip if it's a directory
          try {
            const stat = fs.statSync(fullPath);
            if (!stat.isFile()) return;
          } catch {
            return;
          }

          // Convert to URL path
          const relativePath = path.relative(basePath, fullPath);
          const urlPath = `/examples/${relativePath.replace(/\\/g, "/")}`;

          // Trigger HMR for all connected clients
          if (server.ws) {
            server.ws.send({
              type: "update",
              updates: [
                {
                  type: "js-update",
                  path: urlPath,
                  acceptedPath: urlPath,
                  timestamp: Date.now(),
                },
              ],
            });

            // Also send a custom HMR event that can be handled by the app
            server.ws.send({
              type: "custom",
              event: "example-file-changed",
              data: { path: urlPath, filename },
            });
          }
        });

        return watcher;
      };

      // Watch both example directories
      const watchers = [];
      const rootWatcher = watchExampleFiles(examplesRootPath, examplesRootPath);
      const websiteWatcher = watchExampleFiles(websiteExamplesPath, websiteExamplesPath);

      if (rootWatcher) watchers.push(rootWatcher);
      if (websiteWatcher) watchers.push(websiteWatcher);

      // Clean up watchers on server close
      if (server.httpServer) {
        server.httpServer.once("close", () => {
          watchers.forEach((watcher) => watcher.close());
        });
      }
    },
  };
}

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
    plugins: [
      tailwindcss(),
      nodePolyfills({
        // Include module polyfill for path-browserify compatibility
        globals: {
          Buffer: true,
          global: true,
          process: true,
        },
        protocolImports: true,
        // Exclude Node.js built-ins from polyfills - we want to use real modules in content config
        // Also exclude CommonJS modules that cause issues in module runner context
        // These should use Node.js native modules instead of browser polyfills
        // The polyfills will still be available for browser code if needed via other mechanisms
        exclude: [
          "path",
          "fs",
          "util",
          "stream-http",
          "http",
          "https",
          "events", // CommonJS module used by node-stdlib-browser
          "stream", // Node.js built-in
        ],
      }),
      {
        name: "handle-commonjs-externals",
        enforce: "pre",
        resolveId(id) {
          // Mark CommonJS packages as external so Node.js handles them natively with require()
          // This prevents vite-plugin-node-polyfills from trying to polyfill them
          if (
            id === "stream-http" ||
            id.startsWith("stream-http/") ||
            id === "events" ||
            id.startsWith("events/")
          ) {
            return { id, external: true };
          }
          return null;
        },
      },
      watchExampleFiles(),
      {
        name: "force-node-modules-in-content-config",
        enforce: "pre",
        resolveId(id, importer) {
          // Force Node.js built-ins to use real modules in content config context
          // This prevents browserify polyfills from being used, which have compatibility issues
          // Check if this is being imported in content config or Astro content loader context
          const isContentConfigContext =
            importer &&
            (importer.includes("content.config") ||
              importer.includes("content/loaders") ||
              importer.includes("content/utils") ||
              importer.includes("astro/dist/content") ||
              importer.includes("astro/dist/core/sync") ||
              importer.includes("astro/dist/core/errors"));

          if (isContentConfigContext) {
            // Force real Node.js modules for content collection sync and Astro internals
            if (
              id === "path" ||
              id === "fs" ||
              id === "fs/promises" ||
              id === "url" ||
              id === "util"
            ) {
              return { id, external: true };
            }
          }

          // Mark other Node.js built-ins as external in general
          if (id === "fs/promises" || id === "url" || id === "util") {
            return { id, external: true };
          }
          return null;
        },
      },
      {
        name: "suppress-glob-loader-warnings",
        enforce: "pre",
        buildStart() {
          // Intercept console.warn to suppress duplicate ID warnings from glob-loader
          // These are false positives from Astro's dev server hot reload
          if (typeof console !== "undefined" && console.warn) {
            const originalWarn = console.warn;
            const suppressedPattern = /\[glob-loader\].*Duplicate id/;

            console.warn = (...args) => {
              const message = args[0]?.toString() || "";
              // Suppress only the specific glob-loader duplicate ID warnings
              if (suppressedPattern.test(message)) {
                return; // Suppress this warning
              }
              // Allow all other warnings through
              originalWarn.apply(console, args);
            };
          }
        },
        configureServer(server) {
          // Also suppress in dev mode
          if (typeof console !== "undefined" && console.warn) {
            const originalWarn = console.warn;
            const suppressedPattern = /\[glob-loader\].*Duplicate id/;

            console.warn = (...args) => {
              const message = args[0]?.toString() || "";
              if (suppressedPattern.test(message)) {
                return;
              }
              originalWarn.apply(console, args);
            };
          }
        },
      },
    ],
    server: {
      cors: true,
      watch: {
        // Watch workspace packages for changes
        ignored: [
          "!**/node_modules/@sruja/**",
          "**/*.test.{ts,tsx}",
          "**/*.spec.{ts,tsx}",
          "**/__tests__/**",
        ],
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
      exclude: ["@sruja/shared", "@sruja/ui", "@sruja/designer"],
      // Ensure CommonJS modules are properly transformed during optimization
      esbuildOptions: {
        // This helps esbuild handle CommonJS modules
        format: "esm",
      },
    },
    ssr: {
      // Static site - no SSR, but Vite still uses this config during build
      // Add packages to noExternal so Vite processes them (needed for CSS and module resolution)
      // React must remain external to prevent multiple instances
      // Add packages to noExternal so Vite processes them (needed for CSS and module resolution)
      // React must remain external to prevent multiple instances
      noExternal: ["@sruja/ui", "@sruja/shared", "monaco-editor"],
      // Keep React external to ensure single instance
      // Also keep Node.js built-ins external for content config context
      external: [
        "react",
        "react-dom",
        "react/jsx-runtime",
        "react-dom/client",
        "path",
        "fs",
        "fs/promises",
        "url",
        "util",
        "stream-http", // Externalize so Node.js handles it natively (dependency of vite-plugin-node-polyfills)
        "events", // Externalize CommonJS module
      ],
    },
    build: {
      // Enable CommonJS transformation for mixed ESM/CJS modules
      // This helps Vite handle CommonJS modules like stream-http properly
      commonjsOptions: {
        transformMixedEsModules: true,
        include: [/node_modules/],
      },
    },
    resolve: {
      conditions: ["import", "module", "browser", "default"],
      // Ensure CSS files are resolved as raw assets, not modules
      dedupe: ["react", "react-dom", "@sruja/ui", "@sruja/shared", "@sruja/designer"],
      // Explicitly handle CSS imports from packages
      extensions: [".mjs", ".js", ".mts", ".ts", ".jsx", ".tsx", ".json", ".css"],
      alias: {
        // Map CSS import to actual file path
        "node:buffer": "buffer",
        // Use Node.js built-ins in SSR/build context
        // Alias npm 'util' package to Node.js built-in 'util' to avoid CommonJS issues
        util: "node:util",
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
