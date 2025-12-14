import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";
import fs from "fs";

export default defineConfig({
  plugins: [react()],
  root: "test-app",
  server: {
    port: 3000,
    fs: {
      strict: false,
      allow: [".."],
    },
    watch: {
      // Watch workspace packages for changes
      ignored: ["!**/node_modules/@sruja/**"],
    },
  },
  resolve: {
    alias: {
      // Ensure proper resolution of workspace packages
      "@sruja/layout": path.resolve(__dirname, "../layout/src"),
    },
  },
  optimizeDeps: {
    // Don't pre-bundle workspace packages to ensure changes are picked up
    exclude: ["@sruja/layout"],
  },
});
