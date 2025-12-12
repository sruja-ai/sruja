import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";
import fs from "fs";
import type { Plugin } from "vite";

// Custom plugin to serve examples and wasm from monorepo root
function serveMonorepoAssets(): Plugin {
  return {
    name: "serve-monorepo-assets",
    configureServer(server) {
      server.middlewares.use((req: any, res: any, next: any) => {
        // Serve examples from monorepo root
        if (req.url?.startsWith("/examples/")) {
          const examplesPath = path.resolve(
            __dirname,
            "../../examples",
            req.url.replace("/examples/", "")
          );
          if (fs.existsSync(examplesPath)) {
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
        // Serve wasm from apps/website/public/wasm directory (where WASM files are built)
        if (req.url?.startsWith("/wasm/")) {
          const wasmPath = path.resolve(
            __dirname,
            "../website/public/wasm",
            req.url.replace("/wasm/", "")
          );
          if (fs.existsSync(wasmPath)) {
            if (req.url.endsWith(".wasm")) {
              res.setHeader("Content-Type", "application/wasm");
            } else if (req.url.endsWith(".js")) {
              res.setHeader("Content-Type", "application/javascript");
            }
            fs.createReadStream(wasmPath).pipe(res);
            return;
          }
        }
        next();
      });
    },
    // Also configure preview server (vite preview) for static build tests
    configurePreviewServer(server) {
      server.middlewares.use((req: any, res: any, next: any) => {
        // Serve examples from monorepo root
        if (req.url?.startsWith("/examples/")) {
          const examplesPath = path.resolve(
            __dirname,
            "../../examples",
            req.url.replace("/examples/", "")
          );
          if (fs.existsSync(examplesPath)) {
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
        // Serve wasm from apps/website/public/wasm directory (where WASM files are built)
        if (req.url?.startsWith("/wasm/")) {
          const wasmPath = path.resolve(
            __dirname,
            "../website/public/wasm",
            req.url.replace("/wasm/", "")
          );
          if (fs.existsSync(wasmPath)) {
            if (req.url.endsWith(".wasm")) {
              res.setHeader("Content-Type", "application/wasm");
            } else if (req.url.endsWith(".js")) {
              res.setHeader("Content-Type", "application/javascript");
            }
            fs.createReadStream(wasmPath).pipe(res);
            return;
          }
        }
        next();
      });
    },
  };
}

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), serveMonorepoAssets()],

  // Allow serving files from monorepo root
  server: {
    fs: {
      allow: ["..", "../..", "../../.."],
    },
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
});
