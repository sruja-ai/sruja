import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { nodePolyfills } from "vite-plugin-node-polyfills";
import path from "path";
import fs from "fs";
import type { Plugin } from "vite";

// Custom plugin to serve examples and wasm from monorepo root
function serveMonorepoAssets(): Plugin {
  return {
    name: "serve-monorepo-assets",
    configureServer(server) {
      // Watch example files for HMR
      const examplesRootPath = path.resolve(__dirname, "../../examples");
      const designerExamplesPath = path.resolve(__dirname, "public/examples");

      const watchExampleFiles = (dir: string, basePath: string): fs.FSWatcher | null => {
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
        });

        return watcher;
      };

      // Watch both example directories
      const watchers: fs.FSWatcher[] = [];
      const rootWatcher = watchExampleFiles(examplesRootPath, examplesRootPath);
      const designerWatcher = watchExampleFiles(designerExamplesPath, designerExamplesPath);

      if (rootWatcher) watchers.push(rootWatcher);
      if (designerWatcher) watchers.push(designerWatcher);

      // Clean up watchers on server close
      server.httpServer?.once("close", () => {
        watchers.forEach((watcher) => watcher.close());
      });

      server.middlewares.use(
        (
          req: import("node:http").IncomingMessage,
          res: import("node:http").ServerResponse,
          next: () => void
        ) => {
          // Serve examples from public/examples first, then monorepo root
          // Remove query parameters for file lookup
          const urlPath = req.url?.split("?")[0];

          if (urlPath?.startsWith("/examples/")) {
            const relativePath = urlPath.replace("/examples/", "");

            // Try public/examples first (designer-specific examples)
            let examplesPath = path.resolve(__dirname, "public/examples", relativePath);

            // Fall back to monorepo root examples
            if (!fs.existsSync(examplesPath)) {
              examplesPath = path.resolve(__dirname, "../../examples", relativePath);
            }

            if (fs.existsSync(examplesPath)) {
              // Prevent caching of example files
              res.setHeader("Cache-Control", "no-cache, no-store, must-revalidate");
              res.setHeader("Pragma", "no-cache");
              res.setHeader("Expires", "0");
              if (req.url.endsWith(".json")) {
                res.setHeader("Content-Type", "application/json");
              } else if (req.url.endsWith(".png")) {
                res.setHeader("Content-Type", "image/png");
              } else {
                res.setHeader("Content-Type", "text/plain");
              }
              fs.createReadStream(examplesPath).pipe(res);
              return;
            }
          }
          // Serve wasm from apps/designer/public/wasm first, then apps/website/public/wasm
          if (req.url?.startsWith("/wasm/")) {
            // Remove query parameters for file lookup
            const wasmFile = req.url.split("?")[0].replace("/wasm/", "");

            // Try designer's public/wasm first (for graphvizlib.wasm)
            const designerWasmPath = path.resolve(__dirname, "public/wasm", wasmFile);
            if (fs.existsSync(designerWasmPath)) {
              // Prevent caching of WASM files in development
              res.setHeader("Cache-Control", "no-cache, no-store, must-revalidate");
              res.setHeader("Pragma", "no-cache");
              res.setHeader("Expires", "0");
              // Add ETag based on file modification time for cache validation
              const stats = fs.statSync(designerWasmPath);
              const etag = `"${stats.mtime.getTime()}-${stats.size}"`;
              res.setHeader("ETag", etag);
              if (req.headers["if-none-match"] === etag) {
                res.writeHead(304);
                res.end();
                return;
              }
              if (wasmFile.endsWith(".wasm")) {
                res.setHeader("Content-Type", "application/wasm");
              } else if (wasmFile.endsWith(".js")) {
                res.setHeader("Content-Type", "application/javascript");
              }
              fs.createReadStream(designerWasmPath).pipe(res);
              return;
            }
            // Fallback to website's public/wasm (for sruja.wasm)
            const websiteWasmPath = path.resolve(__dirname, "../website/public/wasm", wasmFile);
            if (fs.existsSync(websiteWasmPath)) {
              // Prevent caching of WASM files in development
              res.setHeader("Cache-Control", "no-cache, no-store, must-revalidate");
              res.setHeader("Pragma", "no-cache");
              res.setHeader("Expires", "0");
              // Add ETag based on file modification time for cache validation
              const stats = fs.statSync(websiteWasmPath);
              const etag = `"${stats.mtime.getTime()}-${stats.size}"`;
              res.setHeader("ETag", etag);
              if (req.headers["if-none-match"] === etag) {
                res.writeHead(304);
                res.end();
                return;
              }
              if (wasmFile.endsWith(".wasm")) {
                res.setHeader("Content-Type", "application/wasm");
              } else if (wasmFile.endsWith(".js")) {
                res.setHeader("Content-Type", "application/javascript");
              }
              fs.createReadStream(websiteWasmPath).pipe(res);
              return;
            }
          }
          next();
        }
      );
    },
    // Also configure preview server (vite preview) for static build tests
    configurePreviewServer(server) {
      server.middlewares.use(
        (
          req: import("node:http").IncomingMessage,
          res: import("node:http").ServerResponse,
          next: () => void
        ) => {
          // Serve examples from public/examples first, then monorepo root
          // Remove query parameters for file lookup
          const urlPath = req.url?.split("?")[0];

          if (urlPath?.startsWith("/examples/")) {
            const relativePath = urlPath.replace("/examples/", "");

            // Try public/examples first (designer-specific examples)
            let examplesPath = path.resolve(__dirname, "public/examples", relativePath);

            // Fall back to monorepo root examples
            if (!fs.existsSync(examplesPath)) {
              examplesPath = path.resolve(__dirname, "../../examples", relativePath);
            }

            if (fs.existsSync(examplesPath)) {
              // Prevent caching of example files
              res.setHeader("Cache-Control", "no-cache, no-store, must-revalidate");
              res.setHeader("Pragma", "no-cache");
              res.setHeader("Expires", "0");
              if (req.url.endsWith(".json")) {
                res.setHeader("Content-Type", "application/json");
              } else if (req.url.endsWith(".png")) {
                res.setHeader("Content-Type", "image/png");
              } else {
                res.setHeader("Content-Type", "text/plain");
              }
              fs.createReadStream(examplesPath).pipe(res);
              return;
            }
          }
          // Serve wasm from apps/designer/public/wasm first, then apps/website/public/wasm
          if (req.url?.startsWith("/wasm/")) {
            // Remove query parameters for file lookup
            const wasmFile = req.url.split("?")[0].replace("/wasm/", "");

            // Try designer's public/wasm first (for graphvizlib.wasm)
            const designerWasmPath = path.resolve(__dirname, "public/wasm", wasmFile);
            if (fs.existsSync(designerWasmPath)) {
              // Prevent caching of WASM files in preview mode
              res.setHeader("Cache-Control", "no-cache, no-store, must-revalidate");
              res.setHeader("Pragma", "no-cache");
              res.setHeader("Expires", "0");
              // Add ETag based on file modification time for cache validation
              const stats = fs.statSync(designerWasmPath);
              const etag = `"${stats.mtime.getTime()}-${stats.size}"`;
              res.setHeader("ETag", etag);
              if (req.headers["if-none-match"] === etag) {
                res.writeHead(304);
                res.end();
                return;
              }
              if (wasmFile.endsWith(".wasm")) {
                res.setHeader("Content-Type", "application/wasm");
              } else if (wasmFile.endsWith(".js")) {
                res.setHeader("Content-Type", "application/javascript");
              }
              fs.createReadStream(designerWasmPath).pipe(res);
              return;
            }
            // Fallback to website's public/wasm (for sruja.wasm)
            const websiteWasmPath = path.resolve(__dirname, "../website/public/wasm", wasmFile);
            if (fs.existsSync(websiteWasmPath)) {
              // Prevent caching of WASM files in preview mode
              res.setHeader("Cache-Control", "no-cache, no-store, must-revalidate");
              res.setHeader("Pragma", "no-cache");
              res.setHeader("Expires", "0");
              // Add ETag based on file modification time for cache validation
              const stats = fs.statSync(websiteWasmPath);
              const etag = `"${stats.mtime.getTime()}-${stats.size}"`;
              res.setHeader("ETag", etag);
              if (req.headers["if-none-match"] === etag) {
                res.writeHead(304);
                res.end();
                return;
              }
              if (wasmFile.endsWith(".wasm")) {
                res.setHeader("Content-Type", "application/wasm");
              } else if (wasmFile.endsWith(".js")) {
                res.setHeader("Content-Type", "application/javascript");
              }
              fs.createReadStream(websiteWasmPath).pipe(res);
              return;
            }
          }
          next();
        }
      );
    },
  };
}

// https://vite.dev/config/
export default defineConfig({
  // Base path for deployment (can be overridden by BASE_PATH env var)
  // For GitHub Pages subdirectory: '/designer/' or '/repo-name/'
  // For root deployment: '/' (default)
  // Default to /designer/ for prod-website deployment
  base: process.env.BASE_PATH || (process.env.NODE_ENV === "production" ? "/designer/" : "/"),

  plugins: [
    react(),
    nodePolyfills(),
    serveMonorepoAssets(),
    // Suppress warnings about Node.js modules in dev mode
    // (packages/shared/src/examples/index.ts conditionally imports them)
    {
      name: "suppress-node-warnings",
      enforce: "pre",
      resolveId(id) {
        // Mark Node.js modules as external to suppress warnings in dev mode
        if (id === "fs/promises" || id === "path" || id === "url") {
          return { id, external: true };
        }
        return null;
      },
    },
  ],

  // Allow serving files from monorepo root
  server: {
    fs: {
      allow: ["..", "../..", "../../.."],
    },
  },
  preview: {
    port: 4322,
    host: true,
  },

  // Resolve aliases
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
    dedupe: ["react", "react-dom"],
  },

  optimizeDeps: {
    exclude: ["monaco-editor"],
  },

  // Externalize Node.js modules that are only used in Node.js context
  // (packages/shared/src/examples/index.ts uses them conditionally)
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          "react-vendor": ["react", "react-dom", "zustand"],
          "ui-vendor": ["@sruja/ui", "lucide-react", "framer-motion"],
          "diagram-vendor": ["@xyflow/react"],
          "core-vendor": ["@sruja/shared"],
          "editor-vendor": ["monaco-editor"],
          "firebase-vendor": ["firebase/app", "firebase/firestore", "firebase/auth"], // Be specific if possible, or just 'firebase'
        },
      },
      external: (id) => {
        // Externalize Node.js built-ins that are conditionally imported
        // Don't externalize 'events' - we're polyfilling it
        if (id === "fs/promises" || id === "path" || id === "url") {
          return true;
        }
        return false;
      },
      onwarn(warning, warn) {
        // Suppress warnings about externalized Node.js modules
        if (
          warning.code === "MODULE_LEVEL_DIRECTIVE" ||
          (warning.message &&
            (warning.message.includes("has been externalized for browser compatibility") ||
              warning.message.includes("fs/promises") ||
              warning.message.includes("path") ||
              warning.message.includes("url") ||
              warning.message.includes("d3-sankey"))) // Suppress d3-sankey warning (optional mermaid dependency)
        ) {
          return;
        }
        // Suppress unresolved import warnings for optional dependencies
        if (warning.code === "UNRESOLVED_IMPORT" && warning.message?.includes("d3-sankey")) {
          return;
        }
        warn(warning);
      },
    },
    chunkSizeWarningLimit: 4000,
  },
});
